use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, content: &str, scale: f64) -> Result<Rendered, String> {
    let mut resources = Resources::new();
    resources.insert(
        "/assets/Inter.ttf".into(),
        include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
    );
    render(
        &format!(
            "<!doctype html><style>body{{margin:0;font:24px/36px Inter}}p{{margin:0}}{css}</style>{content}"
        ),
        resources,
        &RenderOptions {
            width: 160,
            height: 180,
            scale,
            ..Default::default()
        },
    )
}

#[test]
fn vertical_rl_rotates_latin_lines_and_inline_backgrounds() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let content = "<p>ABC<br><span>DEF</span></p>";
        let common = "p{padding:8px;border:2px solid purple}span{background:#eee}";
        let vertical = frame(
            &format!("{common}p{{writing-mode:vertical-rl;width:72px;height:120px}}"),
            content,
            scale,
        )?;
        let rotated = frame(
            &format!(
                "{common}p{{width:120px;height:72px;transform-origin:0 0;transform:translateX(92px) rotate(90deg)}}"
            ),
            content,
            scale,
        )?;
        assert!(
            vertical.rgba == rotated.rgba,
            "DPR {scale}: vertical lines should match the corresponding clockwise horizontal flow"
        );
    }
    Ok(())
}

fn color_center(image: &Rendered, red: bool) -> (f64, f64) {
    let mut x_sum = 0.0;
    let mut y_sum = 0.0;
    let mut count = 0.0;
    for y in 0..image.height {
        for x in 0..image.width {
            let i = ((y * image.width + x) * 4) as usize;
            let p = &image.rgba[i..i + 4];
            let ink = if red {
                p[0] > 180 && p[1] < 80 && p[2] < 80
            } else {
                p[2] > 180 && p[0] < 80 && p[1] < 80
            };
            if ink {
                x_sum += x as f64;
                y_sum += y as f64;
                count += 1.0;
            }
        }
    }
    assert!(count > 20.0);
    (x_sum / count, y_sum / count)
}

#[test]
fn vertical_columns_follow_the_requested_block_direction() -> Result<(), String> {
    for mode in ["vertical-rl", "vertical-lr", "sideways-rl", "sideways-lr"] {
        let image = frame(
            &format!(
                "p{{writing-mode:{mode};width:72px;height:120px}}.first{{color:red}}.second{{color:blue}}"
            ),
            "<p><span class=first>ABC</span><br><span class=second>ABC</span></p>",
            1.0,
        )?;
        let a = color_center(&image, true);
        let b = color_center(&image, false);
        let expected = if mode.ends_with("-rl") { -36.0 } else { 36.0 };
        assert!((b.0 - a.0 - expected).abs() < 0.1, "{mode}: {a:?}, {b:?}");
        assert!((a.1 - b.1).abs() < 0.1);
        assert!(image.diagnostics.is_empty());
    }
    Ok(())
}

#[test]
fn upright_vertical_shaping_reports_its_current_limit() -> Result<(), String> {
    let image = frame(
        "p{writing-mode:vertical-rl;text-orientation:upright;height:120px}",
        "<p>ABC</p>",
        1.0,
    )?;
    assert!(
        image
            .diagnostics
            .iter()
            .any(|d| d.starts_with("html.unsupported-upright-text:"))
    );
    Ok(())
}
