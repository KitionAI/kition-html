use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(style: &str, body: &str, size: u32, scale: f64) -> Result<Rendered, String> {
    let mut resources = Resources::new();
    resources.insert(
        "/assets/Inter.ttf".into(),
        include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
    );
    render(
        &format!(
            "<!doctype html><style>@font-face{{font-family:Inter;src:url(/assets/Inter.ttf)}}body{{margin:0;background:white;font:{size}px/1.5 Inter}}li{{margin:20px 40px;color:#5645d4;{style}}}span{{color:red}}</style><main id=stage>{body}</main>"
        ),
        resources,
        &RenderOptions {
            width: 300,
            height: 160,
            scale,
            ..Default::default()
        },
    )
}
fn red_mask(image: &Rendered) -> Vec<bool> {
    image
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| p[0] > p[1] && p[1] == p[2])
        .collect()
}
fn purple_bounds(image: &Rendered) -> Option<(u32, u32, u32, u32)> {
    let mut bounds = (image.width, image.height, 0, 0);
    let mut found = false;
    for (i, p) in image.rgba.as_chunks::<4>().0.iter().enumerate() {
        if p[2] > p[0] && p[0] > p[1] {
            let x = i as u32 % image.width;
            let y = i as u32 / image.width;
            bounds = (
                bounds.0.min(x),
                bounds.1.min(y),
                bounds.2.max(x),
                bounds.3.max(y),
            );
            found = true;
        }
    }
    found.then_some(bounds)
}

#[test]
fn outside_symbols_preserve_the_principal_text_font_and_line_box() -> Result<(), String> {
    for size in [12, 16, 24] {
        for scale in [1.0, 1.5, 2.0] {
            let reference = frame(
                "list-style:none",
                "<li><span>Alpha</span></li>",
                size,
                scale,
            )?;
            for symbol in [
                "disc",
                "circle",
                "square",
                "disclosure-open",
                "disclosure-closed",
            ] {
                let image = frame(
                    &format!("list-style:{symbol} outside"),
                    "<li><span>Alpha</span></li>",
                    size,
                    scale,
                )?;
                assert!(
                    red_mask(&image) == red_mask(&reference),
                    "{symbol}, size {size}, scale {scale}"
                );
                assert!(purple_bounds(&image).is_some());
                assert_eq!(image.geometry[0]["height"], reference.geometry[0]["height"]);
            }
        }
    }
    Ok(())
}

#[test]
fn inside_disclosure_markers_do_not_enlarge_the_line_height() -> Result<(), String> {
    for size in [12, 16, 24] {
        for scale in [1.0, 2.0] {
            let reference = frame(
                "list-style:none",
                "<li><span>Alpha</span></li>",
                size,
                scale,
            )?;
            for symbol in ["disclosure-open", "disclosure-closed"] {
                let image = frame(
                    &format!("list-style:{symbol} inside"),
                    "<li><span>Alpha</span></li>",
                    size,
                    scale,
                )?;
                assert_eq!(image.geometry[0]["height"], reference.geometry[0]["height"]);
                let bounds = purple_bounds(&image).ok_or("missing marker")?;
                assert_eq!(bounds.0, (40.0 * scale) as u32);
                assert!(bounds.2 - bounds.0 + 1 > (size as f64 * scale * 0.5) as u32);
            }
        }
    }
    Ok(())
}

#[test]
fn closed_disclosure_points_toward_the_inline_end() -> Result<(), String> {
    let left = frame(
        "list-style:disclosure-closed inside",
        "<li><span></span></li>",
        24,
        2.0,
    )?;
    let right = frame(
        "list-style:disclosure-closed inside;direction:rtl",
        "<li><span></span></li>",
        24,
        2.0,
    )?;
    let a = purple_bounds(&left).ok_or("missing LTR marker")?;
    let b = purple_bounds(&right).ok_or("missing RTL marker")?;
    let row_width = |image: &Rendered, bounds: (u32, u32, u32, u32), y: u32| {
        (bounds.0..=bounds.2)
            .filter(|&x| {
                let i = ((y * image.width + x) * 4) as usize;
                image.rgba[i + 2] > image.rgba[i]
            })
            .count()
    };
    assert!(row_width(&left, a, a.1 + 2) < row_width(&left, a, (a.1 + a.3) / 2));
    assert!(row_width(&right, b, b.1 + 2) < row_width(&right, b, (b.1 + b.3) / 2));
    let centroid = |image: &Rendered, bounds: (u32, u32, u32, u32)| {
        let (mut total, mut sum) = (0.0, 0.0);
        for y in bounds.1..=bounds.3 {
            for x in bounds.0..=bounds.2 {
                let i = ((y * image.width + x) * 4) as usize;
                let weight = f64::from(255 - image.rgba[i + 1]);
                total += weight;
                sum += weight * f64::from(x - bounds.0);
            }
        }
        sum / total / f64::from(bounds.2 - bounds.0)
    };
    assert!(centroid(&left, a) < 0.45 && centroid(&right, b) > 0.55);
    Ok(())
}

#[test]
fn marker_none_omits_only_the_symbol() -> Result<(), String> {
    let image = frame(
        "list-style:none",
        "<li><span>Visible text</span></li>",
        16,
        1.0,
    )?;
    assert!(purple_bounds(&image).is_none());
    assert!(red_mask(&image).iter().any(|x| *x));
    Ok(())
}
