//! Editing a page end to end: a new page, a sequence of `html.setSource` edits with undo and
//! redo, saved to `.html`, reopened from those bytes, and compared with what was edited.
use super::*;

const TITLE: &str = "<!doctype html>\n<html lang=\"en\"><head><title>Spring fair</title></head>\n<body><h1>Spring fair</h1></body></html>\n";
const SECTION: &str = "<!doctype html>\n<html lang=\"en\"><head><title>Spring fair</title>\n<style>h1 { color: #5645d4; } .note { font-style: italic; }</style></head>\n<body><h1>Spring fair</h1>\n<p class=\"note\">Saturday 10:00\u{2013}16:00 \u{b7} \u{6625}\u{5b63}\u{5e02}\u{96c6} \u{1f338}</p></body></html>\n";
const LIST: &str = "<!doctype html>\n<html lang=\"en\"><head><title>Spring fair</title>\n<style>h1 { color: #5645d4; } .note { font-style: italic; }</style></head>\n<body><h1>Spring fair</h1>\n<p class=\"note\">Saturday 10:00\u{2013}16:00 \u{b7} \u{6625}\u{5b63}\u{5e02}\u{96c6} \u{1f338}</p>\n<ul><li>Plants</li><li>Bread &amp; cheese</li></ul></body></html>\n";

fn source(doc: &HtmlDocument) -> Result<String, String> {
    String::from_utf8(doc.save(None)?).map_err(|error| error.to_string())
}

/// The described page without the history flags, which a file does not keep.
fn described(doc: &HtmlDocument) -> Result<Value, String> {
    let mut value = doc.inspect()?;
    if let Some(object) = value.as_object_mut() {
        object.remove("canUndo");
        object.remove("canRedo");
    }
    Ok(value)
}

#[test]
fn an_edited_page_saves_and_reopens_as_edited() -> Result<(), String> {
    let mut doc = HtmlDocument::create("Fair.html", 640, 480)?;
    for page in [TITLE, SECTION, LIST] {
        assert_eq!(
            doc.execute("html.setSource", json!({ "source": page }))?["changed"],
            true
        );
    }
    // The same source again is no step at all.
    assert_eq!(
        doc.execute("html.setSource", json!({ "source": LIST }))?["changed"],
        false
    );
    assert_eq!(doc.execute("edit.undo", json!({}))?["done"], true);
    assert_eq!(source(&doc)?, SECTION);
    assert_eq!(doc.execute("edit.redo", json!({}))?["done"], true);
    assert_eq!(source(&doc)?, LIST);
    assert!(doc.dirty());

    let saved = doc.save(None)?;
    doc.mark_saved();
    assert!(!doc.dirty());
    assert_eq!(saved, LIST.as_bytes());

    let mut reopened = HtmlDocument::open("Fair.html", &saved)?;
    reopened.set_viewport(640, 480, false)?;
    assert_eq!(reopened.source(), LIST);
    assert_eq!(described(&reopened)?, described(&doc)?);
    assert_eq!(reopened.save(None)?, saved);
    assert_eq!(reopened.render_png()?, doc.render_png()?);

    // The reopened page edits on: one more step, undone, is the saved page again.
    reopened.execute("html.setSource", json!({ "source": TITLE }))?;
    assert_eq!(reopened.execute("edit.undo", json!({}))?["done"], true);
    assert_eq!(reopened.save(None)?, saved);
    Ok(())
}
