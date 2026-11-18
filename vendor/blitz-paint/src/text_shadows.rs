use super::{Color, draw_glyph_run};
use crate::color::ToColorColor as _;
use anyrender::{
    PaintScene,
    filters::{Filter, FilterEffect},
};
use blitz_dom::{BaseDocument, CopyStyleGetters, node::TextBrush};
use kurbo::{Affine, Rect};
use parley::{GlyphRun, Line, PositionedLayoutItem};
use peniko::{Fill, Mix};
use std::sync::Arc;

#[derive(PartialEq)]
struct Shadow {
    x: f64,
    y: f64,
    sigma: f64,
    color: Color,
}

/// Adjacent runs with the same resolved shadows share one mask, so font and
/// inline style boundaries do not introduce seams or repeat alpha compositing.
pub(super) fn draw(
    scene: &mut impl PaintScene,
    lines: &[Line<'_, TextBrush>],
    doc: &BaseDocument,
    transform: Affine,
    scale: f64,
) {
    let mut groups: Vec<(Vec<Shadow>, Vec<GlyphRun<'_, TextBrush>>)> = Vec::new();
    for line in lines {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            let Some(style) = doc
                .get_node(run.style().brush.id)
                .and_then(|node| node.primary_styles())
            else {
                continue;
            };
            let current = style.clone_color();
            let shadows: Vec<_> = style
                .get_inherited_text()
                .text_shadow
                .0
                .iter()
                .map(|shadow| Shadow {
                    x: shadow.horizontal.px() as f64 * scale,
                    y: shadow.vertical.px() as f64 * scale,
                    sigma: shadow.blur.px() as f64 * scale * 0.5,
                    color: shadow.color.resolve_to_absolute(&current).as_srgb_color(),
                })
                .filter(|shadow| shadow.color.components[3] > 0.0)
                .collect();
            if shadows.is_empty() {
                continue;
            }
            if let Some((_, runs)) = groups
                .last_mut()
                .filter(|(previous, _)| *previous == shadows)
            {
                runs.push(run);
            } else {
                groups.push((shadows, vec![run]));
            }
        }
    }
    for (shadows, runs) in groups {
        for shadow in shadows.iter().rev() {
            let shifted = transform * Affine::translate((shadow.x, shadow.y));
            let mut bounds: Option<Rect> = None;
            for run in &runs {
                let fs = f64::from(run.run().font_size());
                let x = f64::from(run.offset());
                let y = f64::from(run.baseline());
                let rect = Rect::new(
                    x - fs,
                    y - fs * 2.0,
                    x + f64::from(run.advance()) + fs,
                    y + fs,
                );
                bounds = Some(bounds.map_or(rect, |bounds| bounds.union(rect)));
            }
            let Some(bounds) = bounds else {
                continue;
            };
            if shadow.sigma > 0.0 {
                scene.push_layer(
                    Fill::NonZero,
                    Mix::Normal,
                    1.0,
                    shifted,
                    &bounds.inflate(shadow.sigma * 4.0, shadow.sigma * 4.0),
                    Some(Arc::new(Filter::linear_list(std::iter::once(
                        FilterEffect::blur(shadow.sigma as f32),
                    )))),
                    None,
                );
            }
            for run in &runs {
                draw_glyph_run(scene, run, doc, (shifted, transform), scale, shadow.color);
            }
            if shadow.sigma > 0.0 {
                scene.pop_layer();
            }
        }
    }
}
