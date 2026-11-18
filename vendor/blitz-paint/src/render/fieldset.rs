//! Fieldset decoration is below the legend, with a gap in its top border.
use super::ElementCx;
use crate::kurbo_css::CssBox;
use anyrender::PaintScene;
use kurbo::{Rect, Shape};
use peniko::Fill;

impl ElementCx<'_, '_> {
    pub(super) fn fieldset_decorations(&self) -> Self {
        let mut result = self.clone();
        if let Some(layout) = &self.element.fieldset_layout {
            let mut border_box = self.frame.border_box;
            border_box.y0 += f64::from(layout.border_top_offset) * self.scale;
            let mut padding = self.frame.padding_width;
            padding.y0 +=
                f64::from(layout.content_top_extra - layout.border_top_offset) * self.scale;
            result.frame = CssBox::new(
                border_box,
                self.frame.border_width,
                padding,
                self.frame.outline_width,
                self.frame.border_radii,
            );
        }
        result
    }

    pub(crate) fn draw_border(&self, scene: &mut impl PaintScene) {
        let legend = self
            .element
            .fieldset_layout
            .as_ref()
            .and_then(|layout| self.context.dom.get_node(layout.legend));
        if let Some(legend) = legend {
            let layout = legend.final_layout();
            let gap = Rect::new(
                f64::from(layout.location.x) * self.scale,
                0.0,
                f64::from(layout.location.x + layout.size.width) * self.scale,
                (f64::from(layout.location.y + layout.size.height) * self.scale)
                    .max(self.frame.border_box.y0 + self.frame.border_width.y0),
            );
            let mut clip = self.frame.border_box.inflate(10000.0, 10000.0).to_path(0.1);
            clip.extend(gap.path_elements(0.1));
            scene.push_clip_layer(Fill::EvenOdd, self.transform, &clip);
            self.draw_border_unclipped(scene);
            scene.pop_layer();
        } else {
            self.draw_border_unclipped(scene);
        }
    }
}
