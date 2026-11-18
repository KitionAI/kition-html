//! Horizontal multicolumn flow lowered to line-aware paint fragments.
use super::{LayoutPassState, resolve_calc_value};
use crate::{Node, NodeId, dom_node_id};
use style::{
    servo_arc::Arc,
    values::{
        computed::{LengthPercentage, MaxSize, Size as CssSize},
        generics::{
            NonNegative,
            column::ColumnCount,
            length::{LengthPercentageOrAuto, LengthPercentageOrNormal},
        },
        specified::column::ColumnFill,
    },
};
use taffy::{
    AvailableSpace, CoreStyle, LayoutInput, LayoutOutput, MaybeResolve, ResolveOrZero, RunMode,
};

#[derive(Clone, Debug)]
pub struct ColumnFragment {
    /// Source block-axis coordinates relative to the content-box top.
    pub start: f32,
    pub end: f32,
}
#[derive(Clone, Debug)]
pub struct MultiColumnLayout {
    pub width: f32,
    pub gap: f32,
    pub advance: f32,
    pub limited: bool,
    pub fragments: Vec<ColumnFragment>,
    pub rtl: bool,
}

pub(crate) fn applies(node: &Node) -> bool {
    !node.is_anonymous()
        && matches!(
            node.taffy_display(),
            taffy::Display::Block | taffy::Display::FlowRoot
        )
        && node
            .element_data()
            .is_some_and(|el| el.multicol_layout_style.is_none())
        && node.primary_styles().is_some_and(|style| {
            style.get_column().is_multicol() && style.writing_mode.is_horizontal()
        })
}

pub(crate) fn compute(
    tree: &mut LayoutPassState<'_>,
    id: taffy::NodeId,
    inputs: LayoutInput,
) -> LayoutOutput {
    let dom_id = dom_node_id(id);
    let original = tree.nodes[dom_id].primary_styles().unwrap().clone();
    tree.nodes[dom_id]
        .element_data_mut()
        .unwrap()
        .multicol_layout = None;
    // The guard also allows the normal dispatch to measure the outer box.
    tree.nodes[dom_id]
        .element_data_mut()
        .unwrap()
        .multicol_layout_style = Some(original.clone());
    let mut outer_inputs = inputs;
    outer_inputs.vertical_margins_are_collapsible = taffy::Line::FALSE;
    let mut outer = tree.dispatch_child_layout(id, outer_inputs, None);
    let style = tree.nodes[dom_id].layout_style();
    let pb = style
        .padding()
        .resolve_or_zero(inputs.parent_size.width, resolve_calc_value)
        + style
            .border()
            .resolve_or_zero(inputs.parent_size.width, resolve_calc_value);
    let content_width = (outer.size.width - pb.horizontal_axis_sum()).max(0.0);
    let column = original.get_column();
    let gap = match &original.get_position().column_gap {
        LengthPercentageOrNormal::Normal => original.get_font().font_size.used_size().px(),
        LengthPercentageOrNormal::LengthPercentage(value) => value
            .0
            .resolve(style::values::computed::CSSPixelLength::new(content_width))
            .px(),
    };
    let requested_count = match column.column_count {
        ColumnCount::Integer(value) => Some(value.0 as usize),
        ColumnCount::Auto => None,
    };
    let preferred_width = match column.column_width {
        LengthPercentageOrAuto::LengthPercentage(value) => Some(value.0.px()),
        LengthPercentageOrAuto::Auto => None,
    };
    let fits = preferred_width.map(|width| {
        ((content_width + gap) / (width + gap).max(1.0))
            .floor()
            .max(1.0) as usize
    });
    let count = match (requested_count, fits) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        _ => 1,
    }
    .clamp(1, 1000);
    let ideal_width = ((content_width - gap * (count - 1) as f32) / count as f32).max(0.0);
    let width = (ideal_width * 64.0).floor() / 64.0;
    let advance = ideal_width + gap;
    let constrained_height = inputs.known_dimensions.height.is_some()
        || !matches!(original.get_position().height, CssSize::Auto);
    let adjustment = if style.box_sizing() == taffy::BoxSizing::BorderBox {
        pb.vertical_axis_sum()
    } else {
        0.0
    };
    let min_height = style
        .min_size()
        .height
        .maybe_resolve(inputs.parent_size.height, resolve_calc_value)
        .map_or(0.0, |h| (h - adjustment).max(0.0));
    let max_height = if constrained_height {
        Some((outer.size.height - pb.vertical_axis_sum()).max(0.0))
    } else {
        style
            .max_size()
            .height
            .maybe_resolve(inputs.parent_size.height, resolve_calc_value)
            .map(|h| (h - adjustment).max(min_height))
    };
    drop(style);
    if !width.is_finite() || width < 1.0 || inputs.run_mode == RunMode::PerformHiddenLayout {
        tree.nodes[dom_id]
            .element_data_mut()
            .unwrap()
            .multicol_layout_style = None;
        return outer;
    }
    let mut narrow = (*original).clone();
    let position = narrow.mutate_position();
    position.width = CssSize::LengthPercentage(NonNegative(LengthPercentage::new_length(
        style::values::computed::CSSPixelLength::new(width + pb.horizontal_axis_sum()),
    )));
    position.min_width = CssSize::Auto;
    position.max_width = MaxSize::None;
    position.height = CssSize::Auto;
    position.min_height = CssSize::Auto;
    position.max_height = MaxSize::None;
    tree.nodes[dom_id]
        .element_data_mut()
        .unwrap()
        .multicol_layout_style = Some(Arc::new(narrow));
    let mut column_inputs = inputs;
    column_inputs.vertical_margins_are_collapsible = taffy::Line::FALSE;
    column_inputs.known_dimensions.width = Some(width + pb.horizontal_axis_sum());
    column_inputs.known_dimensions.height = None;
    column_inputs.known_dimensions_are_definite.height = false;
    column_inputs.available_space.width =
        AvailableSpace::Definite(width + pb.horizontal_axis_sum());
    column_inputs.available_space.height = AvailableSpace::MaxContent;
    column_inputs.run_mode = RunMode::PerformLayout;
    let flow = tree.dispatch_child_layout(id, column_inputs, None);
    tree.nodes[dom_id]
        .element_data_mut()
        .unwrap()
        .multicol_layout_style = None;
    let flow_height = (flow.size.height - pb.vertical_axis_sum()).max(0.0);
    let mut items = Vec::new();
    collect_items(tree, dom_id, dom_id, 0.0, pb.top, &mut items);
    if items.is_empty() && flow_height > 0.0 {
        items.push(FlowItem::block(0.0, flow_height, false));
    }
    let last_bottom = items.iter().map(|item| item.bottom).fold(0.0_f32, f32::max);
    if flow_height > last_bottom {
        items.push(FlowItem::block(last_bottom, flow_height, true));
    }
    items.sort_by(|a, b| a.top.total_cmp(&b.top).then(a.bottom.total_cmp(&b.bottom)));
    let balanced = column.column_fill == ColumnFill::Balance;
    let target = if balanced {
        let mut low = items
            .iter()
            .filter(|item| !item.splittable)
            .map(|item| item.bottom - item.top)
            .fold(0.0_f32, f32::max);
        let mut high = flow_height.max(low);
        for _ in 0..24 {
            let mid = (low + high) * 0.5;
            if fragments(&items, mid).len() <= count {
                high = mid;
            } else {
                low = mid;
            }
        }
        (high * 64.0).ceil() / 64.0
    } else {
        max_height.unwrap_or(flow_height)
    };
    let target = max_height
        .map_or(target, |height| target.min(height))
        .max(1.0 / 64.0);
    let fragments = fragments(&items, target);
    let used_height = fragments
        .iter()
        .map(|fragment| fragment.end - fragment.start)
        .fold(0.0_f32, f32::max);
    if !constrained_height {
        outer.size.height = max_height
            .map_or(used_height, |max| used_height.min(max))
            .max(min_height)
            + pb.vertical_axis_sum();
    }
    outer.scrollable_overflow_rect.right = outer
        .size
        .width
        .max(pb.left + (width + gap) * fragments.len() as f32 - gap);
    outer.scrollable_overflow_rect.bottom = outer.size.height.max(pb.top + used_height);
    outer.margins_can_collapse_through = false;
    if inputs.run_mode == RunMode::PerformLayout {
        tree.nodes[dom_id]
            .element_data_mut()
            .unwrap()
            .multicol_layout = Some(MultiColumnLayout {
            width,
            gap,
            advance,
            limited: fragments.len() >= MAX_FRAGMENTS,
            fragments,
            rtl: !original.writing_mode.is_bidi_ltr(),
        });
    }
    outer
}

const MAX_FRAGMENTS: usize = 1024;

#[derive(Clone, Copy)]
struct FlowItem {
    top: f32,
    bottom: f32,
    splittable: bool,
    line: Option<(NodeId, usize, usize, usize, usize)>,
}
impl FlowItem {
    fn block(top: f32, bottom: f32, splittable: bool) -> Self {
        Self {
            top,
            bottom,
            splittable,
            line: None,
        }
    }
}

fn fragments(items: &[FlowItem], height: f32) -> Vec<ColumnFragment> {
    let mut result = Vec::new();
    let mut first = 0;
    let mut start = 0.0;
    while first < items.len() && result.len() < MAX_FRAGMENTS {
        let mut next = first;
        while next < items.len() && items[next].bottom - start <= height + 1.0 / 128.0 {
            next += 1;
        }
        if next < items.len() && items[next].splittable && items[next].top < start + height {
            let end = start + height;
            result.push(ColumnFragment { start, end });
            start = end;
            first = next;
            continue;
        }
        if next > first && next < items.len() {
            if let (Some((group, _, _, orphans, widows)), Some((previous, _, _, _, _))) =
                (items[next].line, items[next - 1].line)
            {
                if group == previous {
                    let group_first = (first..next)
                        .find(|&i| items[i].line.is_some_and(|l| l.0 == group))
                        .unwrap_or(first);
                    let remaining = items[next..]
                        .iter()
                        .take_while(|item| item.line.is_some_and(|l| l.0 == group))
                        .count();
                    let shift = widows.saturating_sub(remaining).min(next - group_first);
                    let candidate = next - shift;
                    if candidate - group_first >= orphans {
                        next = candidate;
                    } else if group_first > first {
                        next = group_first;
                    }
                }
            }
        }
        // An oversized unbreakable item still makes forward progress.
        next = next.max(first + 1);
        let end = items[next - 1].bottom;
        result.push(ColumnFragment { start, end });
        first = next;
        if let Some(item) = items.get(first) {
            start = item.top;
        }
    }
    if first < items.len() {
        if let Some(last) = result.last_mut() {
            last.end = items.last().unwrap().bottom;
        }
    }
    result
}

fn collect_items(
    tree: &LayoutPassState<'_>,
    id: NodeId,
    root: NodeId,
    offset: f32,
    root_top: f32,
    items: &mut Vec<FlowItem>,
) {
    let node = &tree.nodes[id];
    if node.is_out_of_flow() || node.taffy_display() == taffy::Display::None {
        return;
    }
    let layout = node.unrounded_layout();
    if id != root
        && node.primary_styles().is_some_and(|s| {
            matches!(
                s.get_box().break_inside,
                style::values::specified::BreakWithin::Avoid
                    | style::values::specified::BreakWithin::AvoidColumn
            )
        })
    {
        items.push(FlowItem::block(
            offset - root_top,
            offset + layout.size.height - root_top,
            false,
        ));
        return;
    }
    // A nested multicol is already fragmented in its own coordinate space.
    if node
        .element_data()
        .is_some_and(|el| el.multicol_layout.is_some())
        && id != root
    {
        items.push(FlowItem::block(
            offset - root_top,
            offset + layout.size.height - root_top,
            false,
        ));
        return;
    }
    let mut has_lines = false;
    if let Some(text) = node
        .element_data()
        .and_then(|el| el.inline_layout_data.as_ref())
    {
        let origin = offset + layout.border.top + layout.padding.top + text.block_offset - root_top;
        let scale = tree.viewport.scale();
        let styles = node.primary_styles();
        let inherited = styles.as_ref().map(|s| s.get_inherited_box());
        let orphans = inherited.map_or(2, |s| s.orphans.max(1) as usize);
        let widows = inherited.map_or(2, |s| s.widows.max(1) as usize);
        let total = text.layout.lines().count();
        for (index, line) in text.layout.lines().enumerate() {
            let metrics = line.metrics();
            items.push(FlowItem {
                top: origin + metrics.block_min_coord / scale,
                bottom: origin + metrics.block_max_coord / scale,
                splittable: false,
                line: Some((id, index, total, orphans, widows)),
            });
            has_lines = true;
        }
    }
    let children = node.layout_children.borrow();
    let children = children.as_deref().unwrap_or(&[]);
    if !has_lines && children.is_empty() && layout.size.height > 0.0 {
        let splittable = !node
            .element_data()
            .is_some_and(|el| super::replaced::is_replaced_element(&el.name.local))
            && node.primary_styles().is_some_and(|s| {
                s.get_box().break_inside == style::values::specified::BreakWithin::Auto
            });
        items.push(FlowItem::block(
            offset - root_top,
            offset + layout.size.height - root_top,
            splittable,
        ));
    }
    if !has_lines {
        for &child in children {
            collect_items(
                tree,
                child,
                root,
                offset + tree.nodes[child].unrounded_layout().location.y,
                root_top,
                items,
            );
        }
    }
}
