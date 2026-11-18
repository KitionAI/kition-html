use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn table(css: &str, caption: &str) -> Result<Rendered, String> {
    let mut resources = Resources::new();
    resources.insert(
        "/assets/Inter.ttf".into(),
        include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
    );
    render(
        &format!(
            "<!doctype html><style>body{{margin:0;font:16px/24px Inter}}table{{width:200px;border-spacing:0;background:#5645d4}}td{{height:40px;padding:0}}caption{{height:20px}}#after{{height:10px;background:#1f9d4c}}{css}</style><main id='stage'><table id='table'>{caption}<tr><td id='cell'></td></tr></table><div id='after'></div></main>"
        ),
        resources,
        &RenderOptions {
            width: 240,
            height: 200,
            ..Default::default()
        },
    )
}

fn rect(image: &Rendered, id: &str) -> [f64; 4] {
    let node = image.geometry.iter().find(|node| node["id"] == id);
    assert!(node.is_some(), "missing {id}");
    let node = node.unwrap_or_else(|| unreachable!());
    ["x", "y", "width", "height"].map(|key| node[key].as_f64().unwrap_or(f64::NAN))
}

fn pixel(image: &Rendered, x: usize, y: usize) -> &[u8] {
    let start = (y * image.width as usize + x) * 4;
    &image.rgba[start..start + 4]
}

#[test]
fn top_caption_reserves_height_and_stays_outside_table_background() -> Result<(), String> {
    let image = table("", "<caption id='caption'></caption>")?;
    assert_eq!(rect(&image, "caption"), [0.0, 0.0, 200.0, 20.0]);
    assert_eq!(rect(&image, "cell"), [0.0, 20.0, 200.0, 40.0]);
    assert_eq!(rect(&image, "after")[1], 60.0);
    assert_eq!(pixel(&image, 10, 10), [255, 255, 255, 255]);
    assert_eq!(pixel(&image, 10, 30), [86, 69, 212, 255]);
    Ok(())
}

#[test]
fn bottom_caption_padding_and_margins_contribute_to_wrapper_height() -> Result<(), String> {
    let image = table(
        "caption{caption-side:bottom;padding:5px;margin:3px 10px}",
        "<caption id='caption'></caption>",
    )?;
    assert_eq!(rect(&image, "caption"), [10.0, 43.0, 180.0, 30.0]);
    assert_eq!(rect(&image, "cell"), [0.0, 0.0, 200.0, 40.0]);
    assert_eq!(rect(&image, "after")[1], 76.0);
    assert_eq!(pixel(&image, 20, 50), [255, 255, 255, 255]);
    Ok(())
}

#[test]
fn hidden_caption_has_no_layout_contribution() -> Result<(), String> {
    let image = table("caption{display:none}", "<caption></caption>")?;
    assert_eq!(rect(&image, "after")[1], 40.0);
    Ok(())
}

#[test]
fn specified_table_height_excludes_caption_height() -> Result<(), String> {
    let image = table("table{height:100px}", "<caption id='caption'></caption>")?;
    assert_eq!(rect(&image, "caption"), [0.0, 0.0, 200.0, 20.0]);
    assert_eq!(rect(&image, "table"), [0.0, 0.0, 200.0, 120.0]);
    assert_eq!(rect(&image, "after")[1], 120.0);
    Ok(())
}

#[test]
fn caption_text_is_shaped_and_padding_surrounds_the_line() -> Result<(), String> {
    let image = table(
        "caption{height:auto;padding:12px}",
        "<caption id='caption'>Quarterly totals</caption>",
    )?;
    assert_eq!(rect(&image, "caption"), [0.0, 0.0, 200.0, 48.0]);
    assert_eq!(rect(&image, "after")[1], 88.0);
    let dark_pixels = (12..36)
        .flat_map(|y| (12..188).map(move |x| (x, y)))
        .filter(|&(x, y)| pixel(&image, x, y)[0] < 128)
        .count();
    assert!(dark_pixels > 100, "caption glyphs must be painted");
    Ok(())
}

#[test]
fn collapsed_borders_are_counted_once_for_inline_and_block_contents() -> Result<(), String> {
    let mut resources = Resources::new();
    resources.insert(
        "/assets/Inter.ttf".into(),
        include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
    );
    for content in ["Label", "<div>Label</div>"] {
        let source = |body: &str| {
            format!(
                "<!doctype html><style>body{{margin:0;font:16px/24px Inter}}table{{border-collapse:collapse;width:200px}}td{{padding:12px;border:2px solid #5645d4}}.reference{{box-sizing:border-box;width:200px;border:2px solid #5645d4;padding:12px}}</style>{body}"
            )
        };
        let actual = render(
            &source(&format!("<table><tr><td>{content}</td></tr></table>")),
            resources.clone(),
            &RenderOptions::default(),
        )?;
        let expected = render(
            &source(&format!("<div class='reference'>{content}</div>")),
            resources.clone(),
            &RenderOptions::default(),
        )?;
        assert_eq!(actual.rgba, expected.rgba, "{content}");
    }
    Ok(())
}

#[test]
fn explicit_caption_width_and_auto_margins_use_table_wrapper_width() -> Result<(), String> {
    let image = table(
        "caption{width:100px;padding:5px;margin:0 auto}",
        "<caption id='caption'></caption>",
    )?;
    assert_eq!(rect(&image, "caption"), [45.0, 0.0, 110.0, 30.0]);
    assert_eq!(rect(&image, "after")[1], 70.0);
    Ok(())
}

#[test]
fn collapsed_bottom_only_borders_paint_without_a_spurious_top_edge() -> Result<(), String> {
    let image = render(
        "<!doctype html><style>body{margin:0}table{border-collapse:collapse;width:120px}td{padding:0;height:20px;background:#e9e6fb;border-bottom:2px solid #5645d4}</style><main id=stage><table id=table><tr><td></td><td></td></tr><tr><td></td><td></td></tr></table></main>",
        Resources::new(),
        &RenderOptions {
            width: 160,
            height: 80,
            ..Default::default()
        },
    )?;
    assert_eq!(rect(&image, "table"), [0., 0., 120., 44.]);
    for x in [0, 30, 60, 119] {
        assert_eq!(pixel(&image, x, 0), [233, 230, 251, 255]);
        assert_eq!(pixel(&image, x, 20), [86, 69, 212, 255]);
        assert_eq!(pixel(&image, x, 42), [86, 69, 212, 255]);
        assert_eq!(pixel(&image, x, 44), [255, 255, 255, 255]);
    }
    Ok(())
}

#[test]
fn collapsed_edges_use_their_own_color_and_resolve_cell_current_color() -> Result<(), String> {
    let image = table(
        "table{border-collapse:collapse}td{color:#1f9d4c;border-top:2px solid transparent;border-bottom:2px solid currentColor}",
        "",
    )?;
    assert_eq!(pixel(&image, 100, 0), [86, 69, 212, 255]);
    assert_eq!(pixel(&image, 100, 42), [31, 157, 76, 255]);
    Ok(())
}
