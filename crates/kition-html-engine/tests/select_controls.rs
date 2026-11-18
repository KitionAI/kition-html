use kition_html_engine::{RenderOptions, Rendered, Session, default_resources, render};

fn source(options: &str, script: &str) -> String {
    format!(
        "<!doctype html><style>body{{margin:0}}select{{display:block;font:14px Inter;width:200px;padding:10px;border:1px solid #c8c4be;border-radius:8px;color:#1a1a1a}}</style><main id=stage><form id=form><select id=choice>{options}</select></form></main><script>{script}</script>"
    )
}
fn options() -> RenderOptions {
    RenderOptions {
        width: 240,
        height: 80,
        ..Default::default()
    }
}
fn frame(options: &str, script: &str) -> Result<Rendered, String> {
    let frame = render(
        &source(options, script),
        default_resources(),
        &self::options(),
    )?;
    assert!(frame.diagnostics.is_empty(), "{:?}", frame.diagnostics);
    Ok(frame)
}
const CHOICES: &str = "<option value=en>English</option><option value=fr>French</option>";

#[test]
fn collapsed_select_uses_font_metrics_and_paints_selected_label() -> Result<(), String> {
    let initial = frame(CHOICES, "")?;
    let select = initial
        .geometry
        .iter()
        .find(|entry| entry["id"] == "choice")
        .ok_or("missing select geometry")?;
    assert_eq!(select["width"], 200.);
    assert_eq!(select["height"], 41.);
    let blank = frame(
        CHOICES,
        "document.getElementById('choice').selectedIndex=-1",
    )?;
    assert_ne!(initial.rgba, blank.rgba, "selected text must paint");
    let selected = frame(CHOICES, "document.getElementById('choice').value='fr'")?;
    assert_ne!(initial.rgba, selected.rgba);
    assert_eq!(
        selected.rgba,
        frame(
            "<option>English</option><option selected>French</option>",
            ""
        )?
        .rgba
    );
    assert_eq!(
        frame("<option label=French>Stored value</option>", "")?.rgba,
        selected.rgba
    );
    Ok(())
}

#[test]
fn selection_and_option_mutations_repaint_an_existing_session() -> Result<(), String> {
    for (change, expected) in [
        (
            "document.getElementById('choice').value='fr'",
            frame(CHOICES, "document.getElementById('choice').value='fr'")?,
        ),
        (
            "document.getElementById('choice').children[0].textContent='Updated option'",
            frame("<option>Updated option</option><option>French</option>", "")?,
        ),
        (
            "document.getElementById('choice').children[0].setAttribute('label','Updated option')",
            frame("<option>Updated option</option><option>French</option>", "")?,
        ),
    ] {
        let mut session = Session::new(
            &source(CHOICES, &format!("setTimeout(()=>{{{change}}},100)")),
            default_resources(),
            &options(),
        )?;
        let before = session.frame(1.)?;
        session.advance(100)?;
        let after = session.frame(1.)?;
        assert!(after.diagnostics.is_empty(), "{:?}", after.diagnostics);
        assert_ne!(before.rgba, after.rgba);
        assert_eq!(expected.rgba, after.rgba);
    }
    Ok(())
}

#[test]
fn form_reset_restores_the_displayed_default_option() -> Result<(), String> {
    let script = "const choice=document.getElementById('choice');choice.value='fr';setTimeout(()=>document.getElementById('form').reset(),100)";
    let mut session = Session::new(&source(CHOICES, script), default_resources(), &options())?;
    assert_eq!(
        session.frame(1.)?.rgba,
        frame(CHOICES, "document.getElementById('choice').value='fr'")?.rgba
    );
    session.advance(100)?;
    assert_eq!(session.frame(1.)?.rgba, frame(CHOICES, "")?.rgba);
    Ok(())
}

#[test]
fn select_glyphs_rebuild_when_the_preview_scale_changes() -> Result<(), String> {
    let html = source(CHOICES, "");
    let mut session = Session::new(&html, default_resources(), &options())?;
    let first = session.frame(1.)?;
    let enlarged = session.frame(2.)?;
    let expected = render(
        &html,
        default_resources(),
        &RenderOptions {
            scale: 2.,
            ..options()
        },
    )?;
    assert_eq!(enlarged.rgba, expected.rgba);
    assert_eq!(session.frame(1.)?.rgba, first.rgba);
    Ok(())
}
