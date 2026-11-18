//! Cell-edge conflict resolution for collapsed tables.
//!
//! Cells reserve half of each resolved border. Individual winning segments
//! keep layout and paint consistent without inventing lines through spans.
use std::collections::BTreeMap;

use style::values::computed::BorderStyle;

use crate::{BaseDocument, NodeId};

use super::table::TableCell;

#[derive(Clone, Copy, Debug)]
pub enum TableBorderSide {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Clone, Copy, Debug)]
pub struct CollapsedTableEdge {
    pub node_id: NodeId,
    pub side: TableBorderSide,
    pub width: f32,
    pub style: BorderStyle,
}

impl CollapsedTableEdge {
    fn rank(self) -> u8 {
        match self.style {
            BorderStyle::Hidden => 9,
            BorderStyle::Double => 8,
            BorderStyle::Solid => 7,
            BorderStyle::Dashed => 6,
            BorderStyle::Dotted => 5,
            BorderStyle::Ridge => 4,
            BorderStyle::Outset => 3,
            BorderStyle::Groove => 2,
            BorderStyle::Inset => 1,
            BorderStyle::None => 0,
        }
    }

    fn wins_over(self, previous: Self) -> bool {
        if self.style == BorderStyle::Hidden || previous.style == BorderStyle::Hidden {
            return self.style == BorderStyle::Hidden && previous.style != BorderStyle::Hidden;
        }
        self.width > previous.width
            || (self.width == previous.width && self.rank() > previous.rank())
    }
}

#[derive(Debug, Default)]
pub struct CollapsedTableBorders {
    /// Keys are (row boundary, column segment), in logical track order.
    pub horizontal: BTreeMap<(usize, usize), CollapsedTableEdge>,
    /// Keys are (row segment, column boundary), in logical track order.
    pub vertical: BTreeMap<(usize, usize), CollapsedTableEdge>,
    pub rows: usize,
    pub columns: usize,
}

impl CollapsedTableBorders {
    pub(crate) fn new(doc: &BaseDocument, cells: &[TableCell], rtl: bool) -> Self {
        let mut result = Self::default();
        for cell in cells {
            result.rows = result.rows.max(cell.row as usize + cell.rowspan as usize);
            result.columns = result
                .columns
                .max(cell.column as usize + cell.colspan as usize);
        }
        for cell in cells {
            let Some(style) = doc.nodes[cell.node_id].primary_styles() else {
                continue;
            };
            let border = style.get_border();
            let edge = |side, width: app_units::Au, style: BorderStyle| CollapsedTableEdge {
                node_id: cell.node_id,
                side,
                width: if style.none_or_hidden() {
                    0.
                } else {
                    width.to_f32_px()
                },
                style,
            };
            let top = edge(
                TableBorderSide::Top,
                border.border_top_width.0,
                border.border_top_style,
            );
            let right = edge(
                TableBorderSide::Right,
                border.border_right_width.0,
                border.border_right_style,
            );
            let bottom = edge(
                TableBorderSide::Bottom,
                border.border_bottom_width.0,
                border.border_bottom_style,
            );
            let left = edge(
                TableBorderSide::Left,
                border.border_left_width.0,
                border.border_left_style,
            );
            let (row, column) = (cell.row as usize, cell.column as usize);
            let (end_row, end_column) =
                (row + cell.rowspan as usize, column + cell.colspan as usize);
            // Rows and cells arrive in logical order. At equal width/style the
            // upper cell, then the inline-start cell, retains the shared edge.
            for column in column..end_column {
                Self::insert(&mut result.horizontal, (row, column), top);
                Self::insert(&mut result.horizontal, (end_row, column), bottom);
            }
            for row in row..end_row {
                Self::insert(
                    &mut result.vertical,
                    (row, column),
                    if rtl { right } else { left },
                );
                Self::insert(
                    &mut result.vertical,
                    (row, end_column),
                    if rtl { left } else { right },
                );
            }
        }
        result
    }

    fn insert(
        edges: &mut BTreeMap<(usize, usize), CollapsedTableEdge>,
        key: (usize, usize),
        edge: CollapsedTableEdge,
    ) {
        edges
            .entry(key)
            .and_modify(|previous| {
                if edge.wins_over(*previous) {
                    *previous = edge;
                }
            })
            .or_insert(edge);
    }

    pub(crate) fn cell_widths(&self, cell: &TableCell, rtl: bool) -> taffy::Rect<f32> {
        let (row, column) = (cell.row as usize, cell.column as usize);
        let (end_row, end_column) = (row + cell.rowspan as usize, column + cell.colspan as usize);
        let horizontal = |boundary| {
            (column..end_column)
                .filter_map(|col| self.horizontal.get(&(boundary, col)))
                .map(|edge| edge.width)
                .fold(0., f32::max)
        };
        let vertical = |boundary| {
            (row..end_row)
                .filter_map(|row| self.vertical.get(&(row, boundary)))
                .map(|edge| edge.width)
                .fold(0., f32::max)
        };
        taffy::Rect {
            top: horizontal(row),
            bottom: horizontal(end_row),
            left: vertical(if rtl { end_column } else { column }),
            right: vertical(if rtl { column } else { end_column }),
        }
    }

    pub(crate) fn outer_widths(&self, rtl: bool) -> taffy::Rect<f32> {
        let horizontal = |boundary| {
            self.horizontal
                .iter()
                .filter(|((row, _), _)| *row == boundary)
                .map(|(_, edge)| edge.width)
                .fold(0., f32::max)
        };
        let vertical = |boundary| {
            self.vertical
                .iter()
                .filter(|((_, column), _)| *column == boundary)
                .map(|(_, edge)| edge.width)
                .fold(0., f32::max)
        };
        taffy::Rect {
            top: horizontal(0),
            bottom: horizontal(self.rows),
            left: vertical(if rtl { self.columns } else { 0 }),
            right: vertical(if rtl { 0 } else { self.columns }),
        }
    }
}
