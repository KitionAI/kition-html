//! Shared listener registration for DOM nodes, window, and standalone targets.
use super::{dom_ctx, node_id_of_value, node_wrapper, to_rust_string};
use crate::state::{DomCtx, Listener, ListenerMap};
use blitz_dom::NodeId;
use boa_engine::{
    Context, Finalize, JsData, JsNativeError, JsResult, JsString, JsValue, Trace, object::JsObject,
};
use boa_gc::GcRefCell;
use std::{cell::Cell, rc::Rc};

#[derive(Trace, Finalize, JsData)]
struct TargetRef {
    listeners: GcRefCell<ListenerMap>,
    #[unsafe_ignore_trace]
    aborted: Option<Rc<Cell<bool>>>,
}

#[derive(Clone)]
pub(crate) enum Target {
    Node(NodeId),
    Window,
    Custom(JsObject),
}

impl Target {
    pub(crate) fn from_value(value: &JsValue, context: &Context) -> JsResult<Self> {
        if let Some(id) = node_id_of_value(value) {
            return Ok(Self::Node(id));
        }
        if let Some(object) = value.as_object() {
            if object.is::<TargetRef>() {
                return Ok(Self::Custom(object));
            }
            if JsObject::equals(&object, &context.global_object()) {
                return Ok(Self::Window);
            }
        }
        Err(JsNativeError::typ()
            .with_message("Invalid event target")
            .into())
    }

    pub(crate) fn value(&self, ctx: &DomCtx, context: &mut Context) -> JsValue {
        match self {
            Self::Node(id) => node_wrapper(ctx, *id, context).into(),
            Self::Window => context.global_object().into(),
            Self::Custom(object) => object.clone().into(),
        }
    }

    pub(crate) fn with_map<T>(&self, ctx: &DomCtx, apply: impl FnOnce(&mut ListenerMap) -> T) -> T {
        match self {
            Self::Node(id) => apply(
                ctx.state
                    .borrow_mut()
                    .node_listeners
                    .entry(*id)
                    .or_default(),
            ),
            Self::Window => apply(&mut ctx.state.borrow_mut().window_listeners),
            Self::Custom(object) => {
                let target = object
                    .downcast_ref::<TargetRef>()
                    .expect("validated event target");
                apply(&mut target.listeners.borrow_mut())
            }
        }
    }
}

pub(crate) fn create(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let target = TargetRef {
        listeners: GcRefCell::default(),
        aborted: args
            .first()
            .is_some_and(JsValue::to_boolean)
            .then(|| Rc::new(Cell::new(false))),
    };
    Ok(JsObject::from_proto_and_data(
        Some(context.intrinsics().constructors().object().prototype()),
        target,
    )
    .into())
}

pub(crate) fn abort(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let flag = signal(args.first().unwrap_or(&JsValue::undefined()))?;
    flag.set(true);
    Ok(JsValue::undefined())
}

fn signal(value: &JsValue) -> JsResult<Rc<Cell<bool>>> {
    value
        .as_object()
        .and_then(|object| {
            object
                .downcast_ref::<TargetRef>()
                .and_then(|target| target.aborted.clone())
        })
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Expected AbortSignal")
                .into()
        })
}

pub(crate) fn add(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let target = Target::from_value(this, context)?;
    let name = to_rust_string(args.first().unwrap_or(&JsValue::undefined()), context)?;
    let callback = args.get(1).and_then(JsValue::as_object);
    let mut capture = false;
    let mut once = false;
    let mut passive = false;
    let mut abort_signal = None;
    if let Some(options) = args.get(2) {
        if let Some(options) = options.as_object() {
            capture = options
                .get(JsString::from("capture"), context)?
                .to_boolean();
            once = options.get(JsString::from("once"), context)?.to_boolean();
            passive = options
                .get(JsString::from("passive"), context)?
                .to_boolean();
            let value = options.get(JsString::from("signal"), context)?;
            if !value.is_null_or_undefined() {
                abort_signal = Some(signal(&value)?);
            }
        } else {
            capture = options.to_boolean();
        }
    }
    let Some(callback) = callback else {
        return Ok(JsValue::undefined());
    };
    if abort_signal.as_ref().is_some_and(|flag| flag.get()) {
        return Ok(JsValue::undefined());
    }
    target.with_map(&ctx, |map| {
        let list = map.entry(name).or_default();
        list.retain(Listener::active);
        if !list
            .iter()
            .any(|entry| entry.capture == capture && JsObject::equals(&entry.callback, &callback))
        {
            list.push(Listener {
                callback,
                capture,
                once,
                passive,
                removed: Default::default(),
                abort_signal,
            });
        }
    });
    Ok(JsValue::undefined())
}

pub(crate) fn remove(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ctx = dom_ctx(context)?;
    let target = Target::from_value(this, context)?;
    let name = to_rust_string(args.first().unwrap_or(&JsValue::undefined()), context)?;
    let callback = args.get(1).and_then(JsValue::as_object);
    let capture = match args.get(2) {
        Some(value) if value.is_object() => value
            .as_object()
            .unwrap()
            .get(JsString::from("capture"), context)?
            .to_boolean(),
        Some(value) => value.to_boolean(),
        None => false,
    };
    if let Some(callback) = callback {
        target.with_map(&ctx, |map| {
            if let Some(list) = map.get_mut(&name) {
                list.retain(|entry| {
                    let remove =
                        entry.capture == capture && JsObject::equals(&entry.callback, &callback);
                    if remove {
                        entry.removed.set(true);
                    }
                    !remove && entry.active()
                });
            }
        });
    }
    Ok(JsValue::undefined())
}
