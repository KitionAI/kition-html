use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:20px;background:white}}div{{width:60px;height:40px}}{css}</style><div></div>"
        ),
        Resources::new(),
        &RenderOptions {
            width: 340,
            height: 230,
            scale,
            ..Default::default()
        },
    )
}
fn pixel(frame: &Rendered, x: usize, y: usize) -> &[u8] {
    let start = (y * frame.width as usize + x) * 4;
    &frame.rgba[start..start + 4]
}
#[test]
fn thin_dots_keep_integral_full_endpoints_at_both_device_scales() -> Result<(), String> {
    // Chrome's 60px, 2px dotted side preserves full endpoint dots and shortens
    // the first and last gaps by one CSS pixel.
    let starts = [0, 3, 7, 11, 15, 19, 23, 27, 31, 35, 39, 43, 47, 51, 55, 58];
    for scale in [1usize, 2] {
        let image = frame("div{border-top:2px dotted #5645d4}", scale as f64)?;
        for x in 0..60 {
            let ink = starts.iter().any(|start| x >= *start && x < start + 2);
            assert_eq!(
                pixel(&image, (20 + x) * scale, 20 * scale),
                if ink { &[86, 69, 212, 255] } else { &[255; 4] },
                "x={x}, scale={scale}"
            );
        }
    }
    Ok(())
}
#[test]
fn short_dashed_sides_become_solid_without_partial_endpoint_dashes() -> Result<(), String> {
    let image = frame(
        "div{width:10px;height:4px;border-top:2px dashed #5645d4}",
        1.0,
    )?;
    for x in 20..30 {
        assert_eq!(pixel(&image, x, 20), [86, 69, 212, 255]);
    }
    assert_eq!(pixel(&image, 30, 20), [255; 4]);
    Ok(())
}
#[test]
fn dash_lengths_stay_fixed_when_the_gap_is_distributed() -> Result<(), String> {
    let image = frame("div{width:252px;border-top:6px dashed #5645d4}", 1.0)?;
    for x in 20..32 {
        assert_eq!(pixel(&image, x, 22), [86, 69, 212, 255]);
    }
    for x in 32..38 {
        assert_eq!(pixel(&image, x, 22), [255; 4]);
    }
    for x in 39..50 {
        assert_eq!(pixel(&image, x, 22), [86, 69, 212, 255]);
    }
    Ok(())
}
#[test]
fn matching_translucent_dot_corners_are_filled_once() -> Result<(), String> {
    let image = frame("div{border:2px dotted rgb(86 69 212 / .5)}", 1.0)?;
    assert_eq!(pixel(&image, 20, 20), pixel(&image, 24, 20));
    assert_eq!(pixel(&image, 20, 20), [170, 162, 233, 255]);
    Ok(())
}
#[test]
fn rounded_dash_contours_start_after_the_upper_left_arc() -> Result<(), String> {
    let image = frame(
        "div{width:253px;height:137px;border:6px dashed #5645d4;border-radius:24px}",
        1.0,
    )?;
    assert_eq!(pixel(&image, 45, 23), [86, 69, 212, 255]);
    assert_eq!(pixel(&image, 55, 23), [86, 69, 212, 255]);
    assert_eq!(pixel(&image, 59, 23), [255; 4]);
    assert_eq!(pixel(&image, 20, 20), [255; 4]);
    Ok(())
}
#[test]
fn mixed_side_colors_do_not_leak_across_the_inner_border_edge() -> Result<(), String> {
    let image = frame(
        "div{width:90px;height:60px;border:6px dashed #5645d4;border-right-color:#e5486a}",
        1.0,
    )?;
    assert_eq!(pixel(&image, 23, 42), [86, 69, 212, 255]);
    assert_eq!(pixel(&image, 119, 42), [229, 72, 106, 255]);
    assert_eq!(pixel(&image, 30, 30), [255; 4]);
    Ok(())
}
