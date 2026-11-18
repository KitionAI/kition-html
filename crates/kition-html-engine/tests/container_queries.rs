use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, body: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:0}}.target{{width:10px;height:10px;background:red}}{css}</style><main id=stage>{body}</main>"
        ),
        Resources::new(),
        &RenderOptions {
            width: 640,
            height: 480,
            scale,
            ..Default::default()
        },
    )
}
fn width(image: &Rendered, id: &str) -> Result<f64, String> {
    image
        .geometry
        .iter()
        .find(|v| v["id"] == id)
        .and_then(|v| v["width"].as_f64())
        .ok_or_else(|| format!("missing {id}"))
}

#[test]
fn size_queries_use_the_content_box_and_do_not_depend_on_device_scale() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        let image = frame(
            "section{container-type:inline-size;box-sizing:border-box;width:240px;padding:10px;border:10px solid black}@container(width:200px){.target{width:100px}}",
            "<section><div class=target id=result></div></section>",
            scale,
        )?;
        assert_eq!(width(&image, "result")?, 100.0);
        assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    }
    Ok(())
}

#[test]
fn nested_named_containers_settle_after_ancestor_queries_change_size() -> Result<(), String> {
    let image = frame(
        "section{container:outer / inline-size;width:400px}article{container:inner / inline-size;width:80px}@container outer (width > 300px){article{width:300px}}@container inner (width >= 250px){.target{width:200px}}",
        "<section><article><div class=target id=result></div></article></section>",
        1.0,
    )?;
    assert_eq!(width(&image, "result")?, 200.0);
    assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    Ok(())
}

#[test]
fn inline_size_containment_does_not_supply_a_block_axis_query() -> Result<(), String> {
    let image = frame(
        "section{container-type:inline-size;width:300px;height:100px}@container(height > 50px){.target{width:200px}}@container(width > 250px){.target{height:20px}}",
        "<section><div class=target id=result></div></section>",
        1.0,
    )?;
    assert_eq!(width(&image, "result")?, 10.0);
    Ok(())
}

#[test]
fn size_containers_supply_both_axes_and_container_relative_units() -> Result<(), String> {
    let image = frame(
        "section{container-type:size;width:300px;height:100px}@container(aspect-ratio > 2/1) and (orientation:landscape){.target{width:50cqw;height:50cqh}}",
        "<section><div class=target id=result></div></section>",
        1.0,
    )?;
    assert_eq!(width(&image, "result")?, 150.0);
    let result = image
        .geometry
        .iter()
        .find(|v| v["id"] == "result")
        .ok_or("missing result")?;
    assert_eq!(result["height"], 50.0);
    Ok(())
}

#[test]
fn display_contents_and_non_atomic_inlines_do_not_create_size_query_boxes() -> Result<(), String> {
    for display in ["contents", "inline"] {
        let image = frame(
            &format!(
                "section{{container:outer / inline-size;width:300px}}span{{container:inner / size;display:{display};width:400px;height:100px}}@container inner (width >= 0px){{.target{{width:200px}}}}@container outer (width > 200px){{.target{{height:20px}}}}"
            ),
            "<section><span><div class=target id=result></div></span></section>",
            1.0,
        )?;
        assert_eq!(width(&image, "result")?, 10.0, "{display}");
    }
    Ok(())
}

#[test]
fn container_units_without_a_qualifying_container_use_the_small_viewport() -> Result<(), String> {
    let image = frame(
        ".target{width:50cqw;height:50cqh}",
        "<div class=target id=result></div>",
        1.0,
    )?;
    assert_eq!(width(&image, "result")?, 320.0);
    Ok(())
}

#[test]
fn size_containment_ignores_descendant_intrinsic_sizes() -> Result<(), String> {
    let image = frame(
        "section{display:inline-block;container-type:size;padding:10px;border:2px solid black}article{width:300px;height:100px}@container(width > 50px){article{width:20px}}",
        "<section id=container><article></article></section>",
        1.0,
    )?;
    assert_eq!(width(&image, "container")?, 24.0);
    let result = image
        .geometry
        .iter()
        .find(|v| v["id"] == "container")
        .ok_or("missing container")?;
    assert_eq!(result["height"], 24.0);
    assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    Ok(())
}

#[test]
fn vertical_inline_size_queries_use_physical_height() -> Result<(), String> {
    let image = frame(
        "section{writing-mode:vertical-rl;container-type:inline-size;width:100px;height:300px}article{container-type:inline-size;height:200px;width:80px}@container(inline-size:200px){.target{width:40px}}",
        "<section><article><div class=target id=result></div></article></section>",
        1.0,
    )?;
    assert_eq!(width(&image, "result")?, 40.0);
    Ok(())
}

#[test]
fn inline_size_containment_preserves_block_fill_and_uncontained_height() -> Result<(), String> {
    let image = frame(
        "section,article{container-type:inline-size}article{display:inline-block}i{display:block;width:300px;height:20px}",
        "<section id=block><i></i></section><article id=inline><i></i></article>",
        1.0,
    )?;
    assert_eq!(width(&image, "block")?, 640.0);
    assert_eq!(width(&image, "inline")?, 0.0);
    for id in ["block", "inline"] {
        let element = image
            .geometry
            .iter()
            .find(|v| v["id"] == id)
            .ok_or("missing container")?;
        assert_eq!(element["height"], 20.0);
    }
    Ok(())
}
