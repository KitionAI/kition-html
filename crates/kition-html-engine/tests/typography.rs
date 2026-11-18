use kition_html_engine::{RenderOptions, Resources, render};

#[test]
fn automatic_optical_size_matches_explicit_axis_and_respects_none() -> Result<(), String> {
    let render_text = |css: &str| {
        let mut resources = Resources::new();
        resources.insert(
            "/assets/Inter.ttf".into(),
            include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
        );
        render(
            &format!(
                "<!doctype html><style>body{{margin:0}}p{{margin:0;font:32px/1.5 Inter;{css}}}</style><p>Optical sizing</p>"
            ),
            resources,
            &RenderOptions {
                width: 320,
                height: 64,
                ..Default::default()
            },
        )
    };
    let automatic = render_text("font-optical-sizing:auto")?;
    let explicit = render_text("font-optical-sizing:none;font-variation-settings:'opsz' 32")?;
    let disabled = render_text("font-optical-sizing:none")?;
    let override_auto = render_text("font-optical-sizing:auto;font-variation-settings:'opsz' 14")?;
    assert_eq!(automatic.rgba, explicit.rgba);
    assert_ne!(automatic.rgba, disabled.rgba);
    assert_eq!(override_auto.rgba, disabled.rgba);
    Ok(())
}

fn render_bidi(css: &str, body: &str) -> Result<kition_html_engine::Rendered, String> {
    let mut resources = Resources::new();
    resources.insert(
        "/assets/Mono.ttf".into(),
        include_bytes!("../../../conformance/assets/JetBrainsMono-Variable.ttf").to_vec(),
    );
    render(
        &format!(
            "<!doctype html><style>body{{margin:0;font:24px/1.5 'JetBrains Mono';font-variant-ligatures:none}}p{{margin:0;text-align:left;{css}}}</style><p>{body}</p>"
        ),
        resources,
        &RenderOptions {
            width: 480,
            height: 64,
            ..Default::default()
        },
    )
}

#[test]
fn bidi_override_reorders_letters_and_digits_without_changing_source() -> Result<(), String> {
    let actual = render_bidi("direction:rtl;unicode-bidi:bidi-override", "ABC 123 DEF")?;
    let expected = render_bidi("", "FED 321 CBA")?;
    assert_eq!(actual.rgba, expected.rgba);
    Ok(())
}

#[test]
fn inline_bidi_override_stays_within_its_span() -> Result<(), String> {
    let actual = render_bidi(
        "",
        "Start <span style='direction:rtl;unicode-bidi:isolate-override'>ABC 123</span> End",
    )?;
    let expected = render_bidi("", "Start 321 CBA End")?;
    assert_eq!(actual.rgba, expected.rgba);
    Ok(())
}
