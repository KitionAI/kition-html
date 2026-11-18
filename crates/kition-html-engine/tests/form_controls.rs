use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, body: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:0;background:white}}input{{position:absolute;left:20px;top:20px;margin:0}}{css}</style><main id=stage>{body}</main>"
        ),
        Resources::new(),
        &RenderOptions {
            width: 100,
            height: 100,
            scale,
            ..Default::default()
        },
    )
}
fn pixel(image: &Rendered, x: usize, y: usize, scale: f64) -> &[u8] {
    let i = ((y as f64 * scale) as usize * image.width as usize + (x as f64 * scale) as usize) * 4;
    &image.rgba[i..i + 4]
}

#[test]
fn intrinsic_and_explicit_sizes_include_the_complete_control() -> Result<(), String> {
    for kind in ["checkbox", "radio"] {
        for size in [13, 24, 40] {
            let css = if size == 13 {
                String::new()
            } else {
                format!("input{{width:{size}px;height:{size}px}}")
            };
            let image = frame(
                &css,
                &format!("<input id=control type={kind} checked>"),
                1.0,
            )?;
            let geometry = image
                .geometry
                .iter()
                .find(|n| n["id"] == "control")
                .ok_or("missing control")?;
            assert_eq!(geometry["width"], f64::from(size));
            assert_eq!(geometry["height"], f64::from(size));
        }
    }
    Ok(())
}

#[test]
fn accent_color_inherits_and_resolves_currentcolor() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        for accent in ["#5645d4", "currentColor"] {
            let image = frame(
                &format!(
                    "body{{accent-color:{accent};color:#5645d4}}input{{width:24px;height:24px}}"
                ),
                "<input type=checkbox checked>",
                scale,
            )?;
            assert_eq!(pixel(&image, 24, 24, scale), [86, 69, 212, 255]);
            let radio = frame(
                &format!(
                    "body{{accent-color:{accent};color:#5645d4}}input{{width:24px;height:24px}}"
                ),
                "<input type=radio checked>",
                scale,
            )?;
            assert_eq!(pixel(&radio, 32, 32, scale), [86, 69, 212, 255]);
        }
    }
    Ok(())
}

#[test]
fn unchecked_and_disabled_controls_do_not_use_the_author_accent() -> Result<(), String> {
    for kind in ["checkbox", "radio"] {
        let css = "input{width:24px;height:24px;accent-color:red}";
        let plain = frame(css, &format!("<input type={kind}>"), 1.0)?;
        assert_eq!(pixel(&plain, 32, 32, 1.0), [255, 255, 255, 255]);
        let disabled = frame(css, &format!("<input type={kind} checked disabled>"), 1.0)?;
        assert!(!disabled.rgba.as_chunks::<4>().0.contains(&[255, 0, 0, 255]));
        assert!(
            disabled
                .rgba
                .as_chunks::<4>()
                .0
                .contains(&[209, 209, 209, 255])
        );
    }
    Ok(())
}

#[test]
fn appearance_none_leaves_author_paint_without_a_native_mark() -> Result<(), String> {
    for kind in ["checkbox", "radio"] {
        let image = frame(
            "input{appearance:none;width:24px;height:24px;background:red}",
            &format!("<input type={kind} checked>"),
            1.0,
        )?;
        for y in 20..44 {
            for x in 20..44 {
                assert_eq!(pixel(&image, x, y, 1.0), [255, 0, 0, 255]);
            }
        }
    }
    Ok(())
}

#[test]
fn bright_accents_use_a_contrasting_dark_checkmark() -> Result<(), String> {
    let image = frame(
        "input{width:24px;height:24px;accent-color:yellow}",
        "<input type=checkbox checked>",
        1.0,
    )?;
    assert!(
        image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| **p == [59, 59, 59, 255])
            .count()
            > 20
    );
    assert!(
        image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| **p == [255, 255, 0, 255])
            .count()
            > 300
    );
    Ok(())
}

#[test]
fn rectangular_controls_center_a_square_native_indicator() -> Result<(), String> {
    let image = frame(
        "input{width:40px;height:24px;accent-color:red}",
        "<input type=checkbox checked>",
        1.0,
    )?;
    assert_eq!(pixel(&image, 24, 30, 1.0), [255, 255, 255, 255]);
    assert_eq!(pixel(&image, 31, 24, 1.0), [255, 0, 0, 255]);
    assert_eq!(pixel(&image, 56, 30, 1.0), [255, 255, 255, 255]);
    Ok(())
}
