//! Regression: a host adds fonts with `add_font`, the CSS generic
//! families (`serif`, `monospace`, `emoji`) map to the faces it has, and Chinese and emoji draw
//! with the fonts it added instead of as boxes. Pages already open pick up fonts added later.
//!
//! Lora and JetBrains Mono come from `conformance/assets`; Droid Sans Fallback and Noto Color Emoji
//! from the kition-fonts checkout (`KITION_FONTS_DIR`, else the repository's sibling `kition-fonts`).
//! The whole regression is one test: added fonts are the process's, so steps must not interleave.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use kition_html_kit::{HtmlDocument, add_font};
use kition_kit::KitDocument;

fn repo() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn kition_fonts() -> PathBuf {
    std::env::var_os("KITION_FONTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo().join("../kition-fonts"))
        .join("fonts")
}

fn read(path: PathBuf) -> Vec<u8> {
    std::fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "{}: {error} (this regression needs the kition-fonts checkout; set KITION_FONTS_DIR)",
            path.display()
        )
    })
}

const W: u32 = 360;
const H: u32 = 80;

fn page(body: &str) -> HtmlDocument {
    let html = format!(
        "<!doctype html><style>body{{margin:0;background:#fff;color:#000}}p{{margin:8px;font-size:40px;line-height:56px}}</style><p {body}</p>"
    );
    let mut doc = HtmlDocument::open("fonts.html", html.as_bytes()).unwrap();
    doc.set_viewport(W, H, false).unwrap();
    doc
}

fn pixels(doc: &HtmlDocument) -> Vec<u8> {
    doc.render(0.0, 0.0, 1.0, W, H).unwrap()
}

fn render(body: &str) -> Vec<u8> {
    pixels(&page(body))
}

fn inked(rgba: &[u8]) -> usize {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[0] < 128)
        .count()
}

/// Whether two renders are the same, without printing whole buffers when they are not.
fn same(a: &[u8], b: &[u8]) -> bool {
    a == b
}

#[test]
fn added_fonts_serve_generic_families_chinese_and_emoji() {
    let assets = repo().join("conformance/assets");
    let lora = read(assets.join("Lora-Variable.ttf"));
    let mono = read(assets.join("JetBrainsMono-Variable.ttf"));
    let cjk = read(kition_fonts().join("droid-sans-fallback/DroidSansFallbackFull.ttf"));
    let emoji = read(kition_fonts().join("noto-color-emoji/Noto-COLRv1.ttf"));

    // A page open before the fonts arrive.
    let open_early = page("style=\"font-family:Lora\">Hamburgefonts");
    let before = pixels(&open_early);
    let inter = render("style=\"font-family:Inter\">Hamburgefonts");
    assert!(
        same(
            &before,
            &render("style=\"font-family:'No Such Face'\">Hamburgefonts")
        ),
        "without Lora, the page draws with the fallback face, as for any family nobody added"
    );

    assert_eq!(
        add_font(b"not a font"),
        0,
        "bytes that are not a font add nothing"
    );
    assert_eq!(add_font(&lora), 1);
    assert_eq!(add_font(&mono), 1);
    assert_eq!(add_font(&lora), 0, "a font added twice adds nothing");
    assert_eq!(add_font(&cjk), 1);
    assert_eq!(add_font(&emoji), 1);

    let named = render("style=\"font-family:Lora\">Hamburgefonts");
    assert!(
        !same(&named, &inter) && !same(&named, &before),
        "the added Lora draws"
    );
    assert!(
        same(&pixels(&open_early), &named),
        "a page already open draws with fonts added later"
    );
    assert!(
        same(
            &render("style=\"font-family:Georgia, serif\">Hamburgefonts"),
            &named
        ),
        "serif is the serif face the host added"
    );
    let jetbrains = render("style=\"font-family:'JetBrains Mono'\">Hamburgefonts");
    assert!(!same(&jetbrains, &inter), "the added JetBrains Mono draws");
    assert!(
        same(
            &render("style=\"font-family:Menlo, monospace\">Hamburgefonts"),
            &jetbrains
        ),
        "monospace is the monospace face the host added"
    );
    assert!(
        same(
            &render("style=\"font-family:sans-serif\">Hamburgefonts"),
            &inter
        ),
        "sans-serif stays Inter"
    );

    // Chinese: distinct characters draw distinct glyphs (missing ones all draw the same box), in
    // any family, from the added Chinese face.
    let zhongwen = render(">中文排版");
    assert!(inked(&zhongwen) > 200, "Chinese draws");
    assert!(
        !same(&zhongwen, &render(">中中中中")),
        "Chinese draws as boxes"
    );
    assert!(
        !same(
            &render("lang=\"zh-CN\" style=\"font-family:serif\">中文排版"),
            &render("lang=\"zh-CN\" style=\"font-family:serif\">中中中中")
        ),
        "Chinese in a serif stack draws as boxes"
    );

    // Emoji: in color, from the emoji face.
    let smile = render(">😀 🎉");
    assert!(!same(&smile, &render(">😎 🎉")), "emoji draw as boxes");
    let colored = smile
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[0] > 200 && px[1] > 150 && px[2] < 100)
        .count();
    assert!(
        colored > 100,
        "emoji draw in color ({colored} yellow pixels)"
    );
}
