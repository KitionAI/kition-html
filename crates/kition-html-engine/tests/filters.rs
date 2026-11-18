use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, html: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:0;background:white}}div{{width:32px;height:32px}}{css}</style>{html}"
        ),
        Resources::new(),
        &RenderOptions {
            width: 96,
            height: 64,
            scale,
            ..Default::default()
        },
    )
}

fn pixel(frame: &Rendered, x: u32, y: u32) -> &[u8] {
    let offset = ((y * frame.width + x) * 4) as usize;
    &frame.rgba[offset..offset + 4]
}

fn near(actual: &[u8], expected: [u8; 4]) {
    assert!(
        actual
            .iter()
            .zip(expected)
            .all(|(&a, b)| a.abs_diff(b) <= 1),
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn color_functions_use_srgb_and_preserve_unfiltered_siblings() -> Result<(), String> {
    for (filter, expected) in [
        ("grayscale(1)", [54, 54, 54, 255]),
        ("grayscale(1) contrast(1.4)", [25, 25, 25, 255]),
        ("invert(1)", [0, 255, 255, 255]),
        ("brightness(.5)", [128, 0, 0, 255]),
        ("contrast(0)", [128, 128, 128, 255]),
        ("saturate(0)", [54, 54, 54, 255]),
        ("sepia(1)", [100, 89, 69, 255]),
        ("hue-rotate(180deg)", [0, 109, 109, 255]),
    ] {
        let image = frame(
            &format!("div{{background:red}}.filtered{{filter:{filter}}}"),
            "<div class=filtered></div><div></div>",
            1.0,
        )?;
        assert!(
            image.diagnostics.is_empty(),
            "{filter}: {:?}",
            image.diagnostics
        );
        near(pixel(&image, 8, 8), expected);
        near(pixel(&image, 8, 40), [255, 0, 0, 255]);
        near(pixel(&image, 40, 8), [255, 255, 255, 255]);
    }
    Ok(())
}

#[test]
fn filter_order_clamps_between_functions_without_byte_rounding() -> Result<(), String> {
    let input = "div{background:rgb(204,51,26)}";
    let first = frame(
        &format!("{input}div{{filter:brightness(2) brightness(.5)}}"),
        "<div></div>",
        1.0,
    )?;
    let second = frame(
        &format!("{input}div{{filter:brightness(.5) brightness(2)}}"),
        "<div></div>",
        1.0,
    )?;
    near(pixel(&first, 8, 8), [128, 51, 26, 255]);
    near(pixel(&second, 8, 8), [204, 51, 26, 255]);
    Ok(())
}

#[test]
fn translucent_colors_are_unpremultiplied_before_filtering() -> Result<(), String> {
    let image = frame(
        "div{background:rgba(200,100,50,.5);filter:invert(1)}",
        "<div></div>",
        1.0,
    )?;
    near(pixel(&image, 8, 8), [155, 205, 230, 255]);
    near(pixel(&image, 40, 8), [255, 255, 255, 255]);
    Ok(())
}

#[test]
fn filter_opacity_applies_to_the_composited_group() -> Result<(), String> {
    let image = frame(
        ".outer{background:red;filter:opacity(.5)}.inner{background:blue;width:16px;height:16px}",
        "<div class=outer><div class=inner></div></div>",
        1.0,
    )?;
    near(pixel(&image, 8, 8), [127, 127, 255, 255]);
    near(pixel(&image, 24, 24), [255, 127, 127, 255]);
    Ok(())
}

#[test]
fn nested_filters_finish_inside_out_before_element_opacity() -> Result<(), String> {
    let image = frame(
        ".outer{background:red;filter:invert(1);opacity:.5}.inner{background:red;filter:invert(1);width:16px;height:16px}",
        "<div class=outer><div class=inner></div></div>",
        1.0,
    )?;
    assert_eq!(pixel(&image, 8, 8), [255, 126, 126, 255]);
    assert_eq!(pixel(&image, 24, 24), [126, 255, 255, 255]);
    Ok(())
}

#[test]
fn filtered_overflow_obeys_ancestor_clips_and_device_scale() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let image = frame(
            ".clip{overflow:hidden;width:24px;height:24px;margin:8px}.paint{background:red;filter:invert(1);transform:translate(8px,8px)}",
            "<div class=clip><div class=paint></div></div>",
            scale,
        )?;
        near(
            pixel(&image, (20.0 * scale) as u32, (20.0 * scale) as u32),
            [0, 255, 255, 255],
        );
        near(
            pixel(&image, (36.0 * scale) as u32, (20.0 * scale) as u32),
            [255, 255, 255, 255],
        );
        near(
            pixel(&image, (12.0 * scale) as u32, (12.0 * scale) as u32),
            [255, 255, 255, 255],
        );
    }
    Ok(())
}

#[test]
fn blur_radius_scales_with_device_pixels_and_css_transforms() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        let image = frame(
            "div{position:absolute;left:20px;top:16px;background:black;filter:blur(4px)}",
            "<div></div>",
            scale,
        )?;
        let edge = pixel(&image, (56.0 * scale) as u32, (32.0 * scale) as u32)[0];
        assert!((211..=230).contains(&edge), "scale {scale}: {edge}");
        let transformed = frame(
            "div{position:absolute;left:20px;top:16px;background:black;filter:blur(2px);transform:scaleX(2);transform-origin:0 0}",
            "<div></div>",
            scale,
        )?;
        let edge = pixel(&transformed, (88.0 * scale) as u32, (32.0 * scale) as u32)[0];
        assert!(
            (211..=230).contains(&edge),
            "transformed scale {scale}: {edge}"
        );
    }
    Ok(())
}

#[test]
fn offscreen_source_pixels_still_contribute_to_visible_blur() -> Result<(), String> {
    let image = frame(
        "div{position:absolute;left:-36px;top:16px;background:black;filter:blur(8px)}",
        "<section><div></div></section>",
        1.0,
    )?;
    assert!(pixel(&image, 0, 32)[0] < 220, "{:?}", pixel(&image, 0, 32));
    assert!(pixel(&image, 28, 32)[0] > 250);
    Ok(())
}
