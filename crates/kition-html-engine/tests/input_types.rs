use blitz_dom::DocumentConfig;
use blitz_vibey_script::ScriptDocument;
use kition_html_engine::{RenderOptions, default_resources, render};

#[test]
fn enumerated_input_type_preserves_attribute_bytes_and_uses_the_text_default() {
    let mut document =
        ScriptDocument::from_html("<!doctype html><input id=input>", DocumentConfig::default())
            .without_timer_thread();
    document.eval(r#"
        const input=document.getElementById('input');
        function check(value){if(!value)throw Error('input type mismatch')}
        check(input.type==='text'&&!input.hasAttribute('type'));
        for(const value of ['', 'unknown-kind', ' TEXT ', 'datetime']){
            input.type=value;check(input.type==='text'&&input.getAttribute('type')===value);
        }
        for(const type of ['text','number','email','tel','search','checkbox','radio','hidden','button','date','datetime-local']){
            input.type=type.toUpperCase();check(input.type===type&&input.getAttribute('type')===type.toUpperCase());
        }
        input.removeAttribute('type');check(input.type==='text');
        input.type='CHECKBOX';input.value='saved';check(input.value==='saved'&&input.getAttribute('value')==='saved');
    "#);
    let errors = document.take_js_errors();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn uppercase_and_invalid_input_types_use_the_same_layout_and_paint_as_their_states()
-> Result<(), String> {
    let options = RenderOptions {
        width: 320,
        height: 100,
        ..Default::default()
    };
    let frame = |kind: &str| {
        render(
            &format!(
                "<!doctype html><style>body{{margin:0}}input{{font:14px Inter}}</style><input {kind} value='42' checked>"
            ),
            default_resources(),
            &options,
        )
    };
    for kind in [
        "text", "number", "email", "tel", "search", "checkbox", "radio", "hidden", "button",
    ] {
        assert_eq!(
            frame(&format!("type={kind}"))?.rgba,
            frame(&format!("type={}", kind.to_uppercase()))?.rgba,
            "{kind}"
        );
    }
    let expected = frame("type=text")?;
    for attributes in ["", "type=''", "type=unknown-kind"] {
        assert_eq!(frame(attributes)?.rgba, expected.rgba, "{attributes}");
    }
    let number = render(
        "<!doctype html><style>body{margin:0}input{font:14px Inter;padding:10px;border:1px solid}</style><main id=stage><input id=number type=number value=42></main>",
        default_resources(),
        &options,
    )?;
    let control = number
        .geometry
        .iter()
        .find(|node| node["id"] == "number")
        .ok_or("missing number")?;
    assert_eq!(control["height"], 39.);
    Ok(())
}

#[test]
fn uppercase_checkable_controls_keep_activation_and_checked_selector_state() {
    let mut document = ScriptDocument::from_html("<!doctype html><input id=box type=CHECKBOX checked><input id=first type=RADIO name=choice checked><input id=second type=radio name=choice>",DocumentConfig::default()).without_timer_thread();
    document.eval(r#"
        const box=document.getElementById('box'),first=document.getElementById('first'),second=document.getElementById('second');
        if(!box.matches(':checked')||!first.matches(':checked'))throw Error('initial checked state');
        box.click();second.click();
        if(box.checked||box.matches(':checked')||first.checked||!second.checked)throw Error('activation state');
    "#);
    let errors = document.take_js_errors();
    assert!(errors.is_empty(), "{errors:?}");
}
