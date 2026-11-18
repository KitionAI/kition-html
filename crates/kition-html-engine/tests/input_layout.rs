use kition_html_engine::{RenderOptions, Rendered, Session, default_resources, render};

fn source(css: &str, value: &str) -> String {
    format!(
        "<!doctype html><style>body{{margin:0;font:16px/24px Inter}}input{{font:14px Inter;width:240px;padding:10px;border:1px solid #c8c4be;color:#1a1a1a}}#after{{height:24px;background:#5645d4}}{css}</style><main id=stage><div id=line><input id=input value='{value}'></div><div id=after></div></main>"
    )
}
fn options() -> RenderOptions {
    RenderOptions {
        width: 320,
        height: 120,
        ..Default::default()
    }
}
fn geometry<'a>(image: &'a Rendered, id: &str) -> Result<&'a serde_json::Value, String> {
    image
        .geometry
        .iter()
        .find(|entry| entry["id"] == id)
        .ok_or_else(|| format!("Missing {id}"))
}
fn ink_bounds(image: &Rendered) -> Option<[usize; 4]> {
    let mut bounds = None::<[usize; 4]>;
    for y in 0..image.height as usize {
        for x in 0..image.width as usize {
            let i = (y * image.width as usize + x) * 4;
            if image.rgba[i..i + 3].iter().all(|&channel| channel < 80) {
                bounds = Some(bounds.map_or([x, y, x, y], |b| {
                    [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)]
                }));
            }
        }
    }
    bounds
}

#[test]
fn single_line_controls_contribute_a_font_baseline_without_extra_descender_space()
-> Result<(), String> {
    for (css, expected_height) in [
        ("", 39.),
        ("input{font-size:10px}", 34.),
        ("input{font-size:24px}", 51.),
        ("input{line-height:2}", 50.),
        ("input{line-height:.5}", 39.),
        ("input{height:64px}", 86.),
    ] {
        for text in ["", "Input text"] {
            let image = render(&source(css, text), default_resources(), &options())?;
            assert_eq!(
                geometry(&image, "input")?["height"],
                expected_height,
                "{css}"
            );
            assert_eq!(
                geometry(&image, "line")?["height"],
                expected_height,
                "{css}"
            );
            assert_eq!(geometry(&image, "after")?["y"], expected_height, "{css}");
        }
    }
    Ok(())
}

#[test]
fn input_text_alignment_uses_the_content_width_and_direction() -> Result<(), String> {
    let frame = |css| render(&source(css, "Input"), default_resources(), &options());
    let left = ink_bounds(&frame("")?).ok_or("missing left text")?;
    let center = ink_bounds(&frame("input{text-align:center}")?).ok_or("missing centered text")?;
    let right = ink_bounds(&frame("input{text-align:right}")?).ok_or("missing right text")?;
    let rtl = ink_bounds(&frame("input{direction:rtl}")?).ok_or("missing RTL text")?;
    assert!(left[0] < 20);
    assert!(center[0] > 100 && center[0] < 130);
    assert!(right[0] > 200);
    assert_eq!(right, rtl);
    assert_eq!(left[1], right[1]);
    Ok(())
}

#[test]
fn long_input_values_stay_on_one_line_and_keep_repeated_spaces() -> Result<(), String> {
    let short = render(
        &source("input{width:80px}", "Input"),
        default_resources(),
        &options(),
    )?;
    let long = render(
        &source("input{width:80px}", "Input text must stay on a single line"),
        default_resources(),
        &options(),
    )?;
    assert_eq!(
        geometry(&short, "after")?["y"],
        geometry(&long, "after")?["y"]
    );
    assert_eq!(
        ink_bounds(&short).ok_or("short text")?[3],
        ink_bounds(&long).ok_or("long text")?[3]
    );
    let spaced = render(
        &source("", "Input    text"),
        default_resources(),
        &options(),
    )?;
    let collapsed = render(&source("", "Input text"), default_resources(), &options())?;
    assert!(
        ink_bounds(&spaced).ok_or("spaced text")?[2]
            > ink_bounds(&collapsed).ok_or("collapsed text")?[2] + 8
    );
    Ok(())
}

#[test]
fn input_alignment_and_baseline_survive_scale_changes() -> Result<(), String> {
    let html = source("input{text-align:right}", "Input text");
    let mut session = Session::new(&html, default_resources(), &options())?;
    let initial = session.frame(1.)?;
    let scaled = session.frame(2.)?;
    let fresh = render(
        &html,
        default_resources(),
        &RenderOptions {
            scale: 2.,
            ..options()
        },
    )?;
    assert_eq!(scaled.rgba, fresh.rgba);
    assert_eq!(session.frame(1.)?.rgba, initial.rgba);
    Ok(())
}
