use kition_html_engine::{RenderOptions, Rendered, Resources, Session, render};

fn html(css: &str) -> String {
    format!(
        "<!doctype html><style>body{{margin:0;min-height:300px}}fieldset{{margin:0;border:2px solid #5645d4;padding:20px;width:320px}}legend{{width:120px;height:24px;padding:0;background:#e5486a}}.content{{width:80px;height:50px;background:#1f9d4c}}{css}</style><main id=stage><fieldset id=field><legend id=legend></legend><div id=first class=content></div><div id=second class=content></div></fieldset></main>"
    )
}
fn geometry<'a>(frame: &'a Rendered, id: &str) -> Result<&'a serde_json::Value, String> {
    frame
        .geometry
        .iter()
        .find(|node| node["id"] == id)
        .ok_or_else(|| format!("Missing geometry for {id}"))
}

fn options() -> RenderOptions {
    RenderOptions {
        width: 640,
        height: 480,
        ..Default::default()
    }
}

#[test]
fn legend_reserves_the_top_border_and_keeps_the_content_formatting_context() -> Result<(), String> {
    for (css, height, second_x, second_y) in [
        ("", 166., 22., 94.),
        ("fieldset{display:flex;gap:8px}", 116., 110., 44.),
        (
            "fieldset{display:grid;grid-template-columns:1fr 1fr}",
            116.,
            182.,
            44.,
        ),
    ] {
        let frame = render(&html(css), Resources::new(), &options())?;
        assert_eq!(geometry(&frame, "field")?["height"], height);
        assert_eq!(geometry(&frame, "legend")?["y"], 0.);
        assert_eq!(geometry(&frame, "legend")?["width"], 120.);
        assert_eq!(geometry(&frame, "first")?["y"], 44.);
        assert_eq!(geometry(&frame, "second")?["x"], second_x);
        assert_eq!(geometry(&frame, "second")?["y"], second_y);
    }
    Ok(())
}

#[test]
fn specified_box_sizes_and_intrinsic_legend_width_are_preserved() -> Result<(), String> {
    for (css, width, height) in [
        ("fieldset{height:100px}", 364., 144.),
        ("fieldset{height:100px;box-sizing:border-box}", 320., 100.),
        ("fieldset{min-height:200px}", 364., 244.),
        ("fieldset{max-height:60px}", 364., 104.),
        ("fieldset{display:inline-block;width:auto}", 164., 166.),
        ("fieldset{border-width:40px}", 440., 220.),
    ] {
        let frame = render(&html(css), Resources::new(), &options())?;
        let field = geometry(&frame, "field")?;
        assert_eq!(field["width"], width, "{css}");
        assert_eq!(field["height"], height, "{css}");
    }
    Ok(())
}

#[test]
fn border_gap_excludes_only_the_legend_box_even_with_auto_margins() -> Result<(), String> {
    let frame = render(
        &html("legend{margin-inline:auto}"),
        Resources::new(),
        &options(),
    )?;
    let pixel = |x: usize, y: usize| &frame.rgba[(y * 640 + x) * 4..(y * 640 + x) * 4 + 4];
    assert_eq!(pixel(30, 11), &[86, 69, 212, 255]);
    assert_eq!(pixel(130, 11), &[229, 72, 106, 255]);
    assert_eq!(pixel(300, 11), &[86, 69, 212, 255]);
    Ok(())
}

#[test]
fn hiding_a_legend_rebuilds_geometry_and_removes_the_border_gap() -> Result<(), String> {
    let source = html("")
        + "<script>document.body.onclick=()=>document.getElementById('legend').style.display='none'</script>";
    let mut session = Session::new(&source, Resources::new(), &options())?;
    let before = session.frame(1.)?;
    assert_eq!(geometry(&before, "first")?["y"], 44.);
    session.click(400., 40.)?;
    let after = session.frame(1.)?;
    assert_eq!(geometry(&after, "first")?["y"], 22.);
    assert_eq!(&after.rgba[(30 * 4)..(30 * 4 + 4)], &[86, 69, 212, 255]);
    Ok(())
}
