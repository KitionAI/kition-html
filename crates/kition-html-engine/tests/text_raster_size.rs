use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn resources() -> Resources {
    Resources::from([
        (
            "/Lora.ttf".into(),
            include_bytes!("../../../conformance/assets/Lora-Variable.ttf").to_vec(),
        ),
        (
            "/JetBrainsMono.ttf".into(),
            include_bytes!("../../../conformance/assets/JetBrainsMono-Variable.ttf").to_vec(),
        ),
    ])
}

fn canvas(family: &str, size: f64, scale: f64) -> Result<Rendered, String> {
    let html = format!(
        "<!doctype html><style>body{{margin:0}}canvas{{display:block}}</style><canvas id=c width=384 height=192></canvas><script>const ctx=document.getElementById('c').getContext('2d');ctx.font=\"{size}px '{family}'\";ctx.scale({scale},{scale});ctx.fillText('H',48/{scale},96/{scale});</script>"
    );
    checked(&html, 1.0)
}

fn checked(html: &str, scale: f64) -> Result<Rendered, String> {
    let result = render(
        html,
        resources(),
        &RenderOptions {
            width: (384.0 / scale) as u32,
            height: (192.0 / scale) as u32,
            scale,
            ..Default::default()
        },
    )?;
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert!(
        result.rgba.as_chunks::<4>().0.iter().any(|p| p[0] < 128),
        "missing glyph"
    );
    Ok(result)
}

fn equivalent(a: &Rendered, b: &Rendered, context: &str) {
    assert_eq!((a.width, a.height), (b.width, b.height));
    let changed = a.rgba.iter().zip(&b.rgba).filter(|(a, b)| a != b).count();
    let max = a
        .rgba
        .iter()
        .zip(&b.rgba)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    assert!(
        max <= 1,
        "{context}: {changed} differing channels, max delta {max}"
    );
}

#[test]
fn canvas_scaling_uses_the_final_raster_font_size() -> Result<(), String> {
    // These supplied fonts have no optical-size axis: changing size cannot change outlines.
    for family in ["Lora", "JetBrains Mono"] {
        for size in [12.0, 16.0, 24.0, 32.0] {
            for scale in [0.75, 1.25, 2.0] {
                equivalent(
                    &canvas(family, size, scale)?,
                    &canvas(family, size * scale, 1.0)?,
                    &format!("{family} {size}px scale {scale}"),
                );
            }
        }
    }
    Ok(())
}

#[test]
fn dom_device_scale_uses_the_same_raster_size_as_canvas() -> Result<(), String> {
    for family in ["Lora", "JetBrains Mono"] {
        for size in [12.0, 16.0, 24.0, 32.0] {
            for scale in [1.0, 1.5, 2.0] {
                let html = format!(
                    "<!doctype html><style>body{{margin:0}}div{{position:absolute;left:{}px;top:0;font:{size}px/48px '{family}'}}i{{display:inline-block;width:0;height:0}}</style><div id=line>H<i id=baseline></i></div><script>const line=document.getElementById('line'),marker=document.getElementById('baseline');line.style.top=({}-marker.getBoundingClientRect().y)+'px';</script>",
                    48.0 / scale,
                    96.0 / scale
                );
                equivalent(
                    &checked(&html, scale)?,
                    &canvas(family, size * scale, 1.0)?,
                    &format!("{family} {size}px DPR {scale}"),
                );
            }
        }
    }
    Ok(())
}

#[test]
fn zero_size_and_collapsed_transforms_do_not_leave_glyph_pixels() -> Result<(), String> {
    for declaration in [
        "font-size:0;transform:scale(2)",
        "font-size:32px;transform:scale(0)",
    ] {
        let frame = render(
            &format!(
                "<!doctype html><style>body{{margin:0}}div{{font-family:Lora;{declaration}}}</style><div>H</div>"
            ),
            resources(),
            &RenderOptions {
                width: 64,
                height: 64,
                ..Default::default()
            },
        )?;
        assert!(frame.diagnostics.is_empty(), "{:?}", frame.diagnostics);
        assert!(
            frame
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| *pixel == [255, 255, 255, 255]),
            "collapsed glyph left ink: {declaration}"
        );
    }
    Ok(())
}
