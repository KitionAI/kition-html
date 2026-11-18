use anyrender::PaintScene;
use blitz_dom::{BaseDocument, CopyStyleGetters, node::TextBrush};
use kurbo::{Affine, BezPath, Ellipse, Rect, Stroke};
use parley::GlyphRun;
use peniko::Fill;

use super::Color;

/// Geometric symbols use the principal font's baseline without drawing a
/// replacement glyph or allowing a fallback font to enlarge the line box.
pub(super) fn draw(
    scene: &mut impl PaintScene,
    run: &GlyphRun<'_, TextBrush>,
    doc: &BaseDocument,
    transform: Affine,
    scale: f64,
    color: Color,
    symbol: char,
) {
    let font_size = run.run().font_size() as f64 / scale;
    let ascent = (run.run().font_metrics().ascent as f64 / scale).round();
    let baseline = run.baseline() as f64 / scale;
    let mut x = run.offset() as f64 / scale;
    let style = doc
        .get_node(run.style().brush.id)
        .and_then(|n| n.primary_styles());
    let inside = style.as_ref().is_some_and(|s| {
        s.clone_list_style_position() == style::computed_values::list_style_position::T::Inside
    });
    let disclosure = matches!(symbol, '\u{25be}' | '\u{25b8}');
    if inside && !disclosure {
        x -= 1.0;
    }
    let transform = transform * Affine::scale(scale);
    if disclosure {
        let size = (font_size * 0.66 * 64.0).floor() / 64.0;
        let y = baseline - size;
        let rtl = style
            .as_ref()
            .is_some_and(|s| s.clone_direction() == style::computed_values::direction::T::Rtl);
        let points = if symbol == '\u{25be}' {
            [(0.0, 0.07), (0.5, 0.93), (1.0, 0.07)]
        } else if rtl {
            [(1.0, 0.0), (0.14, 0.5), (1.0, 1.0)]
        } else {
            [(0.0, 0.0), (0.86, 0.5), (0.0, 1.0)]
        };
        let mut path = BezPath::new();
        path.move_to((x + points[0].0 * size, y + points[0].1 * size));
        for point in &points[1..] {
            path.line_to((x + point.0 * size, y + point.1 * size));
        }
        path.close_path();
        scene.fill(Fill::NonZero, transform, color, None, &path);
    } else {
        let part = (ascent * 2.0 / 3.0).floor();
        let size = ((part + 1.0) / 2.0).floor();
        let y = baseline - ascent + (3.0 * (ascent - part) / 2.0).floor();
        let rect = Rect::new(
            (x + 1.0).round(),
            y.round(),
            (x + 1.0 + size).round(),
            (y + size).round(),
        );
        match symbol {
            '\u{2022}' => scene.fill(
                Fill::NonZero,
                transform,
                color,
                None,
                &Ellipse::from_rect(rect),
            ),
            '\u{25e6}' => scene.stroke(
                &Stroke::new(1.0),
                transform,
                color,
                None,
                &Ellipse::from_rect(rect),
            ),
            '\u{25aa}' => scene.fill(Fill::NonZero, transform, color, None, &rect),
            _ => {}
        }
    }
}
