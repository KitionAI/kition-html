//! Regression: horizontal margin, border and padding of inline
//! elements (and of `::before` and `::after`) take room on the line, and an inline element's
//! background covers its padding, as in a browser.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kition_html_kit::HtmlDocument;
use kition_kit::KitDocument;

const W: u32 = 400;
const H: u32 = 60;

fn render(body: &str) -> Vec<u8> {
    let html = format!(
        "<!doctype html><style>body{{margin:0;background:#fff;color:#000}}p{{margin:0;padding:8px;font:32px/44px Inter}}</style><p>{body}</p>"
    );
    let mut doc = HtmlDocument::open("inline.html", html.as_bytes()).unwrap();
    doc.set_viewport(W, H, false).unwrap();
    doc.render(0.0, 0.0, 1.0, W, H).unwrap()
}

/// The columns that hold a pixel matching `test`.
fn columns(rgba: &[u8], test: impl Fn(&[u8]) -> bool) -> Vec<u32> {
    let mut found: Vec<u32> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .filter(|(_, px)| test(px.as_slice()))
        .map(|(at, _)| at as u32 % W)
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

fn ink(px: &[u8]) -> bool {
    px[0] < 100 && px[1] < 100 && px[2] < 100
}

fn yellow(px: &[u8]) -> bool {
    px[0] > 240 && px[1] > 240 && px[2] < 40
}

/// Where the second glyph's ink starts: the first inked column after the first blank one.
fn second_glyph(rgba: &[u8]) -> u32 {
    let inked = columns(rgba, ink);
    inked
        .windows(2)
        .find(|pair| pair[1] > pair[0] + 1)
        .map(|pair| pair[1])
        .unwrap()
}

#[test]
fn inline_margin_and_padding_take_room() {
    // "I" first, then "B": B moves right by the span's left margin.
    let plain = render("I<span>B</span>");
    let spaced = render("I<span style=\"margin-left:30px\">B</span>");
    let gap = |rgba: &[u8]| i64::from(second_glyph(rgba));
    assert!(
        (gap(&spaced) - gap(&plain) - 30).abs() <= 1,
        "margin-left: {} then {}",
        gap(&plain),
        gap(&spaced)
    );

    // A border takes room only when it is drawn: a width without a style is none.
    let bordered = render("I<span style=\"border-left:5px solid #888\">B</span>");
    assert!(
        (gap(&bordered) - gap(&plain) - 5).abs() <= 1,
        "border-left: {} then {}",
        gap(&plain),
        gap(&bordered)
    );
    assert_eq!(
        gap(&render("I<span style=\"border-left-width:5px\">B</span>")),
        gap(&plain),
        "a border of style none is 0 wide"
    );

    // Padding widens the background by both sides.
    let tight = columns(&render("I<span style=\"background:#ff0\">B</span>"), yellow);
    let padded = columns(
        &render("I<span style=\"background:#ff0;padding:0 12px\">B</span>"),
        yellow,
    );
    assert!(!tight.is_empty());
    assert!(
        (padded.len() as i64 - tight.len() as i64 - 24).abs() <= 1,
        "padding: {} then {} yellow columns",
        tight.len(),
        padded.len()
    );

    // A ::before's right margin parts it from the text after it.
    let style = "<style>b::before{content:'I';font-weight:400}</style>";
    let near = render(&format!("{style}<b>B</b>"));
    let apart = render(&format!(
        "{style}<style>b::before{{margin-right:20px}}</style><b>B</b>"
    ));
    assert!(near != apart, "::before's margin moves what follows");
    let last = |rgba: &[u8]| *columns(rgba, ink).last().unwrap() as i64;
    assert!(
        (last(&apart) - last(&near) - 20).abs() <= 1,
        "::before margin-right: {} then {}",
        last(&near),
        last(&apart)
    );
}
