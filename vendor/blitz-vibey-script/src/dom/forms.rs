//! Form properties whose live state is separate from source markup.
use super::{
    define_accessor, define_method, dom_ctx, js_str, node_wrapper, this_node_id, to_rust_string,
};
use blitz_dom::{BaseDocument, NodeId, local_name};
use boa_engine::object::JsObject;
use boa_engine::{Context, JsResult, JsValue};

pub(crate) fn init(proto: &JsObject, context: &mut Context) {
    define_method(proto, "reset", 0, reset, context);
    define_accessor(proto, "form", Some(form), None, context);
    define_accessor(
        proto,
        "defaultSelected",
        Some(default_selected),
        Some(set_default_selected),
        context,
    );
    define_accessor(
        proto,
        "indeterminate",
        Some(indeterminate),
        Some(set_indeterminate),
        context,
    );
    define_accessor(
        proto,
        "selectedIndex",
        Some(selected_index),
        Some(set_selected_index),
        context,
    );
    define_accessor(
        proto,
        "selected",
        Some(selected),
        Some(set_selected),
        context,
    );
    define_accessor(
        proto,
        "defaultChecked",
        Some(default_checked),
        Some(set_default_checked),
        context,
    );
    define_accessor(
        proto,
        "defaultValue",
        Some(default_value),
        Some(set_default_value),
        context,
    );
    define_accessor(proto, "open", Some(open), Some(set_open), context);
}

pub(crate) fn options(doc: &BaseDocument, id: NodeId) -> Vec<NodeId> {
    doc.select_options(id)
}
fn selection(doc: &BaseDocument, id: NodeId) -> Vec<NodeId> {
    doc.selected_options(id)
}
pub(crate) fn option_value(doc: &BaseDocument, id: NodeId) -> String {
    let Some(node) = doc.get_node(id) else {
        return String::new();
    };
    node.attr(local_name!("value"))
        .map(str::to_owned)
        .unwrap_or_else(|| {
            node.text_content()
                .split_ascii_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
}
pub(crate) fn select_value(doc: &BaseDocument, id: NodeId) -> String {
    selection(doc, id)
        .first()
        .map(|&id| option_value(doc, id))
        .unwrap_or_default()
}
pub(crate) fn set_select_value(doc: &mut BaseDocument, id: NodeId, value: &str) {
    let options = options(doc, id);
    let chosen = options
        .iter()
        .copied()
        .find(|&id| option_value(doc, id) == value);
    doc.set_select_selection(id, chosen);
}
fn selected_index(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let doc = ctx.doc.borrow();
    let selected = selection(&doc, id);
    Ok(options(&doc, id)
        .iter()
        .position(|id| selected.contains(id))
        .map_or(-1, |i| i as i32)
        .into())
}
fn set_selected_index(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let index = args
        .first()
        .unwrap_or(&JsValue::undefined())
        .to_i32(context)?;
    let mut doc = ctx.doc.borrow_mut();
    let chosen = usize::try_from(index)
        .ok()
        .and_then(|index| options(&doc, id).get(index).copied());
    doc.set_select_selection(id, chosen);
    Ok(JsValue::undefined())
}
fn selected(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let doc = ctx.doc.borrow();
    let value = doc.option_selected(id);
    Ok(value.into())
}
fn set_selected(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let value = args.first().is_some_and(JsValue::to_boolean);
    let mut doc = ctx.doc.borrow_mut();
    doc.set_option_selected(id, value);
    Ok(JsValue::undefined())
}
fn attr_bool(this: &JsValue, context: &mut Context, name: &str) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let value = ctx
        .doc
        .borrow()
        .get_node(id)
        .is_some_and(|n| n.data.has_attr(blitz_dom::LocalName::from(name)));
    Ok(value.into())
}
fn set_attr_bool(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
    name: &str,
) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let name = blitz_dom::QualName::new(None, blitz_dom::ns!(), blitz_dom::LocalName::from(name));
    if args.first().is_some_and(JsValue::to_boolean) {
        ctx.doc.borrow_mut().mutate().set_attribute(id, name, "");
    } else {
        ctx.doc.borrow_mut().mutate().clear_attribute(id, name);
    }
    Ok(JsValue::undefined())
}
fn default_checked(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    attr_bool(this, context, "checked")
}
fn set_default_checked(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    set_attr_bool(this, args, context, "checked")
}
fn open(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    attr_bool(this, context, "open")
}
fn set_open(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    set_attr_bool(this, args, context, "open")
}
fn default_value(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let doc = ctx.doc.borrow();
    Ok(js_str(
        &doc.get_node(id)
            .map(|n| {
                if n.data.is_element_with_tag_name(&local_name!("textarea")) {
                    n.text_content()
                } else {
                    n.attr(local_name!("value")).unwrap_or("").to_owned()
                }
            })
            .unwrap_or_default(),
    ))
}
fn set_default_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let textarea = ctx
        .doc
        .borrow()
        .get_node(id)
        .is_some_and(|n| n.data.is_element_with_tag_name(&local_name!("textarea")));
    if textarea {
        // The native textContent setter keeps the original node and replaces children.
        this.as_object().unwrap().set(
            boa_engine::JsString::from("textContent"),
            args.first().cloned().unwrap_or_default(),
            true,
            context,
        )?;
    } else {
        let value = to_rust_string(args.first().unwrap_or(&JsValue::undefined()), context)?;
        ctx.doc
            .borrow_mut()
            .mutate()
            .set_attribute(id, blitz_dom::qual_name!("value"), &value);
    }
    Ok(JsValue::undefined())
}

fn form(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let owner = ctx.doc.borrow().form_owner(id);
    Ok(owner
        .map(|id| node_wrapper(&ctx, id, context).into())
        .unwrap_or_else(JsValue::null))
}
fn default_selected(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    attr_bool(this, context, "selected")
}
fn set_default_selected(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    set_attr_bool(this, args, context, "selected")
}
fn indeterminate(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    let value = ctx
        .doc
        .borrow()
        .get_node(id)
        .is_some_and(|n| n.is_indeterminate());
    Ok(value.into())
}
fn set_indeterminate(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    ctx.doc
        .borrow_mut()
        .set_input_indeterminate(id, args.first().is_some_and(JsValue::to_boolean));
    Ok(JsValue::undefined())
}
fn reset(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let id = this_node_id(this)?;
    if !ctx
        .doc
        .borrow()
        .get_node(id)
        .is_some_and(|n| n.data.is_element_with_tag_name(&local_name!("form")))
    {
        return Err(boa_engine::JsNativeError::typ()
            .with_message("reset requires a form")
            .into());
    }
    reset_form(&ctx, id, context);
    Ok(JsValue::undefined())
}
pub(crate) fn reset_form(ctx: &crate::state::DomCtx, id: NodeId, context: &mut Context) {
    if !ctx.state.borrow_mut().resetting_forms.insert(id) {
        return;
    }
    let path = ctx.doc.borrow().node_chain(id);
    let target = node_wrapper(ctx, id, context).into();
    let event = super::event::create_event(ctx, "reset", true, true, &target, context);
    super::dispatch::dispatch_object(ctx, context, &path, &event, "reset");
    let canceled = event
        .downcast_ref::<super::event::EventRef>()
        .is_some_and(|e| e.prevented.get());
    if !canceled {
        ctx.doc.borrow_mut().reset_form_controls(id);
    }
    ctx.state.borrow_mut().resetting_forms.remove(&id);
}
