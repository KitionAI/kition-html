use kition_html_engine::{RenderOptions, Rendered, default_resources, render};

fn frame(script: &str) -> Result<Rendered, String> {
    let html = format!(
        "<!doctype html><style>body{{margin:0}}canvas{{display:block}}</style><canvas id=canvas width=320 height=180></canvas><script>function check(value,message){{if(!value)throw Error(message)}}const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';{script}</script>"
    );
    let frame = render(
        &html,
        default_resources(),
        &RenderOptions {
            width: 320,
            height: 180,
            ..Default::default()
        },
    )?;
    assert!(frame.diagnostics.is_empty(), "{:?}", frame.diagnostics);
    Ok(frame)
}

#[test]
fn text_metrics_match_supplied_font_reference_and_ignore_drawing_transform() -> Result<(), String> {
    frame(
        r#"
        const m=ctx.measureText('Canvas text');
        check(m instanceof TextMetrics,'metrics interface');
        check(Math.abs(m.width-164.546875)<.001,'reference width');
        check(Math.abs(m.actualBoundingBoxAscent-23.65625)<.001,'reference ascent');
        check(m.fontBoundingBoxAscent===31&&m.fontBoundingBoxDescent===8,'font bounds');
        ctx.scale(2,3);check(ctx.measureText('Canvas text').width===m.width,'transform-independent width');
        check(ctx.measureText('').width===0,'empty string');
        check(ctx.measureText('A\nB\tC').width===ctx.measureText('A B C').width,'ASCII whitespace');
        check(ctx.measureText('A  B').width>ctx.measureText('A B').width,'spaces are preserved');
        check(ctx.measureText('A ').width>ctx.measureText('A').width&&ctx.measureText(' ').width>0,'trailing and isolated spaces');
    "#,
    )?;
    Ok(())
}

#[test]
fn text_state_restores_and_baseline_metrics_follow_the_anchor() -> Result<(), String> {
    frame(
        r#"
        ctx.save();ctx.font='700 48px Inter';ctx.textAlign='center';ctx.textBaseline='top';ctx.restore();
        check(ctx.font==='32px Inter'&&ctx.textAlign==='start'&&ctx.textBaseline==='alphabetic','restored state');
        ctx.font='not a font';check(ctx.font==='32px Inter','invalid font ignored');
        ctx.textBaseline='top';check(Math.abs(ctx.measureText('Canvas text').alphabeticBaseline+25.625)<.001,'em top');
        ctx.textBaseline='middle';check(Math.abs(ctx.measureText('Canvas text').alphabeticBaseline+9.625)<.001,'em middle');
        ctx.textBaseline='bottom';check(Math.abs(ctx.measureText('Canvas text').alphabeticBaseline-6.375)<.001,'em bottom');
        canvas.width=canvas.width;check(ctx.font==='10px sans-serif'&&ctx.textBaseline==='alphabetic','resize resets text state');
    "#,
    )?;
    Ok(())
}

fn ink_bounds(frame: &Rendered) -> Option<(usize, usize, usize, usize)> {
    let mut bounds: Option<(usize, usize, usize, usize)> = None;
    for (index, pixel) in frame.rgba.as_chunks::<4>().0.iter().enumerate() {
        if pixel[..3].iter().any(|&channel| channel < 200) {
            let (x, y) = (index % frame.width as usize, index / frame.width as usize);
            bounds = Some(match bounds {
                None => (x, y, x, y),
                Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
            });
        }
    }
    bounds
}

#[test]
fn fill_stroke_and_max_width_paint_text_without_changing_the_current_path() -> Result<(), String> {
    let fill = frame("ctx.fillStyle='#5645d4';ctx.fillText('Canvas text',20,60);")?;
    let bounds = ink_bounds(&fill).ok_or("text was not painted")?;
    assert!(
        bounds.0 >= 20 && bounds.2 < 190 && bounds.1 >= 35 && bounds.3 <= 61,
        "{bounds:?}"
    );
    let narrow = frame("ctx.fillStyle='#5645d4';ctx.fillText('Canvas text',20,60,80);")?;
    let bounds = ink_bounds(&narrow).ok_or("narrow text was not painted")?;
    assert!(bounds.2 < 102, "{bounds:?}");
    let stroke =
        frame("ctx.strokeStyle='#5645d4';ctx.lineWidth=1;ctx.strokeText('Canvas text',20,60);")?;
    assert!(ink_bounds(&stroke).is_some());
    assert_ne!(fill.rgba, stroke.rgba);
    frame(
        "ctx.beginPath();ctx.rect(0,0,8,8);ctx.fillText('Canvas text',20,60);check(ctx.isPointInPath(4,4)&&!ctx.isPointInPath(24,48),'text leaves current path unchanged');",
    )?;
    Ok(())
}

#[test]
fn text_alignment_and_direction_move_the_same_shaped_run() -> Result<(), String> {
    let left = frame("ctx.fillText('Canvas text',20,60);")?;
    let center = frame(
        "ctx.textAlign='center';ctx.fillText('Canvas text',20+ctx.measureText('Canvas text').width/2,60);",
    )?;
    let right = frame(
        "ctx.textAlign='right';ctx.fillText('Canvas text',20+ctx.measureText('Canvas text').width,60);",
    )?;
    let rtl = frame(
        "ctx.direction='rtl';ctx.fillText('Canvas text',20+ctx.measureText('Canvas text').width,60);",
    )?;
    assert_eq!(left.rgba, center.rgba);
    assert_eq!(left.rgba, right.rgba);
    assert_eq!(left.rgba, rtl.rgba);
    Ok(())
}
