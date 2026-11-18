//! How large a page the kit opens: real web pages of a few megabytes open, edit and save; a page
//! over the limit is refused with the `too-large` reason (and the limit), never as a damaged file.
use super::*;

const LIMIT: usize = 8 * 1024 * 1024;

/// A valid page of exactly `len` bytes: markup around a long paragraph.
fn page(len: usize) -> Vec<u8> {
    let head = "<!doctype html>\n<html lang=\"en\"><head><title>Big page</title></head>\n<body><p>";
    let tail = "</p></body></html>\n";
    let mut out = String::with_capacity(len);
    out.push_str(head);
    while out.len() + tail.len() < len {
        out.push_str("Lorem ipsum dolor sit amet. ");
    }
    out.truncate(len - tail.len());
    out.push_str(tail);
    assert_eq!(out.len(), len);
    out.into_bytes()
}

fn assert_too_large(error: &str) {
    assert!(error.starts_with("too-large:"), "{error}");
    assert!(error.contains("at most 8 MB"), "{error}");
}

#[test]
fn a_page_at_the_limit_opens_edits_and_saves() -> Result<(), String> {
    let bytes = page(LIMIT);
    let mut doc = HtmlDocument::open("big.html", &bytes)?;
    assert_eq!(doc.save(None)?, bytes);
    // A new title, and one sentence fewer so the page stays within the limit.
    let edited = String::from_utf8(bytes)
        .map_err(|e| e.to_string())?
        .replacen("Big page", "Big page, edited", 1)
        .replacen("Lorem ipsum dolor sit amet. ", "", 1);
    doc.execute("html.setSource", json!({ "source": edited }))?;
    assert_eq!(doc.save(None)?, edited.into_bytes());
    Ok(())
}

#[test]
fn a_page_over_the_limit_is_refused_as_too_large() {
    let bytes = page(LIMIT + 1);
    let error = HtmlDocument::open("big.html", &bytes)
        .err()
        .unwrap_or_default();
    assert_too_large(&error);
}

#[test]
fn an_edit_over_the_limit_is_refused_as_too_large() -> Result<(), String> {
    let mut doc = HtmlDocument::open("small.html", &page(1024))?;
    let source = String::from_utf8(page(LIMIT + 1)).map_err(|e| e.to_string())?;
    let error = doc
        .execute("html.setSource", json!({ "source": source }))
        .err()
        .unwrap_or_default();
    assert_too_large(&error);
    Ok(())
}
