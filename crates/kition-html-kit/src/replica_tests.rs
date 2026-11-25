//! Engine contract 2: the page lives in a KDM replica (schema `kit.html` v1, `model/schema.json`).
use super::*;
use kition_kit::kdm::{ActorId, Origin, Replica};
use kition_kit::session::model::ReplicaPort;

/// A byte order mark, CRLF line ends and non-ASCII text: all of it must come back unchanged.
const PAGE: &str = "\u{feff}<!doctype html>\r\n<html lang=\"en\"><body><h1>Caf\u{e9}</h1>\r\n<p>Tickets</p></body></html>\r\n";

fn text(doc: &HtmlDocument) -> Result<String, String> {
    String::from_utf8(doc.save(None)?).map_err(|error| error.to_string())
}

#[test]
fn the_schema_is_kit_html_v1_and_the_manifest_names_it() -> Result<(), String> {
    let schema = model::schema();
    assert_eq!(
        (
            schema.kit.as_str(),
            schema.schema_version,
            schema.root.as_str()
        ),
        ("kit.html", 1, "Page")
    );
    let manifest: Value = serde_json::from_str(include_str!("../../../kit.json"))
        .map_err(|error| error.to_string())?;
    assert_eq!(manifest["engine"]["contract"], 2);
    assert_eq!(
        manifest["model"]["schema"],
        "crates/kition-html-kit/model/schema.json"
    );
    assert_eq!(manifest["model"]["schemaVersion"], 1);
    assert_eq!(manifest["objectModel"]["namespace"], "Html");
    Ok(())
}

#[test]
fn a_page_round_trips_through_its_replica_byte_for_byte() -> Result<(), String> {
    let doc = HtmlDocument::open("page.html", PAGE.as_bytes())?;
    let snapshot = doc
        .session()
        .snapshot()
        .map_err(|error| error.to_string())?;
    // The snapshot conforms to the schema: it loads under it.
    Replica::load(model::schema(), &snapshot, ActorId::random())
        .map_err(|error| error.to_string())?;
    let reopened = HtmlDocument::open_with_replica("page.html", &[], &snapshot, &[])?;
    assert_eq!(text(&reopened)?, PAGE);
    assert_eq!(reopened.doc_id(), doc.doc_id());
    assert!(!reopened.dirty());
    Ok(())
}

#[test]
fn an_unchanged_page_saves_the_same_bytes_and_reopens_with_the_same_object_ids()
-> Result<(), String> {
    let mut doc = HtmlDocument::open("page.html", PAGE.as_bytes())?;
    doc.execute("html.setSource", json!({ "source": "<p>Edited</p>" }))?;
    let saved = doc.save(None)?;
    doc.mark_saved();
    assert_eq!(
        doc.save(None)?,
        saved,
        "an unchanged page saves the same bytes"
    );
    let snapshot = doc
        .session()
        .snapshot()
        .map_err(|error| error.to_string())?;
    let objects = |doc: &HtmlDocument| -> Vec<String> {
        let mut ids: Vec<String> = doc
            .session()
            .replica()
            .objects_of_type("Page")
            .iter()
            .map(ToString::to_string)
            .collect();
        ids.sort();
        ids
    };
    // Reopened on its stored replica with the file it saved: the same objects under the same ids,
    // the same state, no change recorded, and the same bytes saved again.
    let reopened = HtmlDocument::open_with_replica("page.html", &saved, &snapshot, &[])?;
    assert!(!objects(&doc).is_empty());
    assert_eq!(objects(&reopened), objects(&doc));
    assert_eq!(
        reopened
            .session()
            .replica()
            .state_json()
            .map_err(|e| e.to_string())?,
        doc.session()
            .replica()
            .state_json()
            .map_err(|e| e.to_string())?
    );
    assert_eq!(reopened.session().version(), doc.session().version());
    assert_eq!(reopened.save(None)?, saved);
    assert!(!reopened.dirty());
    Ok(())
}

#[test]
fn a_stored_replica_takes_in_the_file_as_it_is_now() -> Result<(), String> {
    let doc = HtmlDocument::open("page.html", b"<p>Before</p>")?;
    let snapshot = doc
        .session()
        .snapshot()
        .map_err(|error| error.to_string())?;
    let reopened = HtmlDocument::open_with_replica("page.html", b"<p>After</p>", &snapshot, &[])?;
    assert_eq!(text(&reopened)?, "<p>After</p>");
    assert_eq!(reopened.doc_id(), doc.doc_id());
    // A damaged replica is ignored and the file opens on its own; with no file it is an error.
    assert_eq!(
        text(&HtmlDocument::open_with_replica(
            "page.html",
            b"<p>File</p>",
            b"junk",
            &[]
        )?)?,
        "<p>File</p>"
    );
    assert!(HtmlDocument::open_with_replica("page.html", &[], b"junk", &[]).is_err());
    assert!(HtmlDocument::open_with_replica("page.html", &[], &snapshot, b"x").is_err());
    Ok(())
}

#[test]
fn an_agent_step_undoes_in_the_agent_scope_only() -> Result<(), String> {
    let agent = Origin::agent("agent:test");
    let mut doc = HtmlDocument::open("page.html", b"<p>One</p>")?;
    doc.execute("html.setSource", json!({ "source": "<p>Two</p>" }))?;
    doc.execute_as(
        "html.setSource",
        json!({ "source": "<p>Two</p><p>Agent</p>" }),
        &agent,
    )?;
    let answer = doc.execute_as("edit.undo", json!({ "scope": "agent" }), &agent)?;
    assert_eq!(answer["done"], true);
    assert_eq!(text(&doc)?, "<p>Two</p>");
    // The person's step is still theirs to undo, and redo brings it back.
    assert_eq!(doc.execute("edit.undo", json!({}))?["done"], true);
    assert_eq!(text(&doc)?, "<p>One</p>");
    assert_eq!(doc.execute("edit.redo", json!({}))?["done"], true);
    assert_eq!(text(&doc)?, "<p>Two</p>");
    Ok(())
}

#[test]
fn a_server_opens_the_replica_alone_and_refuses_a_damaged_one() -> Result<(), String> {
    let doc = HtmlDocument::open("page.html", PAGE.as_bytes())?;
    let snapshot = doc
        .session()
        .snapshot()
        .map_err(|error| error.to_string())?;
    assert_eq!(
        text(&HtmlDocument::model_open_replica(
            "page.html",
            &snapshot,
            &[]
        )?)?,
        PAGE
    );
    assert!(HtmlDocument::model_open_replica("page.html", b"junk", &[]).is_err());
    Ok(())
}
