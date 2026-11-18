// Copyright 2015 The Chromium Authors
// Adapted box-side rounding: see ../LICENSE.chromium.
//! CSS outline styles painted outside the offset border edge.
use super::*;
use crate::kurbo_css::Edge;
use kurbo::Insets;
use style::values::computed::{BorderStyle, OutlineStyle};

impl ElementCx<'_, '_> {
    pub(crate) fn draw_outline(&self, scene: &mut impl PaintScene) {
        let outline = self.style.get_outline();
        let style = match outline.outline_style {
            OutlineStyle::Auto => return,
            OutlineStyle::BorderStyle(style) => style,
        };
        if style.none_or_hidden() || self.frame.outline_width <= 0.0 {
            return;
        }
        let color = outline
            .outline_color
            .resolve_to_absolute(&self.style.clone_color())
            .as_srgb_color();
        if color.components[3] <= 0.0 {
            return;
        }
        let shifted = self
            .frame
            .with_outline_offset(outline.outline_offset.to_f64_px() * self.scale);
        if style == BorderStyle::Solid {
            scene.fill(
                Fill::NonZero,
                self.transform,
                color,
                None,
                &shifted.outline(),
            );
            return;
        }
        let width = shifted.outline_width;
        let mut radii = shifted.border_radii;
        for radius in [
            &mut radii.top_left,
            &mut radii.top_right,
            &mut radii.bottom_left,
            &mut radii.bottom_right,
        ] {
            if radius.x > 0.0 {
                radius.x += width;
            }
            if radius.y > 0.0 {
                radius.y += width;
            }
        }
        let cx = Self {
            frame: CssBox::new(
                shifted.outline_box,
                Insets::uniform(width),
                Insets::ZERO,
                0.0,
                radii,
            ),
            ..self.clone()
        };
        let edges = [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left];
        if matches!(style, BorderStyle::Dotted | BorderStyle::Dashed) {
            if !cx.frame.has_border_radius()
                && (style == BorderStyle::Dashed || width <= 3.0 * self.scale)
            {
                let mut path = kurbo::BezPath::new();
                for edge in edges {
                    path.extend(
                        cx.straight_border_pattern(edge, style == BorderStyle::Dotted)
                            .elements()
                            .iter()
                            .copied(),
                    );
                }
                scene.fill(Fill::NonZero, self.transform, color, None, &path);
            } else {
                for edge in edges {
                    if style == BorderStyle::Dotted {
                        cx.draw_dotted_border_edge(scene, edge, color);
                    } else {
                        cx.draw_dashed_border_edge(scene, edge, color);
                    }
                }
            }
            return;
        }
        let mut fills = Vec::new();
        for edge in edges {
            match style {
                BorderStyle::Double => {
                    let mut path = kurbo::BezPath::new();
                    if width < 3.0 * self.scale {
                        path = cx.frame.border_edge_shape(edge);
                    } else {
                        let stripe = (width / self.scale / 3.0).round() * self.scale / width;
                        path.extend(&cx.frame.border_slice(0.0, stripe).border_edge_shape(edge));
                        path.extend(
                            &cx.frame
                                .border_slice(1.0 - stripe, 1.0)
                                .border_edge_shape(edge),
                        );
                    }
                    fills.push((color, path));
                }
                BorderStyle::Inset | BorderStyle::Outset => {
                    fills.push((
                        border::beveled_edge_color(color, edge, style == BorderStyle::Inset),
                        cx.frame.border_edge_shape(edge),
                    ));
                }
                BorderStyle::Groove | BorderStyle::Ridge => {
                    let (outer, inner) =
                        border::grooved_edge_colors(color, edge, style == BorderStyle::Ridge);
                    let half = width / self.scale * 0.5;
                    let middle = if matches!(edge, Edge::Top | Edge::Left) {
                        half.ceil()
                    } else {
                        half.floor()
                    } * self.scale
                        / width;
                    fills.push((
                        outer,
                        cx.frame.border_slice(0.0, middle).border_edge_shape(edge),
                    ));
                    fills.push((
                        inner,
                        cx.frame.border_slice(middle, 1.0).border_edge_shape(edge),
                    ));
                }
                _ => {}
            }
        }
        // Merge equal shades so their miter seams receive coverage only once.
        while let Some((color, mut path)) = fills.pop() {
            let mut index = 0;
            while index < fills.len() {
                if fills[index].0 == color {
                    path.extend(&fills.swap_remove(index).1);
                } else {
                    index += 1;
                }
            }
            scene.fill(Fill::NonZero, self.transform, color, None, &path);
        }
    }
}
