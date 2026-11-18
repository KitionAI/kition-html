use kition_html_engine::{RenderOptions, default_resources, render};

#[test]
fn text_methods_follow_argument_conversion_and_bitmap_reset_order() -> Result<(), String> {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../conformance/canvas-text-arguments-cases.json"
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
