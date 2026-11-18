//! Collapsed select label and native disclosure chevron.
use super::ElementCx;
use crate::color::ToColorColor;
use anyrender::PaintScene;
use blitz_dom::node::SpecialElementData;
use kurbo::{Affine, BezPath, Rect, Stroke};
use peniko::Fill;

impl ElementCx<'_, '_> {
    pub(super) fn draw_select(&self, scene: &mut impl PaintScene) {
        let SpecialElementData::Select(control) = &self.element.special_data else {
            return;
        };
        let content = self.frame.content_box;
        let left = if control.native {
            if control.rtl { 0. } else { 4. }
        } else {
            0.
        } * self.scale;
        let right = if control.native {
            if control.rtl { 4. } else { 0. }
        } else {
            0.
        } * self.scale;
        let clip = Rect::new(
            content.x0 + left,
            content.y0,
            (content.x1 - right).max(content.x0 + left),
            content.y1,
        );
        if clip.width() > 0. && clip.height() > 0. {
            let layout = &control.label.layout;
            let x = if control.rtl {
                clip.x1 - layout.full_width() as f64
            } else {
                clip.x0
            };
            let y = content.y0 + (content.height() - layout.height() as f64) / 2.;
            let transform = self.transform * Affine::translate((x, y));
            scene.push_clip_layer(Fill::NonZero, self.transform, &clip);
            crate::text::stroke_text(
                scene,
                layout.lines(),
                self.context.dom,
                transform,
                self.scale,
                self.node.id,
                &mut self.context.draw_text_context.borrow_mut(),
            );
            scene.pop_layer();
        }
        if control.native {
            let border = self.frame.border_box;
            let layout = self.node.final_layout();
            let x = if control.rtl {
                border.x0 + layout.border.left as f64 * self.scale + 4. * self.scale
            } else {
                border.x1 - layout.border.right as f64 * self.scale - 12. * self.scale
            };
            let y =
                (border.y0 / self.scale + border.height() / self.scale / 2.).floor() * self.scale;
            let mut arrow = BezPath::new();
            arrow.move_to((x, y - 2. * self.scale));
            arrow.line_to((x + 4. * self.scale, y + 2. * self.scale));
            arrow.line_to((x + 8. * self.scale, y - 2. * self.scale));
            scene.push_clip_layer(Fill::NonZero, self.transform, &border);
            scene.stroke(
                &Stroke::new(2. * self.scale),
                self.transform,
                self.style.get_inherited_text().color.as_srgb_color(),
                None,
                &arrow,
            );
            scene.pop_layer();
        }
    }
}
