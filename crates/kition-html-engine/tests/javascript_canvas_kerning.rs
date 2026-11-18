use kition_html_engine::{RenderOptions, Rendered, default_resources, render};
use serde_json::Value;

fn frame(script: &str) -> Result<Rendered, String> {
    let mut resources = default_resources();
    resources.insert(
        "/assets/Lora.ttf".into(),
        include_bytes!("../../../conformance/assets/Lora-Variable.ttf").to_vec(),
    );
    resources.insert(
        "/assets/JetBrainsMono.ttf".into(),
        include_bytes!("../../../conformance/assets/JetBrainsMono-Variable.ttf").to_vec(),
    );
    let html = format!(
        "<!doctype html><style>body{{margin:0}}canvas{{display:block}}</style><canvas id=canvas width=320 height=128></canvas><script>const canvas=document.getElementById('canvas'),ctx=canvas.getContext('2d');ctx.font='32px Inter';{script}</script>"
    );
    let image = render(
        &html,
        resources,
        &RenderOptions {
            width: 320,
            height: 128,
            ..Default::default()
        },
    )?;
    assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    Ok(image)
}

#[test]
fn kerning_state_matches_chrome_across_validation_and_context_resets() -> Result<(), String> {
    let cases: Vec<Value> = serde_json::from_str(include_str!(
        "../../../conformance/canvas-kerning-cases.json"
    ))
    .map_err(|e| e.to_string())?;
    for case in cases {
        let script = case["script"].as_str().ok_or("missing script")?;
        let id = case["id"].as_str().ok_or("missing id")?;
        frame(&format!(
            "if(!(()=>{{{script}}})())throw Error('kerning state: {id}');"
        ))?;
    }
    Ok(())
}

#[test]
fn kerning_metrics_stay_within_one_layout_unit_of_chrome() -> Result<(), String> {
    let cases = include_str!("../../../conformance/canvas-kerning-metrics.json");
    frame(&format!(
        r#"
        // Compare all profiles within one CSS layout unit; full-image acceptance is separate.
        const cases={cases};
        for(const sample of cases){{
            const family=sample.family==='JetBrainsMono'?'JetBrains Mono':sample.family;
            ctx.font=sample.size+'px "'+family+'"';
            for(const mode of ['auto','normal','none']){{
                ctx.fontKerning=mode;
                const width=ctx.measureText(sample.text).width;
                if(Math.abs(width-sample.widths[mode])>1/64)throw Error(family+' '+sample.size+' '+sample.text+' '+mode+': '+width+' expected '+sample.widths[mode]);
            }}
        }}
    "#
    ))?;
    Ok(())
}

#[test]
fn fill_and_stroke_use_the_same_unkerning_as_measurement() -> Result<(), String> {
    for operation in ["fillText", "strokeText"] {
        let unkerned = frame(&format!(
            "ctx.fontKerning='none';ctx.{operation}('AV',20,64);"
        ))?;
        let separate = frame(&format!(
            "ctx.{operation}('A',20,64);ctx.{operation}('V',20+ctx.measureText('A').width,64);"
        ))?;
        let normal = frame(&format!(
            "ctx.fontKerning='normal';ctx.{operation}('AV',20,64);"
        ))?;
        assert!(
            unkerned
                .rgba
                .iter()
                .zip(&separate.rgba)
                .all(|(a, b)| a.abs_diff(*b) <= 1),
            "{operation} must use unkerned glyph advances"
        );
        assert!(
            unkerned.rgba != normal.rgba,
            "{operation} must respond to the kerning mode"
        );
    }
    Ok(())
}

#[test]
fn alignment_and_max_width_use_the_selected_kerning_metrics() -> Result<(), String> {
    let left = frame("ctx.fontKerning='none';ctx.fillText('AVATAR',20,64,80);")?;
    for (alignment, x) in [("center", 60), ("right", 100)] {
        let aligned = frame(&format!(
            "ctx.fontKerning='none';ctx.textAlign='{alignment}';ctx.fillText('AVATAR',{x},64,80);"
        ))?;
        assert!(
            left.rgba
                .iter()
                .zip(&aligned.rgba)
                .all(|(a, b)| a.abs_diff(*b) <= 1),
            "{alignment} maxWidth alignment"
        );
    }
    frame(
        "ctx.fontKerning='none';const a=ctx.measureText('AVATAR');ctx.translate(.25,.5);ctx.scale(2,3);const b=ctx.measureText('AVATAR');if(a.width!==b.width||a.actualBoundingBoxRight!==b.actualBoundingBoxRight)throw Error('transform changed metrics');",
    )?;
    Ok(())
}
