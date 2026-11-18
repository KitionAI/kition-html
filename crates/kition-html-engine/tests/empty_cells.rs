use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(content: &str, css: &str) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:0}}table{{width:96px;table-layout:fixed;border-spacing:0;empty-cells:hide;background:yellow}}tr{{background:red}}td{{height:40px;padding:0;border:4px solid green;background:blue}}{css}</style><main id=stage><table><tr><td><span></span></td><td id=empty>{content}</td></tr></table></main>"
        ),
        Resources::new(),
        &RenderOptions {
            width: 120,
            height: 80,
            ..Default::default()
        },
    )
}
fn pixel(image: &Rendered, x: u32, y: u32) -> &[u8] {
    let i = ((y * image.width + x) * 4) as usize;
    &image.rgba[i..i + 4]
}

#[test]
fn empty_cells_hide_cell_and_row_backgrounds_but_keep_table_background() -> Result<(), String> {
    for content in [
        "",
        " \n\t ",
        "<!-- comment -->",
        "<span style='display:none'>hidden</span>",
    ] {
        let image = frame(content, "")?;
        assert_eq!(pixel(&image, 70, 20), [255, 255, 0, 255], "{content}");
        assert_eq!(pixel(&image, 49, 1), [255, 255, 0, 255], "{content}");
        assert_eq!(pixel(&image, 20, 20), [0, 0, 255, 255]);
    }
    Ok(())
}

#[test]
fn preserved_whitespace_and_empty_inflow_boxes_make_cells_nonempty() -> Result<(), String> {
    for (content, css) in [
        (" ", "#empty{white-space:pre}"),
        ("<span></span>", ""),
        ("", "#empty::before{content:'x'}"),
    ] {
        let image = frame(content, css)?;
        assert_eq!(pixel(&image, 70, 30), [0, 0, 255, 255], "{content}, {css}");
        assert_eq!(pixel(&image, 49, 1), [0, 128, 0, 255]);
    }
    Ok(())
}

#[test]
fn positioned_descendants_still_draw_when_the_empty_cell_background_is_hidden() -> Result<(), String>
{
    let image = frame(
        "<div style='position:absolute;top:4px;left:4px;width:8px;height:8px;background:red'></div>",
        "#empty{position:relative}",
    )?;
    assert_eq!(pixel(&image, 70, 30), [255, 255, 0, 255]);
    assert_eq!(pixel(&image, 58, 10), [255, 0, 0, 255]);
    Ok(())
}

#[test]
fn collapsed_borders_ignore_empty_cells_hide() -> Result<(), String> {
    let image = frame("", "table{border-collapse:collapse}")?;
    assert_eq!(pixel(&image, 70, 20), [0, 0, 255, 255]);
    Ok(())
}
