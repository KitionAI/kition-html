use kition_html_engine::{RenderOptions, Rendered, default_resources, render};

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
fn spacing_dimensions_reproduce_the_recorded_chrome_grammar() -> Result<(), String> {
    let grammar = include_str!("../../../conformance/canvas-spacing-grammar.json");
    frame(&format!(
        "const grammar={grammar};const differences=[];for(const prop of ['letterSpacing','wordSpacing'])for(const [input,expected]of Object.entries(grammar)){{ctx[prop]='7px';ctx[prop]=input;if(ctx[prop]!==expected)differences.push([prop,input,ctx[prop],expected])}}if(differences.length)throw Error(JSON.stringify(differences));"
    ))?;
    Ok(())
}

#[test]
fn spacing_advances_cover_spaces_ligatures_and_combining_marks() -> Result<(), String> {
    let metrics = include_str!("../../../conformance/canvas-spacing-metrics.json");
    frame(&format!(
        "const cases={metrics};const differences=[];for(const sample of cases){{const family=sample.family==='JetBrainsMono'?'JetBrains Mono':sample.family;ctx.font='32px \"'+family+'\"';ctx.letterSpacing=sample.letter+'px';ctx.wordSpacing=sample.word+'px';const width=ctx.measureText(sample.text??String.fromCodePoint(...sample.codePoints)).width;if(Math.abs(width-sample.width)>.001)differences.push([family,sample.text??sample.codePoints,sample.letter,sample.word,width,sample.width])}}if(differences.length)throw Error(JSON.stringify(differences));"
    ))?;
    Ok(())
}

#[test]
fn spacing_state_matches_chrome() -> Result<(), String> {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../conformance/canvas-spacing-cases.json"
    ))
    .map_err(|error| error.to_string())?;
    for case in cases {
        let script = case["script"].as_str().ok_or("missing script")?;
        let id = case["id"].as_str().ok_or("missing id")?;
        frame(&format!(
            "if(!(()=>{{{script}}})())throw Error('spacing state: {id}');"
        ))?;
    }
    Ok(())
}

#[test]
fn fill_and_stroke_position_spaced_glyphs_like_separate_origins() -> Result<(), String> {
    for operation in ["fillText", "strokeText"] {
        for spacing in [4, -4] {
            let spaced = frame(&format!(
                "ctx.fontKerning='none';ctx.letterSpacing='{spacing}px';ctx.{operation}('AB',20,64);"
            ))?;
            let separate = frame(&format!(
                "ctx.{operation}('A',20,64);ctx.{operation}('B',20+ctx.measureText('A').width+{spacing},64);"
            ))?;
            assert!(
                spaced
                    .rgba
                    .iter()
                    .zip(&separate.rgba)
                    .all(|(a, b)| a.abs_diff(*b) <= 1),
                "{operation} spacing {spacing}"
            );
        }
    }
    Ok(())
}

#[test]
fn spacing_alignment_and_max_width_share_measurement() -> Result<(), String> {
    let setup = "ctx.letterSpacing='2px';ctx.wordSpacing='4px';";
    for operation in ["fillText", "strokeText"] {
        let left = frame(&format!("{setup}ctx.{operation}('A B C',20,64,80);"))?;
        for (align, x) in [("center", 60), ("right", 100)] {
            let aligned = frame(&format!(
                "{setup}ctx.textAlign='{align}';ctx.{operation}('A B C',{x},64,80);"
            ))?;
            assert!(
                left.rgba
                    .iter()
                    .zip(&aligned.rgba)
                    .all(|(a, b)| a.abs_diff(*b) <= 1),
                "{operation} {align} maxWidth"
            );
        }
    }
    Ok(())
}
