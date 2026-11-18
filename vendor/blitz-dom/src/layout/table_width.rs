//! Content-weighted automatic table columns on top of the grid layout primitive.
// Column distribution is informed by Chromium's table_layout_utils.cc (2020
// The Chromium Authors). See LICENSE.chromium and the source links in vendor/README.md.
use std::sync::Arc;

use style::computed_values::border_collapse::T as BorderCollapse;
use taffy::{
    AvailableSpace, GridPlacement, GridTemplateComponent, LayoutInput, LayoutOutput,
    LayoutPartialTree, Line, NodeId, RequestedAxis, ResolveOrZero, RunMode, Size, SizingMode,
    compute_grid_layout, style_helpers,
};

use super::{
    LayoutPassState, resolve_calc_value,
    table::{TableContext, TableTreeWrapper},
};
use crate::taffy_node_id;

pub(super) fn compute_table_grid_layout(
    doc: &mut LayoutPassState<'_>,
    node_id: NodeId,
    ctx: Arc<TableContext>,
    inputs: LayoutInput,
) -> LayoutOutput {
    let mut tree = TableTreeWrapper {
        doc,
        ctx,
        column_style: None,
    };
    let mut initial = compute_grid_layout(&mut tree, node_id, inputs);
    let columns = &tree.ctx.style.grid_template_columns;
    // Fixed layout and unresolved percentage/calc tracks retain their own path.
    // In automatic layout, a definite column width is a preferred width, not
    // an inflexible grid track: its content minimum may be smaller or larger.
    if tree.ctx.is_fixed || columns.is_empty()
        || !columns.iter().all(|column| {
            matches!(column, GridTemplateComponent::Single(track)
                if (track.min.is_auto() || track.min.into_raw().tag() == taffy::CompactLength::LENGTH_TAG)
                    && (track.max.is_auto() || track.max.into_raw().tag() == taffy::CompactLength::LENGTH_TAG))
        })
    {
        return initial;
    }
    let count = columns.len();
    let gap = tree
        .ctx
        .style
        .gap
        .resolve_or_zero(
            Size {
                width: Some(initial.size.width),
                height: None,
            },
            resolve_calc_value,
        )
        .width;
    let padding = tree
        .ctx
        .style
        .padding
        .resolve_or_zero(inputs.parent_size.width, resolve_calc_value);
    let border = tree
        .ctx
        .style
        .border
        .resolve_or_zero(inputs.parent_size.width, resolve_calc_value);
    let mut available = (initial.size.width
        - padding.left
        - padding.right
        - border.left
        - border.right
        - gap * count.saturating_sub(1) as f32)
        .max(0.0);
    // Collapsed tracks exclude gutters. Intrinsic cell measures and column
    // proportions include the two half borders adjoining each column.
    let collapsed = if tree.ctx.border_collapse == BorderCollapse::Collapse {
        gap
    } else {
        0.0
    };
    let mut minimum = vec![0.0_f32; count];
    let mut maximum = vec![0.0_f32; count];
    let mut spanning = Vec::new();
    let mut constrained = vec![false; count];
    let mut percentages = tree.ctx.percentage_columns.clone();
    percentages.resize(count, 0.0);
    let mut percent_spanning = Vec::new();
    for cell in &tree.ctx.cells {
        let span = match cell.style.grid_column.end {
            GridPlacement::Span(span) => usize::from(span),
            _ => 1,
        };
        let start = usize::from(cell.column);
        let end = (start + span).min(count);
        if start >= end {
            continue;
        }
        let mut measure = |width| {
            tree.doc
                .compute_child_layout(
                    taffy_node_id(cell.node_id),
                    LayoutInput {
                        run_mode: RunMode::ComputeSize,
                        sizing_mode: SizingMode::ContentSize,
                        axis: RequestedAxis::Horizontal,
                        known_dimensions: Size::NONE,
                        known_dimensions_are_definite: Size {
                            width: false,
                            height: false,
                        },
                        parent_size: Size::NONE,
                        available_space: Size {
                            width,
                            height: AvailableSpace::MaxContent,
                        },
                        vertical_margins_are_collapsible: Line::FALSE,
                    },
                )
                .size
                .width
                + collapsed
        };
        let min = measure(AvailableSpace::MinContent);
        let mut max = measure(AvailableSpace::MaxContent).max(min);
        let specified = cell.preferred_width.is_some();
        if let Some(preferred) = cell.preferred_width {
            let extra = if cell.style.box_sizing == taffy::BoxSizing::ContentBox {
                collapsed
            } else {
                0.0
            };
            max = max.max(preferred + extra);
        }
        if end - start == 1 {
            minimum[start] = minimum[start].max(min);
            maximum[start] = maximum[start].max(max);
            constrained[start] |= specified;
            percentages[start] = percentages[start].max(cell.preferred_percentage.unwrap_or(0.0));
        } else {
            spanning.push((start, end, min, max));
            if let Some(percent) = cell.preferred_percentage {
                percent_spanning.push((start, end, percent));
            }
        }
    }
    for (index, column) in columns.iter().enumerate() {
        let GridTemplateComponent::Single(track) = column else {
            continue;
        };
        if track.max.into_raw().tag() == taffy::CompactLength::LENGTH_TAG {
            constrained[index] = true;
            // Column widths include the collapsed half-borders; cell widths
            // collected by the grid adapter exclude its border gutters.
            let from_column = tree.ctx.columns.get(index).is_some_and(|column| {
                tree.doc.nodes[column.node_id]
                    .primary_styles()
                    .is_some_and(|style| {
                        let style = stylo_taffy::to_taffy_style(&style);
                        style.size.width.tag() == taffy::CompactLength::LENGTH_TAG
                            && style.size.width.value() > 0.0
                    })
            });
            if from_column {
                maximum[index] = maximum[index].max(track.max.into_raw().value());
            }
        } else if track.min.into_raw().tag() == taffy::CompactLength::LENGTH_TAG {
            minimum[index] = minimum[index].max(track.min.into_raw().value());
            maximum[index] = maximum[index].max(minimum[index]);
        }
    }
    spanning.sort_by_key(|&(start, end, _, _)| end - start);
    for (start, end, min, max) in spanning {
        let spacing = (gap - collapsed) * (end - start - 1) as f32;
        let widths = distribute(
            &minimum[start..end],
            &maximum[start..end],
            &constrained[start..end],
            (min - spacing).max(0.0),
        );
        for (index, width) in (start..end).zip(widths) {
            minimum[index] = minimum[index].max(width);
            maximum[index] = maximum[index].max(minimum[index]);
        }
        let widths = distribute(
            &minimum[start..end],
            &maximum[start..end],
            &constrained[start..end],
            (max - spacing).max(0.0),
        );
        for (index, width) in (start..end).zip(widths) {
            maximum[index] = maximum[index].max(width);
        }
    }
    // A spanning percentage contributes only what its existing tracks lack.
    percent_spanning.sort_by_key(|&(start, end, _)| end - start);
    for (start, end, percent) in percent_spanning {
        let existing: f32 = percentages[start..end].iter().sum();
        let extra = (percent - existing).max(0.0);
        let eligible: Vec<_> = (start..end).filter(|&i| percentages[i] == 0.0).collect();
        let weight: f32 = eligible.iter().map(|&i| maximum[i]).sum();
        for &i in &eligible {
            percentages[i] = extra
                * if weight > 0.0 {
                    maximum[i] / weight
                } else {
                    1.0 / eligible.len() as f32
                };
        }
    }
    // Percentage constraints consume at most 100%, in column order.
    let mut remaining_percent = 1.0_f32;
    for percentage in &mut percentages {
        *percentage = percentage.clamp(0.0, remaining_percent);
        remaining_percent = (remaining_percent - *percentage).max(0.0);
    }
    // Recompute intrinsic automatic table size with border-inclusive column
    // preferences converted to the adapter's border-free tracks. Keep Taffy's
    // handling of parent constraints and table min/max-width in this pass.
    if tree.ctx.style.size.width.is_auto() && inputs.known_dimensions.width.is_none() {
        let mut intrinsic_style = tree.ctx.style.clone();
        let intrinsic_maximum = if percentages.iter().any(|&p| p > 0.0) {
            let total_max: f32 = maximum.iter().sum();
            let non_percent: f32 = maximum
                .iter()
                .zip(&percentages)
                .filter(|&(_, p)| *p == 0.0)
                .map(|(m, _)| m)
                .sum();
            let mut desired = total_max;
            for (&max, &percent) in maximum.iter().zip(&percentages) {
                if percent > 0.0 {
                    desired = desired.max(max / percent);
                }
            }
            if remaining_percent > 0.0 {
                desired = desired.max(non_percent / remaining_percent);
            } else if non_percent > 0.0 {
                desired = desired.max(
                    inputs
                        .available_space
                        .width
                        .into_option()
                        .unwrap_or(total_max),
                );
            }
            distribute_percentages(&minimum, &maximum, &constrained, &percentages, desired)
        } else {
            maximum.clone()
        };
        intrinsic_style.grid_template_columns = minimum
            .iter()
            .zip(&intrinsic_maximum)
            .map(|(&min, &max)| {
                let track: taffy::TrackSizingFunction = style_helpers::minmax(
                    style_helpers::length((min - collapsed).max(0.0)),
                    style_helpers::length((max - collapsed).max(0.0)),
                );
                track.into()
            })
            .collect();
        tree.column_style = Some(intrinsic_style);
        initial = compute_grid_layout(&mut tree, node_id, inputs);
        available = (initial.size.width
            - padding.left
            - padding.right
            - border.left
            - border.right
            - gap * count.saturating_sub(1) as f32)
            .max(0.0);
    }
    let widths = distribute_percentages(
        &minimum,
        &maximum,
        &constrained,
        &percentages,
        available + collapsed * count as f32,
    );
    // Keep the context alive: cloned styles can reference its calc storage.
    let mut style = tree.ctx.style.clone();
    style.grid_template_columns = widths
        .into_iter()
        .map(|width| {
            let track: taffy::TrackSizingFunction =
                style_helpers::length((width - collapsed).max(0.0));
            track.into()
        })
        .collect();
    tree.column_style = Some(style);
    let minimum_outer = minimum.iter().sum::<f32>() - collapsed * count as f32
        + padding.left
        + padding.right
        + border.left
        + border.right
        + gap * count.saturating_sub(1) as f32;
    let mut final_inputs = inputs;
    if minimum_outer > initial.size.width {
        final_inputs.known_dimensions.width = Some(minimum_outer);
    }
    compute_grid_layout(&mut tree, node_id, final_inputs)
}

fn distribute_percentages(
    minimum: &[f32],
    maximum: &[f32],
    constrained: &[bool],
    percentages: &[f32],
    available: f32,
) -> Vec<f32> {
    if !percentages.iter().any(|&p| p > 0.0) {
        return distribute(minimum, maximum, constrained, available);
    }
    let available = available.max(minimum.iter().sum());
    // Resolve percentage targets to layout units before interpolating. The tiny
    // tolerance absorbs f32 subtraction error at an exact 1/64-pixel boundary.
    let targets: Vec<_> = minimum
        .iter()
        .zip(percentages)
        .map(|(&min, &p)| min.max(((available * p * 64.0) + 0.001).floor() / 64.0))
        .collect();
    let percent_sum: f32 = targets.iter().sum();
    if available < percent_sum {
        // Percentages take priority, but cannot violate content minima.
        return distribute(minimum, &targets, &vec![true; minimum.len()], available);
    }
    let remaining_indices: Vec<_> = percentages
        .iter()
        .enumerate()
        .filter(|&(_, p)| *p == 0.0)
        .map(|(i, _)| i)
        .collect();
    if remaining_indices.is_empty() {
        return distribute(&targets, &targets, &vec![true; minimum.len()], available);
    }
    let reserved: f32 = targets
        .iter()
        .zip(percentages)
        .filter(|&(_, p)| *p > 0.0)
        .map(|(v, _)| v)
        .sum();
    let sub_min: Vec<_> = remaining_indices.iter().map(|&i| minimum[i]).collect();
    let sub_max: Vec<_> = remaining_indices.iter().map(|&i| maximum[i]).collect();
    let sub_constrained: Vec<_> = remaining_indices.iter().map(|&i| constrained[i]).collect();
    let widths = distribute(&sub_min, &sub_max, &sub_constrained, available - reserved);
    let mut result = targets;
    for (&i, width) in remaining_indices.iter().zip(widths) {
        result[i] = width;
    }
    // Quantize only after merging; all columns share one table width.
    let last = result.len() - 1;
    let mut remaining = available;
    for (i, width) in result.iter_mut().enumerate() {
        *width = if i == last {
            remaining
        } else {
            (*width * 64.0).floor() / 64.0
        };
        remaining -= *width;
    }
    result
}

fn distribute(minimum: &[f32], maximum: &[f32], constrained: &[bool], available: f32) -> Vec<f32> {
    let min_sum: f32 = minimum.iter().sum();
    let max_sum: f32 = maximum.iter().sum();
    let count = minimum.len();
    let available = available.max(min_sum);
    // First satisfy constrained columns, then grow the automatic columns from
    // their minima to their maxima. Excess goes to automatic columns
    // before constrained columns, in proportion to their preferred widths.
    let constrained_sum: f32 = minimum
        .iter()
        .zip(maximum)
        .zip(constrained)
        .map(|((&min, &max), &fixed)| if fixed { max } else { min })
        .sum();
    let mut widths = if available < max_sum {
        let (lower, upper): (Vec<f32>, Vec<f32>) = if available < constrained_sum {
            (
                minimum.to_vec(),
                minimum
                    .iter()
                    .zip(maximum)
                    .zip(constrained)
                    .map(|((&min, &max), &fixed)| if fixed { max } else { min })
                    .collect(),
            )
        } else {
            (
                minimum
                    .iter()
                    .zip(maximum)
                    .zip(constrained)
                    .map(|((&min, &max), &fixed)| if fixed { max } else { min })
                    .collect(),
                maximum.to_vec(),
            )
        };
        let lower_sum: f32 = lower.iter().sum();
        let upper_sum: f32 = upper.iter().sum();
        let ratio =
            ((available - lower_sum) / (upper_sum - lower_sum).max(f32::EPSILON)).clamp(0.0, 1.0);
        lower
            .iter()
            .zip(&upper)
            .map(|(&min, &max)| min + (max - min) * ratio)
            .collect::<Vec<_>>()
    } else {
        let auto_count = constrained.iter().filter(|&&fixed| !fixed).count();
        let has_auto = auto_count > 0;
        let weights: Vec<f32> = maximum
            .iter()
            .zip(constrained)
            .map(|(&max, &fixed)| if has_auto && fixed { 0.0 } else { max })
            .collect();
        let weight_sum: f32 = weights.iter().sum();
        maximum
            .iter()
            .zip(weights)
            .zip(constrained)
            .map(|((&max, weight), &fixed)| {
                max + (available - max_sum)
                    * if weight_sum > 0.0 {
                        weight / weight_sum
                    } else if has_auto {
                        if fixed { 0.0 } else { 1.0 / auto_count as f32 }
                    } else {
                        1.0 / count as f32
                    }
            })
            .collect()
    };
    // Browser layout units truncate proportional increments at 1/64 CSS px.
    // Put the remainder in the final column so rounding cannot grow the table.
    let mut remaining = available.max(min_sum);
    let last = widths.len() - 1;
    for (index, width) in widths.iter_mut().enumerate() {
        *width = if index == last {
            remaining
        } else {
            (*width * 64.0).floor() / 64.0
        };
        remaining -= *width;
    }
    widths
}
