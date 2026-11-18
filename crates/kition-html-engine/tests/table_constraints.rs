use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(width: f64, columns: &str, rows: &str, css: &str) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:0}}table{{width:{width}px;border-spacing:0}}td{{padding:0}}i{{display:block;height:20px}}{css}</style><main id=stage><table id=table>{columns}{rows}</table></main>"
        ),
        Resources::new(),
        &RenderOptions {
            width: 640,
            height: 260,
            ..Default::default()
        },
    )
}
fn row() -> &'static str {
    "<tr><td id=a><i style='width:20px'></i></td><td id=b><i style='width:40px'></i></td><td id=c><i style='width:60px'></i></td></tr>"
}
fn width(image: &Rendered, id: &str) -> f64 {
    image
        .geometry
        .iter()
        .find(|g| g["id"] == id)
        .and_then(|g| g["width"].as_f64())
        .unwrap_or(f64::NAN)
}
fn widths(image: &Rendered) -> [f64; 3] {
    [width(image, "a"), width(image, "b"), width(image, "c")]
}
#[test]
fn definite_columns_shrink_toward_content_minima_and_grow_proportionally() -> Result<(), String> {
    let cols = "<colgroup><col style='width:180px'><col span=2 style='width:90px'></colgroup>";
    assert_eq!(widths(&frame(240.0, cols, row(), "")?), [100.0, 65.0, 75.0]);
    assert_eq!(
        widths(&frame(540.0, cols, row(), "")?),
        [270.0, 135.0, 135.0]
    );
    Ok(())
}
#[test]
fn automatic_columns_receive_excess_before_definite_columns() -> Result<(), String> {
    let cols = "<colgroup><col style='width:180px'><col><col></colgroup>";
    assert_eq!(
        widths(&frame(360.0, cols, row(), "")?),
        [180.0, 72.0, 108.0]
    );
    assert_eq!(widths(&frame(160.0, cols, row(), "")?), [60.0, 40.0, 60.0]);
    Ok(())
}
#[test]
fn content_minima_expand_both_columns_and_the_table_box() -> Result<(), String> {
    let cols = "<colgroup><col span=3 style='width:10px'></colgroup>";
    let rows = "<tr><td id=a><i style='width:70px'></i></td><td id=b><i style='width:80px'></i></td><td id=c><i style='width:90px'></i></td></tr>";
    let image = frame(120.0, cols, rows, "")?;
    assert_eq!(widths(&image), [70.0, 80.0, 90.0]);
    assert_eq!(width(&image, "table"), 240.0);
    Ok(())
}
#[test]
fn later_rows_contribute_definite_widths_in_automatic_layout() -> Result<(), String> {
    let image = frame(
        360.0,
        "",
        &format!("{}{}", row(), row()),
        "tr:last-child td:first-child{width:180px}",
    )?;
    assert_eq!(widths(&image), [180.0, 72.0, 108.0]);
    Ok(())
}
#[test]
fn colspan_minima_respect_existing_column_preferences() -> Result<(), String> {
    let cols = "<colgroup><col style='width:180px'><col style='width:90px'><col></colgroup>";
    let rows = format!(
        "<tr><td colspan=2><i style='width:240px'></i></td><td><i style='width:60px'></i></td></tr>{}",
        row()
    );
    assert_eq!(widths(&frame(360.0, cols, &rows, "")?), [180.0, 90.0, 90.0]);
    Ok(())
}
#[test]
fn empty_automatic_columns_share_remaining_space() -> Result<(), String> {
    let cols = "<colgroup><col style='width:180px'><col><col></colgroup>";
    let rows = "<tr><td id=a><i style='width:20px'></i></td><td id=b></td><td id=c></td></tr>";
    assert_eq!(widths(&frame(360.0, cols, rows, "")?), [180.0, 90.0, 90.0]);
    Ok(())
}
#[test]
fn fixed_layout_ignores_later_row_width_preferences() -> Result<(), String> {
    let image = frame(
        360.0,
        "",
        &format!("{}{}", row(), row()),
        "table{table-layout:fixed}tr:last-child td:first-child{width:240px}",
    )?;
    assert_eq!(widths(&image), [120.0, 120.0, 120.0]);
    Ok(())
}
#[test]
fn separated_spacing_and_outer_edges_reduce_distributable_width() -> Result<(), String> {
    let cols = "<colgroup><col style='width:180px'><col span=2 style='width:90px'></colgroup>";
    let image = frame(
        360.0,
        cols,
        row(),
        "table{border-spacing:7px;border:3px solid black;padding:4px}",
    )?;
    let actual = widths(&image);
    assert_eq!(actual.iter().sum::<f64>(), 318.0);
    assert_eq!(width(&image, "table"), 360.0);
    assert!(actual[0] > actual[1] && actual[1] < actual[2]);
    Ok(())
}

#[test]
fn automatic_table_width_counts_collapsed_column_edges_once() -> Result<(), String> {
    let cols = "<colgroup><col style='width:180px'><col span=2 style='width:90px'></colgroup>";
    let image = frame(
        360.0,
        cols,
        row(),
        "table{width:auto;border-collapse:collapse}td{border:2px solid black;padding:12px}",
    )?;
    assert_eq!(width(&image, "table"), 362.0);
    // Cell geometry includes the resolved half borders, matching Chrome's
    // getBoundingClientRect rather than the previous border-free gutters.
    assert_eq!(widths(&image), [180.0, 90.0, 90.0]);
    Ok(())
}

#[test]
fn border_box_cell_preference_includes_collapsed_half_borders() -> Result<(), String> {
    let image = frame(
        360.0,
        "",
        row(),
        "table{border-collapse:collapse}td{box-sizing:border-box;border:2px solid black;padding:8px}td:first-child{width:160px}",
    )?;
    assert_eq!(width(&image, "a"), 160.0);
    assert_eq!(width(&image, "b"), 84.4375);
    assert_eq!(width(&image, "c"), 113.5625);
    Ok(())
}

#[test]
fn automatic_percentage_columns_and_cells_share_the_table_width() -> Result<(), String> {
    for (columns, css) in [
        (
            "<colgroup><col style='width:20%'><col style='width:30%'><col></colgroup>",
            "",
        ),
        ("", "td:first-child{width:20%}td:nth-child(2){width:30%}"),
    ] {
        assert_eq!(
            widths(&frame(480.0, columns, row(), css)?),
            [96.0, 144.0, 240.0]
        );
    }
    Ok(())
}

#[test]
fn percentage_preferences_obey_minima_and_take_priority_over_lengths() -> Result<(), String> {
    for (css, expected) in [
        (
            "td:first-child{width:80%}td:nth-child(2){width:80%}",
            [332.0, 88.0, 60.0],
        ),
        (
            "td:first-child{width:10%}td:nth-child(2){width:20%}td:last-child{width:30%}",
            [80.0, 160.0, 240.0],
        ),
        (
            "td:first-child{width:20%}td:nth-child(2){width:180px}",
            [96.0, 180.0, 204.0],
        ),
        (
            "td{padding:10px;border:2px solid black}td:first-child{width:20%}td:nth-child(2){width:30%}",
            [96.0, 144.0, 240.0],
        ),
    ] {
        assert_eq!(widths(&frame(480.0, "", row(), css)?), expected, "{css}");
    }
    assert_eq!(
        widths(&frame(
            100.0,
            "",
            row(),
            "td:first-child{width:20%}td:nth-child(2){width:30%}"
        )?),
        [20.0, 40.0, 60.0]
    );
    Ok(())
}

#[test]
fn percentage_columns_account_for_spacing_and_auto_table_intrinsic_width() -> Result<(), String> {
    let percent = "td:first-child{width:20%}td:nth-child(2){width:30%}";
    assert_eq!(
        widths(&frame(
            480.0,
            "",
            row(),
            &format!("table{{border-spacing:8px}}{percent}")
        )?),
        [89.59375, 134.390625, 224.015625]
    );
    let image = frame(480.0, "", row(), &format!("table{{width:auto}}{percent}"))?;
    assert!((width(&image, "table") - 133.328125).abs() < 1.0 / 64.0);
    let actual = widths(&image);
    for (actual, expected) in actual.into_iter().zip([26.65625, 40.0, 66.671875]) {
        assert!(
            (actual - expected).abs() < 1.0 / 64.0,
            "{actual} != {expected}"
        );
    }
    Ok(())
}
