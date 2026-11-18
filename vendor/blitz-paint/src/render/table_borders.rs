//! Paint the winning edge of each collapsed-table cell boundary.
use super::{ElementCx, PhysicalTracks};
use crate::color::ToColorColor;
use anyrender::PaintScene;
use blitz_dom::{CollapsedTableEdge, CopyStyleGetters, TableBorderSide, node::SpecialElementData};
use kurbo::Rect;
use peniko::{Color, Fill};
use style::values::computed::BorderStyle;

fn bands(tracks: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let Some(first) = tracks.first() else {
        return Vec::new();
    };
    let mut result = vec![(first.0, first.0)];
    result.extend(tracks.windows(2).map(|pair| (pair[0].1, pair[1].0)));
    let end = tracks.last().unwrap().1;
    result.push((end, end));
    result
}

impl ElementCx<'_, '_> {
    fn table_edge_color(&self, edge: CollapsedTableEdge) -> Option<Color> {
        if edge.width <= 0. || edge.style.none_or_hidden() {
            return None;
        }
        let node = self.context.dom.get_node(edge.node_id)?;
        let style = node.primary_styles()?;
        let border = style.get_border();
        let color = match edge.side {
            TableBorderSide::Top => &border.border_top_color,
            TableBorderSide::Right => &border.border_right_color,
            TableBorderSide::Bottom => &border.border_bottom_color,
            TableBorderSide::Left => &border.border_left_color,
        }
        .resolve_to_absolute(&style.clone_color())
        .as_srgb_color();
        (color != Color::TRANSPARENT).then_some(color)
    }

    pub(crate) fn draw_table_borders(&self, scene: &mut impl PaintScene) {
        let SpecialElementData::TableRoot(table) = &self.element.special_data else {
            return;
        };
        let Some(borders) = &table.collapsed_borders else {
            return;
        };
        let Some(grid) = &*table.computed_grid_info.borrow() else {
            return;
        };
        let cols: Vec<_> = PhysicalTracks::from_tracks(&grid.columns)
            .iter()
            .map(|track| (track.start as f64, track.end as f64))
            .collect();
        let rows: Vec<_> = PhysicalTracks::from_tracks(&grid.rows)
            .iter()
            .map(|track| (track.start as f64, track.end as f64))
            .collect();
        if cols.is_empty() || rows.is_empty() {
            return;
        }
        let outer = table
            .style
            .border
            .map(|width| width.into_raw().value() as f64);
        let xs = bands(&cols);
        let ys = bands(&rows);
        let rtl = grid
            .columns
            .positions
            .first()
            .zip(grid.columns.positions.last())
            .is_some_and(|(first, last)| first.start > last.start);
        let offset = table
            .grid_box
            .borrow()
            .as_ref()
            .map_or(0., |grid| grid.location.y as f64);
        let mut fill = |edge: CollapsedTableEdge, rect: Rect| {
            let Some(color) = self.table_edge_color(edge) else {
                return;
            };
            // Collapsed edges snap in CSS pixels before device scaling.
            // At fractional DPR this intentionally preserves half-device-pixel
            // coverage instead of shifting the line to a neighboring pixel.
            let rect = Rect::new(
                rect.x0.round(),
                (rect.y0 + offset).round(),
                rect.x1.round(),
                (rect.y1 + offset).round(),
            )
            .scale_from_origin(self.scale);
            scene.fill(Fill::NonZero, self.transform, color, None, &rect);
        };
        let table_border = self.style.get_border();
        for (&(row, logical_column), &edge) in &borders.vertical {
            if row >= rows.len() || logical_column > cols.len() {
                continue;
            }
            let column = if rtl {
                cols.len() - logical_column
            } else {
                logical_column
            };
            if (column == 0 && table_border.border_left_style == BorderStyle::Hidden)
                || (column == cols.len() && table_border.border_right_style == BorderStyle::Hidden)
            {
                continue;
            }
            let center = (xs[column].0 + xs[column].1) / 2.;
            let half = edge.width as f64 / 2.;
            fill(
                edge,
                Rect::new(
                    center - half,
                    ys[row].0 - if row == 0 { outer.top } else { 0. },
                    center + half,
                    ys[row + 1].1
                        + if row + 1 == rows.len() {
                            outer.bottom
                        } else {
                            0.
                        },
                ),
            );
        }
        for (&(row, logical_column), &edge) in &borders.horizontal {
            if row > rows.len() || logical_column >= cols.len() {
                continue;
            }
            let column = if rtl {
                cols.len() - 1 - logical_column
            } else {
                logical_column
            };
            if (row == 0 && table_border.border_top_style == BorderStyle::Hidden)
                || (row == rows.len() && table_border.border_bottom_style == BorderStyle::Hidden)
            {
                continue;
            }
            let center = (ys[row].0 + ys[row].1) / 2.;
            let half = edge.width as f64 / 2.;
            // Split horizontal junctions at the gutter center. Outer corners
            // include the entire outer gutter, as in the uniform-cell model.
            let start = if column == 0 {
                xs[column].0 - outer.left
            } else {
                (xs[column].0 + xs[column].1) / 2.
            };
            let end = if column + 1 == cols.len() {
                xs[column + 1].1 + outer.right
            } else {
                (xs[column + 1].0 + xs[column + 1].1) / 2.
            };
            fill(edge, Rect::new(start, center - half, end, center + half));
        }
    }
}
