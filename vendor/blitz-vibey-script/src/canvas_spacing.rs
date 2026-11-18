//! Canvas spacing uses one CSS dimension token, not a declaration or calc value.
use crate::state::DomCtx;
use boa_engine::{JsNativeError, JsResult};
use cssparser::{Parser, Token};
use serde_json::{Value, json};
use style::values::specified::LengthUnit;

pub(crate) fn parse(input: &str) -> Value {
    let mut parser = Parser::new(input);
    let Ok(Token::Dimension { value, unit, .. }) = parser.next_including_whitespace() else {
        return Value::Null;
    };
    let value = value.clamp(-f32::MAX, f32::MAX);
    let unit = unit.to_ascii_lowercase();
    if parser.next_including_whitespace().is_ok() || LengthUnit::from_str(&unit).is_err() {
        return Value::Null;
    }
    json!({"value":value,"unit":unit})
}

pub(crate) fn resolve(ctx: &DomCtx, state: &Value, size: f64) -> JsResult<(f32, f32)> {
    let values = [&state["letterSpacingData"], &state["wordSpacingData"]];
    let needs_font = values.iter().any(|value| {
        matches!(
            value["unit"].as_str(),
            Some("ex" | "rex" | "cap" | "rcap" | "ch" | "rch")
        )
    });
    let (x_height, cap_height, zero_width) = if needs_font {
        // The probe has no spacing fields, so it cannot recurse into this branch.
        let probe = crate::canvas_text::shape(
            ctx,
            &json!({
                "text":"0", "state":{"fontData":state["fontData"],"fontKerning":"none"}
            }),
        )?;
        probe
            .spacing_font_metrics()
            .unwrap_or((size * 0.5, size, size * 0.5))
    } else {
        (0.0, 0.0, 0.0)
    };
    let mut result = [0.0; 2];
    for (index, value) in values.into_iter().enumerate() {
        let number = value["value"]
            .as_str()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        let factor = match value["unit"].as_str().unwrap_or("px") {
            "px" => 1.0,
            "pt" => 96.0 / 72.0,
            "pc" => 16.0,
            "in" => 96.0,
            "cm" => 96.0 / 2.54,
            "mm" => 96.0 / 25.4,
            "q" => 96.0 / 101.6,
            "em" | "rem" | "ic" | "ric" => size,
            "ex" | "rex" => x_height,
            "cap" | "rcap" => cap_height,
            "ch" | "rch" => zero_width,
            // Canvas has no viewport, container, or line-height length context.
            _ => 0.0,
        };
        let pixels = number * factor;
        if !pixels.is_finite() || pixels.abs() > 10_000_000.0 {
            return Err(JsNativeError::range()
                .with_message("Canvas text spacing exceeds the preview coordinate limit")
                .into());
        }
        result[index] = pixels as f32;
    }
    Ok((result[0], result[1]))
}
