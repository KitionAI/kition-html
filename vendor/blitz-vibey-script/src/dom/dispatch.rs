//! Shared dispatch for native input and script-created events.
use super::event::{EventRef, create_event};
use super::listeners::Target;
use super::{define_value, dom_ctx, node_id_of_value, node_wrapper, to_rust_string};
use crate::state::{DomCtx, Listener};
use blitz_dom::NodeId;
use boa_engine::object::JsObject;
use boa_engine::{Context, JsNativeError, JsResult, JsString, JsValue, Source};

pub(crate) fn create(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _ = this;
    let ctx = dom_ctx(context)?;
    let name = to_rust_string(args.first().unwrap_or(&JsValue::undefined()), context)?;
    let event = create_event(
        &ctx,
        &name,
        args.get(1).is_some_and(JsValue::to_boolean),
        args.get(2).is_some_and(JsValue::to_boolean),
        &JsValue::null(),
        context,
    );
    if let Some(flags) = event.downcast_ref::<EventRef>() {
        flags.trusted.set(false);
        flags
            .composed
            .set(args.get(3).is_some_and(JsValue::to_boolean));
    }
    Ok(event.into())
}

pub(crate) fn mark_mouse(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    if let Some(object) = args.first().and_then(JsValue::as_object)
        && let Some(event) = object.downcast_ref::<EventRef>()
    {
        event.mouse_event.set(true);
    }
    Ok(JsValue::undefined())
}

pub(crate) fn mark_error(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    if let Some(object) = args.first().and_then(JsValue::as_object)
        && let Some(event) = object.downcast_ref::<EventRef>()
    {
        event.error_event.set(true);
    }
    Ok(JsValue::undefined())
}

pub(crate) fn dispatch(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let event = args
        .first()
        .and_then(JsValue::as_object)
        .filter(|o| o.is::<EventRef>())
        .ok_or_else(|| JsNativeError::typ().with_message("dispatchEvent requires an Event"))?;
    if event
        .downcast_ref::<EventRef>()
        .is_some_and(|e| e.dispatching.get())
    {
        return Err(super::dom_exception(
            context,
            "InvalidStateError",
            "Event is already being dispatched",
        ));
    }
    let name = event
        .downcast_ref::<EventRef>()
        .map(|event| event.type_name.clone())
        .unwrap_or_default();
    if name.is_empty() {
        return Err(super::dom_exception(
            context,
            "InvalidStateError",
            "Event type is empty",
        ));
    }
    let target = Target::from_value(this, context)?;
    let mut chain = Vec::new();
    if let Some(id) = node_id_of_value(this) {
        let doc = ctx.doc.borrow();
        let mut next = Some(id);
        while let Some(id) = next {
            chain.push(id);
            next = doc.get_node(id).and_then(|n| n.parent);
        }
    }
    if let Some(flags) = event.downcast_ref::<EventRef>() {
        flags.trusted.set(false);
    }
    let custom = if let Target::Custom(object) = target {
        Some(object)
    } else {
        None
    };
    dispatch_to(&ctx, context, &chain, &event, &name, custom);
    Ok((!flag(&event, |e| e.prevented.get())).into())
}

fn flag(event: &JsObject, read: impl FnOnce(&EventRef) -> bool) -> bool {
    event.downcast_ref::<EventRef>().is_some_and(|e| read(&e))
}

pub(crate) fn dispatch_object(
    ctx: &DomCtx,
    context: &mut Context,
    chain: &[NodeId],
    event: &JsObject,
    name: &str,
) -> bool {
    dispatch_to(ctx, context, chain, event, name, None)
}

fn dispatch_to(
    ctx: &DomCtx,
    context: &mut Context,
    chain: &[NodeId],
    event: &JsObject,
    name: &str,
    custom: Option<JsObject>,
) -> bool {
    let activation = if name == "click" && flag(event, |e| e.mouse_event.get()) {
        chain
            .first()
            .and_then(|&id| super::activation::prepare(ctx, id))
    } else {
        None
    };
    if let Some(flags) = event.downcast_ref::<EventRef>() {
        flags.activation_handled.set(activation.is_some());
    }
    let target: JsValue = chain
        .first()
        .map(|&id| node_wrapper(ctx, id, context).into())
        .unwrap_or_else(|| {
            custom
                .clone()
                .unwrap_or_else(|| context.global_object())
                .into()
        });
    if let Some(e) = event.downcast_ref::<EventRef>() {
        *e.target.borrow_mut() = target;
        e.dispatching.set(true);
    }
    let window = context.global_object();
    let previous = window
        .get(JsString::from("event"), context)
        .unwrap_or_default();
    define_value(&window, "event", event.clone().into(), context);
    let bubbles = flag(event, |e| e.bubbles);
    let connected = chain
        .last()
        .is_some_and(|id| *id == ctx.doc.borrow().root_node().id);
    let mut called = false;
    // The path is frozen before callbacks; detached trees never reach window.
    let mut path: Vec<Target> = chain.iter().copied().map(Target::Node).collect();
    if let Some(object) = custom {
        path.push(Target::Custom(object));
    } else if connected || path.is_empty() {
        path.push(Target::Window);
    }
    let frozen_path = path
        .iter()
        .map(|target| target.value(ctx, context))
        .collect();
    if let Some(flags) = event.downcast_ref::<EventRef>() {
        *flags.path.borrow_mut() = frozen_path;
    }
    for id in path.iter().skip(1).rev() {
        if flag(event, |e| e.stopped.get()) {
            break;
        }
        called |= invoke(ctx, context, id, event, name, true, 1);
    }
    if !flag(event, |e| e.stopped.get()) {
        called |= invoke(ctx, context, &path[0], event, name, true, 2);
        if !flag(event, |e| e.stopped_immediate.get()) {
            called |= invoke(ctx, context, &path[0], event, name, false, 2);
        }
    }
    if bubbles {
        for id in path.iter().skip(1) {
            if flag(event, |e| e.stopped.get()) {
                break;
            }
            called |= invoke(ctx, context, id, event, name, false, 3);
        }
    }
    define_value(&window, "event", previous, context);
    if let Some(e) = event.downcast_ref::<EventRef>() {
        e.dispatching.set(false);
        *e.current_target.borrow_mut() = JsValue::null();
        e.phase.set(0);
        e.path.borrow_mut().clear();
        e.stopped.set(false);
        e.stopped_immediate.set(false);
    }
    if let Some(activation) = activation {
        let canceled = flag(event, |e| e.prevented.get());
        let changed = super::activation::finish(ctx, activation, canceled);
        if let Some(super::activation::Completion::Change(id)) = changed {
            let path = {
                let doc = ctx.doc.borrow();
                doc.node_chain(id)
            };
            let target = node_wrapper(ctx, id, context).into();
            for name in ["input", "change"] {
                let notification = create_event(ctx, name, true, false, &target, context);
                called |= dispatch_object(ctx, context, &path, &notification, name);
            }
        } else if let Some(super::activation::Completion::Click(id)) = changed {
            let path = ctx.doc.borrow().node_chain(id);
            let target = node_wrapper(ctx, id, context).into();
            let click = create_event(ctx, "click", true, true, &target, context);
            if let Some(flags) = click.downcast_ref::<EventRef>() {
                flags.mouse_event.set(true);
            }
            if let Ok(constructor) = context
                .global_object()
                .get(JsString::from("MouseEvent"), context)
                && let Some(constructor) = constructor.as_object()
                && let Ok(proto) = constructor.get(JsString::from("prototype"), context)
                && let Some(proto) = proto.as_object()
            {
                click.set_prototype(Some(proto));
            }
            called |= dispatch_object(ctx, context, &path, &click, "click");
        } else if let Some(super::activation::Completion::Reset(id)) = changed {
            super::forms::reset_form(ctx, id, context);
        }
    }
    called
}

fn invoke(
    ctx: &DomCtx,
    context: &mut Context,
    target: &Target,
    event: &JsObject,
    name: &str,
    capture: bool,
    phase: i32,
) -> bool {
    let current = target.value(ctx, context);
    if let Some(flags) = event.downcast_ref::<EventRef>() {
        *flags.current_target.borrow_mut() = current.clone();
        flags.phase.set(phase);
    }
    let listeners: Vec<Listener> =
        target.with_map(ctx, |map| map.get(name).cloned().unwrap_or_default());
    let mut called = false;
    for listener in listeners.into_iter().filter(|l| l.capture == capture) {
        if flag(event, |e| e.stopped_immediate.get()) {
            break;
        }
        // A snapshot refers to one registration, not merely a callback. Removing
        // and re-adding the callback must not revive this snapshot's entry.
        let present = target.with_map(ctx, |map| {
            if let Some(list) = map.get_mut(name) {
                let index = list
                    .iter()
                    .position(|l| std::rc::Rc::ptr_eq(&l.removed, &listener.removed) && l.active());
                if listener.once {
                    if let Some(index) = index {
                        list[index].removed.set(true);
                        list.remove(index);
                    }
                }
                index.is_some()
            } else {
                false
            }
        });
        if !present {
            continue;
        }
        called = true;
        if let Some(flags) = event.downcast_ref::<EventRef>() {
            flags.in_passive_listener.set(listener.passive);
        }
        let result = if listener.callback.is_callable() {
            listener
                .callback
                .call(&current, &[event.clone().into()], context)
        } else {
            listener
                .callback
                .get(JsString::from("handleEvent"), context)
                .and_then(|v| {
                    let callback =
                        v.as_object().filter(JsObject::is_callable).ok_or_else(|| {
                            JsNativeError::typ()
                                .with_message("Event listener handleEvent is not callable")
                        })?;
                    callback.call(
                        &listener.callback.clone().into(),
                        &[event.clone().into()],
                        context,
                    )
                })
        };
        if let Some(flags) = event.downcast_ref::<EventRef>() {
            flags.in_passive_listener.set(false);
        }
        if let Err(error) = result {
            crate::runtime::report_js_error(ctx, context, "event listener", &error);
        }
    }
    if !capture && !flag(event, |e| e.stopped_immediate.get()) {
        let property = JsString::from(format!("on{name}"));
        let wrapper = current.as_object();
        let handler = wrapper
            .as_ref()
            .and_then(|w| w.get(property.clone(), context).ok());
        let mut callback = handler
            .as_ref()
            .and_then(JsValue::as_object)
            .filter(JsObject::is_callable);
        // An assigned null disables an attribute handler. Otherwise compile the
        // current attribute, so setAttribute/removeAttribute updates it too.
        if callback.is_none()
            && !wrapper.as_ref().is_some_and(|w| {
                w.has_own_property(property.clone(), context)
                    .unwrap_or(false)
            })
        {
            let id = if let Target::Node(id) = target {
                Some(*id)
            } else {
                None
            };
            let source = id.and_then(|id| {
                ctx.doc
                    .borrow()
                    .get_node(id)
                    .and_then(|n| n.element_data())
                    .and_then(|e| e.attr(blitz_dom::LocalName::from(format!("on{name}"))))
                    .map(str::to_owned)
            });
            if let Some(source) = source {
                match context.eval(Source::from_bytes(&format!(
                    "(function(event){{{source}\n}})"
                ))) {
                    Ok(value) => callback = value.as_object(),
                    Err(error) => {
                        crate::runtime::report_js_error(ctx, context, "event handler", &error)
                    }
                }
            }
        }
        if let Some(callback) = callback {
            called = true;
            let legacy_error = matches!(target, Target::Window)
                && name == "error"
                && flag(event, |event| event.error_event.get());
            let args = if legacy_error {
                ["message", "filename", "lineno", "colno", "error"]
                    .iter()
                    .map(|key| event.get(JsString::from(*key), context).unwrap_or_default())
                    .collect()
            } else {
                vec![event.clone().into()]
            };
            match callback.call(&current, &args, context) {
                Ok(value) if value.as_boolean() == Some(legacy_error) => {
                    if let Some(e) = event.downcast_ref::<EventRef>() {
                        if e.cancelable {
                            e.prevented.set(true);
                        }
                    }
                }
                Err(error) => {
                    crate::runtime::report_js_error(ctx, context, "event handler", &error)
                }
                _ => {}
            }
        }
    }
    called
}
