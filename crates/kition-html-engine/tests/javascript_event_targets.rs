use blitz_dom::DocumentConfig;
use blitz_vibey_script::ScriptDocument;

fn check(script: &str) {
    let mut document = ScriptDocument::from_html(
        "<!doctype html><div id=target></div>",
        DocumentConfig::default(),
    )
    .without_timer_thread();
    document.eval(&format!(
        "function check(value,message){{if(!value)throw Error(message)}}{script}"
    ));
    let errors = document.take_js_errors();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn standalone_targets_support_subclasses_identity_and_event_flags() {
    check(
        r#"
        class Bus extends EventTarget {}
        const bus=new Bus(),other=new Bus();
        check(bus instanceof Bus&&bus instanceof EventTarget,'subclass');
        check(document instanceof EventTarget&&document.body instanceof EventTarget&&window instanceof EventTarget,'DOM inheritance');
        const event=new CustomEvent('paint',{detail:7,cancelable:true});let calls=0;
        bus.addEventListener('paint',function(e){calls++;check(this===bus&&e.target===bus&&e.currentTarget===bus&&e.eventPhase===2&&e.detail===7,'target fields');e.preventDefault()});
        other.dispatchEvent(new Event('paint'));check(calls===0,'independent listeners');
        check(!bus.dispatchEvent(event)&&calls===1,'cancellation');
        check(event.currentTarget===null&&event.eventPhase===0,'dispatch cleanup');
    "#,
    );
}

#[test]
fn abort_signals_preserve_reasons_and_dispatch_once() {
    check(
        r#"
        const controller=new AbortController(),signal=controller.signal,reason={code:7};let calls=0;
        check(!signal.aborted&&signal.reason===undefined&&signal instanceof AbortSignal&&signal instanceof EventTarget,'initial state');
        signal.throwIfAborted();
        signal.addEventListener('abort',function(event){calls++;check(this===signal&&event.target===signal&&signal.aborted&&signal.reason===reason,'abort state visible')});
        controller.abort(reason);controller.abort('ignored');
        check(calls===1&&signal.reason===reason,'abort only once');
        let thrown=false;try{signal.throwIfAborted()}catch(error){thrown=error===reason}check(thrown,'throws original reason');
        check(AbortSignal.abort().reason.name==='AbortError','default reason');
    "#,
    );
}

#[test]
fn aborted_registration_is_removed_from_all_target_types() {
    check(
        r#"
        for(const target of [new EventTarget(),document.body,window]) {
            const controller=new AbortController();let count=0;const later=()=>count++;
            target.addEventListener('paint',()=>controller.abort(),{once:true});
            target.addEventListener('paint',later,{signal:controller.signal});
            target.dispatchEvent(new Event('paint'));check(count===0,'abort during dispatch');
            target.addEventListener('paint',later);target.dispatchEvent(new Event('paint'));check(count===1,'register after abort');
            target.addEventListener('paint',()=>{throw Error('already aborted listener ran')},{signal:AbortSignal.abort()});
            target.dispatchEvent(new Event('paint'));check(count===2,'preaborted listener ignored');
        }
    "#,
    );
}

#[test]
fn abort_dependency_algorithms_are_independent_of_event_propagation() {
    check(
        r#"
        const source=new AbortController(),second=new AbortController();
        source.signal.addEventListener('abort',event=>event.stopImmediatePropagation());
        const combined=AbortSignal.any([source.signal,second.signal]);const nested=AbortSignal.any([combined]);
        const order=[];combined.onabort=()=>{check(nested.aborted,'all dependents marked before listeners');order.push('combined')};nested.onabort=()=>order.push('nested');
        source.abort('done');second.abort('later');
        check(combined.reason==='done'&&nested.reason==='done'&&order.join(',')==='combined,nested','dependent signals');
        check(AbortSignal.any([AbortSignal.abort(7)]).reason===7&&!AbortSignal.any([]).aborted,'preaborted and empty sources');
        let rejected=false;try{document.body.addEventListener('paint',()=>{},{signal:{aborted:false}})}catch(error){rejected=error instanceof TypeError}check(rejected,'signal type validation');
    "#,
    );
}

#[test]
fn event_properties_cannot_rewrite_native_dispatch_flags() {
    check(
        r#"
        const event=new Event('paint',{bubbles:true,composed:true});
        event.type='other';event.cancelable=true;event.bubbles=false;event.composed=false;event.isTrusted=true;
        check(event.type==='paint'&&!event.cancelable&&event.bubbles&&event.composed&&!event.isTrusted,'readonly fields');
        Object.defineProperty(event,'type',{value:'other'});Object.defineProperty(event,'cancelable',{value:true});
        event.preventDefault();check(!event.defaultPrevented,'native cancelable slot');
        let count=0;document.body.addEventListener('paint',e=>{count++;e.target=window;e.currentTarget=window;e.eventPhase=3;check(e.target===document.body&&e.currentTarget===document.body&&e.eventPhase===2,'readonly dispatch fields')});
        document.body.dispatchEvent(event);check(count===1,'native event type');
    "#,
    );
}

#[test]
fn composed_path_is_frozen_during_dispatch_and_cleared_afterwards() {
    check(
        r#"
        const parent=document.createElement('section'),target=document.createElement('div');parent.appendChild(target);document.body.appendChild(parent);
        const event=new Event('paint',{bubbles:true});let calls=0;
        target.addEventListener('paint',e=>{target.remove();const path=e.composedPath();check(path[0]===target&&path[1]===parent&&path.at(-2)===document&&path.at(-1)===window,'frozen DOM path');path.length=0;check(e.composedPath().length>0,'returned path is a copy');calls++});
        parent.addEventListener('paint',()=>calls++);target.dispatchEvent(event);
        check(calls===2&&event.composedPath().length===0&&event.currentTarget===null&&event.eventPhase===0,'path cleanup');
        const bus=new EventTarget();bus.addEventListener('paint',e=>check(e.composedPath().length===1&&e.composedPath()[0]===bus,'standalone path'));bus.dispatchEvent(new Event('paint'));
    "#,
    );
}

#[test]
fn legacy_cancel_properties_obey_passive_and_cancelable_rules() {
    check(
        r#"
        const event=new Event('paint',{bubbles:true,cancelable:true});let count=0;
        document.body.addEventListener('paint',e=>{e.cancelBubble=true;e.cancelBubble=false;e.returnValue=false},{passive:true});
        document.body.addEventListener('paint',()=>count++);document.addEventListener('paint',()=>count+=10);
        check(document.body.dispatchEvent(event)&&event.returnValue&&count===1&&!event.cancelBubble,'passive returnValue and sticky cancelBubble');
        event.returnValue=false;check(event.defaultPrevented&&!event.returnValue,'cancelable returnValue');
    "#,
    );
}

#[test]
fn error_event_uses_legacy_window_handler_arguments_and_true_cancellation() {
    check(
        r#"
        const reason=new Error('example'),event=new ErrorEvent('error',{message:'example',filename:'sample.js',lineno:7,colno:3,error:reason,cancelable:true});
        let called=false;window.onerror=function(message,file,line,column,error){called=this===window&&message==='example'&&file==='sample.js'&&line===7&&column===3&&error===reason;return true};
        check(!window.dispatchEvent(event)&&called&&event.defaultPrevented,'legacy error handler');
        const plain=new Event('error',{cancelable:true});window.onerror=event=>{check(event===plain,'plain Event argument');return false};
        check(!window.dispatchEvent(plain),'plain error event uses normal cancellation');
    "#,
    );
}

#[test]
fn reported_script_errors_use_the_same_listener_rules_and_cleanup() {
    let mut document = ScriptDocument::from_html("<!doctype html>", DocumentConfig::default())
        .without_timer_thread();
    document.eval("let seen=null;let count=0;const controller=new AbortController();window.addEventListener('error',()=>{throw Error('aborted listener ran')},{signal:controller.signal});controller.abort();window.addEventListener('error',{handleEvent(event){seen=event;count++;event.stopImmediatePropagation()}},{once:true});window.onerror=()=>{throw Error('stopped handler ran')};throw Error('reported example')");
    let errors = document.take_js_errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("reported example"));
    document.eval("if(count!==1||!(seen instanceof ErrorEvent)||seen.currentTarget!==null||seen.eventPhase!==0||seen.composedPath().length!==0)throw Error('error dispatch cleanup')");
    assert!(document.take_js_errors().is_empty());
}
