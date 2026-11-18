use kition_html_engine::{RenderOptions, Rendered, default_resources, render};

fn frame(css: &str, markup: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:24px;background:white;color:#1a1a1a;font:16px/24px Inter}}p{{margin:0;width:70px;border:1px solid #5645d4;{css}}}</style><main id=stage><p id=box>{markup}</p></main>"
        ),
        default_resources(),
        &RenderOptions {
            width: 400,
            height: 260,
            scale,
            ..Default::default()
        },
    )
}
fn height(image: &Rendered) -> Result<f64, String> {
    image
        .geometry
        .iter()
        .find(|g| g["id"] == "box")
        .and_then(|g| g["height"].as_f64())
        .ok_or("missing box height".into())
}

#[test]
fn wbr_wraps_without_forcing_a_break_when_the_line_fits() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        let actual = frame("", "alpha<wbr>beta<wbr>gamma<wbr>delta", scale)?;
        let forced = frame("", "alpha<br>beta<br>gamma<br>delta", scale)?;
        assert_eq!(height(&actual)?, height(&forced)?);
        assert!(height(&actual)? > 97.0 && height(&actual)? <= 98.0);
        assert!(actual.rgba == forced.rgba);
        let wide = frame("width:300px", "alpha<wbr>beta<wbr>gamma<wbr>delta", scale)?;
        let plain = frame("width:300px", "alphabetagammadelta", scale)?;
        assert_eq!(height(&wide)?, height(&plain)?);
        assert!(wide.rgba == plain.rgba);
    }
    Ok(())
}

#[test]
fn wbr_keeps_its_discretionary_break_inside_nowrap_and_pre() -> Result<(), String> {
    for css in [
        "white-space:nowrap",
        "white-space:pre",
        "white-space:pre-wrap",
    ] {
        let image = frame(css, "alpha<wbr>beta<wbr>gamma<wbr>delta", 1.0)?;
        assert_eq!(height(&image)?, 98.0, "{css}");
    }
    let no_break = frame("", "<nobr>alpha beta gamma delta</nobr>", 1.0)?;
    assert_eq!(height(&no_break)?, 26.0);
    let explicit = frame("", "<nobr>alpha<wbr>beta<wbr>gamma<wbr>delta</nobr>", 1.0)?;
    assert_eq!(height(&explicit)?, 98.0);
    Ok(())
}

#[test]
fn wbr_does_not_create_a_capitalization_word_boundary() -> Result<(), String> {
    let actual = frame(
        "text-transform:capitalize",
        "alpha<wbr>beta<wbr>gamma<wbr>delta",
        1.0,
    )?;
    let expected = frame("", "Alpha<br>beta<br>gamma<br>delta", 1.0)?;
    assert!(actual.rgba == expected.rgba);
    let spaced = frame(
        "text-transform:capitalize",
        "alpha <wbr> beta <wbr> gamma",
        1.0,
    )?;
    let expected = frame("", "Alpha<br>Beta<br>Gamma", 1.0)?;
    assert!(spaced.rgba == expected.rgba);
    Ok(())
}

#[test]
fn hidden_wbr_and_consecutive_wbr_do_not_add_empty_lines() -> Result<(), String> {
    let hidden = frame("", "alpha<wbr style='display:none'>beta", 1.0)?;
    let plain = frame("", "alphabeta", 1.0)?;
    assert!(hidden.rgba == plain.rgba);
    let invisible = frame("", "alpha<wbr style='visibility:hidden'>beta", 1.0)?;
    assert_eq!(height(&invisible)?, 50.0);
    let repeated = frame("", "<wbr><wbr>alpha<wbr><wbr>beta<wbr><wbr>", 1.0)?;
    let expected = frame("", "alpha<br>beta", 1.0)?;
    assert!(repeated.rgba == expected.rgba);
    Ok(())
}

#[test]
fn wbr_retains_nested_styles_and_bidi_ordering() -> Result<(), String> {
    for css in ["", "direction:rtl"] {
        let actual = frame(css, "alpha<span><wbr></span><b>beta</b><wbr>gamma", 1.0)?;
        let expected = frame(css, "alpha<br><b>beta</b><br>gamma", 1.0)?;
        assert!(actual.rgba == expected.rgba, "{css}");
    }
    Ok(())
}

#[test]
fn mark_uses_html_default_colors_and_author_styles_win() -> Result<(), String> {
    let image = frame("width:300px", "A <mark>highlight</mark> B", 1.0)?;
    let expected = frame(
        "width:300px",
        "A <span style='background:yellow;color:black'>highlight</span> B",
        1.0,
    )?;
    assert!(
        image.rgba == expected.rgba,
        "mark colors: actual {} yellow pixels, expected {}",
        image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| **p == [255, 255, 0, 255])
            .count(),
        expected
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| **p == [255, 255, 0, 255])
            .count()
    );
    assert!(
        image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| **p == [255, 255, 0, 255])
            .count()
            > 100
    );
    let overridden = frame(
        "width:300px",
        "<mark style='background:#5645d4;color:white'>highlight</mark>",
        1.0,
    )?;
    let expected = frame(
        "width:300px",
        "<span style='background:#5645d4;color:white'>highlight</span>",
        1.0,
    )?;
    assert!(overridden.rgba == expected.rgba);
    Ok(())
}

#[test]
fn abbreviation_decoration_requires_a_title_and_can_be_removed() -> Result<(), String> {
    let decorated = frame(
        "width:300px",
        "<abbr title='Hypertext Markup Language'>HTML</abbr>",
        1.0,
    )?;
    let expected = frame(
        "width:300px",
        "<span style='text-decoration:dotted underline'>HTML</span>",
        1.0,
    )?;
    assert!(decorated.rgba == expected.rgba);
    let plain = frame("width:300px", "HTML", 1.0)?;
    assert!(decorated.rgba != plain.rgba);
    for markup in [
        "<abbr>HTML</abbr>",
        "<abbr title='HTML' style='text-decoration:none'>HTML</abbr>",
    ] {
        assert!(frame("width:300px", markup, 1.0)?.rgba == plain.rgba);
    }
    Ok(())
}

#[test]
fn wbr_updates_intrinsic_width_without_adding_letter_spacing() -> Result<(), String> {
    let intrinsic = frame("width:min-content", "alpha<wbr>beta<wbr>gamma", 1.0)?;
    let width = intrinsic
        .geometry
        .iter()
        .find(|g| g["id"] == "box")
        .and_then(|g| g["width"].as_f64())
        .ok_or("missing width")?;
    assert!((width - 57.453125).abs() <= 1.0 / 64.0);
    for scale in [1.0, 1.5, 2.0] {
        let actual = frame(
            "width:90px;letter-spacing:2px",
            "alpha<wbr>beta<wbr>gamma",
            scale,
        )?;
        let expected = frame(
            "width:90px;letter-spacing:2px",
            "alpha<br>beta<br>gamma",
            scale,
        )?;
        assert!(actual.rgba == expected.rgba);
    }
    Ok(())
}

#[test]
fn positioned_and_floating_wbr_still_offer_a_zero_width_break() -> Result<(), String> {
    for style in [
        "position:absolute",
        "float:left",
        "display:inline-block",
        "font-size:80px;line-height:120px",
        "letter-spacing:30px",
    ] {
        let markup = format!("alpha<wbr style='{style}'>beta<wbr>gamma");
        let actual = frame("", &markup, 1.0)?;
        let expected = frame("", "alpha<br>beta<br>gamma", 1.0)?;
        assert_eq!(height(&actual)?, 74.0, "{style}");
        assert!(actual.rgba == expected.rgba, "{style}");
        assert_eq!(height(&frame("width:300px", &markup, 1.0)?)?, 26.0);
    }
    Ok(())
}
