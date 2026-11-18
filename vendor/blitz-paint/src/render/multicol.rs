//! Paint a laid-out flow through the column fragmentainers.
use super::*;
use style::values::computed::BorderStyle;

impl ElementCx<'_, '_> {
    pub(super) fn draw_columns(
        &mut self,
        scene: &mut impl PaintScene,
        content_position: Point,
        clip_rect: Rect,
    ) {
        let Some(columns) = self.element.multicol_layout.as_ref() else {
            return;
        };
        let columns = columns.clone();
        let transform = self.transform;
        let content = self.frame.content_box;
        let depth = self.context.column_depth.get();
        if depth >= 8 {
            self.context.diagnostics.borrow_mut().push(
                "html.unsupported-column-depth: nested column painting is limited to eight levels"
                    .into(),
            );
            return;
        }
        if columns.limited {
            self.context.diagnostics.borrow_mut().push("html.column-fragment-limit: content exceeded the 1024-fragment budget; remaining content stays in the last fragment".into());
        }
        self.context.column_depth.set(depth + 1);
        for (index, fragment) in columns.fragments.iter().enumerate() {
            let remaining = self.context.column_fragments_remaining.get();
            if remaining == 0 {
                self.context.diagnostics.borrow_mut().push(
                    "html.column-paint-limit: the document exceeded the 4096-fragment paint budget"
                        .into(),
                );
                break;
            }
            self.context.column_fragments_remaining.set(remaining - 1);
            let advance =
                (f64::from(columns.advance) * index as f64 * 64.0).round() / 64.0 * self.scale;
            let x = if columns.rtl {
                content.width() - f64::from(columns.width) * self.scale - advance
            } else {
                advance
            };
            let height = f64::from(fragment.end - fragment.start) * self.scale;
            if height <= 0.0 {
                continue;
            }
            let gutter = f64::from(columns.gap) * self.scale * 0.5;
            let clip = Rect::new(
                content.x0 + x - gutter,
                content.y0,
                content.x0 + x + f64::from(columns.width) * self.scale + gutter,
                content.y0 + height,
            );
            scene.push_clip_layer(Fill::NonZero, transform, &clip);
            self.transform =
                transform.pre_translate(Vec2::new(x, -f64::from(fragment.start) * self.scale));
            self.draw_flow_content(scene, content_position, clip_rect, None);
            scene.pop_layer();
        }
        self.transform = transform;
        self.context.column_depth.set(depth);
        let style = self.style.get_column();
        let thickness = style.column_rule_width.0.to_f64_px() * self.scale;
        if style.column_rule_style.none_or_hidden() || thickness <= 0.0 {
            return;
        }
        let color = style
            .column_rule_color
            .resolve_to_absolute(&self.style.clone_color())
            .as_srgb_color();
        for index in 1..columns.fragments.len() {
            let edge = f64::from(columns.advance) * index as f64 - f64::from(columns.gap) * 0.5;
            let x = if columns.rtl {
                content.x1 - edge * self.scale
            } else {
                content.x0 + edge * self.scale
            };
            let mut rect = Rect::new(
                x - thickness * 0.5,
                content.y0,
                x + thickness * 0.5,
                content.y1,
            );
            let [a, b, c, d, tx, ty] = transform.as_coeffs();
            if a == 1.0 && b == 0.0 && c == 0.0 && d == 1.0 {
                rect = Rect::new(
                    ((tx + rect.x0) / self.scale).round() * self.scale - tx,
                    ((ty + rect.y0) / self.scale).round() * self.scale - ty,
                    ((tx + rect.x1) / self.scale).round() * self.scale - tx,
                    ((ty + rect.y1) / self.scale).round() * self.scale - ty,
                );
            }
            match style.column_rule_style {
                BorderStyle::Solid => scene.fill(Fill::NonZero, transform, color, None, &rect),
                BorderStyle::Double => {
                    let third = (thickness / 3.0).round().max(1.0);
                    scene.fill(
                        Fill::NonZero,
                        transform,
                        color,
                        None,
                        &Rect::new(rect.x0, rect.y0, rect.x0 + third, rect.y1),
                    );
                    scene.fill(
                        Fill::NonZero,
                        transform,
                        color,
                        None,
                        &Rect::new(rect.x1 - third, rect.y0, rect.x1, rect.y1),
                    );
                }
                BorderStyle::Dashed | BorderStyle::Dotted => {
                    let path = super::line_pattern::straight_path(
                        rect,
                        true,
                        style.column_rule_style == BorderStyle::Dotted,
                        self.scale,
                    );
                    scene.fill(Fill::NonZero, transform, color, None, &path);
                }
                _ => {
                    self.context.diagnostics.borrow_mut().push("html.unsupported-column-rule: this three-dimensional rule style is painted as a solid rule".into());
                    scene.fill(Fill::NonZero, transform, color, None, &rect);
                }
            }
        }
    }
}
