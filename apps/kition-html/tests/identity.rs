//! `file.identity` and `file.fork` (Kit Process Protocol section 5.6, KitModel M7a)
//! through the real `kition-html` process. A saved page is its HTML source alone
//! (`HtmlModel::export`): the file embeds no replica, and the kit makes a new one each time it
//! opens a file. So a file carries no identity (`docId` is `null`) and its fork is the same bytes.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use kition_kit_host::{HeadlessKit, LaunchOptions};

const LIMIT: Duration = Duration::from_secs(30);

const PAGE: &[u8] = b"<!doctype html><title>Plan</title><h1>Concert day</h1><p>Doors at seven.</p>";

fn workspace(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "kition-html-identity-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::canonicalize(dir).unwrap()
}

async fn launch(dir: &Path) -> HeadlessKit {
    let mut options = LaunchOptions::new(env!("CARGO_BIN_EXE_kition-html"));
    options.workspace = Some(dir.to_path_buf());
    HeadlessKit::launch(options).await.unwrap()
}

#[tokio::test]
async fn a_page_carries_no_identity_and_forks_into_the_same_bytes() {
    let dir = workspace("plain");
    std::fs::write(dir.join("Plan.html"), PAGE).unwrap();
    let kit = launch(&dir).await;
    assert!(kit.serves("file.identity"), "{}", kit.capabilities());
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../../kit.json")).unwrap();
    assert_eq!(
        kit.capabilities()["features"],
        manifest["headless"]["features"],
        "kit.json lists exactly the features kit.initialize answers"
    );

    // A saved page holds no replica: no identity, for the page and for a file another program
    // wrote that is no page at all.
    assert_eq!(kit.identity("Plan.html", LIMIT).await.unwrap().doc_id, None);
    std::fs::write(dir.join("Empty.html"), b"").unwrap();
    assert_eq!(
        kit.identity("Empty.html", LIMIT).await.unwrap().doc_id,
        None
    );

    // Its fork is the same bytes under the new name, with no docId.
    let fork = kit.fork("Plan.html", "Fork.html", LIMIT).await.unwrap();
    assert_eq!(fork.doc_id, None);
    assert_eq!(fork.name, "Fork.html");
    assert!(!fork.version.is_empty());
    assert_eq!(
        std::fs::read(dir.join("Fork.html")).unwrap(),
        PAGE,
        "a file without a replica forks unchanged"
    );
    assert_eq!(
        std::fs::read(dir.join("Plan.html")).unwrap(),
        PAGE,
        "the source is unchanged"
    );

    // The fork opens as a page; a fork of the fork is the same bytes again.
    let state = kit.open("Fork.html", true, LIMIT).await.unwrap();
    assert!(
        state["documentId"].is_string(),
        "the fork opens as a page: {state}"
    );
    let again = kit.fork("Fork.html", "Again.html", LIMIT).await.unwrap();
    assert_eq!(again.doc_id, None);
    assert_eq!(std::fs::read(dir.join("Again.html")).unwrap(), PAGE);

    // A fork never replaces a file.
    std::fs::write(dir.join("Taken.html"), b"<p>Mine</p>").unwrap();
    let taken = kit
        .fork("Plan.html", "Taken.html", LIMIT)
        .await
        .unwrap_err();
    assert_eq!(taken.reason(), Some("file_conflict"), "{taken}");
    assert_eq!(
        std::fs::read(dir.join("Taken.html")).unwrap(),
        b"<p>Mine</p>",
        "the existing file is kept"
    );
    kit.shutdown(Duration::from_secs(5)).await;
    let _ = std::fs::remove_dir_all(dir);
}
