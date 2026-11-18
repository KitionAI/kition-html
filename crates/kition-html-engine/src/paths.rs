//! Flatten non-glyph paths in device-pixel units before backend replay.
use anyrender::{
    Scene,
    recording::{FillCommand, RenderCommand},
};
use peniko::{
    Fill,
    kurbo::{self, Affine, BezPath, PathEl, Shape, StrokeOpts},
};

pub fn prepare(scene: &mut Scene) {
    for command in &mut scene.commands {
        match command {
            RenderCommand::Fill(fill) => refine(&mut fill.shape, fill.transform),
            RenderCommand::PushLayer(layer) => refine(&mut layer.clip, layer.transform),
            RenderCommand::PushClipLayer(layer) => refine(&mut layer.clip, layer.transform),
            RenderCommand::Stroke(stroke) => {
                let Some(tolerance) = tolerance(&stroke.shape, stroke.transform) else {
                    continue;
                };
                let mut shape = kurbo::stroke(
                    stroke.shape.elements().iter().copied(),
                    &stroke.style,
                    &StrokeOpts::default(),
                    tolerance,
                );
                refine(&mut shape, stroke.transform);
                *command = RenderCommand::Fill(FillCommand {
                    fill: Fill::NonZero,
                    transform: stroke.transform,
                    brush: stroke.brush.clone(),
                    brush_transform: stroke.brush_transform,
                    shape,
                });
            }
            _ => {}
        }
    }
}

fn tolerance(path: &BezPath, transform: Affine) -> Option<f64> {
    let bounds = transform.transform_rect_bbox(path.bounding_box());
    if !bounds.is_finite()
        || bounds.width() > 1_000_000.0
        || bounds.height() > 1_000_000.0
        || path.elements().len() > 100_000
    {
        return None;
    }
    let [a, b, c, d, _, _] = transform.as_coeffs();
    let squared = a * a + b * b + c * c + d * d;
    let determinant = a * d - b * c;
    let scale = ((squared
        + (squared * squared - 4.0 * determinant * determinant)
            .max(0.0)
            .sqrt())
        * 0.5)
        .sqrt();
    (scale.is_finite() && scale > 0.0).then_some(0.0625 / scale)
}

fn refine(path: &mut BezPath, transform: Affine) {
    if !path
        .elements()
        .iter()
        .any(|el| matches!(el, PathEl::QuadTo(..) | PathEl::CurveTo(..)))
    {
        return;
    }
    let Some(tolerance) = tolerance(path, transform) else {
        return;
    };
    let mut flat = BezPath::new();
    kurbo::flatten(path.elements().iter().copied(), tolerance, |el| {
        flat.push(el)
    });
    *path = flat;
}
