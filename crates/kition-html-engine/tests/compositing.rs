use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(style: &str, body: &str) -> Result<Rendered, String> {
    render(
        &format!("<!doctype html><style>body{{margin:0}}{style}</style>{body}"),
        Resources::new(),
        &RenderOptions {
            width: 64,
            height: 64,
            ..Default::default()
        },
    )
}

fn pixel(frame: &Rendered, x: usize, y: usize) -> &[u8] {
    let offset = (y * frame.width as usize + x) * 4;
    &frame.rgba[offset..offset + 4]
}

#[test]
fn has_selectors_match_descendants_and_siblings_without_matching_other_parents()
-> Result<(), String> {
    let result = frame(
        "section{width:16px;height:8px;background:red}section:has(>div){background:lime}section:has(+article){background:blue}",
        "<section><div></div></section><section><aside></aside></section><section></section><article></article>",
    )?;
    assert_eq!(pixel(&result, 4, 4), [0, 255, 0, 255]);
    assert_eq!(pixel(&result, 4, 12), [255, 0, 0, 255]);
    assert_eq!(pixel(&result, 4, 20), [0, 0, 255, 255]);
    Ok(())
}

#[test]
fn opacity_composites_overflowing_children_as_a_single_group() -> Result<(), String> {
    let result = frame(
        "section{position:relative;opacity:.5}i,b{position:absolute;width:16px;height:16px;background:red}b{left:8px;top:8px;background:blue}",
        "<section><i></i><b></b></section>",
    )?;
    for (x, y, expected) in [
        (4, 4, [255, 126, 126, 255]),
        (12, 12, [126, 126, 255, 255]),
        (20, 20, [126, 126, 255, 255]),
    ] {
        assert_eq!(pixel(&result, x, y), expected);
    }
    assert_eq!(pixel(&result, 30, 30), [255, 255, 255, 255]);
    Ok(())
}

#[test]
fn opacity_preserves_explicit_overflow_clipping_and_transforms() -> Result<(), String> {
    let result = frame(
        "section{position:relative;width:8px;height:8px;overflow:hidden;opacity:.5;transform:translate(16px,16px)}i{position:absolute;width:24px;height:24px;background:red}",
        "<section><i></i></section>",
    )?;
    assert_eq!(pixel(&result, 20, 20)[0], 255);
    assert!((127..=128).contains(&pixel(&result, 20, 20)[1]));
    assert_eq!(pixel(&result, 28, 28), [255, 255, 255, 255]);
    assert_eq!(pixel(&result, 4, 4), [255, 255, 255, 255]);
    Ok(())
}

#[test]
fn opacity_includes_outlines_in_the_group() -> Result<(), String> {
    let result = frame(
        "section{margin:8px;width:16px;height:16px;outline:4px solid red;opacity:.5}",
        "<section></section>",
    )?;
    assert_eq!(pixel(&result, 6, 12)[0], 255);
    assert!((127..=128).contains(&pixel(&result, 6, 12)[1]));
    Ok(())
}

#[test]
fn outline_offset_moves_the_stroke_without_filling_the_gap() -> Result<(), String> {
    let result = frame(
        "section{margin:16px;width:16px;height:16px;background:blue;outline:4px solid red;outline-offset:4px}",
        "<section></section>",
    )?;
    assert_eq!(pixel(&result, 10, 20), [255, 0, 0, 255]);
    assert_eq!(pixel(&result, 14, 20), [255, 255, 255, 255]);
    assert_eq!(pixel(&result, 20, 20), [0, 0, 255, 255]);
    Ok(())
}

#[test]
fn mix_blend_modes_use_the_backdrop() -> Result<(), String> {
    for (mode, expected) in [
        ("multiply", [0, 0, 0, 255]),
        ("screen", [255, 0, 255, 255]),
        ("difference", [255, 0, 255, 255]),
    ] {
        let result = frame(
            &format!(
                "section{{width:32px;height:32px;background:red}}div{{width:16px;height:16px;background:blue;mix-blend-mode:{mode}}}"
            ),
            "<section><div></div></section>",
        )?;
        assert_eq!(pixel(&result, 8, 8), expected, "{mode}");
        assert_eq!(pixel(&result, 24, 24), [255, 0, 0, 255]);
    }
    Ok(())
}

#[test]
fn svg_backgrounds_repeat_and_respect_no_repeat() -> Result<(), String> {
    for (repeat, far_pixel) in [
        ("repeat", [255, 0, 0, 255]),
        ("no-repeat", [255, 255, 255, 255]),
    ] {
        let mut resources = Resources::new();
        resources.insert("/assets/tile.svg".into(),br#"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><rect width="4" height="8" fill="red"/></svg>"#.to_vec());
        let result = render(
            &format!(
                "<!doctype html><style>body{{margin:0}}div{{width:32px;height:32px;background:url(/assets/tile.svg) 0 0/8px 8px {repeat}}}</style><div></div>"
            ),
            resources,
            &RenderOptions {
                width: 64,
                height: 64,
                ..Default::default()
            },
        )?;
        assert_eq!(pixel(&result, 2, 2), [255, 0, 0, 255]);
        assert_eq!(pixel(&result, 6, 2), [255, 255, 255, 255]);
        assert_eq!(pixel(&result, 26, 26), far_pixel);
        assert_eq!(pixel(&result, 34, 26), [255, 255, 255, 255]);
    }
    Ok(())
}

#[test]
fn background_blending_uses_background_layers_but_not_the_external_backdrop() -> Result<(), String>
{
    for (background, expected) in [
        ("linear-gradient(blue,blue) red", [0, 0, 0, 255]),
        ("linear-gradient(blue,blue)", [0, 0, 255, 255]),
        (
            "linear-gradient(blue,blue),linear-gradient(red,red)",
            [0, 0, 0, 255],
        ),
    ] {
        let result = frame(
            &format!(
                "section{{width:32px;height:32px;background:lime}}div{{width:16px;height:16px;background:{background};background-blend-mode:multiply}}"
            ),
            "<section><div></div></section>",
        )?;
        assert_eq!(pixel(&result, 8, 8), expected, "{background}");
        assert_eq!(pixel(&result, 24, 24), [0, 255, 0, 255]);
    }
    Ok(())
}

#[test]
fn translucent_antialiased_edges_preserve_an_opaque_backdrop() -> Result<(), String> {
    for style in [
        "div{margin:3px;width:39px;height:27px;border-radius:13px;background:rgba(118,118,118,.3)}",
        "div{margin:3px;width:39px;height:27px;border-radius:13px;border:1px solid rgba(118,118,118,.3)}",
        "div{margin:3px;width:39px;height:27px;background:rgba(118,118,118,.3);transform:rotate(17deg)}",
    ] {
        let result = frame(style, "<div></div>")?;
        for (index, rgba) in result.rgba.as_chunks::<4>().0.iter().enumerate() {
            assert_eq!(rgba[3], 255, "pixel {index}: {rgba:?}; {style}");
        }
    }
    Ok(())
}
