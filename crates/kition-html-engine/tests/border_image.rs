use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:20px}}#box{{width:80px;height:40px;border:10px solid black;background:white;border-image:linear-gradient(to right,red 25%,green 25% 75%,blue 75%) 25%;{css}}}</style><main id=stage><div id=box></div></main>"
        ),
        Resources::new(),
        &RenderOptions {
            width: 160,
            height: 120,
            scale,
            ..Default::default()
        },
    )
}

fn pixel(image: &Rendered, x: usize, y: usize, scale: f64) -> &[u8] {
    let x = (x as f64 * scale) as usize;
    let y = (y as f64 * scale) as usize;
    let i = (y * image.width as usize + x) * 4;
    &image.rgba[i..i + 4]
}

#[test]
fn nine_slices_replace_the_border_and_preserve_the_empty_center() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let image = frame("border-radius:20px", scale)?;
        for y in [25, 75] {
            assert_eq!(pixel(&image, 25, y, scale), [255, 0, 0, 255]);
            assert_eq!(pixel(&image, 60, y, scale), [0, 128, 0, 255]);
            assert_eq!(pixel(&image, 115, y, scale), [0, 0, 255, 255]);
        }
        assert_eq!(pixel(&image, 60, 50, scale), [255, 255, 255, 255]);
        assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    }
    Ok(())
}

#[test]
fn fill_and_outset_paint_without_changing_layout_size() -> Result<(), String> {
    let image = frame("border-image-slice:25% fill;border-image-outset:2", 1.0)?;
    assert_eq!(pixel(&image, 2, 2, 1.0), [255, 0, 0, 255]);
    assert_eq!(pixel(&image, 60, 50, 1.0), [0, 128, 0, 255]);
    let box_geometry = image
        .geometry
        .iter()
        .find(|n| n["id"] == "box")
        .ok_or("missing box")?;
    assert_eq!(box_geometry["width"], 100.0);
    assert_eq!(box_geometry["height"], 60.0);
    Ok(())
}

#[test]
fn edge_repeat_round_and_space_use_distinct_tile_placement() -> Result<(), String> {
    let red = [255, 0, 0, 255];
    let blue = [0, 0, 255, 255];
    let white = [255, 255, 255, 255];
    for (repeat, expected) in [
        ("stretch", [red, red, blue, blue]),
        ("repeat", [blue, red, blue, red]),
        ("round", [red, blue, red, blue]),
        ("space", [white, red, blue, white]),
    ] {
        let image = frame(
            &format!("border-image:linear-gradient(to right,red 50%,blue 50%) 15 / 1 / 0 {repeat}"),
            1.0,
        )?;
        for (x, expected) in [35, 55, 80, 105].into_iter().zip(expected) {
            assert_eq!(pixel(&image, x, 25, 1.0), expected, "{repeat} at {x}");
        }
    }
    Ok(())
}

#[test]
fn percentage_border_widths_and_overlapping_slices_do_not_paint_the_center() -> Result<(), String> {
    let image = frame("border-image-width:10%", 1.0)?;
    assert_eq!(pixel(&image, 60, 24, 1.0), [0, 128, 0, 255]);
    assert_eq!(pixel(&image, 60, 27, 1.0), [255, 255, 255, 255]);
    let overlap = frame("border-image-slice:60% fill", 1.0)?;
    assert_eq!(pixel(&overlap, 60, 25, 1.0), [255, 255, 255, 255]);
    assert_eq!(pixel(&overlap, 60, 50, 1.0), [255, 255, 255, 255]);
    Ok(())
}

fn source_image(svg: bool) -> Result<Vec<u8>, String> {
    if svg {
        return Ok(br#"<svg xmlns="http://www.w3.org/2000/svg" width="30" height="30" viewBox="0 0 30 30"><path fill="red" d="M0 0h10v30H0z"/><path fill="green" d="M10 0h10v30H10z"/><path fill="blue" d="M20 0h10v30H20z"/></svg>"#.to_vec());
    }
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, 30, 30);
        encoder.set_color(png::ColorType::Rgba);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        let mut pixels = Vec::new();
        for _ in 0..30 {
            for x in 0..30 {
                pixels.extend_from_slice(match x {
                    0..10 => &[255, 0, 0, 255],
                    10..20 => &[0, 128, 0, 255],
                    _ => &[0, 0, 255, 255],
                });
            }
        }
        writer
            .write_image_data(&pixels)
            .map_err(|e| e.to_string())?;
    }
    Ok(bytes)
}

#[test]
fn raster_and_svg_sources_resolve_slices_auto_widths_and_shared_background_resources()
-> Result<(), String> {
    for svg in [false, true] {
        for scale in [1.0, 1.5, 2.0] {
            let path = if svg {
                "/assets/border.svg"
            } else {
                "/assets/border.png"
            };
            let resources = Resources::from([(path.into(), source_image(svg)?)]);
            let image = render(
                &format!(
                    "<!doctype html><style>body{{margin:20px}}div{{width:80px;height:40px;border:4px solid black;background:white;border-image:url({path}) 10 fill / auto;image-rendering:pixelated}}aside{{width:30px;height:30px;background:url({path})}}</style><div></div><aside></aside>"
                ),
                resources,
                &RenderOptions {
                    width: 160,
                    height: 120,
                    scale,
                    ..Default::default()
                },
            )?;
            for y in [25, 44, 63] {
                for (x, expected) in [
                    (25, [255, 0, 0, 255]),
                    (60, [0, 128, 0, 255]),
                    (103, [0, 0, 255, 255]),
                ] {
                    assert_eq!(
                        pixel(&image, x, y, scale),
                        expected,
                        "svg={svg}, scale={scale}, ({x},{y})"
                    );
                }
            }
            assert_eq!(pixel(&image, 25, 75, scale), [255, 0, 0, 255]);
            assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
        }
    }
    Ok(())
}

#[test]
fn unavailable_border_image_retains_the_ordinary_border() -> Result<(), String> {
    let image = frame("border-image-source:url(/missing.png)", 1.0)?;
    assert_eq!(pixel(&image, 25, 25, 1.0), [0, 0, 0, 255]);
    assert_eq!(pixel(&image, 60, 50, 1.0), [255, 255, 255, 255]);
    assert!(
        image
            .diagnostics
            .iter()
            .any(|message| message.contains("resource-unavailable"))
    );
    Ok(())
}
