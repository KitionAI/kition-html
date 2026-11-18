use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, body: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!("<!doctype html><style>body{{margin:0;background:white}}{css}</style>{body}"),
        Resources::new(),
        &RenderOptions {
            width: 100,
            height: 80,
            scale,
            ..Default::default()
        },
    )
}
fn same(left: &Rendered, right: &Rendered) {
    assert_eq!((left.width, left.height), (right.width, right.height));
    let differences = left
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(right.rgba.as_chunks::<4>().0.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        differences, 0,
        "paint ownership must not alter absolute positioning"
    );
}

#[test]
fn static_opacity_ancestors_keep_absolute_descendants_in_their_group() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        let actual = frame(
            ".cb{position:relative;display:flow-root;left:8px;top:8px}.sc{opacity:.5;margin:12px;padding:5px}i{position:absolute;left:3px;top:4px;width:20px;height:12px;background:red}",
            "<div class=cb><section class=sc><i></i></section></div>",
            scale,
        )?;
        let expected = frame(
            "i{position:absolute;left:11px;top:12px;width:20px;height:12px;background:red;opacity:.5}",
            "<i></i>",
            scale,
        )?;
        same(&actual, &expected);
    }
    Ok(())
}

#[test]
fn ordinary_intermediate_blocks_do_not_take_paint_ownership() -> Result<(), String> {
    let css = ".cb{position:relative;width:80px;height:60px}.sc{opacity:.5;margin:12px;padding:5px}article{margin:9px;padding:3px}i{position:absolute;left:3px;top:4px;width:20px;height:12px;background:red}";
    same(
        &frame(
            css,
            "<div class=cb><section class=sc><article><i></i></article></section></div>",
            1.0,
        )?,
        &frame(
            css,
            "<div class=cb><section class=sc><i></i></section></div>",
            1.0,
        )?,
    );
    Ok(())
}

#[test]
fn nested_static_opacity_groups_preserve_each_isolation_boundary() -> Result<(), String> {
    let css = ".cb{position:relative}.outer{opacity:.5}.inner{opacity:.4}i{position:absolute;left:0;top:0;width:20px;height:12px;background:#5645d4}";
    let actual = frame(
        css,
        "<div class=cb><section class=outer><article class=inner><i></i></article></section></div>",
        1.0,
    )?;
    let positioned = frame(
        &format!("{css}.inner{{position:relative}}"),
        "<div class=cb><section class=outer><article class=inner><i></i></article></section></div>",
        1.0,
    )?;
    same(&actual, &positioned);
    assert_eq!(&actual.rgba[..4], [221, 218, 247, 255]);
    Ok(())
}

#[test]
fn fixed_descendants_keep_viewport_coordinates_inside_static_opacity() -> Result<(), String> {
    let actual = frame(
        "section{margin:20px;padding:10px;opacity:.5}i{position:fixed;left:4px;top:6px;width:20px;height:12px;background:red}",
        "<section><i></i></section>",
        1.0,
    )?;
    let expected = frame(
        "i{position:fixed;left:4px;top:6px;width:20px;height:12px;background:red;opacity:.5}",
        "<i></i>",
        1.0,
    )?;
    same(&actual, &expected);
    Ok(())
}

#[test]
fn external_containing_blocks_escape_overflow_but_keep_opacity() -> Result<(), String> {
    let css = ".cb{position:relative}.sc{opacity:.5;width:8px;height:8px;overflow:hidden}b{display:block;width:20px;height:20px;background:blue}i{position:absolute;left:10px;top:10px;width:20px;height:12px;background:red}";
    let actual = frame(
        css,
        "<div class=cb><section class=sc><b></b><i></i></section></div>",
        1.0,
    )?;
    let expected = frame(
        &format!("{css}.sc{{overflow:visible}}b{{width:8px;height:8px}}"),
        "<div class=cb><section class=sc><b></b><i></i></section></div>",
        1.0,
    )?;
    same(&actual, &expected);
    Ok(())
}

#[test]
fn static_mix_blend_contexts_enclose_absolute_children() -> Result<(), String> {
    let image = frame(
        "body{background:red}.cb{position:relative}.sc{mix-blend-mode:multiply}i{position:absolute;left:0;top:0;width:20px;height:12px;background:lime}",
        "<div class=cb><section class=sc><i></i></section></div>",
        1.0,
    )?;
    assert_eq!(&image.rgba[..4], [0, 0, 0, 255]);
    let outside = (30 * image.width as usize + 30) * 4;
    assert_eq!(&image.rgba[outside..outside + 4], [255, 0, 0, 255]);
    Ok(())
}

#[test]
fn explicit_isolation_keeps_descendant_blending_off_the_external_backdrop() -> Result<(), String> {
    let image = frame(
        "body{background:red}.cb{position:relative}.sc{isolation:isolate}i{position:absolute;left:0;top:0;width:20px;height:12px;background:lime;mix-blend-mode:multiply}",
        "<div class=cb><section class=sc><i></i></section></div>",
        1.0,
    )?;
    assert_eq!(&image.rgba[..4], [0, 255, 0, 255]);
    Ok(())
}

#[test]
fn unsupported_outer_overflow_escape_is_reported_once() -> Result<(), String> {
    let image = frame(
        ".cb{position:relative}.outer{opacity:.5;overflow:hidden;width:8px;height:8px}.inner{opacity:.4}i,b{position:absolute;left:10px;top:10px;width:20px;height:12px;background:red}",
        "<div class=cb><section class=outer><article class=inner><i></i><b></b></article></section></div>",
        1.0,
    )?;
    assert_eq!(
        image
            .diagnostics
            .iter()
            .filter(|d| d.starts_with("html.unsupported-overflow-ancestry:"))
            .count(),
        1
    );
    Ok(())
}
