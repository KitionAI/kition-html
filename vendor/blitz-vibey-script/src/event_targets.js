// Standalone event targets use the same native listener and dispatch rules as nodes.
(() => {
    const create = __blitz_create_target;
    const markAborted = __blitz_abort_signal;
    const signals = new WeakMap(), controllers = new WeakMap();
    const methods = Object.fromEntries(['addEventListener','removeEventListener','dispatchEvent'].map(name=>[name,Node.prototype[name]]));
    class EventTarget {
        constructor(token) {
            const target=create(token===signals);
            Object.setPrototypeOf(target,new.target.prototype);
            return target;
        }
    }
    for(const [name,value] of Object.entries(methods))Object.defineProperty(EventTarget.prototype,name,{value,writable:true,configurable:true});
    Object.setPrototypeOf(Node.prototype,EventTarget.prototype);
    Object.setPrototypeOf(Window.prototype,EventTarget.prototype);

    function data(signal) {
        const value=signals.get(signal);
        if(!value)throw new TypeError('Invalid AbortSignal receiver');
        return value;
    }
    function abort(signal,reason) {
        if(data(signal).aborted)return;
        if(reason===undefined)reason=new DOMException('The operation was aborted','AbortError');
        const pending=[signal], changed=[];
        for(let index=0;index<pending.length;index++) {
            const current=pending[index],state=data(current);
            if(state.aborted)continue;
            state.aborted=true;state.reason=reason;markAborted(current);
            changed.push(current);pending.push(...state.followers);state.followers.clear();
        }
        // All dependent flags change before listeners run. Event propagation
        // cannot suppress the cancellation algorithms of dependent signals.
        for(const current of changed)current.dispatchEvent(new Event('abort'));
    }
    class AbortSignal extends EventTarget {
        constructor(token) {
            if(token!==signals)throw new TypeError('Illegal constructor');
            super(signals);
            signals.set(this,{aborted:false,reason:undefined,followers:new Set()});
        }
        get aborted(){return data(this).aborted;}
        get reason(){return data(this).reason;}
        throwIfAborted(){const state=data(this);if(state.aborted)throw state.reason;}
        static abort(reason){const signal=new AbortSignal(signals);abort(signal,reason);return signal;}
        static timeout(milliseconds) {
            milliseconds=Number(milliseconds);
            if(!Number.isFinite(milliseconds)||milliseconds<0||milliseconds>Number.MAX_SAFE_INTEGER)throw new TypeError('Invalid abort timeout');
            const signal=new AbortSignal(signals);
            setTimeout(()=>abort(signal,new DOMException('The operation timed out','TimeoutError')),Math.trunc(milliseconds));
            return signal;
        }
        static any(values) {
            const sources=Array.from(values);for(const source of sources)data(source);
            const signal=new AbortSignal(signals),finished=sources.find(source=>data(source).aborted);
            if(finished)abort(signal,data(finished).reason);
            else for(const source of sources)data(source).followers.add(signal);
            return signal;
        }
    }
    Object.defineProperty(AbortSignal.prototype,'onabort',{value:null,writable:true,configurable:true});
    class AbortController {
        constructor(){controllers.set(this,new AbortSignal(signals));}
        get signal(){const signal=controllers.get(this);if(!signal)throw new TypeError('Invalid AbortController receiver');return signal;}
        abort(reason){abort(this.signal,reason);}
    }
    Object.assign(globalThis,{EventTarget,AbortSignal,AbortController});
})();
