use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(content: &str, css: &str) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:0}}table{{width:360px;border-spacing:0}}td{{padding:0}}i{{display:block;height:10px}}{css}</style><main id=stage><table>{content}</table></main>"
        ),
        Resources::new(),
        &RenderOptions {
            width: 400,
            height: 120,
            ..Default::default()
        },
    )
}

fn width(image: &Rendered, id: &str) -> f64 {
    image
        .geometry
        .iter()
        .find(|node| node["id"] == id)
        .and_then(|node| node["width"].as_f64())
        .unwrap_or(f64::NAN)
}

#[test]
fn auto_columns_distribute_excess_width_in_proportion_to_content() -> Result<(), String> {
    let image = frame(
        "<tr><td id=a><i style='width:40px'></i></td><td id=b><i style='width:80px'></i></td></tr>",
        "",
    )?;
    assert_eq!(width(&image, "a"), 120.0);
    assert_eq!(width(&image, "b"), 240.0);
    Ok(())
}

#[test]
fn spanning_cells_contribute_to_all_covered_columns() -> Result<(), String> {
    let image = frame(
        "<tr><td colspan=2><i style='width:180px'></i></td><td><i style='width:60px'></i></td></tr><tr><td id=a><i style='width:20px'></i></td><td id=b><i style='width:40px'></i></td><td id=c><i style='width:60px'></i></td></tr>",
        "",
    )?;
    assert_eq!(
        [width(&image, "a"), width(&image, "b"), width(&image, "c")],
        [90.0, 180.0, 90.0]
    );
    Ok(())
}

#[test]
fn border_spacing_is_removed_before_content_weighted_distribution() -> Result<(), String> {
    let image = frame(
        "<tr><td id=a><i style='width:40px'></i></td><td id=b><i style='width:80px'></i></td></tr>",
        "table{border-spacing:8px;border:6px solid black;padding:3px}",
    )?;
    assert_eq!(width(&image, "a"), 106.0);
    assert_eq!(width(&image, "b"), 212.0);
    Ok(())
}

#[test]
fn fixed_layout_keeps_equal_columns_despite_unequal_content() -> Result<(), String> {
    let image = frame(
        "<tr><td id=a><i style='width:40px'></i></td><td id=b><i style='width:80px'></i></td></tr>",
        "table{table-layout:fixed}",
    )?;
    assert_eq!(width(&image, "a"), 180.0);
    assert_eq!(width(&image, "b"), 180.0);
    Ok(())
}
