//! Lower color and Gaussian filter groups to CPU images before replaying the remaining scene.
use anyrender::{
    PaintScene, Scene,
    filters::{Filter, FilterEffect, FilterId, FilterInput, FilterSource},
    recording::{LayerCommand, RenderCommand},
    render_to_buffer,
};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use peniko::{
    Blob, Fill, ImageAlphaType, ImageBrush, ImageData, ImageFormat, ImageQuality, ImageSampler,
    kurbo::{Affine, Rect, Shape},
};
use std::sync::Arc;

/// Color matrices act on an isolated group's unpremultiplied sRGB channels,
/// before the group's opacity, blend, and parent clips are applied. The pinned
/// CPU backend cannot execute them itself. Lower innermost groups first so
/// nested filters retain their order without recursive Rust calls.
pub fn prepare_filters(
    scene: &mut Scene,
    width: u32,
    height: u32,
    diagnostics: &mut Vec<String>,
) -> Result<(), String> {
    let commands = std::mem::take(&mut scene.commands);
    let mut stack = Vec::new();
    for mut command in commands {
        match &mut command {
            RenderCommand::PushLayer(layer) => {
                let mut cpu_filter = None;
                if let Some(filter) = &layer.filter {
                    if is_color_chain(filter) || blur_sigmas(filter, layer).is_some() {
                        cpu_filter = layer.filter.take();
                    } else if !is_backend_filter(filter) {
                        layer.filter = None;
                        diagnostics.push("html.unsupported-filter: the CPU backend cannot render this filter or filter chain; the unfiltered content is shown".into());
                    }
                }
                if layer.backdrop_filter.take().is_some() {
                    diagnostics.push("html.unsupported-backdrop-filter: the CPU backend does not implement backdrop filters".into());
                }
                stack.push((scene.commands.len(), cpu_filter));
            }
            RenderCommand::PushClipLayer(_) => stack.push((scene.commands.len(), None)),
            RenderCommand::PopLayer => {
                let (start, filter) = stack.pop().ok_or("unbalanced scene layer")?;
                if let Some(filter) = filter {
                    let children = Scene {
                        tolerance: scene.tolerance,
                        commands: scene.commands.split_off(start + 1),
                    };
                    let RenderCommand::PushLayer(layer) = &scene.commands[start] else {
                        return Err("CPU filter has no scene layer".into());
                    };
                    if let Some((image, origin)) =
                        filter_group(children, layer, &filter, width, height)
                    {
                        scene.fill(
                            Fill::NonZero,
                            Affine::translate(origin),
                            image.as_ref(),
                            None,
                            &Rect::new(
                                0.0,
                                0.0,
                                image.image.width as f64,
                                image.image.height as f64,
                            ),
                        );
                    }
                }
            }
            _ => {}
        }
        scene.commands.push(command);
    }
    if !stack.is_empty() {
        return Err("unclosed scene layer".into());
    }
    Ok(())
}

fn is_backend_filter(filter: &Filter) -> bool {
    filter.nodes().len() == 1
        && matches!(
            filter.nodes()[0].effect,
            FilterEffect::Flood(_)
                | FilterEffect::GaussianBlur(_)
                | FilterEffect::DropShadow(_)
                | FilterEffect::Offset(_)
        )
}

fn is_color_chain(filter: &Filter) -> bool {
    !filter.nodes().is_empty()
        && filter.output() == FilterId((filter.nodes().len() - 1) as u16)
        && filter.nodes().iter().enumerate().all(|(index, node)| {
            matches!(node.effect, FilterEffect::ColorMatrix(_))
                && node.inputs.secondary.is_none()
                && if index == 0 {
                    node.inputs.primary.is_none()
                        || node.inputs.primary
                            == Some(FilterInput::Source(FilterSource::SourceGraphic))
                } else {
                    node.inputs.primary == Some(FilterInput::Result(FilterId((index - 1) as u16)))
                }
        })
}

fn blur_sigmas(filter: &Filter, layer: &LayerCommand) -> Option<(f64, f64)> {
    if filter.nodes().len() != 1 {
        return None;
    }
    let FilterEffect::GaussianBlur(blur) = &filter.nodes()[0].effect else {
        return None;
    };
    if blur.edge_mode != anyrender::filters::EdgeMode::None {
        return None;
    }
    let [a, b, c, d, _, _] = layer.transform.as_coeffs();
    if !((b.abs() < 1e-8 && c.abs() < 1e-8) || (a.abs() < 1e-8 && d.abs() < 1e-8)) {
        return None;
    }
    let sigma = f64::from(blur.std_deviation);
    let x = sigma * a.hypot(c);
    let y = sigma * b.hypot(d);
    (x <= 135.0 && y <= 135.0).then_some((x, y))
}

fn filter_group(
    children: Scene,
    layer: &LayerCommand,
    filter: &Filter,
    width: u32,
    height: u32,
) -> Option<(ImageBrush, peniko::kurbo::Vec2)> {
    // Retain offscreen source pixels within the blur support. Integer origins
    // preserve rasterization phase when replaying into an isolated image.
    let sigmas = blur_sigmas(filter, layer);
    let pad = sigmas.map_or(0.0, |(x, y)| x.max(y) * 3.0 + 2.0);
    let bounds = layer
        .transform
        .transform_rect_bbox(layer.clip.bounding_box())
        .expand()
        .intersect(Rect::new(0.0, 0.0, width as f64, height as f64).inflate(pad, pad));
    if bounds.width() <= 0.0 || bounds.height() <= 0.0 {
        return None;
    }
    let origin = bounds.origin().to_vec2();
    let width = bounds.width() as u32;
    let height = bounds.height() as u32;
    let mut rgba = render_to_buffer::<VelloCpuImageRenderer, _>(
        |painter| painter.append_scene(children, Affine::translate(-origin)),
        width,
        height,
    );
    if let Some((x, y)) = sigmas {
        crate::blur::apply(&mut rgba, width as usize, height as usize, x, y);
    } else {
        apply_color_chain(&mut rgba, filter);
    }
    Some((
        ImageBrush {
            image: ImageData {
                data: Blob::new(Arc::new(rgba)),
                format: ImageFormat::Rgba8,
                width,
                height,
                alpha_type: ImageAlphaType::AlphaPremultiplied,
            },
            sampler: ImageSampler {
                quality: ImageQuality::Low,
                ..Default::default()
            },
        },
        origin,
    ))
}

fn apply_color_chain(rgba: &mut [u8], filter: &Filter) {
    for pixel in rgba.as_chunks_mut::<4>().0 {
        let alpha = f64::from(pixel[3]) / 255.0;
        let mut color = [0.0, 0.0, 0.0, alpha];
        if alpha > 0.0 {
            for channel in 0..3 {
                color[channel] = f64::from(pixel[channel]) / (255.0 * alpha);
            }
        }
        for node in filter.nodes() {
            let FilterEffect::ColorMatrix(matrix) = &node.effect else {
                continue;
            };
            color = std::array::from_fn(|row| {
                let coefficients = &matrix.0[row * 5..row * 5 + 5];
                (color
                    .iter()
                    .zip(coefficients)
                    .map(|(c, m)| c * f64::from(*m))
                    .sum::<f64>()
                    + f64::from(coefficients[4]))
                .clamp(0.0, 1.0)
            });
        }
        for channel in 0..3 {
            pixel[channel] = (color[channel] * color[3] * 255.0).round() as u8;
        }
        pixel[3] = (color[3] * 255.0).round() as u8;
    }
}
