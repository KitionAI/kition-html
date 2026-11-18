use kition_html_engine::{RenderOptions, Rendered, default_resources, render};

fn frame(script: &str) -> Result<Rendered, String> {
    let html = format!(
        "<!doctype html><style>body{{margin:0}}canvas{{display:block}}</style><canvas id=c width=160 height=128></canvas><script>const ctx=document.getElementById('c').getContext('2d');ctx.font='16px Inter';{script}</script>"
    );
    let frame = render(
        &html,
        default_resources(),
        &RenderOptions {
            width: 160,
            height: 128,
            ..Default::default()
        },
    )?;
    assert!(frame.diagnostics.is_empty(), "{:?}", frame.diagnostics);
    assert!(
        frame.rgba.as_chunks::<4>().0.iter().any(|p| p[0] < 128),
        "glyph is absent"
    );
    Ok(frame)
}

fn same(a: &Rendered, b: &Rendered) -> bool {
    a.rgba.iter().zip(&b.rgba).all(|(a, b)| a.abs_diff(*b) <= 1)
}

#[test]
fn fill_and_stroke_origins_use_bitmap_phases_after_the_current_transform() -> Result<(), String> {
    for operation in ["fillText", "strokeText"] {
        for scale in [1.0, 1.5, 2.0] {
            let mut frames = Vec::new();
            for (x, y) in [
                (0., 0.),
                (0.0625, 0.25),
                (0.25, 0.),
                (0.3125, 0.25),
                (0., 0.625),
            ] {
                frames.push(frame(&format!(
                    "ctx.scale({scale},{scale});ctx.{operation}('H',{},{});",
                    (16. + x) / scale,
                    (80. + y) / scale
                ))?);
            }
            assert!(
                same(&frames[0], &frames[1]),
                "origin phase at scale {scale}"
            );
            assert!(
                same(&frames[2], &frames[3]),
                "quarter phase at scale {scale}"
            );
            assert!(
                !same(&frames[0], &frames[2]),
                "horizontal phase lost at scale {scale}"
            );
            assert!(
                !same(&frames[0], &frames[4]),
                "vertical pixel lost at scale {scale}"
            );
        }
    }
    Ok(())
}

#[test]
fn text_metrics_do_not_snap_to_bitmap_origins() -> Result<(), String> {
    frame(
        "const before=ctx.measureText('Hg');ctx.translate(.1875,.3125);ctx.scale(1.25,1.5);ctx.fillText('Hg',16,50,8);const after=ctx.measureText('Hg');if(before.width!==after.width||before.actualBoundingBoxRight!==after.actualBoundingBoxRight)throw Error('metrics changed with paint transform');",
    )?;
    Ok(())
}

#[test]
fn fill_alignment_and_max_width_share_the_same_bitmap_origin() -> Result<(), String> {
    let left = frame("ctx.fillText('Hamburg',16.1875,80.3125,48);")?;
    let centered = frame("ctx.textAlign='center';ctx.fillText('Hamburg',40.1875,80.3125,48);")?;
    let right = frame("ctx.textAlign='right';ctx.fillText('Hamburg',64.1875,80.3125,48);")?;
    assert!(
        same(&left, &centered) && same(&left, &right),
        "alignment must precede origin snapping"
    );
    Ok(())
}
