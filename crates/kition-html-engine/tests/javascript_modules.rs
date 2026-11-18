use kition_html_engine::{RenderOptions, Resources, render};

fn capture(
    map: &str,
    module: &str,
    resources: Resources,
) -> Result<kition_html_engine::Rendered, String> {
    let html = format!(
        r#"<!doctype html><style>body{{margin:0}}#target{{width:32px;height:32px;background:red}}</style><div id=target></div><script type=importmap>{map}</script><script type=module>{module}</script>"#
    );
    render(
        &html,
        resources,
        &RenderOptions {
            width: 64,
            height: 64,
            ..Default::default()
        },
    )
}

#[test]
fn import_maps_resolve_bare_prefix_scoped_and_url_keys() -> Result<(), String> {
    let resources = Resources::from([
        (
            "/assets/palette.js".into(),
            b"export const color='#5645d4'".to_vec(),
        ),
        (
            "/assets/red.js".into(),
            b"export const color='red'".to_vec(),
        ),
        (
            "/assets/main.js".into(),
            b"import {color} from 'palette';export {color};".to_vec(),
        ),
    ]);
    for (map, specifier) in [
        (r#"{"imports":{"palette":"/assets/palette.js"}}"#, "palette"),
        (r#"{"imports":{"colors/":"/assets/"}}"#, "colors/palette.js"),
        (
            r#"{"imports":{"./original.js":"/assets/palette.js"}}"#,
            "./original.js",
        ),
        (
            r#"{"imports":{"palette":"/assets/red.js"},"scopes":{"/assets/":{"palette":"/assets/palette.js"}}}"#,
            "/assets/main.js",
        ),
        (
            r#"{"imports":{"palette":"/assets/red.js"},"scopes":{"/cases/":{"palette":"/assets/palette.js"}}}"#,
            "palette",
        ),
    ] {
        let frame = capture(
            map,
            &format!(
                "import {{color}} from '{specifier}';document.getElementById('target').style.background=color"
            ),
            resources.clone(),
        )?;
        assert!(
            frame.diagnostics.is_empty(),
            "{map}: {:?}",
            frame.diagnostics
        );
        assert_eq!(&frame.rgba[..4], &[86, 69, 212, 255], "{map}");
    }
    Ok(())
}

#[test]
fn invalid_blocked_unmapped_and_escaping_imports_report_errors() -> Result<(), String> {
    for (map, specifier) in [
        ("{}", "missing"),
        (r#"{"imports":{"blocked":null}}"#, "blocked"),
        (
            r#"{"imports":{"colors/":"/assets/"}}"#,
            "colors/../escape.js",
        ),
        (r#"{"imports":[]}"#, "missing"),
        ("{invalid}", "missing"),
        (r#"{"imports":{"remote":"file:///etc/passwd"}}"#, "remote"),
    ] {
        let frame = capture(map, &format!("import '{specifier}'"), Resources::new())?;
        assert!(!frame.diagnostics.is_empty(), "{map} {specifier}");
        assert_eq!(&frame.rgba[..4], &[255, 0, 0, 255]);
    }
    Ok(())
}

#[test]
fn named_exports_accept_automatic_semicolon_insertion_at_end_of_module() -> Result<(), String> {
    for ending in [
        "export {color}",
        "export {color}\n",
        "export {color} // final export",
        "export {color} /* final export */",
    ] {
        let resources = Resources::from([(
            "/assets/palette.js".into(),
            format!("const color='#5645d4';{ending}").into_bytes(),
        )]);
        let frame = capture(
            "{}",
            "import {color} from '/assets/palette.js';document.getElementById('target').style.background=color",
            resources,
        )?;
        assert!(
            frame.diagnostics.is_empty(),
            "{ending}: {:?}",
            frame.diagnostics
        );
        assert_eq!(&frame.rgba[..4], &[86, 69, 212, 255]);
    }
    Ok(())
}
