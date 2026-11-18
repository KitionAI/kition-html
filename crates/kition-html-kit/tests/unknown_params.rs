//! A command with a parameter schema refuses a key the schema does not name, and says which
//! (platform rule E7a, kit requirement AX15): an agent that misspells a parameter is told instead
//! of the command running without it.
#![allow(clippy::unwrap_used, clippy::panic)]

use kition_html_kit::HtmlDocument;
use kition_kit::KitDocument;
use serde_json::json;

fn doc() -> HtmlDocument {
    HtmlDocument::create("Page.html", 640, 480).unwrap()
}

#[test]
fn every_command_refuses_an_unknown_parameter() {
    if let Err(problems) = kition_kit::testing::check_unknown_params(doc, &[]) {
        panic!("{} problems:\n{}", problems.len(), problems.join("\n"));
    }
}

#[test]
fn a_misspelled_parameter_names_the_one_it_resembles() {
    let error = doc()
        .execute("html.setSource", json!({ "sourec": "<p>Hi</p>" }))
        .unwrap_err();
    assert!(
        error.contains("`sourec`") && error.contains("did you mean `source`"),
        "{error}"
    );
}
