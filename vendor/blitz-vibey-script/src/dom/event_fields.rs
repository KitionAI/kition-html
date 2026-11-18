//! Event properties read native slots; assigning public fields cannot alter dispatch.
use super::{define_accessor, define_method, event::EventRef, js_str};
use boa_engine::{
    Context, JsNativeError, JsResult, JsValue,
    object::{JsObject, builtins::JsArray},
};

fn read(this: &JsValue, apply: impl FnOnce(&EventRef) -> JsValue) -> JsResult<JsValue> {
    this.as_object()
        .and_then(|object| object.downcast_ref::<EventRef>().map(|event| apply(&event)))
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Invalid Event receiver")
                .into()
        })
}

pub(crate) fn init(proto: &JsObject, context: &mut Context) {
    macro_rules! getter {
        ($name:literal, $event:ident, $value:expr) => {
            define_accessor(
                proto,
                $name,
                Some(|this, _, _| read(this, |$event| $value)),
                None,
                context,
            );
        };
    }
    getter!("type", event, js_str(&event.type_name));
    getter!("bubbles", event, event.bubbles.into());
    getter!("cancelable", event, event.cancelable.into());
    getter!("target", event, event.target.borrow().clone());
    getter!("srcElement", event, event.target.borrow().clone());
    getter!(
        "currentTarget",
        event,
        event.current_target.borrow().clone()
    );
    getter!("eventPhase", event, event.phase.get().into());
    getter!("isTrusted", event, event.trusted.get().into());
    getter!("composed", event, event.composed.get().into());
    getter!("timeStamp", event, event.timestamp.into());
    define_accessor(
        proto,
        "cancelBubble",
        Some(|this, _, _| read(this, |event| event.stopped.get().into())),
        Some(|this, args, _| {
            read(this, |event| {
                if args.first().is_some_and(JsValue::to_boolean) {
                    event.stopped.set(true);
                }
                JsValue::undefined()
            })
        }),
        context,
    );
    define_accessor(
        proto,
        "returnValue",
        Some(|this, _, _| read(this, |event| (!event.prevented.get()).into())),
        Some(|this, args, context| {
            if !args.first().is_some_and(JsValue::to_boolean) {
                super::event::prevent_default(this, &[], context)?;
            }
            Ok(JsValue::undefined())
        }),
        context,
    );
    define_method(
        proto,
        "composedPath",
        0,
        |this, _, context| {
            let path = this
                .as_object()
                .and_then(|object| {
                    object
                        .downcast_ref::<EventRef>()
                        .map(|event| event.path.borrow().clone())
                })
                .ok_or_else(|| JsNativeError::typ().with_message("Invalid Event receiver"))?;
            Ok(JsArray::from_iter(path, context).into())
        },
        context,
    );
}
