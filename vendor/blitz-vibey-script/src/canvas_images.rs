//! Canvas image sources are snapshots of engine-owned data.
use crate::canvas::{dimension, image_brush};
use crate::dom::this_node_id;
use crate::state::DomCtx;
use anyrender::PaintScene;
use blitz_dom::{
    local_name,
    node::{ImageData, RasterImageData},
};
use boa_engine::{JsNativeError, JsResult, JsValue};
use peniko::{
    Extend, Fill, ImageQuality,
    kurbo::{Affine, Rect},
};
use serde_json::Value;
use std::sync::Arc;

pub(crate) fn source(ctx: &DomCtx, value: &JsValue) -> JsResult<Option<ImageData>> {
    let id = this_node_id(value)?;
    let doc = ctx.doc.borrow();
    let element = doc
        .get_node(id)
        .and_then(|node| node.element_data())
        .ok_or_else(|| JsNativeError::typ().with_message("Invalid image source"))?;
    if element.name.local == local_name!("img") {
        return Ok(element
            .image_data()
            .filter(|image| !matches!(image, ImageData::None))
            .cloned());
    }
    if element.name.local != local_name!("canvas") {
        return Err(JsNativeError::typ()
            .with_message("Expected an image or canvas")
            .into());
    }
    let width = dimension(element.attr(local_name!("width")), 300);
    let height = dimension(element.attr(local_name!("height")), 150);
    if width > 8192 || height > 8192 || u64::from(width) * u64::from(height) > 16_777_216 {
        return Err(JsNativeError::range()
            .with_message("Source canvas exceeds the preview bitmap limit")
            .into());
    }
    Ok(Some(ImageData::Raster(
        element.canvas_bitmap.clone().unwrap_or_else(|| {
            RasterImageData::new(
                width,
                height,
                Arc::new(vec![0; width as usize * height as usize * 4]),
            )
        }),
    )))
}

pub(crate) fn geometry(
    source: &ImageData,
    request: &Value,
    transform: Affine,
) -> Option<(Rect, Affine)> {
    let (width, height) = match source {
        ImageData::Raster(image) => (image.width as f64, image.height as f64),
        ImageData::Svg(image) => (
            image.tree.size().width() as f64,
            image.tree.size().height() as f64,
        ),
        ImageData::None => return None,
    };
    let values = request["args"].as_array()?;
    let n = |index: usize| values.get(index).and_then(Value::as_f64).unwrap_or(0.0);
    let (mut sx, mut sy, mut sw, mut sh, mut dx, mut dy, mut dw, mut dh) = match values.len() {
        2 => (0., 0., width, height, n(0), n(1), width, height),
        4 => (0., 0., width, height, n(0), n(1), n(2), n(3)),
        8 => (n(0), n(1), n(2), n(3), n(4), n(5), n(6), n(7)),
        _ => return None,
    };
    // Negative dimensions grow in the opposite direction without flipping pixels.
    if sw < 0. {
        sx += sw;
        sw = -sw;
    }
    if sh < 0. {
        sy += sh;
        sh = -sh;
    }
    if dw < 0. {
        dx += dw;
        dw = -dw;
    }
    if dh < 0. {
        dy += dh;
        dh = -dh;
    }
    if sw == 0. || sh == 0. || dw == 0. || dh == 0. || width == 0. || height == 0. {
        return None;
    }
    let source_rect = Rect::new(
        sx.max(0.),
        sy.max(0.),
        (sx + sw).min(width),
        (sy + sh).min(height),
    );
    if source_rect.width() <= 0. || source_rect.height() <= 0. {
        return None;
    }
    let image_transform = transform
        * Affine::translate((dx, dy))
        * Affine::scale_non_uniform(dw / sw, dh / sh)
        * Affine::translate((-sx, -sy));
    Some((source_rect, image_transform))
}

pub(crate) fn draw(
    scene: &mut impl PaintScene,
    source: &ImageData,
    request: &Value,
    (source_rect, image_transform): (Rect, Affine),
) {
    scene.push_clip_layer(Fill::NonZero, image_transform, &source_rect);
    match source {
        ImageData::Raster(image) => {
            let mut brush = image_brush(image);
            brush.sampler.x_extend = Extend::Pad;
            brush.sampler.y_extend = Extend::Pad;
            brush.sampler.quality = if request["state"]["imageSmoothingEnabled"] == false {
                ImageQuality::Low
            } else {
                ImageQuality::Medium
            };
            scene.draw_image(brush.as_ref(), image_transform);
        }
        ImageData::Svg(image) => {
            anyrender_svg::render_svg_tree(scene, &image.tree, image_transform)
        }
        ImageData::None => {}
    }
    scene.pop_layer();
}
