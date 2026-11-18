//! HTML fragment insertion and rendered-text replacement.
use super::{dom_ctx, dom_exception, this_node_id, to_rust_string};
use boa_engine::{Context, JsResult, JsValue};

pub(crate) fn set_inner_text(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let node_id = this_node_id(this)?;
    let text = to_rust_string(args.first().unwrap_or(&JsValue::undefined()), context)?;
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut doc = ctx.doc.borrow_mut();
    let mut mutr = doc.mutate();
    for child in mutr.child_ids(node_id) {
        mutr.remove_node(child);
    }
    let mut children = Vec::new();
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            children.push(mutr.create_element(blitz_dom::qual_name!("br", html), Vec::new()));
        }
        if !line.is_empty() {
            children.push(mutr.create_text_node(line));
        }
    }
    mutr.append_children(node_id, &children);
    Ok(JsValue::undefined())
}

pub(crate) fn insert_adjacent_html(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let node_id = this_node_id(this)?;
    let position = to_rust_string(args.first().unwrap_or(&JsValue::undefined()), context)?
        .to_ascii_lowercase();
    let html = to_rust_string(args.get(1).unwrap_or(&JsValue::undefined()), context)?;
    if !matches!(
        position.as_str(),
        "beforebegin" | "afterbegin" | "beforeend" | "afterend"
    ) {
        return Err(dom_exception(
            context,
            "SyntaxError",
            "invalid adjacent HTML position",
        ));
    }
    let outside = matches!(position.as_str(), "beforebegin" | "afterend");
    let parent_id = if outside {
        let parent = ctx
            .doc
            .borrow()
            .get_node(node_id)
            .and_then(|node| node.parent);
        let Some(parent) = parent.filter(|id| {
            ctx.doc
                .borrow()
                .get_node(*id)
                .is_some_and(|node| !matches!(node.data, blitz_dom::node::NodeData::Document(_)))
        }) else {
            return Err(dom_exception(
                context,
                "NoModificationAllowedError",
                "adjacent HTML requires a non-document parent",
            ));
        };
        parent
    } else {
        node_id
    };
    let name = ctx
        .doc
        .borrow()
        .get_node(parent_id)
        .and_then(|node| node.element_data())
        .map(|element| element.name.clone());
    let name = name
        .filter(|name| name.local != blitz_dom::local_name!("html"))
        .unwrap_or_else(|| blitz_dom::qual_name!("body", html));
    let mut doc = ctx.doc.borrow_mut();
    let mut mutr = doc.mutate();
    // A detached context preserves table/select/foreign-content fragment parsing.
    let fragment = mutr.create_element(name, Vec::new());
    mutr.set_inner_html(fragment, &html);
    let children = mutr.child_ids(fragment);
    for child in &children {
        mutr.remove_node(*child);
    }
    let reference = match position.as_str() {
        "beforebegin" => Some(node_id),
        "afterbegin" => mutr.child_ids(node_id).first().copied(),
        "afterend" => mutr.next_sibling_id(node_id),
        _ => None,
    };
    if let Some(reference) = reference {
        mutr.insert_nodes_before(reference, &children);
    } else {
        mutr.append_children(parent_id, &children);
    }
    mutr.remove_and_drop_node(fragment);
    Ok(JsValue::undefined())
}

pub(crate) fn set_outer_html(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let node_id = this_node_id(this)?;
    if ctx
        .doc
        .borrow()
        .get_node(node_id)
        .and_then(|node| node.parent)
        .is_none()
    {
        return Ok(JsValue::undefined());
    }
    insert_adjacent_html(
        this,
        &[
            super::js_str("beforebegin"),
            args.first().cloned().unwrap_or_default(),
        ],
        context,
    )?;
    ctx.doc.borrow_mut().mutate().remove_node(node_id);
    Ok(JsValue::undefined())
}
