use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, body: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:0;background:white}}section{{position:relative}}i,b{{position:absolute;width:40px;height:30px;background:#5645d4}}b{{left:20px;top:10px;background:#e5486a}}{css}</style>{body}"
        ),
        Resources::new(),
        &RenderOptions {
            width: 80,
            height: 60,
            scale,
            ..Default::default()
        },
    )
}
fn pixel(image: &Rendered, x: usize, y: usize) -> &[u8] {
    let i = (y * image.width as usize + x) * 4;
    &image.rgba[i..i + 4]
}

#[test]
fn single_paint_quantizes_alpha_before_rgb_premultiplication() -> Result<(), String> {
    // These sRGB samples are from independent Chrome captures, including
    // alpha values whose RGB and alpha rounding disagree in float pipelines.
    for (opacity, expected) in [
        (0.1, [238, 236, 251, 255]),
        (0.75, [128, 116, 223, 255]),
        (0.9, [103, 87, 216, 255]),
    ] {
        for scale in [1.0, 1.5, 2.0] {
            let image = frame(
                &format!("section{{opacity:{opacity}}}"),
                "<section><i></i></section>",
                scale,
            )?;
            assert_eq!(
                pixel(&image, 10, 10),
                expected,
                "opacity={opacity}, scale={scale}"
            );
        }
    }
    Ok(())
}

#[test]
fn isolated_groups_preserve_one_rounding_step_over_the_backdrop() -> Result<(), String> {
    for (opacity, expected) in [
        (0.1, [237, 235, 250, 255]),
        (0.5, [169, 161, 233, 255]),
        (0.9, [102, 87, 216, 255]),
    ] {
        let image = frame(
            &format!("section{{opacity:{opacity}}}"),
            "<section><i></i><b></b></section>",
            1.0,
        )?;
        assert_eq!(pixel(&image, 10, 10), expected);
        assert_eq!(pixel(&image, 70, 50), [255; 4]);
    }
    Ok(())
}

#[test]
fn nested_single_draws_retain_the_inner_isolation_boundary() -> Result<(), String> {
    for (opacity, expected) in [
        (0.1, [248, 247, 254, 255]),
        (0.5, [221, 218, 247, 255]),
        (0.9, [194, 188, 240, 255]),
    ] {
        let image = frame(
            &format!("section{{opacity:{opacity}}}article{{opacity:.4;position:relative}}"),
            "<section><article><i></i></article></section>",
            1.0,
        )?;
        assert_eq!(pixel(&image, 10, 10), expected);
    }
    Ok(())
}

#[test]
fn transparent_children_and_colored_backdrops_keep_group_order() -> Result<(), String> {
    let translucent = frame(
        "section{opacity:.5}i{background:#5645d480}b{background:#e5486a80}",
        "<section><i></i><b></b></section>",
        1.0,
    )?;
    assert_eq!(pixel(&translucent, 10, 10), [212, 208, 244, 255]);
    // The existing inner source-over path differs by one channel level;
    // strict conformance metrics retain that error.
    assert!(
        pixel(&translucent, 25, 15)
            .iter()
            .zip([226, 185, 211, 255])
            .all(|(actual, reference)| actual.abs_diff(reference) <= 1)
    );
    let colored = frame(
        "body{background:#1f9d4c}section{opacity:.5}",
        "<section><i></i><b></b></section>",
        1.0,
    )?;
    assert_eq!(pixel(&colored, 10, 10), [58, 112, 144, 255]);
    assert_eq!(pixel(&colored, 25, 15), [130, 114, 91, 255]);
    Ok(())
}

#[test]
fn quantized_zero_opacity_never_darkens_the_backdrop() -> Result<(), String> {
    let image = frame(
        "section{opacity:.001}",
        "<section><i></i><b></b></section>",
        1.0,
    )?;
    assert!(image.rgba.iter().all(|channel| *channel == 255));
    Ok(())
}
