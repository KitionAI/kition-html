//! Numeric IDL properties for progress and meter elements.
use super::{define_accessor, dom_ctx, js_str, this_node_id, to_rust_string};
use boa_engine::{Context, JsNativeError, JsResult, JsValue, object::JsObject};

pub(crate) fn init(proto: &JsObject, context: &mut Context) {
    define_accessor(proto, "min", Some(min), Some(set_min), context);
    define_accessor(proto, "max", Some(max), Some(set_max), context);
    define_accessor(proto, "low", Some(low), Some(set_low), context);
    define_accessor(proto, "high", Some(high), Some(set_high), context);
    define_accessor(proto, "optimum", Some(optimum), Some(set_optimum), context);
    define_accessor(proto, "position", Some(position), None, context);
}
fn get(this: &JsValue, context: &mut Context, name: &str) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let doc = ctx.doc.borrow();
    let element = doc.get_node(id).and_then(|n| n.element_data());
    if let Some(range) = element.and_then(|e| e.range_control()) {
        let value = match name {
            "min" => range.min,
            "max" => range.max,
            "low" => range.low,
            "high" => range.high,
            "optimum" => range.optimum,
            "position" => {
                if range.determinate {
                    range.fraction()
                } else {
                    -1.
                }
            }
            _ => range.value,
        };
        return Ok(value.into());
    }
    Ok(js_str(
        element
            .and_then(|e| e.attr(blitz_dom::LocalName::from(name)))
            .unwrap_or(""),
    ))
}
pub(crate) fn set(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
    name: &str,
) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let range = ctx
        .doc
        .borrow()
        .get_node(id)
        .and_then(|n| n.element_data())
        .and_then(|e| e.range_control());
    let value = args.first().cloned().unwrap_or_default();
    let text = if let Some(range) = range {
        let mut number = value.to_number(context)?;
        if !number.is_finite() {
            return Err(JsNativeError::typ()
                .with_message("Range control properties require a finite number")
                .into());
        }
        if range.progress {
            if name == "max" && number <= 0. {
                number = 1.;
            } else if name == "value" {
                number = number.max(0.);
            }
        }
        number.to_string()
    } else {
        to_rust_string(&value, context)?
    };
    ctx.doc
        .borrow_mut()
        .mutate()
        .set_attribute(id, super::element::attr_name(name), &text);
    Ok(JsValue::undefined())
}
macro_rules! property {
    ($get:ident,$set:ident,$name:literal) => {
        fn $get(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
            get(this, context, $name)
        }
        fn $set(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
            set(this, args, context, $name)
        }
    };
}
property!(min, set_min, "min");
property!(max, set_max, "max");
property!(low, set_low, "low");
property!(high, set_high, "high");
property!(optimum, set_optimum, "optimum");
fn position(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    get(this, context, "position")
}
