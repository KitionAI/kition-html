use kition_html_engine::{RenderOptions, Resources, render};

fn pixels(commands: &str) -> Result<Vec<u8>, String> {
    let html = format!(
        "<!doctype html><style>body{{margin:0}}canvas{{display:block}}</style><canvas id=canvas width=320 height=200></canvas><script>const ctx=document.getElementById('canvas').getContext('2d');ctx.strokeStyle='#5645d4';ctx.lineWidth=8;{commands}</script>"
    );
    let frame = render(
        &html,
        Resources::new(),
        &RenderOptions {
            width: 320,
            height: 200,
            ..Default::default()
        },
    )?;
    assert!(frame.diagnostics.is_empty(), "{:?}", frame.diagnostics);
    Ok(frame.rgba)
}

#[test]
fn tangent_arc_matches_an_explicit_quarter_circle() -> Result<(), String> {
    let tangent =
        pixels("ctx.moveTo(20,20);ctx.arcTo(240,20,240,160,40);ctx.lineTo(240,160);ctx.stroke();")?;
    let explicit = pixels(
        "ctx.moveTo(20,20);ctx.lineTo(200,20);ctx.arc(200,60,40,-Math.PI/2,0);ctx.lineTo(240,160);ctx.stroke();",
    )?;
    assert_eq!(tangent, explicit);
    Ok(())
}

#[test]
fn tangent_arc_recovers_current_point_after_transform_changes() -> Result<(), String> {
    let plain =
        pixels("ctx.moveTo(20,20);ctx.arcTo(240,20,240,160,40);ctx.lineTo(240,160);ctx.stroke();")?;
    let translated = pixels(
        "ctx.moveTo(20,20);ctx.translate(40,20);ctx.arcTo(200,0,200,140,40);ctx.lineTo(200,140);ctx.stroke();",
    )?;
    let retained = pixels(
        "const path=new Path2D();path.moveTo(20,20);path.arcTo(240,20,240,160,40);path.lineTo(240,160);ctx.stroke(path);",
    )?;
    assert_eq!(plain, translated);
    assert_eq!(plain, retained);
    Ok(())
}

#[test]
fn zero_radius_and_collinear_points_produce_straight_segments() -> Result<(), String> {
    assert_eq!(
        pixels("ctx.moveTo(20,20);ctx.arcTo(240,20,240,160,0);ctx.lineTo(240,160);ctx.stroke();")?,
        pixels("ctx.moveTo(20,20);ctx.lineTo(240,20);ctx.lineTo(240,160);ctx.stroke();")?
    );
    assert_eq!(
        pixels("ctx.moveTo(20,20);ctx.arcTo(140,80,260,140,40);ctx.stroke();")?,
        pixels("ctx.moveTo(20,20);ctx.lineTo(140,80);ctx.stroke();")?
    );
    assert_eq!(
        pixels("ctx.arcTo(20,20,240,160,40);ctx.lineTo(240,160);ctx.stroke();")?,
        pixels("ctx.moveTo(20,20);ctx.lineTo(240,160);ctx.stroke();")?
    );
    Ok(())
}
