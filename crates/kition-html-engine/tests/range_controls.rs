use blitz_dom::DocumentConfig;
use blitz_vibey_script::ScriptDocument;
use kition_html_engine::{RenderOptions, Resources, Session, render};

fn check(body: &str, script: &str) {
    let mut doc =
        ScriptDocument::from_html(&format!("<!doctype html>{body}"), DocumentConfig::default())
            .without_timer_thread();
    doc.eval(&format!(
        "function check(value,message){{if(!value)throw Error(message)}}{script}"
    ));
    let errors = doc.take_js_errors();
    assert!(errors.is_empty(), "{errors:?}");
}
#[test]
fn progress_properties_clamp_live_values_and_validate_assignments() {
    check(
        "<progress id=control></progress>",
        r#"
        const control=document.getElementById('control');
        check(control.max===1&&control.value===0&&control.position===-1&&control.matches(':indeterminate'),'defaults');
        control.max=100;control.value=65;
        check(control.value===65&&control.max===100&&control.position===.65&&!control.matches(':indeterminate'),'numeric properties');
        control.value=120;check(control.value===100&&control.getAttribute('value')==='120','clamped live value');
        control.value=-1;check(control.value===0&&control.getAttribute('value')==='0','negative assignment');
        control.max=-1;check(control.max===1&&control.getAttribute('max')==='1','invalid max assignment');
        let rejected=false;try{control.value=NaN}catch(error){rejected=error instanceof TypeError}check(rejected,'nonfinite value rejected');
        control.removeAttribute('value');check(control.position===-1&&control.matches(':indeterminate'),'indeterminate restoration');
    "#,
    );
}
#[test]
fn meter_properties_resolve_constraints_without_rewriting_attributes() {
    check(
        "<meter id=control min=20 max=100 low=30 high=70 value=72></meter>",
        r#"
        const control=document.getElementById('control');
        check(control.value===72&&control.min===20&&control.max===100&&control.optimum===60,'range');
        control.low=200;check(control.low===100&&control.high===100&&control.getAttribute('low')==='200','low clamping');
        control.value=-1;check(control.value===20&&control.getAttribute('value')==='-1','value clamping');
        control.max=-2;check(control.max===20&&control.value===20,'collapsed range');
        let rejected=false;try{control.optimum=Infinity}catch(error){rejected=error instanceof TypeError}check(rejected,'nonfinite optimum rejected');
    "#,
    );
}
fn source(tag: &str, attrs: &str) -> String {
    format!(
        "<!doctype html><style>body{{margin:0}}progress,meter{{display:block;width:200px;height:24px;accent-color:#5645d4}}</style><{tag} id=control {attrs}>Fallback text must stay hidden</{tag}>"
    )
}
#[test]
fn native_tracks_have_numeric_fills_and_hide_fallback_children() -> Result<(), String> {
    let options = RenderOptions {
        width: 240,
        height: 80,
        ..Default::default()
    };
    for (tag, attrs, color) in [
        ("progress", "max=100 value=50", [86, 69, 212, 255]),
        ("meter", "max=100 value=50", [16, 124, 16, 255]),
        (
            "meter",
            "max=100 value=50 low=30 high=70 optimum=0",
            [255, 185, 0, 255],
        ),
        (
            "meter",
            "max=100 value=80 low=30 high=70 optimum=0",
            [216, 59, 1, 255],
        ),
    ] {
        let frame = render(&source(tag, attrs), Resources::new(), &options)?;
        let pixel = |x: usize, y: usize| &frame.rgba[(y * 240 + x) * 4..(y * 240 + x) * 4 + 4];
        assert_eq!(pixel(20, 12), &color, "{tag} {attrs}");
        assert_eq!(pixel(180, 12), &[239, 239, 239, 255]);
        assert_eq!(pixel(20, 2), &[255, 255, 255, 255]);
    }
    Ok(())
}
#[test]
fn script_updates_repaint_an_existing_progress_without_changing_its_box() -> Result<(), String> {
    let html = source("progress", "value=0 max=100")
        + "<script>document.body.onclick=()=>document.getElementById('control').value=75</script>";
    let options = RenderOptions {
        width: 240,
        height: 80,
        ..Default::default()
    };
    let mut session = Session::new(&html, Resources::new(), &options)?;
    let before = session.frame(1.)?;
    let offset = (12 * 240 + 20) * 4;
    assert_eq!(&before.rgba[offset..offset + 4], &[239, 239, 239, 255]);
    session.click(20., 12.)?;
    let after = session.frame(1.)?;
    assert_eq!(&after.rgba[offset..offset + 4], &[86, 69, 212, 255]);
    Ok(())
}
