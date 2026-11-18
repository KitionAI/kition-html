//! Container size snapshots bridge Stylo's style pass and Taffy's layout pass.
use app_units::Au;
use euclid::default::Size2D;
use style::values::specified::{
    ContainerType, Display,
    box_::{DisplayInside, DisplayOutside},
};

use crate::{BaseDocument, node::Node};

pub(crate) fn query_size(node: &Node, display: &Display) -> Size2D<Option<Au>> {
    if !supports_size_containment(display) {
        return Size2D::new(None, None);
    }
    let Some(style) = node.primary_styles() else {
        return Size2D::new(None, None);
    };
    let kind = style.get_box().container_type;
    if !kind.is_size_container_type() {
        return Size2D::new(None, None);
    }
    let layout = node.unrounded_layout();
    let width = (layout.size.width
        - layout.border.left
        - layout.border.right
        - layout.padding.left
        - layout.padding.right
        - layout.scrollbar_size.width)
        .max(0.0);
    let height = (layout.size.height
        - layout.border.top
        - layout.border.bottom
        - layout.padding.top
        - layout.padding.bottom
        - layout.scrollbar_size.height)
        .max(0.0);
    // Taffy stores the unrounded box in its placing algorithm's writing mode.
    #[cfg(feature = "writing-mode")]
    let (width, height) = if node
        .containing_block()
        .map(|id| node.with(id).layout_data().writing_mode.is_vertical())
        .unwrap_or_else(|| node.layout_data().writing_mode.is_vertical())
    {
        (height, width)
    } else {
        (width, height)
    };
    let both = kind.contains(ContainerType::SIZE);
    let horizontal = style.writing_mode.is_horizontal();
    Size2D::new(
        (both || horizontal).then(|| Au::from_f32_px(width)),
        (both || !horizontal).then(|| Au::from_f32_px(height)),
    )
}

impl BaseDocument {
    pub(crate) fn container_size_snapshot(
        &self,
    ) -> Vec<(blitz_traits::node_id::NodeId, Size2D<Option<Au>>)> {
        self.nodes
            .iter()
            .filter_map(|(id, node)| {
                let style = node.primary_styles()?;
                if !style.get_box().container_type.is_size_container_type() {
                    return None;
                }
                Some((id, query_size(node, &style.get_box().display)))
            })
            .collect()
    }

    /// Whether the most recent resolve hit the bounded style/layout dependency iteration limit.
    pub fn layout_dependencies_unstable(&self) -> bool {
        self.layout_dependencies_unstable
    }
}

impl super::LayoutPassState<'_> {
    /// Size containment measures the box as if it had no content, then lays out
    /// descendants normally within the resulting contained dimensions.
    pub(crate) fn contained_layout_inputs(
        &self,
        node_id: taffy::NodeId,
        mut inputs: taffy::LayoutInput,
    ) -> taffy::LayoutInput {
        use taffy::LayoutPartialTree;
        if inputs.run_mode == taffy::RunMode::PerformHiddenLayout {
            return inputs;
        }
        let node = &self.nodes[crate::dom_node_id(node_id)];
        let Some(computed) = node.primary_styles() else {
            return inputs;
        };
        let kind = computed.get_box().container_type;
        let display = computed.get_box().display;
        if !kind.is_size_container_type() || !supports_size_containment(&display) {
            return inputs;
        }
        let style = self.get_core_container_style(node_id);
        let empty =
            taffy::compute_leaf_layout(inputs, &style, super::resolve_calc_value, |_, _| {
                taffy::Size::ZERO
            });
        inputs.known_dimensions.width = Some(empty.size.width);
        inputs.known_dimensions_are_definite.width = true;
        if kind.contains(ContainerType::SIZE) {
            inputs.known_dimensions.height = Some(empty.size.height);
            inputs.known_dimensions_are_definite.height = true;
        }
        inputs
    }
}

fn supports_size_containment(display: &Display) -> bool {
    !display.is_none()
        && !display.is_contents()
        && display.inside() != DisplayInside::Table
        && display.outside() != DisplayOutside::InternalTable
        && !(display.outside() == DisplayOutside::Inline && display.inside() == DisplayInside::Flow)
}
