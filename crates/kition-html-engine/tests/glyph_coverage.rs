use kition_html_engine::{RenderOptions, Rendered, default_resources, render};

fn frame(css: &str, body: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:0;background:black}}span{{position:absolute;left:20px;top:20px;font:32px/48px Inter;color:white}}{css}</style>{body}"
        ),
        default_resources(),
        &RenderOptions {
            width: 240,
            height: 120,
            scale,
            ..Default::default()
        },
    )
}

#[test]
fn grayscale_glyph_coverage_depends_on_foreground_for_upright_and_transformed_text()
-> Result<(), String> {
    for scale in [1.0, 2.0] {
        for style in ["", "font-style:italic", "transform:rotate(12deg)"] {
            let light = frame(&format!("span{{{style}}}"), "<span>Hamburg</span>", scale)?;
            let dark = frame(
                &format!("body{{background:white}}span{{color:black;{style}}}"),
                "<span>Hamburg</span>",
                scale,
            )?;
            let mut adjusted = 0;
            for (light, dark) in light
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .zip(dark.rgba.as_chunks::<4>().0.iter())
            {
                let black_coverage = 255 - dark[0];
                assert!(light[0].abs_diff(light[1]) <= 1);
                assert!(light[0].abs_diff(light[2]) <= 1);
                assert!(light[0] >= black_coverage.saturating_sub(1));
                if (16..240).contains(&black_coverage) && light[0] > black_coverage + 8 {
                    adjusted += 1;
                }
            }
            assert!(
                adjusted > 50,
                "DPR {scale}, {style}: {adjusted} edge pixels"
            );
        }
    }
    Ok(())
}

#[test]
fn coverage_is_independent_of_background_and_composes_with_text_alpha() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let opaque = frame("", "<span>Hamburg</span>", scale)?;
        let red_background = frame("body{background:red}", "<span>Hamburg</span>", scale)?;
        let translucent = frame(
            "span{color:rgb(255 255 255 / 50%)}",
            "<span>Hamburg</span>",
            scale,
        )?;
        for ((opaque, red), translucent) in opaque
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .zip(red_background.rgba.as_chunks::<4>().0.iter())
            .zip(translucent.rgba.as_chunks::<4>().0.iter())
        {
            assert_eq!(opaque[1], red[1]);
            assert_eq!(opaque[2], red[2]);
            assert!((u16::from(translucent[0]) * 2).abs_diff(u16::from(opaque[0])) <= 2);
        }
    }
    Ok(())
}

#[test]
fn glyph_coverage_does_not_change_regular_path_antialiasing() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let shape =
            "<svg width=100 height=100><circle cx=50.25 cy=50.25 r=30.5 fill=currentColor /></svg>";
        let light = frame("svg{color:white}", shape, scale)?;
        let dark = frame("body{background:white}svg{color:black}", shape, scale)?;
        let mut edges = 0;
        for (light, dark) in light
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .zip(dark.rgba.as_chunks::<4>().0.iter())
        {
            assert!(light[0].abs_diff(255 - dark[0]) <= 1);
            edges += usize::from((1..255).contains(&light[0]));
        }
        assert!(edges > 50);
    }
    Ok(())
}

#[test]
fn glyph_color_does_not_modify_previously_recorded_coverage() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let both = frame(
            "#second{left:120px;color:#5645d4}",
            "<span>H</span><span id=second>H</span>",
            scale,
        )?;
        let first = frame("", "<span>H</span>", scale)?;
        let second = frame("span{left:120px;color:#5645d4}", "<span>H</span>", scale)?;
        let split = (100.0 * scale) as u32;
        for y in 0..both.height {
            for x in 0..both.width {
                let offset = ((y * both.width + x) * 4) as usize;
                let reference = if x < split { &first.rgba } else { &second.rgba };
                assert_eq!(
                    &both.rgba[offset..offset + 4],
                    &reference[offset..offset + 4]
                );
            }
        }
    }
    Ok(())
}
