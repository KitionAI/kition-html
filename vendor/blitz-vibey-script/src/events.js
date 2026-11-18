// Constructors share the native event flags used for input dispatch.
(() => {
    const create = __blitz_create_event;
    const markMouse = __blitz_mark_mouse_event;
    const markError = __blitz_mark_error_event;
    const nativePrototype = Object.getPrototypeOf(create("", false, false));
    class Event {
        constructor(type, options = {}) {
            if (!arguments.length) throw new TypeError("Event type is required");
            const event = create(String(type), Boolean(options?.bubbles), Boolean(options?.cancelable), Boolean(options?.composed));
            Object.setPrototypeOf(event, new.target.prototype);
            return event;
        }
    }
    Object.setPrototypeOf(Event.prototype, nativePrototype);
    for (const [name, value] of Object.entries({NONE:0,CAPTURING_PHASE:1,AT_TARGET:2,BUBBLING_PHASE:3})) {
        Object.defineProperty(Event, name, {value});
        Object.defineProperty(Event.prototype, name, {value});
    }
    class CustomEvent extends Event {
        constructor(type, options = {}) { super(type, options); this.detail = options?.detail ?? null; }
    }
    class UIEvent extends Event {
        constructor(type, options = {}) { super(type, options); this.detail = Number(options?.detail ?? 0); this.view = options?.view ?? null; }
    }
    class MouseEvent extends UIEvent {
        constructor(type, options = {}) {
            super(type, options);
            markMouse(this);
            for (const key of ["screenX","screenY","clientX","clientY","button","buttons"])
                this[key] = Number(options?.[key] ?? 0);
            for (const key of ["ctrlKey","shiftKey","altKey","metaKey"]) this[key] = Boolean(options?.[key]);
            this.relatedTarget = options?.relatedTarget ?? null;
        }
        get x() { return this.clientX; }
        get y() { return this.clientY; }
    }
    class ErrorEvent extends Event {
        constructor(type, options={}) {
            super(type,options);markError(this);
            for(const [key,value] of Object.entries({message:String(options?.message??''),filename:String(options?.filename??''),lineno:Number(options?.lineno??0)>>>0,colno:Number(options?.colno??0)>>>0,error:options?.error??null}))
                Object.defineProperty(this,key,{value,enumerable:true});
        }
    }
    Object.assign(globalThis, {Event, CustomEvent, UIEvent, MouseEvent, ErrorEvent});
    const clicking = new WeakSet();
    HTMLElement.prototype.click = function () {
        if (clicking.has(this) || (this.matches(":disabled") && /^(BUTTON|INPUT|SELECT|TEXTAREA)$/.test(this.tagName))) return;
        clicking.add(this);
        try { this.dispatchEvent(new MouseEvent("click", {bubbles:true, cancelable:true, composed:true, view:window})); }
        finally { clicking.delete(this); }
    };
})();
