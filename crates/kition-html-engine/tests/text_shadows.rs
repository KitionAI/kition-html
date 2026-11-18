use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, body: &str, scale: f64) -> Result<Rendered, String> {
    let mut resources = Resources::new();
    resources.insert(
        "/assets/Inter.ttf".into(),
        include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
    );
    render(
        &format!(
            "<!doctype html><style>@font-face{{font-family:Inter;src:url(/assets/Inter.ttf)}}body{{margin:0;background:white;font:32px/48px Inter}}p{{margin:20px;position:relative}}{css}</style><main id=stage>{body}</main>"
        ),
        resources,
        &RenderOptions {
            width: 320,
            height: 160,
            scale,
            ..Default::default()
        },
    )
}

#[test]
fn zero_blur_shadow_uses_the_same_glyphs_and_device_scaled_offsets() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        let shadow = frame(
            "p{color:transparent;text-shadow:40px 24px 0 red}",
            "<p>Shadow</p>",
            scale,
        )?;
        let reference = frame("p{color:red;left:40px;top:24px}", "<p>Shadow</p>", scale)?;
        assert_eq!(shadow.rgba, reference.rgba, "scale {scale}");
    }
    Ok(())
}

#[test]
fn first_shadow_is_on_top_and_transparent_shadows_do_not_abort() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let reference = frame(
            "p{color:transparent;text-shadow:40px 24px 0 red}",
            "<p>Shadow</p>",
            scale,
        )?;
        let layered = frame(
            "p{color:transparent;text-shadow:40px 24px 0 transparent,40px 24px 0 red,40px 24px 0 blue}",
            "<p>Shadow</p>",
            scale,
        )?;
        // Fully covered glyph interiors must use the first opaque shadow.
        let mut interiors = 0;
        for (a, b) in reference
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .zip(layered.rgba.as_chunks::<4>().0.iter())
        {
            if *a == [255, 0, 0, 255] {
                assert_eq!(a, b);
                interiors += 1;
            }
        }
        assert!(interiors > 100);
    }
    Ok(())
}

#[test]
fn inherited_shadows_have_no_seams_at_equal_inline_styles() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let plain = frame(
            "p{color:transparent;text-shadow:3px 4px 4px #e5486a88}",
            "<p>Shadow test</p>",
            scale,
        )?;
        let split = frame(
            "p{color:transparent;text-shadow:3px 4px 4px #e5486a88}",
            "<p>Sha<span>dow</span> test</p>",
            scale,
        )?;
        assert_eq!(plain.rgba, split.rgba, "scale {scale}");
        assert!(plain.rgba.as_chunks::<4>().0.iter().any(|p| p[1] < 250));
    }
    Ok(())
}

#[test]
fn currentcolor_is_resolved_per_run_and_none_suppresses_inheritance() -> Result<(), String> {
    let inherited = frame(
        "p{color:red;text-shadow:0 60px currentColor}span{color:blue;text-shadow:none}",
        "<p>Red <span>Blue</span></p>",
        1.0,
    )?;
    let reference = frame(
        "p{color:red}span{color:blue}#copy{position:absolute;top:60px;left:0;color:red}",
        "<p>Red <span>Blue</span></p><p id=copy>Red </p>",
        1.0,
    )?;
    assert_eq!(inherited.rgba, reference.rgba);
    Ok(())
}

#[test]
fn text_shadow_offsets_follow_css_rotation() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let shadow = frame(
            "p{color:transparent;text-shadow:20px 0 red;transform:rotate(90deg);transform-origin:0 0;left:100px}",
            "<p>Shadow</p>",
            scale,
        )?;
        let reference = frame(
            "p{color:red;transform:rotate(90deg);transform-origin:0 0;left:100px;top:20px}",
            "<p>Shadow</p>",
            scale,
        )?;
        assert_eq!(shadow.rgba, reference.rgba);
    }
    Ok(())
}
