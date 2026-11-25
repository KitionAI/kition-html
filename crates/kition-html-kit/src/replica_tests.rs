//! Engine contract 2: the page lives in a KDM replica (schema `kit.html` v1, `model/schema.json`).
use super::*;
use kition_kit::kdm::{ActorId, Origin, Replica};
use kition_kit::session::KitSessionDocument;
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
    doc.execute("file.setSource", json!({ "source": "<p>Edited</p>" }))?;
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
    doc.execute("file.setSource", json!({ "source": "<p>Two</p>" }))?;
    doc.execute_as(
        "file.setSource",
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

#[test]
fn an_imported_change_is_a_new_revision_the_preview_and_inspect_see() -> Result<(), String> {
    // Two replicas of one page on one line of changes: b takes in a's edit.
    let mut a = HtmlDocument::open("page.html", b"<h1>Title</h1><p>Body</p>")?;
    let snapshot = a.session().snapshot().map_err(|error| error.to_string())?;
    let mut b = HtmlDocument::open_with_replica("page.html", &[], &snapshot, &[])?;
    let base = a.session().version();
    a.execute(
        "file.setSource",
        json!({ "source": "<h1>New title</h1><p>Body text</p>" }),
    )?;
    let changes = a
        .session()
        .export_changes(&base)
        .map_err(|error| error.to_string())?;
    let revision = b.revision();
    KitSessionDocument::import_changes(&mut b, &changes)?;
    assert_eq!(text(&b)?, "<h1>New title</h1><p>Body text</p>");
    assert_eq!(text(&a)?, text(&b)?);
    assert!(b.revision() > revision);
    assert_eq!(b.inspect()?["source"], "<h1>New title</h1><p>Body text</p>");
    Ok(())
}

/// A draft of `doc` (its replica as it is now), and the version it was made at.
fn draft_of(doc: &HtmlDocument) -> Result<(HtmlDocument, Vec<u8>), String> {
    let snapshot = doc
        .session()
        .snapshot()
        .map_err(|error| error.to_string())?;
    let since = kition_kit::session::__version(doc);
    Ok((
        HtmlDocument::open_with_replica("page.html", &[], &snapshot, &[])?,
        since,
    ))
}

#[test]
fn an_agents_merged_draft_is_one_undo_step_of_the_person_and_undoes_exactly() -> Result<(), String>
{
    let agent = Origin::agent("agent:test");
    let mut doc = HtmlDocument::open("page.html", b"<h1>Title</h1><p>Body</p>")?;
    doc.execute(
        "file.setSource",
        json!({ "source": "<h1>Title</h1><p>Body</p><p>Mine</p>" }),
    )?;
    let (mut draft, since) = draft_of(&doc)?;
    draft.execute_as(
        "file.setSource",
        json!({ "source": "<h1>New title</h1><p>Body</p><p>Mine</p>" }),
        &agent,
    )?;
    let changes = kition_kit::session::__export_changes(&draft, &since)?;
    let before = doc.session().state_hash().map_err(|e| e.to_string())?;
    let revision = doc.revision();
    let answer: Value = serde_json::from_str(&kition_kit::session::__merge_changes(
        &mut doc,
        &changes,
        r#"["agent","agent:test",null]"#,
    )?)
    .map_err(|e| e.to_string())?;
    assert!(answer["summary"].is_object(), "{answer}");
    let merged = "<h1>New title</h1><p>Body</p><p>Mine</p>";
    assert_eq!(text(&doc)?, merged);
    assert!(doc.revision() > revision, "the merge is a new revision");
    assert_eq!(doc.inspect()?["source"], merged, "inspect sees the merge");
    let after = doc.session().state_hash().map_err(|e| e.to_string())?;
    // The person approved it: the agent's Undo leaves it, the person's takes it back exactly.
    assert_eq!(
        doc.execute_as("edit.undo", json!({}), &agent)?["done"],
        false
    );
    assert_eq!(text(&doc)?, merged);
    let revision = doc.revision();
    assert!(KitSessionDocument::undo(&mut doc)?.0);
    assert_eq!(text(&doc)?, "<h1>Title</h1><p>Body</p><p>Mine</p>");
    assert_eq!(
        doc.session().state_hash().map_err(|e| e.to_string())?,
        before
    );
    assert!(
        doc.revision() > revision,
        "the engine's undo() is a new revision"
    );
    assert_eq!(
        doc.inspect()?["source"],
        "<h1>Title</h1><p>Body</p><p>Mine</p>"
    );
    assert!(KitSessionDocument::redo(&mut doc)?.0);
    assert_eq!(
        doc.session().state_hash().map_err(|e| e.to_string())?,
        after
    );
    assert_eq!(doc.inspect()?["source"], merged);
    // The person's own edit below the merge is the next step.
    assert_eq!(doc.execute("edit.undo", json!({}))?["done"], true);
    assert_eq!(doc.execute("edit.undo", json!({}))?["done"], true);
    assert_eq!(text(&doc)?, "<h1>Title</h1><p>Body</p>");
    Ok(())
}

#[test]
fn a_draft_on_a_page_edited_meanwhile_is_refused_and_changes_nothing() -> Result<(), String> {
    // The page is one text field (`Page.source`): an edit of the person since the draft was made
    // and the draft's edit both change it, which a merge refuses (data model 6.6, D5), whatever
    // the ranges. Nothing changes.
    let mut doc = HtmlDocument::open("page.html", b"<h1>Title</h1><p>Body</p>")?;
    let (mut draft, since) = draft_of(&doc)?;
    draft.execute_as(
        "file.setSource",
        json!({ "source": "<h1>New title</h1><p>Body</p>" }),
        &Origin::agent("agent:test"),
    )?;
    doc.execute(
        "file.setSource",
        json!({ "source": "<h1>Title</h1><p>Body text</p>" }),
    )?;
    let changes = kition_kit::session::__export_changes(&draft, &since)?;
    let before = doc.session().state_hash().map_err(|e| e.to_string())?;
    let revision = doc.revision();
    let Err(error) =
        KitSessionDocument::merge_changes(&mut doc, &changes, &Origin::agent("agent:test"))
    else {
        return Err("the merge was taken".into());
    };
    assert!(error.starts_with("stale: "), "{error}");
    assert!(error.contains("rule D5"), "{error}");
    assert_eq!(
        doc.session().state_hash().map_err(|e| e.to_string())?,
        before
    );
    assert_eq!(doc.revision(), revision);
    assert_eq!(text(&doc)?, "<h1>Title</h1><p>Body text</p>");
    Ok(())
}

#[test]
fn a_restored_state_is_one_undo_step_and_undoes_exactly() -> Result<(), String> {
    let person = Origin::person("user:test");
    let mut doc = HtmlDocument::open("page.html", b"<p>One</p>")?;
    let earlier = doc.session().snapshot().map_err(|e| e.to_string())?;
    doc.execute("file.setSource", json!({ "source": "<p>Two</p>" }))?;
    let before = doc.session().state_hash().map_err(|e| e.to_string())?;
    let revision = doc.revision();
    KitSessionDocument::restore_state(&mut doc, &earlier, &person)?;
    assert_eq!(text(&doc)?, "<p>One</p>");
    assert!(doc.revision() > revision, "the restore is a new revision");
    assert_eq!(doc.inspect()?["source"], "<p>One</p>");
    let restored = doc.session().state_hash().map_err(|e| e.to_string())?;
    assert!(KitSessionDocument::undo(&mut doc)?.0);
    assert_eq!(
        doc.session().state_hash().map_err(|e| e.to_string())?,
        before
    );
    assert_eq!(doc.inspect()?["source"], "<p>Two</p>");
    assert!(KitSessionDocument::redo(&mut doc)?.0);
    assert_eq!(
        doc.session().state_hash().map_err(|e| e.to_string())?,
        restored
    );
    assert_eq!(doc.inspect()?["source"], "<p>One</p>");
    Ok(())
}

#[test]
fn an_agents_reply_comes_back_unchanged_as_the_schema_has_no_tool_ids() -> Result<(), String> {
    // `kit.html` declares no tool ids: `modelReply` answers every reply as it is, the page's own
    // object id included, without reading the document.
    let doc = HtmlDocument::open("page.html", PAGE.as_bytes())?;
    assert!(!model::schema().has_tool_ids());
    let page = doc
        .session()
        .replica()
        .objects_of_type("Page")
        .first()
        .map(ToString::to_string)
        .ok_or("no page")?;
    let mut keyed = serde_json::Map::new();
    keyed.insert(page.clone(), json!({ "source": "<p>x</p>" }));
    for reply in [
        json!({ "page": page, "objects": keyed }),
        json!({ "source": "<p>x</p>", "width": 800 }),
    ] {
        assert_eq!(
            kition_kit::session::__model_reply(&doc, &reply.to_string())?,
            reply.to_string()
        );
    }
    Ok(())
}
