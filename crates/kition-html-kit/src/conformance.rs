//! The KHP conformance fixtures beside the renderer's own cases in `conformance/`: `basic.html`
//! (no diagnostics), `edit.json` (steps that change it), one `check-<code>.html` per catalogue code,
//! `golden/` previews of `basic`, and the check catalogue
//! (`agent/checks.json`, from [`crate::check::CATALOGUE`]).
//!
//! `KITION_WRITE_CONFORMANCE=1 cargo test -p kition-html-kit conformance` writes them; without the
//! variable the test only checks that every fixture reports its code.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use kition_kit::KitDocument;
use serde_json::{Value, json};

use crate::HtmlDocument;
use crate::check::{CATALOGUE, catalogue_json};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const BASIC: &str = "<!doctype html>\n<html><head><style>body{margin:24px;font-family:sans-serif;background:#fff}h1{color:#5645d4}.card{border:1px solid #ddd;border-radius:12px;padding:16px}</style></head>\n<body><h1>Spring concert</h1><div class=\"card\"><p>Saturday 8 pm, Town Hall</p></div></body></html>\n";

fn fixture(code: &str) -> String {
    let extra = match code {
        "html.resource-unavailable" => {
            r#"<img src="images/poster.png" alt="Poster" width="120" height="80">"#
        }
        "html.javascript-error" => {
            r#"<script>document.querySelector("h1").textContent = missingTitle;</script>"#
        }
        "html.unsupported-filter" => {
            r#"<div style="width:80px;height:40px;background:#5645d4;filter:drop-shadow(2px 2px 2px black) hue-rotate(20deg)"></div>"#
        }
        "html.unsupported-backdrop-filter" => {
            r#"<div style="width:80px;height:40px;backdrop-filter:blur(4px)"></div>"#
        }
        other => panic!("no fixture for {other}"),
    };
    BASIC.replace("</body>", &format!("{extra}</body>"))
}

fn edit() -> Value {
    json!({ "steps": [
        { "command": "html.setSource", "params": { "source": BASIC.replace("Saturday 8 pm", "Sunday 6 pm") } },
    ] })
}

fn png(doc: &HtmlDocument, scale: f64) -> Vec<u8> {
    let (width, height) = (
        (f64::from(doc.width()) * scale) as u32,
        (f64::from(doc.height()) * scale) as u32,
    );
    let rgba = doc.render(0.0, 0.0, scale, width, height).expect("render");
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .expect("png header")
            .write_image_data(&rgba)
            .expect("png data");
    }
    bytes
}

fn codes(doc: &HtmlDocument) -> Vec<String> {
    doc.check()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn every_fixture_reports_its_code_and_basic_is_clean() {
    let write = std::env::var_os("KITION_WRITE_CONFORMANCE").is_some();
    let out = repo().join("conformance");
    let basic = HtmlDocument::open("basic.html", BASIC.as_bytes()).expect("basic");
    assert_eq!(codes(&basic), Vec::<String>::new(), "basic.html is clean");
    if write {
        std::fs::create_dir_all(out.join("golden")).expect("conformance/golden");
        std::fs::create_dir_all(repo().join("agent")).expect("agent");
        std::fs::write(out.join("basic.html"), BASIC).expect("basic");
        std::fs::write(out.join("edit.json"), format!("{:#}\n", edit())).expect("edit.json");
        std::fs::write(out.join("golden/basic.png"), png(&basic, 1.0)).expect("golden");
        std::fs::write(out.join("golden/basic-scale-2.png"), png(&basic, 2.0))
            .expect("golden scale 2");
        let tolerance = json!({
            "maxChannelDelta": 2,
            "maxDifferentPixels": 1500,
            "note": "The kit's own renderer draws the preview; glyph edges may differ by a step between platforms and fonts."
        });
        std::fs::write(
            out.join("golden/tolerance.json"),
            format!("{tolerance:#}\n"),
        )
        .expect("tolerance");
        std::fs::write(
            repo().join("agent/checks.json"),
            format!("{:#}\n", catalogue_json()),
        )
        .expect("checks.json");
    }
    for entry in CATALOGUE {
        let source = fixture(entry.code);
        let doc = HtmlDocument::open("fixture.html", source.as_bytes()).expect("open");
        assert!(
            codes(&doc).iter().any(|code| code == entry.code),
            "{}: {:?}",
            entry.code,
            codes(&doc)
        );
        if write {
            std::fs::write(
                out.join(format!(
                    "check-{}.html",
                    entry.code.trim_start_matches("html.")
                )),
                &source,
            )
            .expect("fixture");
        }
    }
}
