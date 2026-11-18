// Copyright (C) 2013 Google Inc. All rights reserved.
// Copyright 2015 The Chromium Authors
// Adapted pattern arithmetic: see LICENSE.chromium and LICENSE.google-stroke.
//! Browser-style dash spacing and integral thin-dot endpoint treatment.

/// Pick the integral dash count whose gap is closest to the nominal gap.
/// Dash lengths stay fixed; open paths include a complete dash at both ends.
pub(super) fn best_gap(length: f64, dash: f64, gap: f64, closed: bool) -> f64 {
    let available = if closed { length } else { length + gap };
    let count = (available / (dash + gap)).floor();
    let gaps = if closed { count } else { count - 1.0 };
    let small = (length - count * dash) / gaps;
    let large = (length - (count + 1.0) * dash) / (gaps + 1.0);
    if large <= 0.0 || (small - gap).abs() < (large - gap).abs() {
        small
    } else {
        large
    }
}

/// None means the line is too short for a gap and should be solid.
pub(super) fn dashes(length: f64, width: f64, closed: bool) -> Option<(f64, f64)> {
    let (dash, gap) = if width >= 3.0 {
        (2.0 * width, width)
    } else {
        (3.0 * width, 2.0 * width)
    };
    if length <= 2.0 * dash {
        return None;
    }
    let pair = 2.0 * dash + if closed { 2.0 * gap } else { gap };
    if length <= pair {
        let multiplier = length / pair;
        return Some((dash * multiplier, gap * multiplier));
    }
    Some((dash, best_gap(length, dash, gap, closed)))
}

/// Fully covered rectangles for 1-, 2-, and 3-CSS-pixel dots. Endpoint
/// adjustments preserve full dots while keeping every intervening edge integral.
pub(super) fn thin_dots(length: i32, width: i32) -> Vec<(f64, f64)> {
    let length = length.max(0);
    let width = width.clamp(1, 3);
    let m4 = length % 4;
    let m6 = length % 6;
    let mut start_dot = false;
    let mut start_growth = 0;
    let mut offset = 0;
    let mut end_dot = false;
    let mut end_growth = 0;
    if (width == 1 && length % 2 == 0) || (width == 3 && m6 == 0) {
        start_dot = true;
        start_growth = 1;
        offset = 1;
    }
    if (width == 2 && (m4 == 0 || m4 == 1)) || (width == 3 && (m6 == 1 || m6 == 2)) {
        start_dot = true;
        offset = -1;
    }
    if (width == 2 && m4 == 0) || (width == 3 && m6 == 1) {
        end_dot = true;
    }
    if (width == 2 && m4 == 3) || (width == 3 && (m6 == 4 || m6 == 5)) {
        start_dot = true;
        offset = 1;
    }
    if width == 3 && m6 == 5 {
        end_dot = true;
    } else if width == 3 && m6 == 0 {
        end_dot = true;
        end_growth = 1;
    }
    let mut spans = Vec::new();
    let mut start = 0;
    let mut end = length;
    if start_dot {
        spans.push((0.0, f64::from(width + start_growth)));
        start += 2 * width + offset;
    }
    if end_dot {
        spans.push((f64::from(length - width - end_growth), f64::from(length)));
        end -= width + end_growth + 1;
    }
    if length <= 2 * width {
        if end > start {
            spans.push((f64::from(start), f64::from(end)));
        }
    } else {
        let dash = if length <= 3 * width {
            f64::from(length) / 3.0
        } else {
            f64::from(width)
        };
        let mut position = f64::from(start);
        while position < f64::from(end) && spans.len() < 16384 {
            spans.push((position, (position + dash).min(f64::from(end))));
            position += 2.0 * dash;
        }
    }
    spans
}

/// A straight rule or border side, expressed as one fill to avoid seams where
/// same-color sides meet. Geometry is measured in CSS pixels before scaling.
pub(super) fn straight_path(
    rect: kurbo::Rect,
    vertical: bool,
    dotted: bool,
    scale: f64,
) -> kurbo::BezPath {
    use kurbo::{BezPath, Circle, Rect, Shape as _};
    let (length, width) = if vertical {
        (rect.height() / scale, rect.width() / scale)
    } else {
        (rect.width() / scale, rect.height() / scale)
    };
    let mut path = BezPath::new();
    if length <= 0.0 || width <= 0.0 {
        return path;
    }
    let segment = |start: f64, end: f64| {
        if vertical {
            Rect::new(
                rect.x0,
                rect.y0 + start * scale,
                rect.x1,
                rect.y0 + end * scale,
            )
        } else {
            Rect::new(
                rect.x0 + start * scale,
                rect.y0,
                rect.x0 + end * scale,
                rect.y1,
            )
        }
    };
    if dotted && width <= 3.0 {
        for (start, end) in thin_dots(length.round() as i32, width.round() as i32) {
            path.extend(segment(start.max(0.0), end.min(length)).path_elements(0.01));
        }
    } else if dotted {
        let step = if length < 2.0 * width {
            2.0 * width
        } else {
            best_gap(length, width, width, false) + width - 0.01
        };
        let mut position = width * 0.5;
        let mut count = 0;
        while position <= length - width * 0.5 + 0.0001 && count < 16384 {
            let center = if vertical {
                (rect.center().x, rect.y0 + position * scale)
            } else {
                (rect.x0 + position * scale, rect.center().y)
            };
            path.extend(Circle::new(center, width * scale * 0.5).path_elements(0.01));
            position += step;
            count += 1;
        }
    } else if let Some((dash, gap)) = dashes(length, width, false) {
        let mut position = 0.0;
        let mut count = 0;
        while position < length && count < 16384 {
            path.extend(segment(position, (position + dash).min(length)).path_elements(0.01));
            position += dash + gap;
            count += 1;
        }
    } else {
        path.extend(rect.path_elements(0.01));
    }
    path
}
