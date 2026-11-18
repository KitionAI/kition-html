use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:40px;background:white}}div{{width:140px;height:70px;outline-offset:8px}}{css}</style><main id=stage><div id=box></div></main>"
        ),
        Resources::new(),
        &RenderOptions {
            width: 260,
            height: 180,
            scale,
            ..Default::default()
        },
    )
}
fn pixel(image: &Rendered, x: usize, y: usize) -> &[u8] {
    let start = (y * image.width as usize + x) * 4;
    &image.rgba[start..start + 4]
}

#[test]
fn dashed_and_dotted_outlines_have_gaps_and_preserve_the_offset() -> Result<(), String> {
    for scale in [1usize, 2] {
        for style in ["dashed", "dotted"] {
            let image = frame(&format!("div{{outline:2px {style} #5645d4}}"), scale as f64)?;
            let row: Vec<_> = (32 * scale..188 * scale)
                .map(|x| pixel(&image, x, 30 * scale))
                .collect();
            assert!(row.iter().filter(|p| **p == [86, 69, 212, 255]).count() > 25 * scale);
            assert!(row.iter().filter(|p| **p == [255; 4]).count() > 25 * scale);
            assert_eq!(pixel(&image, 100 * scale, 35 * scale), [255; 4]);
        }
    }
    Ok(())
}

#[test]
fn double_outline_snaps_its_stripes_and_thin_double_is_solid() -> Result<(), String> {
    for scale in [1usize, 2] {
        let image = frame("div{outline:5px double #5645d4}", scale as f64)?;
        for (y, ink) in [
            (27, true),
            (28, true),
            (29, false),
            (30, true),
            (31, true),
            (32, false),
        ] {
            assert_eq!(
                pixel(&image, 100 * scale, y * scale),
                if ink { &[86, 69, 212, 255] } else { &[255; 4] }
            );
        }
        let thin = frame("div{outline:2px double #5645d4}", scale as f64)?;
        let solid = frame("div{outline:2px solid #5645d4}", scale as f64)?;
        assert_eq!(thin.rgba, solid.rgba);
    }
    Ok(())
}

#[test]
fn inset_and_outset_reverse_the_same_edge_shades() -> Result<(), String> {
    let inset = frame("div{outline:5px inset #5645d4}", 1.0)?;
    let outset = frame("div{outline:5px outset #5645d4}", 1.0)?;
    assert_eq!(pixel(&inset, 100, 28), pixel(&outset, 100, 120));
    assert_eq!(pixel(&inset, 100, 120), pixel(&outset, 100, 28));
    assert!(pixel(&inset, 100, 28)[2] < pixel(&inset, 100, 120)[2]);
    Ok(())
}

#[test]
fn odd_groove_halves_follow_top_left_to_bottom_right_pixel_snapping() -> Result<(), String> {
    for scale in [1usize, 2] {
        let image = frame("div{outline:5px groove #5645d4}", scale as f64)?;
        for offset in 0..5 {
            let top = pixel(&image, 100 * scale, (27 + offset) * scale);
            let bottom = pixel(&image, 100 * scale, (118 + offset) * scale);
            assert_eq!(top, bottom, "offset {offset}, DPR {scale}");
            assert_eq!(top[2] < 200, offset < 3);
        }
    }
    Ok(())
}

#[test]
fn outline_style_does_not_replace_the_border_or_change_layout() -> Result<(), String> {
    let base = frame("div{border:4px solid red}", 1.0)?;
    for style in ["dotted", "dashed", "double", "inset", "groove"] {
        let image = frame(
            &format!("div{{border:4px solid red;outline:2px {style} blue}}"),
            1.0,
        )?;
        assert_eq!(base.geometry, image.geometry);
        assert_eq!(pixel(&image, 100, 40), [255, 0, 0, 255]);
        assert_eq!(pixel(&image, 100, 35), [255; 4]);
    }
    Ok(())
}

#[test]
fn translucent_dot_corners_are_not_composited_twice() -> Result<(), String> {
    let image = frame("div{outline:2px dotted rgb(86 69 212 / .5)}", 1.0)?;
    assert_eq!(pixel(&image, 30, 30), [170, 162, 233, 255]);
    assert_eq!(pixel(&image, 30, 30), pixel(&image, 34, 30));
    Ok(())
}

#[test]
fn odd_width_curved_dashes_keep_the_opposite_edge_phase() -> Result<(), String> {
    // Integral half-width contour insets prevent an extra dash around the
    // ellipse from shifting the opposite edge by half a pattern period.
    for scale in [1usize, 2] {
        let image = frame(
            "div{outline:5px dashed #5645d4;border-radius:45% / 30%;background:#ede9fa}",
            scale as f64,
        )?;
        for (x, ink) in [(105, true), (112, false), (122, true), (128, false)] {
            assert_eq!(
                pixel(&image, x * scale, 119 * scale),
                if ink { &[86, 69, 212, 255] } else { &[255; 4] }
            );
        }
    }
    Ok(())
}

#[test]
fn inset_outline_is_visible_over_its_own_background() -> Result<(), String> {
    for scale in [1usize, 2] {
        for style in ["solid", "double"] {
            let image = frame(
                &format!(
                    "div{{background:#e5486a;outline:6px {style} #5645d4;outline-offset:-12px}}"
                ),
                scale as f64,
            )?;
            assert_eq!(pixel(&image, 100 * scale, 46 * scale), [86, 69, 212, 255]);
            assert_eq!(pixel(&image, 100 * scale, 60 * scale), [229, 72, 106, 255]);
        }
    }
    Ok(())
}

#[test]
fn positioned_children_remain_above_an_inset_outline() -> Result<(), String> {
    for position in ["relative", "absolute"] {
        let image = render(
            &format!(
                "<!doctype html><style>body{{margin:40px}}section{{position:relative;width:140px;height:70px;background:#e5486a;outline:6px solid #5645d4;outline-offset:-12px}}b{{display:block;position:{position};inset:0;width:140px;height:70px;background:#1f9d4c}}</style><section><b></b></section>"
            ),
            Resources::new(),
            &RenderOptions {
                width: 260,
                height: 180,
                ..Default::default()
            },
        )?;
        assert_eq!(pixel(&image, 100, 46), [31, 157, 76, 255]);
    }
    Ok(())
}
