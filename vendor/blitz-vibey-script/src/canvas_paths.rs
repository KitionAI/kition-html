//! Geometry shared by retained and current Canvas 2D paths.
use peniko::kurbo::{Affine, Arc, BezPath, ParamCurveNearest, PathEl, Point, Shape};
use serde_json::Value;

pub(crate) fn arc_to(path: &mut BezPath, args: &Value, transform: Affine) {
    let n = |i| args[i].as_f64().unwrap_or(0.0);
    let corner = Point::new(n(0), n(1));
    let end = Point::new(n(2), n(3));
    let radius = n(4);
    let Some(current) = path.current_position() else {
        path.move_to(transform * corner);
        return;
    };
    // The current point was recorded under its own transform. Recover it in
    // this command's coordinate system before constructing the tangent circle.
    if transform.determinant() == 0. {
        path.line_to(transform * corner);
        return;
    }
    let start = transform.inverse() * current;
    let incoming = start - corner;
    let outgoing = end - corner;
    if radius == 0. || incoming.hypot() == 0. || outgoing.hypot() == 0. {
        path.line_to(transform * corner);
        return;
    }
    let a = incoming.normalize();
    let b = outgoing.normalize();
    let cross = a.cross(b);
    if cross.abs() < 1e-14 {
        path.line_to(transform * corner);
        return;
    }
    let dot = a.dot(b).clamp(-1., 1.);
    let distance = radius * (1. + dot) / cross.abs();
    let tangent = corner + a * distance;
    let center = tangent + peniko::kurbo::Vec2::new(-a.y, a.x) * (radius * cross.signum());
    let arc = Arc::new(
        center,
        (radius, radius),
        (tangent - center).atan2(),
        (-cross).atan2(-dot),
        0.,
    );
    for element in (transform * arc.to_path(0.01)).elements() {
        match *element {
            PathEl::MoveTo(point) => path.line_to(point),
            other => path.push(other),
        }
    }
}

pub(crate) fn round_rect(args: &Value) -> BezPath {
    let n = |i| args[i].as_f64().unwrap_or(0.0);
    let (x, y, w, h) = (n(0), n(1), n(2), n(3));
    let (width, height) = (w.abs(), h.abs());
    let mut radii = [(n(4), n(5)), (n(6), n(7)), (n(8), n(9)), (n(10), n(11))];
    let ratio = |length: f64, total: f64| if total > 0. { length / total } else { 1. };
    let factor = 1_f64
        .min(ratio(width, radii[0].0 + radii[1].0))
        .min(ratio(width, radii[2].0 + radii[3].0))
        .min(ratio(height, radii[0].1 + radii[3].1))
        .min(ratio(height, radii[1].1 + radii[2].1));
    for (rx, ry) in &mut radii {
        *rx *= factor;
        *ry *= factor;
    }
    let [a, b, c, d] = radii;
    let mut path = BezPath::new();
    path.move_to((a.0, 0.));
    for ((rx, ry), center, start, edge) in [
        (
            b,
            (width - b.0, b.1),
            -std::f64::consts::FRAC_PI_2,
            (width, 0.),
        ),
        (c, (width - c.0, height - c.1), 0., (width, height)),
        (
            d,
            (d.0, height - d.1),
            std::f64::consts::FRAC_PI_2,
            (0., height),
        ),
        (a, (a.0, a.1), std::f64::consts::PI, (0., 0.)),
    ] {
        if rx == 0. || ry == 0. {
            path.line_to(edge);
            continue;
        }
        let arc = Arc::new(center, (rx, ry), start, std::f64::consts::FRAC_PI_2, 0.);
        for element in arc.path_elements(0.01) {
            match element {
                PathEl::MoveTo(point) => path.line_to(point),
                other => path.push(other),
            }
        }
    }
    path.close_path();
    let flip = Affine::translate((x, y))
        * Affine::scale_non_uniform(if w < 0. { -1. } else { 1. }, if h < 0. { -1. } else { 1. });
    flip * path
}

pub(crate) fn svg(text: &str) -> Result<BezPath, String> {
    if text.len() > 65536 {
        return Err("SVG path input limit exceeded".into());
    }
    // Invalid SVG syntax produces an empty retained path; browser prefix parsing
    // of malformed input is outside the supported subset.
    let path = BezPath::from_svg(text).unwrap_or_default();
    if path.elements().len() > 16384 {
        return Err("SVG path command limit exceeded".into());
    }
    let valid = |p: Point| {
        p.x.is_finite() && p.y.is_finite() && p.x.abs() <= 10_000_000. && p.y.abs() <= 10_000_000.
    };
    if !path.elements().iter().all(|element| match *element {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => valid(p),
        PathEl::QuadTo(a, b) => valid(a) && valid(b),
        PathEl::CurveTo(a, b, c) => valid(a) && valid(b) && valid(c),
        PathEl::ClosePath => true,
    }) {
        return Err("SVG path exceeds the preview coordinate limit".into());
    }
    Ok(path)
}

pub(crate) fn contains(path: &BezPath, point: Point, evenodd: bool) -> bool {
    let mut closed = BezPath::new();
    let mut open = false;
    for element in path.elements() {
        match element {
            PathEl::MoveTo(_) => {
                if open {
                    closed.close_path();
                }
                open = true;
            }
            PathEl::ClosePath => open = false,
            _ => {}
        }
        closed.push(*element);
    }
    if open {
        closed.close_path();
    }
    let winding = closed.winding(point);
    (if evenodd {
        winding % 2 != 0
    } else {
        winding != 0
    }) || closed
        .segments()
        .any(|segment| segment.nearest(point, 1e-6).distance_sq <= 1e-12)
}
