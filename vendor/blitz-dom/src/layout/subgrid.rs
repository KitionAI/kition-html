//! Resolve inherited grid tracks after the parent's placement and sizing pass.
use super::resolve_calc_value;
use crate::{BaseDocument, node::Node};
use style::{
    Atom,
    properties::ComputedValues,
    servo_arc::Arc,
    values::{
        CustomIdent,
        computed::{GridTemplateComponent, Length, LengthPercentage},
        generics::{
            NonNegative,
            grid::{
                GenericGridTemplateComponent as Template, LineNameList, LineNameListValue,
                RepeatCount, TrackBreadth, TrackList, TrackListValue, TrackSize,
            },
            length::LengthPercentageOrNormal,
        },
        specified::box_::DisplayInside,
    },
};
use taffy::{DetailedGridTracksInfo, DetailedLayoutInfo, GridContainerStyle, ResolveOrZero, Size};

impl BaseDocument {
    pub(crate) fn clear_subgrid_layout_styles(&mut self) {
        let mut changed = false;
        for (_, node) in self.nodes.iter_mut() {
            if let Some(el) = node.element_data_mut() {
                changed |= el.subgrid_layout_style.take().is_some();
            }
        }
        if changed {
            for (_, node) in self.nodes.iter_mut() {
                node.invalidate_layout_cache();
            }
        }
    }

    pub(crate) fn resolve_subgrid_layout_styles(&mut self) -> bool {
        let updates: Vec<_> = self
            .nodes
            .iter()
            .filter_map(|(id, node)| {
                let style = resolved_style(node);
                (style.is_some()
                    || node
                        .element_data()
                        .is_some_and(|el| el.subgrid_layout_style.is_some()))
                .then_some((id, style))
            })
            .collect();
        let mut changed = false;
        for (id, style) in updates {
            let Some(el) = self.nodes[id].element_data_mut() else {
                continue;
            };
            let Some(style) = style else {
                changed |= el.subgrid_layout_style.take().is_some();
                continue;
            };
            let position = style.get_position();
            let same = el.subgrid_layout_style.as_ref().is_some_and(|old| {
                let old = old.get_position();
                old.grid_template_columns == position.grid_template_columns
                    && old.grid_template_rows == position.grid_template_rows
                    && old.column_gap == position.column_gap
                    && old.row_gap == position.row_gap
            });
            changed |= !same;
            el.subgrid_layout_style = Some(Arc::new(style));
        }
        if changed {
            for (_, node) in self.nodes.iter_mut() {
                node.invalidate_layout_cache();
            }
        }
        changed
    }
}

fn resolved_style(node: &Node) -> Option<ComputedValues> {
    let primary = node.primary_styles()?;
    if primary.get_box().display.inside() != DisplayInside::Grid || node.is_out_of_flow() {
        return None;
    }
    let position = primary.get_position();
    let columns = match &position.grid_template_columns {
        Template::Subgrid(names) => Some(names),
        _ => None,
    };
    let rows = match &position.grid_template_rows {
        Template::Subgrid(names) => Some(names),
        _ => None,
    };
    if columns.is_none() && rows.is_none() {
        return None;
    }
    let parent = node.with(node.layout_parent.get()?);
    let parent_style = parent.primary_styles()?;
    if parent_style.get_box().display.inside() != DisplayInside::Grid {
        return None;
    }
    // Orthogonal subgrids need explicit axis conversion and are not lowered here.
    if !primary.writing_mode.is_horizontal() || !parent_style.writing_mode.is_horizontal() {
        return None;
    }
    let DetailedLayoutInfo::Grid(grid) = &parent.element_data()?.detailed_layout_info else {
        return None;
    };
    let children = parent.layout_children.borrow();
    let mut in_flow = children.as_ref()?.iter().filter(|&&id| {
        let child = node.with(id);
        !child.is_out_of_flow()
            && child
                .primary_styles()
                .is_some_and(|s| !s.get_box().display.is_none())
    });
    let index = in_flow.position(|&id| id == node.id)?;
    let item = grid.items.get(index)?;
    let layout = node.unrounded_layout();
    let content = Size {
        width: Some(
            (layout.size.width
                - layout.padding.left
                - layout.padding.right
                - layout.border.left
                - layout.border.right)
                .max(0.0),
        ),
        height: Some(
            (layout.size.height
                - layout.padding.top
                - layout.padding.bottom
                - layout.border.top
                - layout.border.bottom)
                .max(0.0),
        ),
    };
    let gaps = node
        .layout_style()
        .gap()
        .resolve_or_zero(content, resolve_calc_value);
    let mut result = (**primary).clone();
    let output = result.mutate_position();
    if let Some(names) = columns {
        let normal = matches!(position.column_gap, LengthPercentageOrNormal::Normal);
        let start = usize::from(item.column_start.checked_sub(1)?);
        let end = usize::from(item.column_end.checked_sub(1)?);
        let (tracks, gap) = inherited_axis(
            &grid.columns,
            start,
            end,
            names,
            (!normal).then_some(gaps.width),
            if primary.writing_mode.is_bidi_ltr() {
                layout.padding.left + layout.border.left + layout.margin.left
            } else {
                layout.padding.right + layout.border.right + layout.margin.right
            },
            if primary.writing_mode.is_bidi_ltr() {
                layout.padding.right + layout.border.right + layout.margin.right
            } else {
                layout.padding.left + layout.border.left + layout.margin.left
            },
            primary.writing_mode.is_bidi_ltr() != parent_style.writing_mode.is_bidi_ltr(),
        )?;
        output.grid_template_columns = tracks;
        output.column_gap = LengthPercentageOrNormal::LengthPercentage(NonNegative(
            LengthPercentage::new_length(Length::new(gap)),
        ));
    }
    if let Some(names) = rows {
        let normal = matches!(position.row_gap, LengthPercentageOrNormal::Normal);
        let start = usize::from(item.row_start.checked_sub(1)?);
        let end = usize::from(item.row_end.checked_sub(1)?);
        let (tracks, gap) = inherited_axis(
            &grid.rows,
            start,
            end,
            names,
            (!normal).then_some(gaps.height),
            layout.padding.top + layout.border.top + layout.margin.top,
            layout.padding.bottom + layout.border.bottom + layout.margin.bottom,
            false,
        )?;
        output.grid_template_rows = tracks;
        output.row_gap = LengthPercentageOrNormal::LengthPercentage(NonNegative(
            LengthPercentage::new_length(Length::new(gap)),
        ));
    }
    Some(result)
}

#[allow(clippy::too_many_arguments)]
fn inherited_axis(
    parent: &DetailedGridTracksInfo<Atom>,
    start: usize,
    end: usize,
    local_names: &LineNameList<i32>,
    explicit_gap: Option<f32>,
    inset_start: f32,
    inset_end: f32,
    reverse: bool,
) -> Option<(GridTemplateComponent, f32)> {
    let tracks = parent.positions.get(start..end)?;
    if tracks.is_empty() || tracks.len() > 1000 {
        return None;
    }
    let gap_at = |pair: &[taffy::Line<f32>]| {
        (pair[1].start - pair[0].end)
            .max(pair[0].start - pair[1].end)
            .max(0.0)
    };
    let inherited_gap = tracks.windows(2).next().map_or(0.0, gap_at);
    // Collapsed auto-fit gutters can be nonuniform; retain the original path.
    if tracks
        .windows(2)
        .any(|pair| (gap_at(pair) - inherited_gap).abs() > 0.001)
    {
        return None;
    }
    let gap = explicit_gap.unwrap_or(inherited_gap);
    let adjustment = (inherited_gap - gap) * 0.5;
    let mut widths: Vec<_> = tracks.iter().map(|t| (t.end - t.start).abs()).collect();
    if reverse {
        widths.reverse();
    }
    let count = widths.len();
    let values: Vec<_> = widths
        .into_iter()
        .enumerate()
        .map(|(index, width)| {
            let width = width
                + if index > 0 { adjustment } else { -inset_start }
                + if index + 1 < count {
                    adjustment
                } else {
                    -inset_end
                };
            TrackListValue::TrackSize(TrackSize::Breadth(TrackBreadth::Breadth(
                LengthPercentage::new_length(Length::new(width.max(0.0))),
            )))
        })
        .collect();
    let negative = usize::from(parent.negative_implicit_tracks);
    let mut names: Vec<Vec<CustomIdent>> = (start..=end)
        .map(|index| {
            index
                .checked_sub(negative)
                .map(|index| {
                    parent
                        .line_names
                        .line(index)
                        .iter()
                        .cloned()
                        .map(CustomIdent)
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();
    if reverse {
        names.reverse();
    }
    let mut index = 0;
    for item in local_names.line_names.iter() {
        match item {
            LineNameListValue::LineNames(local) => {
                if let Some(names) = names.get_mut(index) {
                    names.extend(local.iter().cloned());
                }
                index += 1;
            }
            LineNameListValue::Repeat(repeat) => {
                let repetitions = match repeat.count {
                    RepeatCount::Number(n) => n.max(0) as usize,
                    RepeatCount::AutoFill => {
                        (count + 1).saturating_sub(local_names.expanded_line_names_length)
                            / repeat.line_names.len().max(1)
                    }
                    RepeatCount::AutoFit => 0,
                };
                for _ in 0..repetitions.min(count + 1) {
                    for local in repeat.line_names.iter() {
                        if let Some(names) = names.get_mut(index) {
                            names.extend(local.iter().cloned());
                        }
                        index += 1;
                    }
                }
            }
        }
    }
    Some((
        Template::TrackList(Box::new(TrackList {
            auto_repeat_index: usize::MAX,
            values: values.into(),
            line_names: names.into_iter().map(Into::into).collect::<Vec<_>>().into(),
        })),
        gap,
    ))
}
