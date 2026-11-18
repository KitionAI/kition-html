//! Fieldsets reserve the top border area for their first rendered legend.
use super::{LayoutPassState, resolve_calc_value};
use crate::node::FieldsetLayout;
use crate::{Node, NodeId, dom_node_id, local_name, taffy_node_id};
use style::{
    servo_arc::Arc,
    values::{
        computed::{
            CSSPixelLength, LengthPercentage, MaxSize, Size as CssSize, border::BorderSideWidth,
        },
        generics::NonNegative,
    },
};
use taffy::{
    AvailableSpace, BoxSizing, CoreStyle, Layout, LayoutInput, LayoutOutput, LayoutPartialTree,
    Line, MaybeResolve, Point, ResolveOrZero, RunMode, Size, SizingMode,
};

pub(crate) fn legend(node: &Node) -> Option<NodeId> {
    let element = node.element_data()?;
    if element.name.local != local_name!("fieldset")
        || element.fieldset_layout_style.is_some()
        || !node
            .primary_styles()
            .is_some_and(|s| s.writing_mode.is_horizontal())
    {
        return None;
    }
    node.children
        .iter()
        .copied()
        .find(|&id| {
            let child = node.with(id);
            child.data.is_element_with_tag_name(&local_name!("legend"))
                && !child.is_out_of_flow()
                && !matches!(child.taffy_display(), taffy::Display::None)
                && child
                    .primary_styles()
                    .is_some_and(|s| s.get_box().float == style::values::computed::Float::None)
        })
        .filter(|id| {
            node.layout_children
                .borrow()
                .as_ref()
                .is_some_and(|children| children.contains(id))
        })
}
fn length(value: f32) -> CssSize {
    CssSize::LengthPercentage(NonNegative(LengthPercentage::new_length(
        CSSPixelLength::new(value.max(0.0)),
    )))
}

pub(crate) fn compute(
    tree: &mut LayoutPassState<'_>,
    id: taffy::NodeId,
    legend: NodeId,
    inputs: LayoutInput,
) -> LayoutOutput {
    let dom_id = dom_node_id(id);
    let original = tree.nodes[dom_id].primary_styles().unwrap().clone();
    tree.nodes[dom_id]
        .element_data_mut()
        .unwrap()
        .fieldset_layout_style = Some(original.clone());
    // This provisional pass resolves the outer width and definite height through
    // the same sizing code as ordinary block, flex and grid containers.
    let outer = tree.dispatch_child_layout(id, inputs, None);
    let style = tree.nodes[dom_id].layout_style();
    let padding = style
        .padding()
        .resolve_or_zero(inputs.parent_size.width, resolve_calc_value);
    let border = style
        .border()
        .resolve_or_zero(inputs.parent_size.width, resolve_calc_value);
    let content_width =
        (outer.size.width - padding.horizontal_axis_sum() - border.horizontal_axis_sum()).max(0.0);
    let definite_height = inputs.known_dimensions.height.is_some()
        || style
            .size()
            .height
            .maybe_resolve(inputs.parent_size.height, resolve_calc_value)
            .is_some();
    let min_height = style
        .min_size()
        .height
        .maybe_resolve(inputs.parent_size.height, resolve_calc_value);
    let max_height = style
        .max_size()
        .height
        .maybe_resolve(inputs.parent_size.height, resolve_calc_value);
    let content_box = style.box_sizing() == BoxSizing::ContentBox;
    drop(style);
    let legend_style = tree.nodes[legend].layout_style();
    let margin_style = legend_style.margin();
    let mut margin = margin_style.resolve_or_zero(Some(content_width), resolve_calc_value);
    margin.top = 0.0;
    let legend_padding = legend_style
        .padding()
        .resolve_or_zero(Some(content_width), resolve_calc_value);
    let legend_border = legend_style
        .border()
        .resolve_or_zero(Some(content_width), resolve_calc_value);
    let auto_width = legend_style.size().width.is_auto();
    drop(legend_style);
    let mut legend_inputs = LayoutInput {
        known_dimensions: Size::NONE,
        known_dimensions_are_definite: Size {
            width: false,
            height: false,
        },
        parent_size: Size {
            width: Some(content_width),
            height: None,
        },
        available_space: Size {
            width: AvailableSpace::MaxContent,
            height: AvailableSpace::MaxContent,
        },
        vertical_margins_are_collapsible: Line::FALSE,
        sizing_mode: SizingMode::InherentSize,
        ..inputs
    };
    let mut measured = tree.compute_child_layout(taffy_node_id(legend), legend_inputs);
    if auto_width {
        legend_inputs.known_dimensions.width = Some(
            measured
                .size
                .width
                .min((content_width - margin.left - margin.right).max(0.0)),
        );
        legend_inputs.available_space.width = AvailableSpace::Definite(content_width);
        measured = tree.compute_child_layout(taffy_node_id(legend), legend_inputs);
    }
    let available = (content_width - measured.size.width - margin.left - margin.right).max(0.0);
    let auto_margins =
        u8::from(margin_style.left.is_auto()) + u8::from(margin_style.right.is_auto());
    if auto_margins > 0 {
        let share = available / f32::from(auto_margins);
        if margin_style.left.is_auto() {
            margin.left = share;
        }
        if margin_style.right.is_auto() {
            margin.right = share;
        }
    }
    let legend_y = ((border.top - measured.size.height) / 2.0).max(0.0);
    let top = (measured.size.height + margin.bottom).max(border.top);
    let extra = top - border.top;
    let mut amended = (*original).clone();
    amended.mutate_border().border_top_width = BorderSideWidth(app_units::Au::from_f32_px(top));
    if content_box {
        let position = amended.mutate_position();
        if let Some(min) = min_height {
            position.min_height = length(min - extra);
        }
        if let Some(max) = max_height {
            position.max_height = MaxSize::LengthPercentage(NonNegative(
                LengthPercentage::new_length(CSSPixelLength::new((max - extra).max(0.0))),
            ));
        }
    }
    tree.nodes[dom_id]
        .element_data_mut()
        .unwrap()
        .fieldset_layout_style = Some(Arc::new(amended));
    let children = tree.nodes[dom_id].layout_children.borrow().clone();
    tree.nodes[dom_id]
        .layout_children
        .borrow_mut()
        .as_mut()
        .unwrap()
        .retain(|&child| child != legend);
    let mut content_inputs = inputs;
    content_inputs.vertical_margins_are_collapsible = Line::FALSE;
    content_inputs.known_dimensions.width = Some(outer.size.width);
    if definite_height {
        content_inputs.known_dimensions.height = Some(outer.size.height);
    }
    let mut output = tree.dispatch_child_layout(id, content_inputs, None);
    *tree.nodes[dom_id].layout_children.borrow_mut() = children;
    tree.nodes[dom_id]
        .element_data_mut()
        .unwrap()
        .fieldset_layout_style = None;
    if inputs.run_mode == RunMode::PerformLayout {
        let rtl = !original.writing_mode.is_bidi_ltr();
        let x = if rtl {
            outer.size.width - border.right - padding.right - margin.right - measured.size.width
        } else {
            border.left + padding.left + margin.left
        };
        let layout = Layout {
            size: measured.size,
            location: Point { x, y: legend_y },
            padding: legend_padding,
            border: legend_border,
            margin,
            scrollable_overflow_rect: measured.scrollable_overflow_rect,
            ..Default::default()
        };
        measured.oof_candidates.translate(layout.location);
        output.oof_candidates.append(&mut measured.oof_candidates);
        tree.set_unrounded_layout(taffy_node_id(legend), &layout);
        tree.nodes[dom_id]
            .element_data_mut()
            .unwrap()
            .fieldset_layout = Some(FieldsetLayout {
            legend,
            border_top_offset: ((measured.size.height - border.top) / 2.0).max(0.0),
            content_top_extra: extra,
        });
    }
    output
}
