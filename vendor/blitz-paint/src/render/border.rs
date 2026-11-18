use anyrender::PaintScene;
use blitz_dom::CopyStyleGetters;
use kurbo::{BezPath, Cap, Circle, Insets, Join, PathEl, Point, Rect, Shape as _, Stroke, Vec2};
use peniko::{Color, Fill};
use smallvec::SmallVec;
use style::values::computed::BorderStyle;

use crate::{
    color::{ToColorColor as _, contrast_ratio},
    kurbo_css::{CssBox, Edge},
    render::ElementCx,
};

/// A darker version of a colour (mirrors WebKit/Blink's `Color::Dark`): the
/// brightest channel is reduced by a fixed amount and the others scaled to match,
/// preserving hue.
// Copyright (C) 2003, 2004, 2005, 2006, 2008 Apple Inc. All rights reserved.
// Chromium color shading quantizes with the representable value just below 256.
// Adapted arithmetic retains its BSD terms in ../LICENSE.apple-color.
fn quantize_shade(value: f32) -> f32 {
    let scale = f32::from_bits(256.0_f32.to_bits() - 1);
    (value.clamp(0.0, 1.0) * scale) as u32 as f32 / 255.0
}

fn darken(color: Color) -> Color {
    let [r, g, b, a] = color.components;
    let v = r.max(g).max(b);
    if v == 0.0 {
        return color;
    }
    let multiplier = ((v - 0.33) / v).max(0.0);
    Color::new([
        quantize_shade(r * multiplier),
        quantize_shade(g * multiplier),
        quantize_shade(b * multiplier),
        a,
    ])
}

/// A lighter version of a colour (mirrors WebKit/Blink's `Color::Light`): the
/// brightest channel is increased by a fixed amount and the others scaled to
/// match, preserving hue.
fn lighten(color: Color) -> Color {
    let [r, g, b, a] = color.components;
    let v = r.max(g).max(b);
    if v == 0.0 {
        // Pure black: WebKit uses a fixed dark grey (0x545454).
        return Color::new([0.33, 0.33, 0.33, a]);
    }
    let multiplier = (v + 0.33).min(1.0) / v;
    Color::new([
        quantize_shade(r * multiplier),
        quantize_shade(g * multiplier),
        quantize_shade(b * multiplier),
        a,
    ])
}

/// The colour of a single edge of an `inset`/`outset` border.
///
/// An `outset` border is raised: the top/left edges use the "lighter" shade and
/// the bottom/right edges the "darker" shade (`inset` is the reverse). The exact
/// shading matches Chrome's `CalculateInsetOutsetColor`:
///
/// * The darker edge is always darkened.
/// * The lighter edge keeps the base colour unchanged, *except* for colours dark
///   enough that the darkened edge wouldn't have enough contrast against the base
///   colour, which are lightened instead so the bevel stays visible.
pub(super) fn beveled_edge_color(color: Color, edge: Edge, inset: bool) -> Color {
    let top_or_left = matches!(edge, Edge::Top | Edge::Left);
    let should_darken = top_or_left == inset;

    let dark_color = darken(color);
    if should_darken {
        return dark_color;
    }

    // Lighter edge. Chrome uses the base colour as-is unless the colour is dark
    // enough that the darkened edge lacks contrast against it. The red/green
    // shortcut mirrors Chrome: for those colours the contrast is always
    // sufficient, so the base colour is used without a contrast check.
    let [r, g, _, _] = color.components;
    if r >= 150.0 / 255.0 || g >= 92.0 / 255.0 {
        return color;
    }

    // Lighten only when the darkened edge would be too low-contrast to read.
    const MIN_CONTRAST_RATIO: f32 = 1.75;
    if contrast_ratio(color, dark_color) < MIN_CONTRAST_RATIO {
        lighten(color)
    } else {
        color
    }
}

/// The (outer half, inner half) colours of a single edge of a `groove`/`ridge`
/// border.
///
/// A `groove` looks carved into the page: its outer half is shaded like `inset`
/// and its inner half like `outset`. `ridge` is the reverse (raised).
pub(super) fn grooved_edge_colors(color: Color, edge: Edge, ridge: bool) -> (Color, Color) {
    let outer = beveled_edge_color(color, edge, !ridge);
    let inner = beveled_edge_color(color, edge, ridge);
    (outer, inner)
}

/// Return `count` points spaced `spacing` apart (by arc length) along `path`,
/// starting at its beginning. Used to place dots along a rounded border.
fn sample_points_along_path(path: &BezPath, spacing: f64, count: usize) -> Vec<Point> {
    // Flatten to a polyline so we can walk it by arc length.
    let mut poly: Vec<Point> = Vec::new();
    kurbo::flatten(path.iter(), 0.01, |el| match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => poly.push(p),
        PathEl::ClosePath => {
            if let Some(&first) = poly.first() {
                poly.push(first);
            }
        }
        _ => {}
    });

    let mut points = Vec::with_capacity(count);
    if poly.len() < 2 {
        return points;
    }

    let mut seg = 0;
    let mut seg_start = 0.0;
    let mut seg_len = (poly[1] - poly[0]).hypot();
    for i in 0..count {
        let target = i as f64 * spacing;
        while seg + 2 < poly.len() && target > seg_start + seg_len {
            seg_start += seg_len;
            seg += 1;
            seg_len = (poly[seg + 1] - poly[seg]).hypot();
        }
        let t = if seg_len > 0.0 {
            ((target - seg_start) / seg_len).clamp(0.0, 1.0)
        } else {
            0.0
        };
        points.push(poly[seg].lerp(poly[seg + 1], t));
    }
    points
}

fn browser_contour(mut path: BezPath) -> BezPath {
    let elements = path.elements();
    let Some(PathEl::MoveTo(origin)) = elements.first().copied() else {
        return path;
    };
    let Some(first_line) = elements
        .iter()
        .position(|el| matches!(el, PathEl::LineTo(_)))
    else {
        path.close_path();
        return path;
    };
    if first_line <= 1 {
        path.close_path();
        return path;
    }
    let start = match elements[first_line - 1] {
        PathEl::CurveTo(_, _, p) | PathEl::QuadTo(_, p) | PathEl::LineTo(p) | PathEl::MoveTo(p) => {
            p
        }
        PathEl::ClosePath => origin,
    };
    let mut reordered = BezPath::new();
    reordered.move_to(start);
    reordered.extend(elements[first_line..].iter().copied());
    reordered.line_to(origin);
    reordered.extend(elements[1..first_line].iter().copied());
    reordered.close_path();
    reordered
}

impl ElementCx<'_, '_> {
    /// Draw all borders for a node
    pub(super) fn draw_border_unclipped(&self, scene: &mut impl PaintScene) {
        let collapsed_cell = self.node.layout_parent.get().and_then(|id| self.context.dom.get_node(id))
            .and_then(|parent| parent.element_data()).is_some_and(|element| {
                matches!(&element.special_data, blitz_dom::node::SpecialElementData::TableRoot(table)
                    if table.collapsed_cell_styles.contains_key(&self.node.id))
            });
        if collapsed_cell || super::table::hides_empty_cell(self.context.dom, self.node) {
            return;
        }
        if self.draw_border_image(scene) {
            return;
        }
        let style = &*self.style;
        let border = style.get_border();
        let current_color = style.clone_color();

        if !self.frame.has_border_radius() {
            let style = border.border_top_style;
            let same_style = [
                border.border_right_style,
                border.border_bottom_style,
                border.border_left_style,
            ]
            .iter()
            .all(|s| *s == style);
            let color = border
                .border_top_color
                .resolve_to_absolute(&current_color)
                .as_srgb_color();
            let same_color = [
                &border.border_right_color,
                &border.border_bottom_color,
                &border.border_left_color,
            ]
            .iter()
            .all(|c| c.resolve_to_absolute(&current_color).as_srgb_color() == color);
            let bw = self.frame.border_width;
            let uniform_width = bw.x0 > 0.0 && bw.x0 == bw.y0 && bw.x0 == bw.x1 && bw.x0 == bw.y1;
            if same_style
                && same_color
                && uniform_width
                && matches!(style, BorderStyle::Dashed | BorderStyle::Dotted)
                && (style == BorderStyle::Dashed || bw.x0 <= 3.0 * self.scale)
            {
                let mut path = BezPath::new();
                for edge in [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left] {
                    path.extend(
                        self.straight_border_pattern(edge, style == BorderStyle::Dotted)
                            .elements()
                            .iter()
                            .copied(),
                    );
                }
                scene.fill(Fill::NonZero, self.transform, color, None, &path);
                return;
            }
        }

        // Fast path: a uniform-width, single-color solid border whose corners are
        // all circular arcs (equal x and y radius — sharp rectangles, ordinary
        // rounded rectangles and circles all qualify) can be drawn as a single
        // stroke along its center-line instead of a fill per edge. Besides being
        // faster, this works around Vello seam artifacts at the quarter-arc joins
        // when a thin circular/elliptical border is drawn as the
        // border-box/padding-box annulus.
        //
        // A full ellipse (`border-radius: 50%` on a non-square box) has the same
        // seam problem but no rounded-rect representation, so it is stroked as an
        // ellipse.
        let all_solid = border.border_top_style == BorderStyle::Solid
            && border.border_right_style == BorderStyle::Solid
            && border.border_bottom_style == BorderStyle::Solid
            && border.border_left_style == BorderStyle::Solid;
        if all_solid {
            let color = border
                .border_top_color
                .resolve_to_absolute(&current_color)
                .as_srgb_color();
            let same_color = [
                &border.border_right_color,
                &border.border_bottom_color,
                &border.border_left_color,
            ]
            .iter()
            .all(|c| c.resolve_to_absolute(&current_color).as_srgb_color() == color);
            let bw = self.frame.border_width;
            let uniform_width = bw.x0 == bw.y0 && bw.x0 == bw.x1 && bw.x0 == bw.y1 && bw.x0 > 0.0;
            if same_color && uniform_width && color.components[3] > 0.0 {
                let bb = self.frame.border_box;
                let pb = self.frame.padding_box;
                let width = (bb.width() - pb.width()) / 2.0;
                // Miter join keeps sharp corners square. It has no effect on rounded
                // corners, whose arcs meet the straight edges tangentially.
                let stroke = Stroke::new(width).with_join(Join::Miter);

                if self.frame.is_uniform_corner_border() {
                    let radii = self.frame.border_radii;
                    // The stroke matches the CSS border shape only when each corner is
                    // sharp or its radius is at least the border width (outer radius
                    // `r`, inner radius `r - width`, concentric). For 0 < r < width CSS
                    // draws a rounded outer edge against a sharp inner corner, which a
                    // stroke can't represent — leave those to the per-edge fill.
                    let is_exact = |r: Vec2| r.x == 0.0 || r.x >= width;
                    let stroke_is_exact = is_exact(radii.top_left)
                        && is_exact(radii.top_right)
                        && is_exact(radii.bottom_right)
                        && is_exact(radii.bottom_left);
                    if stroke_is_exact {
                        let half_width = width / 2.0;
                        // Center-line rect, and corner radii shrunk to match.
                        let centerline_box = bb - Insets::uniform(half_width);
                        let centerline_radius = |r: Vec2| (r.x - half_width).max(0.0);
                        let radii = kurbo::RoundedRectRadii::new(
                            centerline_radius(radii.top_left),
                            centerline_radius(radii.top_right),
                            centerline_radius(radii.bottom_right),
                            centerline_radius(radii.bottom_left),
                        );
                        let rr = kurbo::RoundedRect::from_rect(centerline_box, radii);
                        scene.stroke(&stroke, self.transform, color, None, &rr);
                        return;
                    }
                } else if self.frame.is_elliptical_border() {
                    // Center-line radii, straight from the boxes.
                    let rx = (bb.width() + pb.width()) / 4.0;
                    let ry = (bb.height() + pb.height()) / 4.0;
                    let e = kurbo::Ellipse::new(bb.center(), (rx, ry), 0.0);
                    scene.stroke(&stroke, self.transform, color, None, &e);
                    return;
                }
            }
        }

        // (colour, path) pairs to be filled. Several entries may share a colour;
        // they are grouped before filling so that adjacent same-coloured regions
        // are drawn together, avoiding anti-aliasing seams between them.
        //
        // At most 8 entries: 4 edges, each of which can contribute 2 (the outer
        // and inner halves of a `groove`/`ridge`).
        let mut borders: SmallVec<[(Color, BezPath); 8]> = SmallVec::new();

        for &edge in &[Edge::Top, Edge::Right, Edge::Bottom, Edge::Left] {
            let (color, edge_style) = match edge {
                Edge::Top => (&border.border_top_color, border.border_top_style),
                Edge::Right => (&border.border_right_color, border.border_right_style),
                Edge::Bottom => (&border.border_bottom_color, border.border_bottom_style),
                Edge::Left => (&border.border_left_color, border.border_left_style),
            };
            let color = color.resolve_to_absolute(&current_color).as_srgb_color();

            if color.components[3] <= 0.0 {
                continue;
            }

            match edge_style {
                // `none`/`hidden` produce a zero-width border during layout, but
                // guard against drawing them anyway.
                BorderStyle::None | BorderStyle::Hidden => {}

                // Dashed and dotted edges are drawn immediately as their own
                // (clipped) shapes rather than being batched with the solid edges.
                BorderStyle::Dotted => self.draw_dotted_border_edge(scene, edge, color),
                BorderStyle::Dashed => self.draw_dashed_border_edge(scene, edge, color),

                // A double border is two solid lines separated by a gap, splitting
                // the border width into three equal parts (outer line / gap / inner
                // line). Both lines share a color, so both rings are placed into a
                // single path.
                BorderStyle::Double => {
                    // Needs at least 3px (one device pixel per line and per gap) to
                    // render as two lines; thinner borders fall back to a solid
                    // fill, matching browser behaviour.
                    let path = if self.edge_width(edge) < 3.0 * self.scale {
                        self.frame.border_edge_shape(edge)
                    } else {
                        let mut path = self
                            .frame
                            .border_slice(0.0, 1.0 / 3.0)
                            .border_edge_shape(edge);
                        path.extend(
                            &self
                                .frame
                                .border_slice(2.0 / 3.0, 1.0)
                                .border_edge_shape(edge),
                        );
                        path
                    };
                    borders.push((color, path));
                }

                // `inset`/`outset` are solid, but each edge is shaded lighter or
                // darker to give a bevelled (3D) appearance.
                BorderStyle::Inset | BorderStyle::Outset => {
                    let inset = edge_style == BorderStyle::Inset;
                    let shade = beveled_edge_color(color, edge, inset);
                    borders.push((shade, self.frame.border_edge_shape(edge)));
                }

                // `groove`/`ridge` split each edge into an outer and an inner half,
                // each shaded as if it were `inset`/`outset`, producing a carved or
                // raised ridge.
                BorderStyle::Groove | BorderStyle::Ridge => {
                    let ridge = edge_style == BorderStyle::Ridge;
                    let (outer, inner) = grooved_edge_colors(color, edge, ridge);
                    borders.push((
                        outer,
                        self.frame.border_slice(0.0, 0.5).border_edge_shape(edge),
                    ));
                    borders.push((
                        inner,
                        self.frame.border_slice(0.5, 1.0).border_edge_shape(edge),
                    ));
                }

                // Solid fills the whole edge region with the border colour.
                BorderStyle::Solid => {
                    borders.push((color, self.frame.border_edge_shape(edge)));
                }
            }
        }

        if borders.is_empty() {
            return;
        }

        // Group together identical colors by sorting, then fill each group as a
        // single path.
        borders.sort_unstable_by(|a, b| {
            a.0.components
                .partial_cmp(&b.0.components)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut start = 0;
        while start < borders.len() {
            let color = borders[start].0;
            let mut path = std::mem::take(&mut borders[start].1);
            let mut next = start + 1;
            while next < borders.len() && borders[next].0 == color {
                path.extend(&borders[next].1);
                next += 1;
            }
            scene.fill(Fill::NonZero, self.transform, color, None, &path);
            start = next;
        }
    }

    /// The border width (in device pixels) of a single edge.
    fn edge_width(&self, edge: Edge) -> f64 {
        match edge {
            Edge::Top => self.frame.border_width.y0,
            Edge::Bottom => self.frame.border_width.y1,
            Edge::Left => self.frame.border_width.x0,
            Edge::Right => self.frame.border_width.x1,
        }
    }

    pub(super) fn straight_border_pattern(&self, edge: Edge, dotted: bool) -> BezPath {
        let bb = self.frame.border_box;
        let thickness = self.edge_width(edge);
        let (rect, vertical) = match edge {
            Edge::Top => (Rect::new(bb.x0, bb.y0, bb.x1, bb.y0 + thickness), false),
            Edge::Bottom => (Rect::new(bb.x0, bb.y1 - thickness, bb.x1, bb.y1), false),
            Edge::Left => (Rect::new(bb.x0, bb.y0, bb.x0 + thickness, bb.y1), true),
            Edge::Right => (Rect::new(bb.x1 - thickness, bb.y0, bb.x1, bb.y1), true),
        };
        super::line_pattern::straight_path(rect, vertical, dotted, self.scale)
    }

    /// Chromium truncates each half-width in CSS pixels before constructing
    /// the curved pattern contour. Odd widths therefore use an integral inset.
    fn pattern_centerline(&self) -> BezPath {
        let width = self.frame.border_width;
        let half = |value: f64| (value / self.scale * 0.5).floor() * self.scale;
        let frame = CssBox::new(
            self.frame.border_box,
            Insets::new(
                half(width.x0),
                half(width.y0),
                half(width.x1),
                half(width.y1),
            ),
            Insets::ZERO,
            0.0,
            self.frame.border_radii,
        );
        browser_contour(frame.padding_box_path())
    }

    /// Draw a single `dashed` border edge.
    ///
    /// The dashes are produced by stroking the centre line of the border with a
    /// dash pattern (butt caps give them flat, square ends). Everything is clipped
    /// to the edge's region (the same trapezoid used by the solid path) so corners
    /// are mitred, adjacent edges of different colors don't overlap, and any
    /// border-radius is respected.
    pub(super) fn draw_dashed_border_edge(
        &self,
        scene: &mut impl PaintScene,
        edge: Edge,
        color: Color,
    ) {
        let thickness = self.edge_width(edge);
        if thickness <= 0.0 {
            return;
        }

        if !self.frame.has_border_radius() {
            let path = self.straight_border_pattern(edge, false);
            scene.push_clip_layer(
                Fill::NonZero,
                self.transform,
                &self.frame.border_edge_shape(edge),
            );
            scene.fill(Fill::NonZero, self.transform, color, None, &path);
            scene.pop_layer();
            return;
        }
        let centerline = self.pattern_centerline();
        let length = (centerline.perimeter(0.01) / self.scale).floor();
        if length <= 0.0 {
            return;
        }
        let (dash, gap) = super::line_pattern::dashes(length, thickness / self.scale, true)
            .unwrap_or((length, 0.0));
        let stroke = Stroke::new(2.2 * thickness)
            .with_caps(Cap::Butt)
            .with_dashes(0.0, [dash * self.scale, gap * self.scale]);
        let clip = self.frame.border_edge_shape(edge);
        scene.push_clip_layer(Fill::NonZero, self.transform, &clip);
        scene.stroke(&stroke, self.transform, color, None, &centerline);
        scene.pop_layer();
    }

    /// Draw a single `dotted` border edge.
    ///
    /// Dots are filled circles (diameter == border thickness). Unlike dashes they
    /// can't be produced by stroking, because kurbo doesn't emit zero-length dashes
    /// (so a round-capped dash pattern would render nothing); drawing them
    /// explicitly also lets us anchor a dot in each square corner. Everything is
    /// clipped to the edge's region, as for the other styles.
    pub(super) fn draw_dotted_border_edge(
        &self,
        scene: &mut impl PaintScene,
        edge: Edge,
        color: Color,
    ) {
        let thickness = self.edge_width(edge);
        if thickness <= 0.0 {
            return;
        }
        if !self.frame.has_border_radius() {
            let path = self.straight_border_pattern(edge, true);
            scene.push_clip_layer(
                Fill::NonZero,
                self.transform,
                &self.frame.border_edge_shape(edge),
            );
            scene.fill(Fill::NonZero, self.transform, color, None, &path);
            scene.pop_layer();
            return;
        }
        let radius = thickness / 2.0;
        let centerline = self.pattern_centerline();
        let perimeter = centerline.perimeter(0.01);
        if perimeter <= 0.0 {
            return;
        }
        if thickness <= 3.0 * self.scale {
            let stroke = Stroke::new(2.2 * thickness)
                .with_caps(Cap::Butt)
                .with_dashes(0.0, [thickness, thickness]);
            scene.push_clip_layer(
                Fill::NonZero,
                self.transform,
                &self.frame.border_edge_shape(edge),
            );
            scene.stroke(&stroke, self.transform, color, None, &centerline);
            scene.pop_layer();
            return;
        }
        let length = (perimeter / self.scale).floor();
        let width = thickness / self.scale;
        let spacing =
            (super::line_pattern::best_gap(length, width, width, true) + width - 0.01) * self.scale;
        // A closed contour has one mark at the seam; do not duplicate its
        // antialiased coverage with an almost coincident final mark.
        let count = (perimeter / spacing).floor().clamp(1.0, 16384.0) as usize;
        let mut path = BezPath::new();
        for center in sample_points_along_path(&centerline, spacing, count) {
            path.extend(Circle::new(center, radius).path_elements(0.01));
        }

        let clip = self.frame.border_edge_shape(edge);
        scene.push_clip_layer(Fill::NonZero, self.transform, &clip);
        scene.fill(Fill::NonZero, self.transform, color, None, &path);
        scene.pop_layer();
    }
}
