//! Preserve fractional text origins independently of snapped box edges.
use super::{BlitzDomPainter, ElementCx};
use blitz_dom::Node;
use kurbo::{Affine, Vec2};

impl BlitzDomPainter<'_, '_> {
    pub(super) fn record_text_origin(&self, node: &Node, parent_transform: Affine) {
        if self.column_depth.get() > 0 {
            return;
        }
        let (_, precise) = self.dom.physical_unrounded_geometry(node.id);
        let rounded = node.final_layout();
        let local = Vec2::new(
            f64::from(precise.location.x - rounded.location.x) * self.scale,
            f64::from(precise.location.y - rounded.location.y) * self.scale,
        );
        let [a, b, c, d, _, _] = parent_transform.as_coeffs();
        let mut offsets = self.text_origin_offsets.borrow_mut();
        let inherited = node
            .containing_block()
            .and_then(|parent| offsets.get(&parent).copied())
            .unwrap_or(Vec2::ZERO);
        offsets.insert(
            node.id,
            inherited + Vec2::new(a * local.x + c * local.y, b * local.x + d * local.y),
        );
    }
}

impl ElementCx<'_, '_> {
    pub(super) fn text_content_transform(&self) -> Affine {
        // Column fragments already retain fractional origins and padding.
        if self.context.column_depth.get() > 0 {
            return self.transform;
        }
        let (_, precise) = self.context.dom.physical_unrounded_geometry(self.node.id);
        let rounded = self.node.final_layout();
        let x = f64::from(
            precise.border.left + precise.padding.left - rounded.border.left - rounded.padding.left,
        ) * self.scale;
        let y = f64::from(
            precise.border.top + precise.padding.top - rounded.border.top - rounded.padding.top,
        ) * self.scale;
        let offset = self
            .context
            .text_origin_offsets
            .borrow()
            .get(&self.node.id)
            .copied()
            .unwrap_or(Vec2::ZERO);
        Affine::translate(offset) * self.transform * Affine::translate((x, y))
    }
}
