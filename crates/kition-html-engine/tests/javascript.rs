use kition_html_engine::{RenderOptions, Resources, Session, render};

fn source(script: &str) -> String {
    format!(
        r#"<!doctype html><style>body{{margin:0}}#target{{width:32px;height:32px;background:red}}</style><main id="stage"><div id="target"></div></main><script>{script}</script>"#
    )
}
fn options() -> RenderOptions {
    RenderOptions {
        width: 64,
        height: 64,
        ..Default::default()
    }
}
fn purple(frame: &kition_html_engine::Rendered) {
    assert_eq!(&frame.rgba[..4], &[86, 69, 212, 255]);
    assert!(frame.diagnostics.is_empty(), "{:?}", frame.diagnostics);
}

#[test]
fn dom_cssom_promises_and_lifecycle() -> Result<(), String> {
    for script in [
        "document.getElementById('target').style.background='#5645d4'",
        "document.querySelector('#target').innerHTML='<div style=\"width:32px;height:32px;background:#5645d4\"></div>'",
        "Promise.resolve().then(()=>document.querySelector('#target').style.background='#5645d4')",
        "document.addEventListener('DOMContentLoaded',()=>document.querySelector('#target').style.background='#5645d4')",
        "window.addEventListener('load',()=>document.querySelector('#target').style.background='#5645d4')",
        "requestAnimationFrame(()=>document.querySelector('#target').style.background='#5645d4')",
        "setTimeout(()=>document.querySelector('#target').style.background='#5645d4',0)",
    ] {
        purple(&render(&source(script), Resources::new(), &options())?);
    }
    Ok(())
}

#[test]
fn live_events_timers_and_zoom_keep_the_same_context() -> Result<(), String> {
    let html = source(
        "let clicks=0; document.querySelector('#target').addEventListener('click',()=>{clicks++; document.querySelector('#target').style.background=clicks===2?'#5645d4':'blue'});setTimeout(()=>document.querySelector('#target').style.width='48px',100)",
    );
    let mut session = Session::new(&html, Resources::new(), &options())?;
    session.frame(2.0)?;
    assert!(session.click(4.0, 4.0)?);
    session.frame(1.0)?;
    assert!(session.click(4.0, 4.0)?);
    purple(&session.frame(1.0)?);
    session.advance(100)?;
    let frame = session.frame(1.0)?;
    assert_eq!(frame.geometry[0]["width"], 48.0);
    Ok(())
}

#[test]
fn external_scripts_modules_and_fetch_use_only_supplied_resources() -> Result<(), String> {
    let resources=Resources::from([
        ("/assets/main.js".into(),b"import {color} from './color.js';document.querySelector('#target').style.background=color".to_vec()),
        ("/assets/color.js".into(),b"export const color='#5645d4'".to_vec()),
        ("/assets/color.txt".into(),b"#5645d4".to_vec()),
    ]);
    let html = source("") + "<script type=module src=/assets/main.js></script>";
    purple(&render(&html, resources.clone(), &options())?);
    purple(&render(
        &source(
            "fetch('/assets/color.txt').then(r=>r.text()).then(color=>document.querySelector('#target').style.background=color)",
        ),
        resources,
        &options(),
    )?);
    for url in [
        "file:///etc/passwd",
        "https://example.com/secret.js",
        "http://127.0.0.1:8080/secret.js",
    ] {
        let html = source("") + &format!("<script src='{url}'></script>");
        let frame = render(&html, Resources::new(), &options())?;
        assert!(
            frame
                .diagnostics
                .iter()
                .any(|message| message.contains("resource-unavailable")),
            "{:?}",
            frame.diagnostics
        );
        assert_eq!(&frame.rgba[..4], &[255, 0, 0, 255]);
    }
    Ok(())
}

#[test]
fn supplied_resource_fetch_honors_abort_before_loading_and_before_body_consumption()
-> Result<(), String> {
    let resources = Resources::from([("/assets/color.txt".into(), b"#5645d4".to_vec())]);
    for operation in [
        "fetch('/not-supplied',{signal:AbortSignal.abort('done')})",
        "(()=>{const controller=new AbortController();const pending=fetch('/not-supplied',{signal:controller.signal});controller.abort('done');return pending})()",
        "(()=>{const controller=new AbortController();return fetch('/assets/color.txt',{signal:controller.signal}).then(response=>{controller.abort('done');return response.text()})})()",
        "(()=>{const controller=new AbortController();return fetch('/assets/color.txt',{signal:controller.signal}).then(response=>{const copy=response.clone();controller.abort('done');return copy.text()})})()",
    ] {
        let script = format!(
            "{operation}.then(()=>{{throw Error('aborted fetch resolved')}},reason=>{{if(reason!=='done')throw Error('wrong abort reason');document.getElementById('target').style.background='#5645d4'}})"
        );
        purple(&render(&source(&script), resources.clone(), &options())?);
    }
    purple(&render(
        &source(
            "fetch('/not-supplied',{signal:{aborted:false}}).then(()=>{throw Error('invalid signal accepted')},reason=>{if(!(reason instanceof TypeError))throw Error('wrong signal error');document.getElementById('target').style.background='#5645d4'})",
        ),
        resources,
        &options(),
    )?);
    Ok(())
}

#[test]
fn exceptions_and_runaway_scripts_return_diagnostics() -> Result<(), String> {
    for script in [
        "throw new Error('fixture failure')",
        "while(true){}",
        "function again(){again()}again()",
        "function again(){queueMicrotask(again)}again()",
    ] {
        let frame = render(&source(script), Resources::new(), &options())?;
        assert!(
            frame
                .diagnostics
                .iter()
                .any(|message| message.starts_with("html.javascript-error:")),
            "{script}: {:?}",
            frame.diagnostics
        );
    }
    Ok(())
}

#[test]
fn recursive_zero_delay_timers_are_bounded() -> Result<(), String> {
    let frame = render(
        &source("function again(){setTimeout(again,0)}again()"),
        Resources::new(),
        &options(),
    )?;
    assert!(
        frame
            .diagnostics
            .iter()
            .any(|message| message.contains("timer-limit"))
    );
    Ok(())
}

#[test]
fn script_types_defer_order_and_noscript_match_enabled_scripting() -> Result<(), String> {
    let resources = Resources::from([(
        "/assets/deferred.js".into(),
        b"document.querySelector('#target').style.background=ready?'#5645d4':'red'".to_vec(),
    )]);
    let html = source("")
        + r#"<script defer src="/assets/deferred.js"></script><script>var ready=true</script><script nomodule>throw new Error('nomodule executed')</script><script type="application/json">{"invalid JS":true}</script><noscript><div style="position:absolute;inset:0;background:red"></div></noscript>"#;
    purple(&render(&html, resources, &options())?);
    purple(&render(
        &(source("")
            + r#"<script type="text/ecmascript">document.querySelector('#target').style.background='#5645d4'</script>"#),
        Resources::new(),
        &options(),
    )?);
    Ok(())
}

#[test]
fn scripting_media_queries_and_noscript_share_enabled_state() -> Result<(), String> {
    let html = source("")
        + r#"<style>@media(scripting:enabled){#target{background:#5645d4}}@media(scripting:none){#target{background:red}}@media(scripting:initial-only){#target{background:red}}</style><noscript id="fallback"><div>Fallback</div></noscript><script>if(getComputedStyle(document.getElementById('fallback')).display!=='none')throw new Error('noscript is visible')</script>"#;
    purple(&render(&html, Resources::new(), &options())?);
    Ok(())
}

#[test]
fn intervals_can_cancel_themselves_and_click_coordinates_are_preserved() -> Result<(), String> {
    let html = source(
        "let ticks=0;const timer=setInterval(()=>{if(++ticks===3){clearInterval(timer);document.querySelector('#target').style.background='#5645d4'}},20);document.querySelector('#target').onclick=event=>{if(event.clientX!==4||event.clientY!==5)throw new Error('wrong click coordinates')}",
    );
    let mut session = Session::new(&html, Resources::new(), &options())?;
    assert!(session.advance(100)?);
    purple(&session.frame(1.0)?);
    assert!(!session.advance(100)?);
    session.click(4.0, 5.0)?;
    purple(&session.frame(1.0)?);
    Ok(())
}
