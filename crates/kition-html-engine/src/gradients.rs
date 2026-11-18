//! Device-space gradient sampling with ordered color quantization.
//!
//! An 8x8 Bayer matrix distributes the final 8-bit quantization error, as in
//! Skia's raster pipeline. This is part of brush rendering, before shape coverage,
//! filtering, opacity, and blending; captured PNGs are never postprocessed.
use anyrender::{Paint, Scene, recording::RenderCommand};
use peniko::{
    Blob, Extend, Gradient, GradientKind, ImageAlphaType, ImageBrush, ImageData, ImageFormat,
    ImageQuality, ImageSampler, InterpolationAlphaSpace,
    color::{ColorSpaceTag, Srgb},
    kurbo::{Affine, Point, Rect, Shape},
};
use std::sync::Arc;

pub fn prepare_gradients(scene: &mut Scene, width: u32, height: u32) {
    let viewport = Rect::new(0.0, 0.0, width as f64, height as f64);
    for command in &mut scene.commands {
        let (brush, transform, brush_transform, bounds) = match command {
            RenderCommand::Fill(fill) => (
                &mut fill.brush,
                fill.transform,
                &mut fill.brush_transform,
                fill.shape.bounding_box(),
            ),
            RenderCommand::Stroke(stroke) => (
                &mut stroke.brush,
                stroke.transform,
                &mut stroke.brush_transform,
                stroke.shape.bounding_box().inflate(
                    stroke.style.width * stroke.style.miter_limit.max(1.0),
                    stroke.style.width * stroke.style.miter_limit.max(1.0),
                ),
            ),
            _ => continue,
        };
        let Paint::Gradient(gradient) = brush else {
            continue;
        };
        if !supported(gradient) {
            continue;
        }
        let device_bounds = transform
            .transform_rect_bbox(bounds)
            .expand()
            .intersect(viewport);
        if device_bounds.width() <= 0.0 || device_bounds.height() <= 0.0 {
            continue;
        }
        let gradient_transform = transform * brush_transform.unwrap_or(Affine::IDENTITY);
        if gradient_transform.determinant().abs() < f64::MIN_POSITIVE {
            continue;
        }
        let inverse = gradient_transform.inverse();
        if !inverse.as_coeffs().iter().all(|v| v.is_finite()) {
            continue;
        }
        let stops: Vec<_> = gradient
            .stops
            .iter()
            .map(|stop| {
                let mut rgba = stop
                    .color
                    .to_alpha_color::<Srgb>()
                    .components
                    .map(f64::from);
                if gradient.interpolation_alpha_space == InterpolationAlphaSpace::Premultiplied {
                    for c in 0..3 {
                        rgba[c] *= rgba[3];
                    }
                }
                (f64::from(stop.offset), rgba)
            })
            .collect();
        let width = device_bounds.width() as u32;
        let height = device_bounds.height() as u32;
        let origin = device_bounds.origin().to_vec2();
        let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
        for y in 0..height {
            for x in 0..width {
                let global_x = x + origin.x as u32;
                let global_y = y + origin.y as u32;
                let point = inverse * Point::new(global_x as f64 + 0.5, global_y as f64 + 0.5);
                let position = sample_position(&gradient.kind, point);
                let position = match gradient.extend {
                    Extend::Pad => position.clamp(0.0, 1.0),
                    Extend::Repeat => position.rem_euclid(1.0),
                    Extend::Reflect => {
                        let t = position.rem_euclid(2.0);
                        if t > 1.0 { 2.0 - t } else { t }
                    }
                };
                let mut rgba = sample_stops(&stops, position);
                if gradient.interpolation_alpha_space == InterpolationAlphaSpace::Unpremultiplied {
                    for c in 0..3 {
                        rgba[c] *= rgba[3];
                    }
                }
                let offset = ((y * width + x) * 4) as usize;
                let noise = bayer_noise(global_x, global_y);
                for c in 0..3 {
                    pixels[offset + c] =
                        ((rgba[c] * 255.0 + noise).clamp(0.0, rgba[3] * 255.0)).round() as u8;
                }
                pixels[offset + 3] = (rgba[3] * 255.0).round() as u8;
            }
        }
        *brush = Paint::Image(ImageBrush {
            image: ImageData {
                data: Blob::new(Arc::new(pixels)),
                format: ImageFormat::Rgba8,
                width,
                height,
                alpha_type: ImageAlphaType::AlphaPremultiplied,
            },
            sampler: ImageSampler {
                quality: ImageQuality::Low,
                ..Default::default()
            },
        });
        *brush_transform = Some(transform.inverse() * Affine::translate(origin));
    }
}

fn supported(gradient: &Gradient) -> bool {
    if gradient.interpolation_cs != ColorSpaceTag::Srgb
        || gradient.stops.is_empty()
        || !gradient
            .stops
            .windows(2)
            .all(|pair| pair[0].offset <= pair[1].offset)
    {
        return false;
    }
    match &gradient.kind {
        GradientKind::Linear(line) => line.start.distance_squared(line.end) > 0.0,
        GradientKind::Radial(radial) => {
            radial.start_center == radial.end_center && radial.end_radius > radial.start_radius
        }
        GradientKind::Sweep(sweep) => sweep.end_angle > sweep.start_angle,
    }
}

fn sample_position(kind: &GradientKind, point: Point) -> f64 {
    match kind {
        GradientKind::Linear(line) => {
            let delta = line.end - line.start;
            (point - line.start).dot(delta) / delta.hypot2()
        }
        GradientKind::Radial(radial) => {
            (point.distance(radial.start_center) - f64::from(radial.start_radius))
                / f64::from(radial.end_radius - radial.start_radius)
        }
        GradientKind::Sweep(sweep) => {
            let delta = point - sweep.center;
            let angle = delta.y.atan2(delta.x).rem_euclid(std::f64::consts::TAU);
            (angle - f64::from(sweep.start_angle)) / f64::from(sweep.end_angle - sweep.start_angle)
        }
    }
}

fn sample_stops(stops: &[(f64, [f64; 4])], position: f64) -> [f64; 4] {
    let end = stops.partition_point(|(offset, _)| *offset <= position);
    if end == 0 {
        return stops[0].1;
    }
    if end == stops.len() {
        return stops[end - 1].1;
    }
    let (start_offset, start) = stops[end - 1];
    let (end_offset, end_color) = stops[end];
    let t = (position - start_offset) / (end_offset - start_offset);
    std::array::from_fn(|i| start[i] + (end_color[i] - start[i]) * t)
}

fn bayer_noise(x: u32, y: u32) -> f64 {
    // Recursively construct Bayer's matrix from [0, 3; 2, 1]. The least
    // significant coordinate bits select the largest threshold contribution.
    let mut rank = 0;
    for bit in 0..3 {
        let column = (x >> bit) & 1;
        let row = (y >> bit) & 1;
        let quadrant = match (column, row) {
            (0, 0) => 0,
            (1, 0) => 3,
            (0, 1) => 2,
            _ => 1,
        };
        rank = rank * 4 + quadrant;
    }
    f64::from(rank) / 64.0 - 63.0 / 128.0
}
