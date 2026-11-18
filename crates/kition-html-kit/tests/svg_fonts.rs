//! Regression: SVG `<text>` draws with the page's fonts (the bundled
//! Inter, the page's font resources and the fonts the host added), never the machine's, and
//! follows `font-family` and its generic families as HTML text does. A test binary of its own:
//! added fonts are the process's.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use kition_html_kit::{HtmlDocument, add_font};
use kition_kit::KitDocument;

const W: u32 = 360;
const H: u32 = 80;

fn svg(family: &str) -> Vec<u8> {
    let html = format!(
        "<!doctype html><style>body{{margin:0;background:#fff}}</style><svg width=\"{W}\" height=\"{H}\" viewBox=\"0 0 {W} {H}\"><text x=\"8\" y=\"56\" font-size=\"40\" font-family=\"{family}\" fill=\"#000\">Hamburgefonts</text></svg>"
    );
    let mut doc = HtmlDocument::open("svg.html", html.as_bytes()).unwrap();
    doc.set_viewport(W, H, false).unwrap();
    doc.render(0.0, 0.0, 1.0, W, H).unwrap()
}

fn inked(rgba: &[u8]) -> usize {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[0] < 128)
        .count()
}

#[test]
fn svg_text_draws_with_the_pages_fonts_only() {
    let inter = svg("Inter");
    assert!(inked(&inter) > 300, "the text draws");
    // Families the page does not have fall back to its own faces, whatever the machine has.
    for family in [
        "Helvetica",
        "Arial",
        "Times New Roman",
        "serif",
        "monospace",
    ] {
        assert!(
            svg(family) == inter,
            "{family} draws with Inter, the page's only face"
        );
    }
    let lora = std::fs::read(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../conformance/assets/Lora-Variable.ttf"
    )))
    .unwrap();
    assert_eq!(add_font(&lora), 1);
    let named = svg("Lora");
    assert!(named != inter, "an added face draws by its name");
    assert!(
        svg("serif") == named,
        "serif is the serif face the host added"
    );
    assert!(
        svg("Helvetica, sans-serif") == inter,
        "sans-serif stays Inter"
    );
}
