//! CPU-backed Canvas 2D drawing. The host browser never produces these pixels.
use crate::dom::{dom_ctx, js_str, this_node_id, to_rust_string};
use anyrender::{Paint, PaintScene, render_to_buffer};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::{local_name, node::RasterImageData};
use boa_engine::{Context, JsNativeError, JsResult, JsValue};
use peniko::kurbo::{
    Affine, Arc as EllipseArc, BezPath, Cap, Join, PathEl, Point, Rect, Shape, Stroke,
};
use peniko::{
    BlendMode, Color, Compose, Fill, Gradient, ImageAlphaType, ImageBrush, ImageData, ImageFormat,
    ImageQuality, ImageSampler, Mix,
};
use serde_json::{Value, json};
use std::sync::Arc;

const MAX_PIXELS: u64 = 16_777_216;
fn error(message: &str) -> boa_engine::JsError {
    JsNativeError::range()
        .with_message(message.to_owned())
        .into()
}
fn number(value: &Value) -> f64 {
    value.as_f64().filter(|n| n.is_finite()).unwrap_or(0.0)
}
fn bounded(value: &Value) -> bool {
    match value {
        Value::Number(n) => n
            .as_f64()
            .is_some_and(|n| n.is_finite() && n.abs() <= 10_000_000.0),
        Value::Array(items) => items.iter().all(bounded),
        Value::Object(items) => items.values().all(bounded),
        _ => true,
    }
}
fn affine(value: &Value) -> Affine {
    if value.as_array().is_some_and(|v| v.len() == 6) {
        Affine::new(std::array::from_fn(|i| number(&value[i])))
    } else {
        Affine::IDENTITY
    }
}
fn rule(value: &Value) -> Fill {
    if value.as_str() == Some("evenodd") {
        Fill::EvenOdd
    } else {
        Fill::NonZero
    }
}
fn color(value: &Value) -> Color {
    color::parse_color(value.as_str().unwrap_or("black"))
        .map(|c| c.to_alpha_color())
        .unwrap_or(Color::TRANSPARENT)
}
pub(crate) fn dimension(value: Option<&str>, fallback: u32) -> u32 {
    value
        .map(str::trim)
        .map(|s| s.strip_prefix('+').unwrap_or(s))
        .and_then(|s| {
            s.split(|c: char| !c.is_ascii_digit())
                .next()?
                .parse::<u32>()
                .ok()
        })
        .filter(|&n| n <= 2147483647)
        .unwrap_or(fallback)
}
pub(crate) fn image_brush(image: &RasterImageData) -> ImageBrush {
    ImageBrush {
        image: ImageData {
            data: image.data.clone(),
            width: image.width,
            height: image.height,
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
        },
        sampler: ImageSampler {
            quality: ImageQuality::Low,
            ..Default::default()
        },
    }
}

pub(crate) fn request(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(args.first().unwrap_or(&JsValue::undefined()))?;
    let request = to_rust_string(args.get(1).unwrap_or(&JsValue::undefined()), context)?;
    let request: Value =
        serde_json::from_str(&request).map_err(|_| error("Invalid canvas command"))?;
    let (width, height, revision, previous) = {
        let doc = ctx.doc.borrow();
        let element = doc
            .get_node(id)
            .and_then(|n| n.element_data())
            .ok_or_else(|| error("Canvas no longer exists"))?;
        if element.name.local != local_name!("canvas") {
            return Err(JsNativeError::typ()
                .with_message("Expected a canvas")
                .into());
        }
        (
            dimension(element.attr(local_name!("width")), 300),
            dimension(element.attr(local_name!("height")), 150),
            element.canvas_revision,
            element.canvas_bitmap.clone(),
        )
    };
    let op = request["op"].as_str().unwrap_or("");
    let source = if op == "drawImage" {
        let source =
            crate::canvas_images::source(&ctx, args.get(2).unwrap_or(&JsValue::undefined()))?;
        if source.is_none() {
            return Ok(js_str("null"));
        }
        source
    } else {
        None
    };
    if op == "info" {
        return Ok(js_str(
            &json!({"width":width,"height":height,"revision":revision}).to_string(),
        ));
    }
    if op == "spacing" {
        return Ok(js_str(
            &crate::canvas_spacing::parse(request["value"].as_str().unwrap_or("")).to_string(),
        ));
    }
    if op == "color" {
        let parsed = request["value"]
            .as_str()
            .and_then(|v| color::parse_color(v).ok())
            .map(|color| {
                let rgba = color.to_alpha_color::<color::Srgb>().to_rgba8();
                if rgba.a == 255 {
                    format!("#{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b)
                } else {
                    format!(
                        "rgba({}, {}, {}, {})",
                        rgba.r,
                        rgba.g,
                        rgba.b,
                        rgba.a as f64 / 255.0
                    )
                }
            });
        return Ok(js_str(&json!(parsed).to_string()));
    }
    let text = if matches!(op, "fillText" | "strokeText" | "measureText") {
        if !bounded(&request) {
            return Err(error(
                "Canvas geometry exceeds the preview coordinate limit",
            ));
        }
        let text = crate::canvas_text::shape(&ctx, &request)?;
        if op == "measureText" {
            return Ok(js_str(&text.metrics(&request["state"]).to_string()));
        }
        let property = if op == "strokeText" {
            "strokeStyle"
        } else {
            "fillStyle"
        };
        if !request["state"][property].is_string() {
            return Err(JsNativeError::typ()
                .with_message("Canvas text currently requires a solid paint")
                .into());
        }
        Some(text)
    } else {
        None
    };
    if width > 8192 || height > 8192 || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(error(
            "Canvas bitmap exceeds the 16-megapixel preview limit",
        ));
    }
    if matches!(op, "hitFill" | "hitStroke") {
        if !bounded(&request) {
            return Err(error(
                "Canvas geometry exceeds the preview coordinate limit",
            ));
        }
        let mut path = build_path(&request["path"])?;
        let mut point = Point::new(number(&request["args"][0]), number(&request["args"][1]));
        if op == "hitStroke" {
            let transform = affine(&request["state"]["transform"]);
            if transform.determinant() == 0. {
                return Ok(js_str("false"));
            }
            path = transform.inverse() * path;
            point = transform.inverse() * point;
            path = peniko::kurbo::stroke(
                path.elements().iter().copied(),
                &stroke_style(&request["state"]),
                &Default::default(),
                0.01,
            );
        }
        let inside = crate::canvas_paths::contains(
            &path,
            point,
            op == "hitFill" && request["rule"] == "evenodd",
        );
        return Ok(js_str(if inside { "true" } else { "false" }));
    }
    if op == "read" {
        let mut x = number(&request["args"][0]) as i64;
        let mut y = number(&request["args"][1]) as i64;
        let mut w = number(&request["args"][2]) as i64;
        let mut h = number(&request["args"][3]) as i64;
        if w < 0 {
            x = x.saturating_add(w);
            w = w.saturating_neg();
        }
        if h < 0 {
            y = y.saturating_add(h);
            h = h.saturating_neg();
        }
        if w <= 0 || h <= 0 || w > 8192 || h > 8192 || w as u64 * h as u64 > MAX_PIXELS {
            return Err(error("Invalid canvas readback dimensions"));
        }
        let mut data = vec![0u8; w as usize * h as usize * 4];
        if let Some(image) = previous {
            for row in 0..h {
                for col in 0..w {
                    let sx = x.saturating_add(col);
                    let sy = y.saturating_add(row);
                    if sx >= 0 && sy >= 0 && sx < i64::from(width) && sy < i64::from(height) {
                        let src = (sy as usize * width as usize + sx as usize) * 4;
                        let dest = (row as usize * w as usize + col as usize) * 4;
                        data[dest..dest + 4].copy_from_slice(&image.data.data()[src..src + 4]);
                    }
                }
            }
        }
        return Ok(js_str(
            &json!({"width":w,"height":h,"data":data}).to_string(),
        ));
    }
    if width == 0 || height == 0 {
        return Ok(js_str("null"));
    }
    let pixels = if op == "write" {
        write_pixels(&request, width, height, previous.as_ref())?
    } else {
        draw(
            &request,
            width,
            height,
            previous.as_ref(),
            source.as_ref(),
            text.as_ref(),
            ctx.state.borrow().canvas_scene_preparer,
        )?
    };
    let mut doc = ctx.doc.borrow_mut();
    if let Some(node) = doc.get_node_mut(id) {
        node.element_data_mut().unwrap().canvas_bitmap =
            Some(RasterImageData::new(width, height, Arc::new(pixels)));
        node.mark_ancestors_dirty();
    }
    Ok(js_str("null"))
}

fn write_pixels(
    request: &Value,
    width: u32,
    height: u32,
    previous: Option<&RasterImageData>,
) -> JsResult<Vec<u8>> {
    let sw = request["width"].as_u64().unwrap_or(0);
    let sh = request["height"].as_u64().unwrap_or(0);
    if sw == 0 || sh == 0 || sw > 8192 || sh > 8192 || sw * sh > MAX_PIXELS {
        return Err(error("Invalid image data dimensions"));
    }
    let Some(data) = request["data"]
        .as_array()
        .filter(|d| d.len() == (sw * sh * 4) as usize)
    else {
        return Err(error("Invalid image data length"));
    };
    let dx = number(&request["args"][0]) as i64;
    let dy = number(&request["args"][1]) as i64;
    let args = request["args"].as_array().map(Vec::len).unwrap_or(0);
    let (mut x, mut y, mut w, mut h) = if args >= 6 {
        (
            number(&request["args"][2]) as i64,
            number(&request["args"][3]) as i64,
            number(&request["args"][4]) as i64,
            number(&request["args"][5]) as i64,
        )
    } else {
        (0, 0, sw as i64, sh as i64)
    };
    if w < 0 {
        x = x.saturating_add(w);
        w = w.saturating_neg();
    }
    if h < 0 {
        y = y.saturating_add(h);
        h = h.saturating_neg();
    }
    let mut pixels = previous
        .map(|p| p.data.data().to_vec())
        .unwrap_or_else(|| vec![0; width as usize * height as usize * 4]);
    for sy in y.max(0)..y.saturating_add(h).min(sh as i64) {
        for sx in x.max(0)..x.saturating_add(w).min(sw as i64) {
            let tx = dx.saturating_add(sx);
            let ty = dy.saturating_add(sy);
            if tx >= 0 && ty >= 0 && tx < i64::from(width) && ty < i64::from(height) {
                let dest = (ty as usize * width as usize + tx as usize) * 4;
                let src = (sy as usize * sw as usize + sx as usize) * 4;
                for channel in 0..4 {
                    pixels[dest + channel] =
                        data[src + channel].as_u64().unwrap_or(0).min(255) as u8;
                }
            }
        }
    }
    Ok(pixels)
}

fn build_path(commands: &Value) -> JsResult<BezPath> {
    let mut path = BezPath::new();
    let Some(commands) = commands.as_array() else {
        return Ok(path);
    };
    for command in commands.iter().take(16384) {
        let args = &command[1];
        let transform = affine(&command[2]);
        let p = |i| transform * Point::new(number(&args[i]), number(&args[i + 1]));
        match command[0].as_str().unwrap_or("") {
            "svg" => {
                let svg = crate::canvas_paths::svg(args[0].as_str().unwrap_or(""))
                    .map_err(|message| error(&message))?;
                path.extend((transform * svg).elements().iter().copied());
            }
            "roundRect" => path.extend(
                (transform * crate::canvas_paths::round_rect(args))
                    .elements()
                    .iter()
                    .copied(),
            ),
            "move" => path.move_to(p(0)),
            "arcTo" => crate::canvas_paths::arc_to(&mut path, args, transform),
            "line" => {
                if path.elements().is_empty() {
                    path.move_to(p(0))
                } else {
                    path.line_to(p(0))
                }
            }
            "quadratic" => {
                if path.elements().is_empty() {
                    path.move_to(p(0));
                }
                path.quad_to(p(0), p(2));
            }
            "bezier" => {
                if path.elements().is_empty() {
                    path.move_to(p(0));
                }
                path.curve_to(p(0), p(2), p(4));
            }
            "close" => {
                if !path.elements().is_empty() {
                    path.close_path();
                }
            }
            "rect" => {
                let x = number(&args[0]);
                let y = number(&args[1]);
                let w = number(&args[2]);
                let h = number(&args[3]);
                path.move_to(transform * Point::new(x, y));
                path.line_to(transform * Point::new(x + w, y));
                path.line_to(transform * Point::new(x + w, y + h));
                path.line_to(transform * Point::new(x, y + h));
                path.close_path();
            }
            "ellipse" => {
                let start = number(&args[5]);
                let end = number(&args[6]);
                let ccw = number(&args[7]) != 0.;
                let tau = std::f64::consts::TAU;
                let delta = end - start;
                let sweep = if !ccw {
                    if delta >= tau {
                        tau
                    } else {
                        delta.rem_euclid(tau)
                    }
                } else if -delta >= tau {
                    -tau
                } else {
                    -(-delta).rem_euclid(tau)
                };
                let arc = EllipseArc::new(
                    (number(&args[0]), number(&args[1])),
                    (number(&args[2]), number(&args[3])),
                    start,
                    sweep,
                    number(&args[4]),
                );
                for element in (transform * arc.to_path(0.01)).elements() {
                    match *element {
                        PathEl::MoveTo(p) => {
                            if path.elements().is_empty() {
                                path.move_to(p)
                            } else {
                                path.line_to(p)
                            }
                        }
                        element => path.push(element),
                    }
                }
            }
            _ => {}
        }
    }
    Ok(path)
}

fn brush(value: &Value) -> (Paint, Option<Affine>) {
    if value.is_string() {
        return (Paint::Solid(color(value)), None);
    }
    let args = &value["args"];
    let gradient = if value["kind"] == "radial" {
        Gradient::new_two_point_radial(
            (number(&args[0]), number(&args[1])),
            number(&args[2]) as f32,
            (number(&args[3]), number(&args[4])),
            number(&args[5]) as f32,
        )
    } else {
        Gradient::new_linear(
            (number(&args[0]), number(&args[1])),
            (number(&args[2]), number(&args[3])),
        )
    };
    let stops: Vec<_> = value["stops"]
        .as_array()
        .map(|stops| {
            stops
                .iter()
                .map(|stop| (number(&stop[0]) as f32, color(&stop[1])))
                .collect()
        })
        .unwrap_or_default();
    if stops.is_empty() {
        return (Paint::Solid(Color::TRANSPARENT), None);
    }
    (
        Paint::Gradient(gradient.with_stops(stops.as_slice())),
        Some(affine(&value["transform"])),
    )
}
fn blend(value: &Value) -> BlendMode {
    let compose = match value.as_str().unwrap_or("") {
        "source-in" => Compose::SrcIn,
        "source-out" => Compose::SrcOut,
        "source-atop" => Compose::SrcAtop,
        "destination-over" => Compose::DestOver,
        "destination-in" => Compose::DestIn,
        "destination-out" => Compose::DestOut,
        "destination-atop" => Compose::DestAtop,
        "lighter" => Compose::Plus,
        "copy" => Compose::Copy,
        "xor" => Compose::Xor,
        _ => Compose::SrcOver,
    };
    let mix = match value.as_str().unwrap_or("") {
        "multiply" => Mix::Multiply,
        "screen" => Mix::Screen,
        "overlay" => Mix::Overlay,
        "darken" => Mix::Darken,
        "lighten" => Mix::Lighten,
        "difference" => Mix::Difference,
        "exclusion" => Mix::Exclusion,
        _ => Mix::Normal,
    };
    BlendMode::new(mix, compose)
}
fn stroke_style(state: &Value) -> Stroke {
    let cap = match state["lineCap"].as_str() {
        Some("round") => Cap::Round,
        Some("square") => Cap::Square,
        _ => Cap::Butt,
    };
    let join = match state["lineJoin"].as_str() {
        Some("round") => Join::Round,
        Some("bevel") => Join::Bevel,
        _ => Join::Miter,
    };
    let mut style = Stroke::new(number(&state["lineWidth"]))
        .with_caps(cap)
        .with_join(join);
    style.miter_limit = number(&state["miterLimit"]);
    if let Some(dash) = state["dash"]
        .as_array()
        .filter(|v| v.iter().any(|v| number(v) > 0.0))
    {
        style = style.with_dashes(number(&state["lineDashOffset"]), dash.iter().map(number));
    }
    style
}
fn draw(
    request: &Value,
    width: u32,
    height: u32,
    previous: Option<&RasterImageData>,
    source: Option<&blitz_dom::node::ImageData>,
    text: Option<&crate::canvas_text::TextLayout>,
    prepare: Option<fn(&mut anyrender::Scene, u32, u32)>,
) -> JsResult<Vec<u8>> {
    let op = request["op"].as_str().unwrap_or("");
    if !matches!(
        op,
        "fillRect"
            | "strokeRect"
            | "clearRect"
            | "fill"
            | "stroke"
            | "drawImage"
            | "fillText"
            | "strokeText"
    ) {
        return Err(JsNativeError::typ()
            .with_message("Unsupported canvas command")
            .into());
    }
    if !bounded(request) {
        return Err(error(
            "Canvas geometry exceeds the preview coordinate limit",
        ));
    }
    let state = &request["state"];
    let transform = affine(&state["transform"]);
    let image_geometry =
        source.and_then(|source| crate::canvas_images::geometry(source, request, transform));
    if (source.is_some() && image_geometry.is_none()) || text.is_some_and(|text| text.is_empty()) {
        return Ok(previous
            .map(|p| p.data.data().to_vec())
            .unwrap_or_else(|| vec![0; width as usize * height as usize * 4]));
    }
    let path = if op.ends_with("Rect") {
        build_path(&json!([["rect", request["args"], state["transform"]]]))
    } else {
        build_path(&request["path"])
    }?;
    let clips: Vec<_> = state["clips"]
        .as_array()
        .map(|clips| {
            clips
                .iter()
                .take(128)
                .map(|clip| Ok((build_path(&clip["path"])?, rule(&clip["rule"]))))
                .collect::<JsResult<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();
    let stroke = op == "stroke" || op == "strokeRect" || op == "strokeText";
    let clear = op == "clearRect";
    let (paint, paint_transform) = if clear {
        (Paint::Solid(Color::WHITE), None)
    } else {
        brush(&state[if stroke { "strokeStyle" } else { "fillStyle" }])
    };
    let blend = if clear {
        BlendMode::new(Mix::Normal, Compose::DestOut)
    } else {
        blend(&state["globalCompositeOperation"])
    };
    let alpha = if clear {
        1.0
    } else {
        number(&state["globalAlpha"]).clamp(0.0, 1.0) as f32
    };
    let mut scene = anyrender::Scene::new();
    {
        if let Some(previous) = previous {
            scene.draw_image(image_brush(previous).as_ref(), Affine::IDENTITY);
        }
        for (path, rule) in &clips {
            scene.push_clip_layer(*rule, Affine::IDENTITY, path);
        }
        scene.push_layer(
            Fill::NonZero,
            blend,
            alpha,
            Affine::IDENTITY,
            &Rect::new(0., 0., width as f64, height as f64),
            None,
            None,
        );
        if let Some(text) = text {
            let stroke_style = stroke.then(|| stroke_style(state));
            text.draw(
                &mut scene,
                request,
                transform,
                &paint,
                stroke_style.as_ref(),
            );
        } else if let Some((source, geometry)) = source.zip(image_geometry) {
            crate::canvas_images::draw(&mut scene, source, request, geometry);
        } else if stroke {
            if transform.determinant() != 0.0 {
                let inverse = transform.inverse();
                let local_path = inverse * path.clone();
                let style = stroke_style(state);
                scene.stroke(
                    &style,
                    transform,
                    paint.as_ref(),
                    paint_transform.map(|p| inverse * p),
                    &local_path,
                );
            }
        } else {
            scene.fill(
                rule(&request["rule"]),
                Affine::IDENTITY,
                paint.as_ref(),
                paint_transform,
                &path,
            );
        }
        scene.pop_layer();
        for _ in &clips {
            scene.pop_layer();
        }
    }
    if let Some(prepare) = prepare {
        prepare(&mut scene, width, height);
    }
    let pixels = render_to_buffer::<VelloCpuImageRenderer, _>(
        |target| target.append_scene(scene, Affine::IDENTITY),
        width,
        height,
    );
    // Vello exports premultiplied pixels; DOM image resources use straight RGBA.
    let mut straight = pixels;
    for pixel in straight.chunks_exact_mut(4) {
        let alpha = u32::from(pixel[3]);
        if alpha > 0 && alpha < 255 {
            for channel in &mut pixel[..3] {
                *channel = ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
            }
        }
    }
    Ok(straight)
}
