use kition_html_engine::{RenderOptions, default_resources, render};

#[test]
fn text_setters_observe_resets_and_exception_order_during_string_conversion() -> Result<(), String>
{
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../conformance/canvas-text-coercion-cases.json"
    ))
    .map_err(|error| error.to_string())?;
    let mut failures = Vec::new();
    for case in cases {
        let script = case["script"].as_str().ok_or("missing script")?;
        let id = case["id"].as_str().ok_or("missing id")?;
        let html = format!(
            "<!doctype html><canvas id=canvas></canvas><script>const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';if(!(()=>{{{script}}})())throw Error('{id}');</script>"
        );
        let image = render(
            &html,
            default_resources(),
            &RenderOptions {
                width: 320,
                height: 128,
                ..Default::default()
            },
        )?;
        if !image.diagnostics.is_empty() {
            failures.push((id.to_owned(), image.diagnostics));
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
    Ok(())
}
