//! Regression: fonts a page supplies as resources take the generic
//! family their family name says, whatever the resource's path is called. A separate test binary
//! from `fonts.rs`, whose fonts are added for the whole process.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use kition_html_kit::HtmlDocument;
use kition_kit::KitDocument;

const W: u32 = 360;
const H: u32 = 80;

#[test]
fn resource_fonts_take_the_generic_family_of_their_name() {
    let lora = std::fs::read(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../conformance/assets/Lora-Variable.ttf"
    )))
    .unwrap();
    let draw = |family: &str| {
        let html = format!(
            "<!doctype html><style>body{{margin:0;background:#fff}}p{{margin:8px;font:40px {family}}}</style><p>Hamburgefonts</p>"
        );
        let mut doc = HtmlDocument::open("resources.html", html.as_bytes()).unwrap();
        doc.set_viewport(W, H, false).unwrap();
        doc.set_resource("/assets/face-1.ttf".into(), lora.clone())
            .unwrap();
        doc.render(0.0, 0.0, 1.0, W, H).unwrap()
    };
    let named = draw("Lora");
    assert!(named != draw("Inter"), "the Lora resource draws");
    assert!(draw("serif") == named, "serif is the Lora resource");
}
