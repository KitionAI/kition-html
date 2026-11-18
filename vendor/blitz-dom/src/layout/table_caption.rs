//! Captions share the table wrapper's width, but sit outside the painted grid.
use std::sync::Arc;

use style::values::computed::table::CaptionSide;
use taffy::{
    AvailableSpace, CoreStyle, Layout, LayoutInput, LayoutOutput, LayoutPartialTree, Line, NodeId,
    Point, ResolveOrZero, RunMode, Size,
};

use super::{
    LayoutPassState, resolve_calc_value, table::TableContext,
    table_width::compute_table_grid_layout,
};
use crate::taffy_node_id;

pub(super) fn compute_table_layout(
    doc: &mut LayoutPassState<'_>,
    node_id: NodeId,
    ctx: Arc<TableContext>,
    inputs: LayoutInput,
) -> LayoutOutput {
    if ctx.captions.is_empty() {
        return compute_table_grid_layout(doc, node_id, ctx, inputs);
    }
    let mut grid_inputs = inputs;
    if !ctx.captions.is_empty() {
        // The parent measures the wrapper including captions. Its final known
        // height must not be passed straight through to the inner grid.
        grid_inputs.known_dimensions.height = None;
    }
    let mut output = compute_table_grid_layout(doc, node_id, Arc::clone(&ctx), grid_inputs);
    let width = output.size.width;
    let mut captions = Vec::new();
    let mut top_height = 0.0;
    let mut bottom_height = 0.0;
    for id in &ctx.captions {
        let node = &doc.nodes[*id];
        let bottom = node
            .primary_styles()
            .is_some_and(|style| *style.get_caption_side() == CaptionSide::Bottom);
        let style = node.layout_style();
        let margin_style = style.margin();
        let mut margin = margin_style.resolve_or_zero(Some(width), resolve_calc_value);
        let auto_width = style.size().width.is_auto();
        let padding = style
            .padding()
            .resolve_or_zero(Some(width), resolve_calc_value);
        let border = style
            .border()
            .resolve_or_zero(Some(width), resolve_calc_value);
        drop(style);
        let caption_width = (width - margin.left - margin.right).max(0.0);
        let child = doc.compute_child_layout(
            taffy_node_id(*id),
            LayoutInput {
                known_dimensions: Size {
                    width: auto_width.then_some(caption_width),
                    height: None,
                },
                known_dimensions_are_definite: Size {
                    width: true,
                    height: false,
                },
                parent_size: Size {
                    width: Some(width),
                    height: None,
                },
                available_space: Size {
                    width: AvailableSpace::Definite(caption_width),
                    height: AvailableSpace::MaxContent,
                },
                vertical_margins_are_collapsible: Line::FALSE,
                ..inputs
            },
        );
        let free_space = (width - child.size.width - margin.left - margin.right).max(0.0);
        let auto_margins =
            u8::from(margin_style.left.is_auto()) + u8::from(margin_style.right.is_auto());
        if auto_margins > 0 {
            let share = free_space / f32::from(auto_margins);
            if margin_style.left.is_auto() {
                margin.left = share;
            }
            if margin_style.right.is_auto() {
                margin.right = share;
            }
        }
        let cursor = if bottom {
            &mut bottom_height
        } else {
            &mut top_height
        };
        let layout = Layout {
            size: child.size,
            location: Point {
                x: margin.left,
                y: *cursor + margin.top,
            },
            padding,
            border,
            margin,
            scrollable_overflow_rect: child.scrollable_overflow_rect,
            ..Layout::default()
        };
        *cursor += margin.top + child.size.height + margin.bottom;
        captions.push((*id, bottom, layout, child.oof_candidates));
    }
    let caption_height = top_height + bottom_height;
    if !ctx.captions.is_empty()
        && let Some(height) = inputs.known_dimensions.height
    {
        grid_inputs.known_dimensions.height = Some((height - caption_height).max(0.0));
        output = compute_table_grid_layout(doc, node_id, Arc::clone(&ctx), grid_inputs);
    }
    let grid_height = output.size.height;
    if inputs.run_mode == RunMode::PerformLayout {
        *ctx.grid_box.borrow_mut() = Some(Layout {
            size: output.size,
            location: Point {
                x: 0.0,
                y: top_height,
            },
            ..Layout::default()
        });
        for cell in &ctx.cells {
            doc.nodes[cell.node_id].unrounded_layout_mut().location.y += top_height;
        }
        output.oof_candidates.translate(Point {
            x: 0.0,
            y: top_height,
        });
        if let Some(area) = &mut output.oof_positioning_area {
            area.offset.y += top_height;
        }
        for (index, (id, bottom, mut layout, mut candidates)) in captions.into_iter().enumerate() {
            if bottom {
                layout.location.y += top_height + grid_height;
            }
            layout.order = (ctx.cells.len() + index) as u32;
            candidates.translate(layout.location);
            output.oof_candidates.append(&mut candidates);
            doc.set_unrounded_layout(taffy_node_id(id), &layout);
        }
    }
    output.size.height += caption_height;
    output.baselines.first = output.baselines.first.map(|v| v + top_height);
    output.baselines.last = output.baselines.last.map(|v| v + top_height);
    output.scrollable_overflow_rect.bottom += caption_height;
    output
}
