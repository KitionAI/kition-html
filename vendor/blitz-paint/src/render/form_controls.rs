use super::ElementCx;
use crate::color::{Color, ToColorColor as _, contrast_ratio};
use anyrender::PaintScene;
use blitz_dom::CopyStyleGetters;
use kurbo::{BezPath, Cap, Join, Rect, Stroke};
use peniko::Fill;
use style::{
    dom::TElement as _,
    values::{computed::Appearance, generics::color::ColorOrAuto},
};

impl ElementCx<'_, '_> {
    pub(super) fn draw_input(&self, scene: &mut impl PaintScene) {
        if self.node.local_name() != "input" || self.style.get_box().appearance == Appearance::None
        {
            return;
        }
        let Some(checked) = self.element.checkbox_input_checked() else {
            return;
        };
        let is_radio = self.element.input_type() == "radio";
        let disabled = self.node.is_effectively_disabled();
        let indeterminate = !is_radio && self.node.is_indeterminate();
        let checked = checked || indeterminate;
        let accent = if disabled {
            Color::from_rgba8(209, 209, 209, 255)
        } else {
            match &self.style.get_inherited_ui().accent_color {
                ColorOrAuto::Auto => Color::from_rgba8(0, 117, 255, 255),
                ColorOrAuto::Color(color) => color
                    .resolve_to_absolute(&self.style.clone_color())
                    .as_srgb_color(),
            }
        };
        let dark = Color::from_rgba8(59, 59, 59, 255);
        let mark = if disabled {
            Color::from_rgba8(237, 237, 237, 255)
        } else if checked && contrast_ratio(accent, dark) > contrast_ratio(accent, Color::WHITE) {
            dark
        } else {
            Color::WHITE
        };
        let background = if disabled {
            Color::from_rgba8(248, 248, 248, 255)
        } else {
            mark
        };
        let border = if checked {
            accent
        } else if disabled {
            Color::from_rgba8(209, 209, 209, 255)
        } else {
            Color::from_rgba8(118, 118, 118, 255)
        };
        let box_rect = self.frame.border_box;
        let side = box_rect.width().min(box_rect.height());
        if side <= 0.0 {
            return;
        }
        let center = box_rect.center();
        let rect = Rect::new(
            center.x - side / 2.0,
            center.y - side / 2.0,
            center.x + side / 2.0,
            center.y + side / 2.0,
        );
        let radius = if is_radio {
            side / 2.0
        } else {
            2.0 * self.scale
        };
        if side <= 2.0 * self.scale {
            scene.fill(Fill::NonZero, self.transform, border, None, &rect);
            return;
        }
        if is_radio || !checked {
            // Keep the background just inside the stroke's antialiased exterior.
            scene.fill(
                Fill::NonZero,
                self.transform,
                background,
                None,
                &rect
                    .inflate(-0.2 * self.scale, -0.2 * self.scale)
                    .to_rounded_rect(radius),
            );
            scene.stroke(
                &Stroke::new(self.scale),
                self.transform,
                border,
                None,
                &rect
                    .inflate(-0.5 * self.scale, -0.5 * self.scale)
                    .to_rounded_rect(radius),
            );
        }
        if !checked {
            return;
        }
        if is_radio {
            scene.fill(
                Fill::NonZero,
                self.transform,
                accent,
                None,
                &rect
                    .inflate(-side * 0.2, -side * 0.2)
                    .to_rounded_rect(radius),
            );
        } else {
            scene.fill(
                Fill::NonZero,
                self.transform,
                accent,
                None,
                &rect.to_rounded_rect(radius),
            );
            if indeterminate {
                let half_width = side * 0.25;
                scene.fill(
                    Fill::NonZero,
                    self.transform,
                    mark,
                    None,
                    &Rect::new(
                        center.x - half_width,
                        center.y - self.scale,
                        center.x + half_width,
                        center.y + self.scale,
                    ),
                );
                return;
            }
            let mut tick = BezPath::new();
            tick.move_to((rect.x0 + side * 0.2, rect.y0 + side * 0.5));
            tick.line_to((rect.x0 + side * 0.4, rect.y0 + side * 0.7));
            tick.line_to((rect.x0 + side * 0.8, rect.y0 + side * 0.2));
            scene.stroke(
                &Stroke::new(side * 0.16)
                    .with_caps(Cap::Butt)
                    .with_join(Join::Miter),
                self.transform,
                mark,
                None,
                &tick,
            );
        }
    }
}
