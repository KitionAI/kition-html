//! The headless executable (`kition-html --kition-kit`) starts through `run_manifest_model`: it
//! serves the page's object model (`model.describe`, `model.batch`) and the replica methods
//! (`model.open`, `model.import`, `model.export`, `model.save`) over the engine's change exchange.
//! `kit.json` lists exactly the features `kit.initialize` answers (platform rule K9), so a host
//! knows them before it starts the executable.
use kition_headless::{FileIdentity, Headless, KitInfo};
use kition_html_kit::HtmlDocument;
use kition_kit::KitDocument;
use kition_kit::session::KitSessionDocument;
use kition_kit_server::{KitBackend, with_model_features};
use serde_json::{Value, json};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const MANIFEST: &str = include_str!("../../../kit.json");
const NAME: &str = "page.html";

/// The backend `main` serves, built from the same manifest with the same arguments.
fn kit() -> Result<Headless<HtmlDocument>> {
    Ok(
        Headless::with_replica_port(KitInfo::from_manifest(MANIFEST, &[], &["width", "height"])?)
            .with_file_identity(FileIdentity::NotEmbedded),
    )
}

fn call(kit: &mut Headless<HtmlDocument>, method: &str, params: Value) -> Result<Value> {
    kit.model(method, params)
        .map_err(|error| format!("{method}: {error:?}").into())
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn encode(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let at = |i: usize| u32::from(chunk.get(i).copied().unwrap_or(0));
        let n = (at(0) << 16) | (at(1) << 8) | at(2);
        for k in 0..4 {
            let digit = BASE64.get(((n >> (18 - 6 * k)) & 63) as usize).copied();
            out.push(if k <= chunk.len() {
                char::from(digit.unwrap_or(b'='))
            } else {
                '='
            });
        }
    }
    out
}

fn decode(text: &str) -> Result<Vec<u8>> {
    let (mut out, mut buffer, mut bits) = (Vec::new(), 0u32, 0u32);
    for byte in text.trim_end_matches('=').bytes() {
        let value = BASE64
            .iter()
            .position(|known| *known == byte)
            .ok_or("not base64")?;
        buffer = (buffer << 6) | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Ok(out)
}

fn text(doc: &HtmlDocument) -> Result<String> {
    Ok(String::from_utf8(doc.save(None)?)?)
}

#[test]
fn kit_json_lists_the_features_kit_initialize_answers() -> Result {
    let manifest: Value = serde_json::from_str(MANIFEST)?;
    let mut kit = kit()?;
    let serves = kit.serves_model();
    // What `kit.initialize` answers in `capabilities.features`: the server adds the object
    // model's features, then `document.identity` for a backend that says how its files carry
    // their identity (Kit Process Protocol section 5.6).
    let mut answered = with_model_features(kit.capabilities(), serves)["features"].clone();
    if kit.file_identity().is_some()
        && let Some(features) = answered.as_array_mut()
    {
        features.push(json!("document.identity"));
    }
    assert_eq!(manifest["headless"]["features"], answered);
    Ok(())
}

#[test]
fn the_page_is_served_through_its_object_model_and_replica() -> Result {
    let mut kit = kit()?;
    // The room's page, as a collaborator holds it.
    let mut peer = HtmlDocument::open(NAME, b"<h1>Title</h1><p>Body</p>")?;
    let snapshot = peer.session().snapshot().map_err(|e| e.to_string())?;
    let opened = call(
        &mut kit,
        "model.open",
        json!({ "name": NAME, "snapshot": encode(&snapshot) }),
    )?;
    let id = opened["documentId"]
        .as_str()
        .ok_or("model.open answered no documentId")?
        .to_owned();

    let described = call(&mut kit, "model.describe", json!({ "documentId": id }))?;
    assert!(described["types"].get("Page").is_some(), "{described}");

    // The object model reads the page's source and writes it as a text edit.
    let read = |kit: &mut Headless<HtmlDocument>| -> Result<Value> {
        let loaded = call(
            kit,
            "model.batch",
            json!({ "documentId": id, "ops": [], "load": [{ "target": "root", "fields": ["source"] }] }),
        )?;
        Ok(loaded["loaded"][0]["fields"]["source"].clone())
    };
    assert_eq!(read(&mut kit)?, "<h1>Title</h1><p>Body</p>");
    call(
        &mut kit,
        "model.batch",
        json!({ "documentId": id, "ops": [{ "op": "text.insert", "target": "root", "field": "source", "index": 21, "text": " text" }] }),
    )?;
    assert_eq!(read(&mut kit)?, "<h1>Title</h1><p>Body text</p>");

    // The collaborator's concurrent edit comes in, and the kit's goes out: both converge.
    let since = peer.session().version();
    peer.execute(
        "html.setSource",
        json!({ "source": "<h1>New title</h1><p>Body</p>" }),
    )?;
    let changes = peer
        .session()
        .export_changes(&since)
        .map_err(|e| e.to_string())?;
    call(
        &mut kit,
        "model.import",
        json!({ "documentId": id, "changes": encode(&changes) }),
    )?;
    let exported = call(
        &mut kit,
        "model.export",
        json!({ "documentId": id, "since": encode(&peer.session().version().encode()) }),
    )?;
    let changes = decode(exported["changes"].as_str().ok_or("no changes")?)?;
    KitSessionDocument::import_changes(&mut peer, &changes)?;
    assert_eq!(text(&peer)?, "<h1>New title</h1><p>Body text</p>");

    let saved = call(&mut kit, "model.save", json!({ "documentId": id }))?;
    assert_eq!(saved["format"], "html");
    let data = decode(
        saved["data"]
            .as_str()
            .ok_or("model.save answered no data")?,
    )?;
    assert_eq!(
        String::from_utf8(data)?,
        "<h1>New title</h1><p>Body text</p>"
    );

    // A replica the kit cannot read is refused, never opened as an empty page.
    assert!(
        kit.model(
            "model.open",
            json!({ "name": NAME, "snapshot": encode(b"junk") })
        )
        .is_err()
    );
    Ok(())
}
