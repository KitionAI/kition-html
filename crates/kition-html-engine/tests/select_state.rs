use kition_html_engine::{RenderOptions, Session, default_resources, render};
use serde_json::Value;

#[test]
fn option_state_transitions_match_recorded_chrome_traces() -> Result<(), String> {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../../../conformance/select-state-cases.json"))
            .map_err(|error| error.to_string())?;
    for case in cases {
        let expected =
            serde_json::to_string(&case["expected"]).map_err(|error| error.to_string())?;
        // Compare parsed objects rather than object-key serialization order.
        let html = format!(
            "<!doctype html>{}<script>{};const expected={};if(trace.length!==expected.length||trace.some((step,i)=>Object.keys(expected[i]).length!==Object.keys(step).length||Object.keys(step).some(key=>JSON.stringify(step[key])!==JSON.stringify(expected[i][key]))))throw Error('Unexpected trace: '+JSON.stringify(trace));</script>",
            case["body"].as_str().ok_or("Missing scenario body")?,
            case["script"].as_str().ok_or("Missing scenario script")?,
            expected,
        );
        let image = render(&html, default_resources(), &RenderOptions::default())?;
        assert!(
            image.diagnostics.is_empty(),
            "{}: {:?}",
            case["id"],
            image.diagnostics
        );
    }
    Ok(())
}

#[test]
fn selection_repaints_after_live_moves_without_rewriting_source_defaults() -> Result<(), String> {
    let prefix = "<!doctype html><style>body{margin:0}select{font:14px Inter}</style>";
    let options = "<option>English</option><option>French</option>";
    let html = format!(
        "{prefix}<select id=s>{options}</select><script>setTimeout(()=>{{const s=document.getElementById('s'),g=document.createElement('optgroup');while(s.firstChild)g.appendChild(s.firstChild);s.appendChild(g);if(s.value!=='French'||s.querySelectorAll('[selected]').length)throw Error('Selection or source default mismatch')}},100)</script>"
    );
    let expected = format!(
        "{prefix}<select><optgroup><option>English</option><option selected>French</option></optgroup></select>"
    );
    let mut session = Session::new(&html, default_resources(), &RenderOptions::default())?;
    session.advance(100)?;
    for scale in [1., 1.5, 2., 1.] {
        let actual = session.frame(scale)?;
        assert!(actual.diagnostics.is_empty(), "{:?}", actual.diagnostics);
        let expected = render(
            &expected,
            default_resources(),
            &RenderOptions {
                scale,
                ..Default::default()
            },
        )?;
        assert_eq!(actual.rgba, expected.rgba, "scale {scale}");
    }
    Ok(())
}

#[test]
fn shallow_and_deep_option_clones_start_from_source_defaults() -> Result<(), String> {
    let html = "<!doctype html><select id=s><option id=a>A</option><option id=b selected>B</option></select><script>const s=document.getElementById('s'),a=document.getElementById('a'),b=document.getElementById('b');a.selected=true;for(const deep of [false,true]){const ac=a.cloneNode(deep),bc=b.cloneNode(deep);if(ac.selected||!bc.selected)throw Error('Clone copied live selectedness');bc.defaultSelected=false;if(bc.selected)throw Error('Clone copied dirtiness')}const copy=s.cloneNode(true);if(copy.value!=='B'||copy.selectedIndex!==1)throw Error('Select clone did not reset defaults')</script>";
    let image = render(html, default_resources(), &RenderOptions::default())?;
    assert!(image.diagnostics.is_empty(), "{:?}", image.diagnostics);
    Ok(())
}

#[test]
fn checked_option_state_invalidates_ancestor_styles() -> Result<(), String> {
    let html = "<!doctype html><style>html,body{margin:0;width:100%;height:100%;background:white}select{display:none}body:has(option:checked){background:#5645d4}</style><select id=s><option>A</option><option>B</option></select><script>setTimeout(()=>document.getElementById('s').selectedIndex=-1,100);setTimeout(()=>document.getElementById('s').firstChild.defaultSelected=true,200)</script>";
    let mut session = Session::new(html, default_resources(), &RenderOptions::default())?;
    assert_eq!(&session.frame(1.)?.rgba[..4], &[86, 69, 212, 255]);
    session.advance(100)?;
    assert_eq!(&session.frame(1.)?.rgba[..4], &[255; 4]);
    session.advance(100)?;
    assert_eq!(&session.frame(1.)?.rgba[..4], &[86, 69, 212, 255]);
    Ok(())
}
