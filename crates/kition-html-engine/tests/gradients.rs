use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:0}}div{{width:256px;height:32px}}{css}</style><div></div>"
        ),
        Resources::new(),
        &RenderOptions {
            width: 256,
            height: 96,
            ..Default::default()
        },
    )
}
fn pixel(image: &Rendered, x: u32, y: u32) -> &[u8] {
    let i = ((y * image.width + x) * 4) as usize;
    &image.rgba[i..i + 4]
}

#[test]
fn gradient_quantization_preserves_the_mean_without_color_banding() -> Result<(), String> {
    let image = frame("div{background:linear-gradient(to right,black,white)}")?;
    for x in 8..240 {
        let expected = 255.0 * (x as f64 + 0.5) / 256.0;
        for y in 0..24 {
            let actual = pixel(&image, x, y)[0];
            assert!((f64::from(actual) - expected).abs() < 1.0);
            assert_eq!(pixel(&image, x, y), pixel(&image, x, y + 8));
        }
    }
    // The Bayer pattern distributes error over a whole 8x8 tile, rather than
    // requiring each individual row or column to have a zero mean error.
    for start in (8..240).step_by(8) {
        let mut actual = 0.0;
        let mut expected = 0.0;
        for y in 0..8 {
            for x in start..start + 8 {
                actual += f64::from(pixel(&image, x, y)[0]);
                expected += 255.0 * (x as f64 + 0.5) / 256.0;
            }
        }
        assert!((actual - expected).abs() / 64.0 < 0.15);
    }
    Ok(())
}

#[test]
fn transparent_gradient_stops_use_premultiplied_color() -> Result<(), String> {
    let image = frame("div{width:32px;background:linear-gradient(to right,transparent,blue)}")?;
    for x in 1..31 {
        let actual = pixel(&image, x, 8);
        let expected = (255.0 * (1.0 - (x as f64 + 0.5) / 32.0)).round() as u8;
        assert!(actual[0].abs_diff(expected) <= 1 && actual[1].abs_diff(expected) <= 1);
        assert!(actual[2] >= 254);
        assert_eq!(actual[3], 255);
    }
    Ok(())
}

#[test]
fn gradient_sampling_keeps_css_transform_and_clipping() -> Result<(), String> {
    let image = frame(
        "div{width:64px;height:32px;transform-origin:0 0;transform:translate(48px,8px) rotate(90deg);background:linear-gradient(to right,red,blue)}",
    )?;
    let start = pixel(&image, 32, 12);
    let end = pixel(&image, 32, 68);
    assert!(start[0] > 230 && start[2] < 25, "{start:?}");
    assert!(end[2] > 230 && end[0] < 25, "{end:?}");
    assert_eq!(pixel(&image, 12, 12), [255, 255, 255, 255]);
    assert_eq!(pixel(&image, 52, 12), [255, 255, 255, 255]);
    Ok(())
}

#[test]
fn repeated_hard_stops_remain_exact_solid_colors() -> Result<(), String> {
    let image = frame(
        "div{background:repeating-linear-gradient(to right,red 0px,red 8px,blue 8px,blue 16px)}",
    )?;
    for x in 0..256 {
        assert_eq!(
            pixel(&image, x, 8),
            if x % 16 < 8 {
                [255, 0, 0, 255]
            } else {
                [0, 0, 255, 255]
            }
        );
    }
    Ok(())
}
