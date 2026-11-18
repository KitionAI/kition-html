use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn glyph(css: &str, scale: f64) -> Result<Rendered, String> {
    text(css, "H", scale)
}

fn text(css: &str, content: &str, scale: f64) -> Result<Rendered, String> {
    let mut resources = Resources::new();
    resources.insert(
        "/assets/Mono.ttf".into(),
        include_bytes!("../../../conformance/assets/JetBrainsMono-Variable.ttf").to_vec(),
    );
    render(
        &format!(
            "<!doctype html><style>body{{margin:32px;font:64px/80px 'JetBrains Mono';{css}}}</style>{content}"
        ),
        resources,
        &RenderOptions {
            width: 160,
            height: 160,
            scale,
            ..Default::default()
        },
    )
}

// Compare the ink centroids of the two vertical stem ends. This tests the
// visible slant without relying on a particular antialiasing implementation.
fn slant(image: &Rendered) -> f64 {
    let ink = |x: u32, y: u32| f64::from(255 - image.rgba[((y * image.width + x) * 4) as usize]);
    let rows: Vec<_> = (0..image.height)
        .filter(|&y| (0..image.width).any(|x| ink(x, y) > 64.0))
        .collect();
    assert!(rows.len() > 20);
    let third = rows.len() / 3;
    let center = |ys: &[u32]| {
        let mut weight = 0.0;
        let mut position = 0.0;
        for &y in ys {
            for x in 0..image.width {
                let value = ink(x, y);
                weight += value;
                position += value * f64::from(x);
            }
        }
        position / weight
    };
    center(&rows[..third]) - center(&rows[rows.len() - third..])
}

#[test]
fn synthetic_italic_and_signed_oblique_lean_in_css_direction() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let upright = glyph("font-style:normal", scale)?;
        let italic = glyph("font-style:italic", scale)?;
        let positive = glyph("font-style:oblique 14deg", scale)?;
        let negative = glyph("font-style:oblique -14deg", scale)?;
        assert!(slant(&upright).abs() < 0.2);
        assert!(slant(&italic) > 5.0 * scale);
        assert!(slant(&positive) > 5.0 * scale);
        assert!(slant(&negative) < -5.0 * scale);
        assert!(italic.rgba == positive.rgba);
    }
    Ok(())
}

#[test]
fn font_synthesis_none_suppresses_faux_italic() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let upright = glyph("font-style:normal", scale)?;
        for css in [
            "font-style:italic;font-synthesis:none",
            "font-style:oblique -14deg;font-synthesis-style:none",
        ] {
            let disabled = glyph(css, scale)?;
            assert!(
                disabled.rgba == upright.rgba,
                "{css}: slant {} vs {}",
                slant(&disabled),
                slant(&upright)
            );
        }
    }
    Ok(())
}

#[test]
fn synthesis_suppression_is_scoped_to_the_declaring_span() -> Result<(), String> {
    let actual = text(
        "font-style:italic",
        "H<span style='font-synthesis-style:none'>H</span>H",
        1.0,
    )?;
    let expected = text(
        "font-style:italic",
        "H<span style='font-style:normal'>H</span>H",
        1.0,
    )?;
    assert!(actual.rgba == expected.rgba);
    Ok(())
}
