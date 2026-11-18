use kition_html_engine::{RenderOptions, Rendered, Resources, Session, render};

fn source(css: &str, rows: &str) -> String {
    format!(
        "<!doctype html><style>body{{margin:0}}table{{border-collapse:collapse;width:120px}}td{{padding:0;height:20px;background:#e9e6fb;border:2px solid #5645d4}}{css}</style><main id=stage><table id=table>{rows}</table></main>"
    )
}

fn pixel(image: &Rendered, x: usize, y: usize) -> &[u8] {
    let offset = (y * image.width as usize + x) * 4;
    &image.rgba[offset..offset + 4]
}

fn height(image: &Rendered) -> f64 {
    image
        .geometry
        .iter()
        .find(|node| node["id"] == "table")
        .and_then(|node| node["height"].as_f64())
        .unwrap_or(f64::NAN)
}

#[test]
fn footer_without_border_has_no_extra_line_or_reserved_bottom_width() -> Result<(), String> {
    let image = render(
        &source(
            "td{border:0;border-bottom:2px solid #5645d4}tfoot td{border:0}",
            "<tbody><tr><td></td></tr><tr><td></td></tr></tbody><tfoot><tr><td></td></tr></tfoot>",
        ),
        Resources::new(),
        &RenderOptions::default(),
    )?;
    assert_eq!(height(&image), 64.);
    assert_eq!(pixel(&image, 40, 42), [86, 69, 212, 255]);
    assert_eq!(pixel(&image, 40, 63), [233, 230, 251, 255]);
    assert_eq!(pixel(&image, 40, 64), [255, 255, 255, 255]);
    Ok(())
}

#[test]
fn hidden_shared_edge_suppresses_paint_and_layout_width() -> Result<(), String> {
    let image = render(
        &source(
            "tr:last-child td{border-top-style:hidden}",
            "<tr><td></td></tr><tr><td></td></tr>",
        ),
        Resources::new(),
        &RenderOptions::default(),
    )?;
    assert_eq!(height(&image), 44.);
    for y in 2..42 {
        assert_eq!(pixel(&image, 40, y), [233, 230, 251, 255]);
    }
    assert_eq!(pixel(&image, 40, 42), [86, 69, 212, 255]);
    Ok(())
}

#[test]
fn rowspan_and_colspan_have_no_internal_border() -> Result<(), String> {
    let image = render(
        &source(
            "",
            "<tr><td rowspan=2></td><td></td><td></td></tr><tr><td colspan=2></td></tr>",
        ),
        Resources::new(),
        &RenderOptions::default(),
    )?;
    assert_eq!(height(&image), 46.);
    assert_eq!(pixel(&image, 20, 23), [233, 230, 251, 255]);
    assert_eq!(pixel(&image, 70, 23), [86, 69, 212, 255]);
    assert_eq!(pixel(&image, 80, 35), [233, 230, 251, 255]);
    Ok(())
}

#[test]
fn equal_width_edges_keep_the_upper_cell_color() -> Result<(), String> {
    let image = render(
        &source(
            "td{border-color:currentColor;color:#5645d4}tr:last-child td{color:#e5486a}",
            "<tr><td></td></tr><tr><td></td></tr>",
        ),
        Resources::new(),
        &RenderOptions::default(),
    )?;
    assert_eq!(pixel(&image, 40, 22), [86, 69, 212, 255]);
    assert_eq!(pixel(&image, 40, 44), [229, 72, 106, 255]);
    Ok(())
}

#[test]
fn later_cell_border_is_used_when_first_cell_has_none() -> Result<(), String> {
    let image = render(
        &source(
            "td:first-child{border:0}td:last-child{border-color:#e5486a}",
            "<tr><td></td><td></td></tr>",
        ),
        Resources::new(),
        &RenderOptions::default(),
    )?;
    assert_eq!(pixel(&image, 90, 0), [229, 72, 106, 255]);
    assert_eq!(pixel(&image, 119, 10), [229, 72, 106, 255]);
    assert_eq!(pixel(&image, 10, 10), [233, 230, 251, 255]);
    Ok(())
}

#[test]
fn live_border_changes_rebuild_the_conflict_map() -> Result<(), String> {
    let html = source("", "<tr><td></td></tr><tr><td id=last></td></tr>")
        + "<script>document.body.onclick=()=>document.getElementById('last').style.borderBottomColor='#e5486a'</script>";
    let mut session = Session::new(&html, Resources::new(), &RenderOptions::default())?;
    assert_eq!(pixel(&session.frame(1.)?, 40, 44), [86, 69, 212, 255]);
    session.click(40., 10.)?;
    assert_eq!(pixel(&session.frame(1.)?, 40, 44), [229, 72, 106, 255]);
    Ok(())
}

#[test]
fn fixed_columns_include_the_first_cells_resolved_half_borders() -> Result<(), String> {
    for (css, expected) in [
        ("td:first-child{width:60px}", 62.),
        ("td:first-child{width:50%}", 61.),
        ("td:first-child{width:60px;box-sizing:border-box}", 60.),
    ] {
        let image = render(
            &source(
                &format!("table{{table-layout:fixed}}{css}"),
                "<tr><td id=first></td><td></td></tr>",
            ),
            Resources::new(),
            &RenderOptions::default(),
        )?;
        let width = image
            .geometry
            .iter()
            .find(|node| node["id"] == "first")
            .and_then(|node| node["width"].as_f64())
            .unwrap_or(f64::NAN);
        assert_eq!(width, expected, "{css}");
    }
    Ok(())
}

#[test]
fn caption_keeps_its_own_border_above_the_collapsed_grid() -> Result<(), String> {
    let image = render(
        &source(
            "caption{height:20px;border:4px solid #1f9d4c}",
            "<caption></caption><tr><td></td><td></td></tr>",
        ),
        Resources::new(),
        &RenderOptions::default(),
    )?;
    assert_eq!(pixel(&image, 40, 1), [31, 157, 76, 255]);
    assert_eq!(pixel(&image, 1, 10), [31, 157, 76, 255]);
    assert_eq!(pixel(&image, 40, 28), [86, 69, 212, 255]);
    Ok(())
}

#[test]
fn collapsed_edges_snap_in_css_pixels_before_device_scaling() -> Result<(), String> {
    let html = source("", "<tr><td></td><td></td><td></td></tr>");
    let options = RenderOptions {
        width: 140,
        height: 40,
        ..Default::default()
    };
    let one = render(&html, Resources::new(), &options)?;
    let two = render(
        &html,
        Resources::new(),
        &RenderOptions {
            scale: 2.,
            ..options
        },
    )?;
    for y in 0..40 {
        for x in 0..140 {
            for dy in 0..2 {
                for dx in 0..2 {
                    assert_eq!(
                        pixel(&two, x * 2 + dx, y * 2 + dy),
                        pixel(&one, x, y),
                        "at {x}, {y}"
                    );
                }
            }
        }
    }
    Ok(())
}
