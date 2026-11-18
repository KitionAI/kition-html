use kition_html_engine::{RenderOptions, Resources, render};

#[test]
fn line_baselines_and_normal_heights_are_invariant_in_css_pixels() -> Result<(), String> {
    let mut resources = Resources::new();
    for (name, bytes) in [
        (
            "Inter",
            include_bytes!("../../../conformance/assets/Inter-Variable.ttf").as_slice(),
        ),
        (
            "Lora",
            include_bytes!("../../../conformance/assets/Lora-Variable.ttf").as_slice(),
        ),
        (
            "JetBrains Mono",
            include_bytes!("../../../conformance/assets/JetBrainsMono-Variable.ttf").as_slice(),
        ),
    ] {
        resources.insert(format!("/assets/{name}.ttf"), bytes.to_vec());
    }
    for family in ["Inter", "Lora", "JetBrains Mono"] {
        for size in [12, 16, 24, 32, 48] {
            for line_height in ["72px", "normal"] {
                let html = format!(
                    "<!doctype html><style>body{{margin:0;font:{size}px/{line_height} '{family}'}}i{{display:inline-block;width:1px;height:0}}</style><main id=stage><div id=line>Typography<i id=baseline></i></div></main>"
                );
                let mut expected = None;
                for scale in [1.0, 1.5, 2.0, 3.0] {
                    let image = render(
                        &html,
                        resources.clone(),
                        &RenderOptions {
                            width: 400,
                            height: 100,
                            scale,
                            ..Default::default()
                        },
                    )?;
                    let find = |id, field| {
                        image
                            .geometry
                            .iter()
                            .find(|n| n["id"] == id)
                            .and_then(|n| n[field].as_f64())
                            .unwrap_or(f64::NAN)
                    };
                    let metrics = [find("baseline", "y"), find("line", "height")];
                    if let Some(expected) = expected {
                        assert_eq!(
                            metrics, expected,
                            "{family} {size}px/{line_height}, DPR {scale}"
                        );
                    } else {
                        expected = Some(metrics);
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn fractional_device_baselines_keep_the_same_glyph_mask() -> Result<(), String> {
    let resources = Resources::from([(
        "/assets/JetBrainsMono-Variable.ttf".into(),
        include_bytes!("../../../conformance/assets/JetBrainsMono-Variable.ttf").to_vec(),
    )]);
    let mut frames = Vec::new();
    for top in [0, 1, 2] {
        frames.push(render(
            &format!("<!doctype html><style>body{{margin:0}}div{{position:absolute;left:8px;top:{top}px;font:16px/72px 'JetBrains Mono'}}</style><div>Hamburg</div>"),
            resources.clone(),
            &RenderOptions { width:160,height:100,scale:1.5,..Default::default() },
        )?);
    }
    let stride = frames[0].width as usize * 4;
    for (frame, shift) in frames.iter().skip(1).zip([1, 3]) {
        let original = &frames[0].rgba[..(frames[0].height as usize - shift) * stride];
        let shifted = &frame.rgba[shift * stride..];
        assert!(
            original
                .iter()
                .zip(shifted)
                .all(|(a, b)| a.abs_diff(*b) <= 1),
            "glyph mask changed after device baseline translation {shift}"
        );
    }
    Ok(())
}

#[test]
fn horizontal_glyph_origins_share_quarter_device_pixel_phases() -> Result<(), String> {
    let fonts = [
        (
            "Inter",
            include_bytes!("../../../conformance/assets/Inter-Variable.ttf").as_slice(),
        ),
        (
            "Lora",
            include_bytes!("../../../conformance/assets/Lora-Variable.ttf").as_slice(),
        ),
        (
            "JetBrains Mono",
            include_bytes!("../../../conformance/assets/JetBrainsMono-Variable.ttf").as_slice(),
        ),
    ];
    for (family, bytes) in fonts {
        let resources = Resources::from([(format!("/assets/{family}.ttf"), bytes.to_vec())]);
        for size in [12, 16, 24] {
            for scale in [1.0, 1.5, 2.0] {
                let mut phases = Vec::new();
                for device_offset in [0.0, 0.0625, 0.125, 0.1875, 0.25, 0.3125] {
                    let left = 16.0 + device_offset / scale;
                    phases.push(render(
                        &format!("<!doctype html><style>body{{margin:0}}div{{position:absolute;left:{left}px;top:16px;font:{size}px/48px '{family}';color:black}}</style><div>H</div>"),
                        resources.clone(),
                        &RenderOptions {width:100,height:100,scale,..Default::default()},
                    )?);
                }
                for (first, second) in [(0, 1), (2, 3), (2, 4), (2, 5)] {
                    assert!(
                        phases[first]
                            .rgba
                            .iter()
                            .zip(&phases[second].rgba)
                            .all(|(a, b)| a.abs_diff(*b) <= 1),
                        "{family} {size}px at DPR {scale}: masks in the same phase differ ({first}, {second})"
                    );
                }
                assert!(
                    phases[0].rgba != phases[2].rgba,
                    "{family} {size}px at DPR {scale}: distinct subpixel phases were lost"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn nested_fractional_text_origins_match_flattened_placement() -> Result<(), String> {
    // Precise offsets must accumulate through containing blocks, including
    // padding and scaled ancestors, without changing layout metrics.
    for scale in [1.0, 1.5, 2.0] {
        for transform in ["none", "scale(1.25)", "rotate(90deg)"] {
            let prefix = format!(
                "<!doctype html><style>body{{margin:0}}main{{position:absolute;left:32px;top:24px;transform:{transform};transform-origin:0 0}}div{{position:absolute;font:16px/48px Inter;white-space:pre}}</style>"
            );
            let nested = format!(
                "{prefix}<main><section style='position:absolute;left:.25px;top:.25px;padding:.125px'><div style='left:.25px;top:.25px;padding:.125px'>Hamburg</div></section></main>"
            );
            let flat =
                format!("{prefix}<main><div style='left:.625px;top:.625px'>Hamburg</div></main>");
            let options = RenderOptions {
                width: 240,
                height: 200,
                scale,
                ..Default::default()
            };
            let nested = render(&nested, Resources::new(), &options)?;
            let flat = render(&flat, Resources::new(), &options)?;
            assert!(
                nested
                    .rgba
                    .iter()
                    .zip(&flat.rgba)
                    .all(|(a, b)| a.abs_diff(*b) <= 1),
                "nested text origin differs at DPR {scale}, transform {transform}"
            );
        }
    }
    Ok(())
}
