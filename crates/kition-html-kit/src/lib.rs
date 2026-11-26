//! Experimental KitDocument adapter. Rendering uses the same CPU engine as the CLI. Under engine
//! contract 2 the page is a KDM replica ([`model`], schema `kit.html` v1): commands are its
//! transactions, undo is per person and per agent, and replicas exchange changes.
use kition_html_engine::{RenderOptions, Rendered, Resources, SessionHandle, default_resources};

/// Adds a font file for every page (Kition platform E9): see [`kition_html_engine::add_font`].
pub use kition_html_engine::add_font;
use kition_kit::kdm::{ActorId, Origin, Schema};
use kition_kit::session::{
    Damage, KitModel, KitSession, KitSessionDocument, MemoryBlobs, history_scope,
};
use kition_kit::{KitCommand, KitDocument, raster};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::sync::Arc;

/// The editor state `inspect()` reports: the viewport the page is laid out in (`width` and
/// `height`), which the editor sets, not the page. kition-headless leaves it out of what an agent
/// reads (its volatile keys), and `kit.json` declares the same list as `engine.editorState`.
pub const EDITOR_STATE: &[&str] = &["width", "height"];

pub mod check;
#[cfg(test)]
mod conformance;
#[cfg(test)]
mod edit_tests;
mod files;
pub mod model;
#[cfg(test)]
mod replica_tests;
#[cfg(test)]
mod size_tests;

use model::{HtmlModel, validate_source, write_source};

/// The person a command from the in-page editor runs for when the host names none.
fn local_person() -> Origin {
    Origin::person("person:local")
}

const NEW_PAGE: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Untitled HTML</title></head><body></body></html>";

pub struct HtmlDocument {
    session: KitSession<HtmlModel>,
    saved: String,
    pending_save: RefCell<Option<String>>,
    revision: u64,
    options: RenderOptions,
    resources: Resources,
    cache: RefCell<Option<Rendered>>,
    script: RefCell<Option<SessionHandle>>,
    /// The added fonts the page was last laid out with ([`kition_html_engine::fonts_generation`]).
    fonts: std::cell::Cell<u64>,
}

impl HtmlDocument {
    /// Each command's parameter schema, as the catalogue offers it to agents; read once, since a
    /// command's parameters do not change with the page.
    fn parameter_schemas(&self) -> &'static std::collections::HashMap<String, Value> {
        static SCHEMAS: std::sync::OnceLock<std::collections::HashMap<String, Value>> =
            std::sync::OnceLock::new();
        SCHEMAS.get_or_init(|| {
            self.commands()
                .into_iter()
                .filter_map(|command| Some((command.id, command.schema?)))
                .collect()
        })
    }

    fn from_session(session: KitSession<HtmlModel>) -> Self {
        Self {
            saved: session.model().source().to_owned(),
            session,
            pending_save: RefCell::new(None),
            revision: 0,
            options: RenderOptions::default(),
            resources: default_resources(),
            cache: RefCell::new(None),
            script: RefCell::new(None),
            fonts: std::cell::Cell::new(kition_html_engine::fonts_generation()),
        }
    }

    /// A page laid out before the host added fonts draws again with them.
    fn follow_fonts(&self) {
        let generation = kition_html_engine::fonts_generation();
        if self.fonts.get() != generation {
            self.fonts.set(generation);
            self.cache.replace(None);
            self.script.replace(None);
        }
    }

    /// The page's HTML source.
    pub fn source(&self) -> &str {
        self.session.model().source()
    }

    pub fn set_resource(&mut self, path: String, bytes: Vec<u8>) -> Result<(), String> {
        if !path.starts_with("/assets/") || path.split('/').any(|part| part == "..") {
            return Err("resource paths must be under /assets/".into());
        }
        let total: usize = self
            .resources
            .iter()
            .filter(|(key, _)| *key != &path)
            .map(|(_, v)| v.len())
            .sum();
        if total.saturating_add(bytes.len()) > 64 * 1024 * 1024 {
            return Err("resource bundle exceeds 64 MiB".into());
        }
        self.resources.insert(path, bytes);
        self.cache.replace(None);
        self.script.replace(None);
        Ok(())
    }

    pub fn set_viewport(&mut self, width: u32, height: u32, dark: bool) -> Result<(), String> {
        if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 16_777_216 {
            return Err("viewport must contain 1 to 16,777,216 pixels".into());
        }
        self.options = RenderOptions {
            width,
            height,
            dark,
            scale: 1.0,
        };
        self.cache.replace(None);
        self.script.replace(None);
        Ok(())
    }

    fn with_session<T>(
        &self,
        call: impl FnOnce(&SessionHandle) -> Result<T, String>,
    ) -> Result<T, String> {
        self.follow_fonts();
        if self.script.borrow().is_none() {
            self.script.replace(Some(SessionHandle::new(
                self.source().to_owned(),
                self.resources.clone(),
                self.options.clone(),
            )?));
        }
        let script = self.script.borrow();
        call(script.as_ref().ok_or("script session is unavailable")?)
    }

    fn with_frame<T>(&self, call: impl FnOnce(&Rendered) -> T) -> Result<T, String> {
        self.follow_fonts();
        if self.cache.borrow().is_none() {
            let frame = self.with_session(|session| session.frame(1.0))?;
            self.cache.replace(Some(frame));
        }
        let cache = self.cache.borrow();
        Ok(call(cache.as_ref().ok_or("render cache is unavailable")?))
    }

    pub fn render_png(&self) -> Result<Vec<u8>, String> {
        self.with_frame(Rendered::png)?
    }

    /// The page changed (a command, an undo, another replica's changes): a new revision, and the
    /// preview starts again from the new source.
    fn changed(&mut self, damage: &Damage) {
        if *damage == Damage::None {
            return;
        }
        self.revision += 1;
        self.cache.replace(None);
        self.script.replace(None);
    }

    /// The replica's document id (stable across saves; never the file's path).
    pub fn doc_id(&self) -> String {
        self.session.replica().doc_id().to_string()
    }

    /// Opens the file `bytes` with the replica the host stored for it (`snapshot`), so the page
    /// keeps its KDM identity and history across sessions. What the file holds that the replica
    /// does not is recorded as one `system` import change. Empty `bytes` open the replica alone.
    /// A replica that does not open (damaged, of another kit, a newer schema) is ignored and the
    /// file opens as [`KitDocument::open`] opens it. A page references no blobs.
    pub fn open_with_replica(
        name: &str,
        bytes: &[u8],
        snapshot: &[u8],
        blobs: &[u8],
    ) -> Result<Self, String> {
        check_no_blobs(blobs)?;
        let mut session = match KitSession::<HtmlModel>::open_snapshot(
            snapshot,
            Box::new(MemoryBlobs::default()),
            ActorId::random(),
        ) {
            Ok(session) => session,
            Err(error) if bytes.is_empty() => {
                return Err(format!("the stored replica does not open: {error}"));
            }
            Err(_) => return Self::open(name, bytes),
        };
        if !bytes.is_empty() {
            let incoming = std::str::from_utf8(bytes).map_err(|_| "HTML must be UTF-8")?;
            validate_source(incoming)?;
            let current = session.model().source().to_owned();
            if current != incoming {
                session.execute(&Origin::system("import"), |_, tx| {
                    write_source(tx, &current, incoming)
                })?;
            }
        }
        Ok(Self::from_session(session))
    }

    /// Every blob the replica references: none.
    pub fn blobs(&self) -> Vec<u8> {
        Vec::new()
    }

    /// Takes a peer's blobs: a page has none, so only an empty bundle is accepted.
    pub fn add_blobs(&mut self, bytes: &[u8]) -> Result<(), String> {
        check_no_blobs(bytes)
    }
}

/// A page takes no blobs: an empty bundle is fine, anything else is refused.
fn check_no_blobs(blobs: &[u8]) -> Result<(), String> {
    if blobs.is_empty() {
        Ok(())
    } else {
        Err("an HTML page references no blobs; the bundle must be empty".into())
    }
}

impl KitDocument for HtmlDocument {
    fn open(name: &str, bytes: &[u8]) -> Result<Self, String> {
        Ok(Self::from_session(KitSession::<HtmlModel>::import(
            name,
            bytes,
            Box::new(MemoryBlobs::default()),
            ActorId::random(),
        )?))
    }
    fn create(_name: &str, width: u32, height: u32) -> Result<Self, String> {
        let replica =
            model::create_replica(NEW_PAGE, ActorId::random(), &Origin::system("create"))?;
        let mut doc = Self::from_session(KitSession::from_replica(
            replica,
            Box::new(MemoryBlobs::default()),
        )?);
        doc.set_viewport(width, height, false)?;
        Ok(doc)
    }
    fn width(&self) -> u32 {
        self.options.width
    }
    fn height(&self) -> u32 {
        self.options.height
    }
    fn revision(&self) -> u64 {
        self.revision
    }
    fn dirty(&self) -> bool {
        self.source() != self.saved
    }
    fn render(
        &self,
        x: f64,
        y: f64,
        scale: f64,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, String> {
        if !x.is_finite()
            || !y.is_finite()
            || !scale.is_finite()
            || scale <= 0.0
            || scale > 4.0
            || width == 0
            || height == 0
            || width > kition_kit::MAX_RENDER_SIDE
            || height > kition_kit::MAX_RENDER_SIDE
            || u64::from(width) * u64::from(height) > 16_777_216
            || (f64::from(width) / scale).ceil() * (f64::from(height) / scale).ceil() > 16_777_216.0
        {
            return Err("invalid render region".into());
        }
        if scale > 1.0 {
            let frame = self.with_session(|session| session.frame(scale))?;
            return Ok(raster::render(
                raster::Frame {
                    rgba: &frame.rgba,
                    width: frame.width,
                    height: frame.height,
                },
                x * scale,
                y * scale,
                1.0,
                width,
                height,
            ));
        }
        self.with_frame(|frame| {
            raster::render(
                raster::Frame {
                    rgba: &frame.rgba,
                    width: frame.width,
                    height: frame.height,
                },
                x,
                y,
                scale,
                width,
                height,
            )
        })
    }
    fn render_area(
        &self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, String> {
        if x1 <= x0
            || y1 <= y0
            || width == 0
            || height == 0
            || width > kition_kit::MAX_RENDER_SIDE
            || height > kition_kit::MAX_RENDER_SIDE
            || u64::from(width) * u64::from(height) > 16_777_216
            || (i64::from(x1) - i64::from(x0)).saturating_mul(i64::from(y1) - i64::from(y0))
                > 16_777_216
        {
            return Err("invalid render area".into());
        }
        self.with_frame(|frame| {
            raster::render_area(
                raster::Frame {
                    rgba: &frame.rgba,
                    width: frame.width,
                    height: frame.height,
                },
                x0,
                y0,
                x1,
                y1,
                width,
                height,
            )
        })
    }
    fn execute(&mut self, command: &str, params: Value) -> Result<Value, String> {
        self.execute_as(command, params, &local_person())
    }
    fn inspect(&self) -> Result<Value, String> {
        Ok(
            json!({"canUndo":self.session.can_undo(),"canRedo":self.session.can_redo(),
        "width":self.width(),"height":self.height(),"source":self.source(),"javascript":true,"engine":"Blitz CPU"}),
        )
    }
    fn commands(&self) -> Vec<KitCommand> {
        let mut commands = [
            (
                "file.setSource",
                "Replace HTML",
                "Replaces the whole page with `source` (an HTML document, at most 8 MiB) as one undo step; answers `changed: false` when the source is already the same. The only way to edit the page.",
                true,
                Some("{\"source\":string}"),
            ),
            (
                "preview.click",
                "Click preview",
                "Clicks the rendered page at `x`, `y` (CSS pixels from the page's top-left) so its scripts react, without changing the saved source; answers whether the rendered page changed.",
                true,
                Some("{\"x\":number,\"y\":number}"),
            ),
            (
                "preview.advance",
                "Advance preview time",
                "Runs the page's timers and animations forward by `millis` milliseconds (a whole number), without changing the saved source; answers whether the rendered page changed.",
                true,
                Some("{\"millis\":int}"),
            ),
            (
                "edit.undo",
                "Undo",
                "Takes back the caller's last `file.setSource` (the person's or, with `scope: \"agent\"`, the agent's) without touching the other's edits.",
                self.session.can_undo(),
                Some("{ scope?: \"person\" | \"agent\" }"),
            ),
            (
                "edit.redo",
                "Redo",
                "Applies again the `file.setSource` the caller's last undo took back, in the same scope as `edit.undo`.",
                self.session.can_redo(),
                Some("{ scope?: \"person\" | \"agent\" }"),
            ),
        ]
        .into_iter()
        .map(|(id, title, description, enabled, params)| {
            KitCommand {
                id: id.into(),
                title: title.into(),
                description: description.into(),
                output_schema: matches!(id, "preview.click" | "preview.advance").then(|| {
                    json!({"type":"object","additionalProperties":false,"properties":{"changed":{"type":"boolean","required":true,"description":"whether the rendered page changed"}}})
                }),
                menu: if matches!(id, "preview.click" | "preview.advance") {
                    None
                } else {
                    Some(vec!["Edit".into()])
                },
                mutates: !matches!(id, "preview.click" | "preview.advance"),
                enabled,
                params: params.map(str::to_owned),
                ..Default::default()
            }
            .with_kit_schema()
        })
        .collect::<Vec<_>>();
        kition_kit::files::declare(&mut commands, files::FILES);
        commands
    }
    fn save(&self, name: Option<&str>) -> Result<Vec<u8>, String> {
        if name.is_some_and(|name| {
            let name = name.to_ascii_lowercase();
            !name.ends_with(".html") && !name.ends_with(".htm")
        }) {
            return Err("save requires .html or .htm".into());
        }
        self.pending_save.replace(Some(self.source().to_owned()));
        Ok(self.session.export(name.unwrap_or("page.html"))?)
    }
    fn mark_saved(&mut self) {
        if let Some(source) = self.pending_save.take() {
            self.saved = source;
        }
    }
    fn check(&self) -> Vec<kition_kit::diagnostics::Diagnostic> {
        match self.with_frame(|frame| frame.diagnostics.clone()) {
            Ok(messages) => check::diagnostics(messages),
            // A page the engine cannot render is the render call's error (`render_png`, inspect,
            // the preview), which names it; the check has nothing to say about a page it cannot see.
            Err(_) => Vec::new(),
        }
    }
}

impl KitSessionDocument for HtmlDocument {
    type Model = HtmlModel;

    fn session(&self) -> &KitSession<HtmlModel> {
        &self.session
    }

    fn session_mut(&mut self) -> &mut KitSession<HtmlModel> {
        &mut self.session
    }

    fn execute_as(
        &mut self,
        command: &str,
        params: Value,
        origin: &Origin,
    ) -> Result<Value, String> {
        // A key the command's schema does not name is refused, not ignored (platform rule E7a).
        if let Some(schema) = self.parameter_schemas().get(command) {
            kition_kit::schema::refuse_unknown(command, schema, &params)?;
        }
        match command {
            "preview.advance" | "preview.click" => {
                let changed = if command == "preview.advance" {
                    let millis = params
                        .get("millis")
                        .and_then(Value::as_u64)
                        .ok_or("millis must be a nonnegative integer")?;
                    self.with_session(|session| session.advance(millis))?
                } else {
                    let x = params
                        .get("x")
                        .and_then(Value::as_f64)
                        .ok_or("x must be a number")?;
                    let y = params
                        .get("y")
                        .and_then(Value::as_f64)
                        .ok_or("y must be a number")?;
                    self.with_session(|session| session.click(x as f32, y as f32))?
                };
                if changed {
                    self.cache.replace(None);
                }
                Ok(json!({"changed":changed}))
            }
            "file.setSource" => {
                let source = params
                    .get("source")
                    .and_then(Value::as_str)
                    .ok_or("source must be a string")?;
                validate_source(source)?;
                if source == self.source() {
                    return Ok(json!({"changed":false}));
                }
                let (_, damage) = self
                    .session
                    .execute(origin, |model, tx| write_source(tx, model.source(), source))?;
                self.changed(&damage);
                Ok(json!({"changed":true}))
            }
            // In the scope its `scope` parameter names, else the origin's (E14a, M12a).
            "edit.undo" => {
                let (done, damage) =
                    KitSessionDocument::undo_in(self, history_scope(&params, origin)?)?;
                Ok(json!({ "done": done, "damage": damage }))
            }
            "edit.redo" => {
                let (done, damage) =
                    KitSessionDocument::redo_in(self, history_scope(&params, origin)?)?;
                Ok(json!({ "done": done, "damage": damage }))
            }
            _ => Err(format!("unknown command: {command}")),
        }
    }

    fn import_changes(&mut self, bytes: &[u8]) -> Result<Damage, String> {
        let damage = self.session.import_changes(bytes)?;
        self.changed(&damage);
        Ok(damage)
    }

    /// A draft merged in (`mergeChanges`, one step of the person's Undo, M12a): a new revision the
    /// preview and `inspect` see, as after a command.
    fn merge_changes(
        &mut self,
        bytes: &[u8],
        origin: &Origin,
    ) -> Result<(Option<kition_kit::kdm::UndoScope>, Damage), String> {
        let (scope, damage) = self.session.merge_changes(bytes, origin)?;
        self.changed(&damage);
        Ok((scope, damage))
    }

    /// An earlier state written over the page (`restoreState`), taken in as a merged draft is.
    fn restore_state(&mut self, snapshot: &[u8], origin: &Origin) -> Result<Damage, String> {
        let damage = self.session.restore_state(snapshot, origin)?;
        self.changed(&damage);
        Ok(damage)
    }

    /// The engine's `undo()` and `redo()` members: the person's, keeping the revision current.
    fn undo(&mut self) -> Result<(bool, Damage), String> {
        self.undo_in(kition_kit::kdm::UndoScope::Person)
    }

    fn redo(&mut self) -> Result<(bool, Damage), String> {
        self.redo_in(kition_kit::kdm::UndoScope::Person)
    }

    fn undo_in(&mut self, scope: kition_kit::kdm::UndoScope) -> Result<(bool, Damage), String> {
        let (done, damage) = self.session.undo_in(scope)?;
        self.changed(&damage);
        Ok((done, damage))
    }

    fn redo_in(&mut self, scope: kition_kit::kdm::UndoScope) -> Result<(bool, Damage), String> {
        let (done, damage) = self.session.redo_in(scope)?;
        self.changed(&damage);
        Ok((done, damage))
    }
}

/// The object model (`KitObjectModel 1.0`) headless, over the document's change exchange.
impl kition_kit::session::model::ReplicaPort for HtmlDocument {
    fn model_schema(&self) -> Arc<Schema> {
        HtmlModel::schema()
    }

    fn model_export(&mut self, since: &[u8]) -> Result<Vec<u8>, String> {
        kition_kit::session::__export_changes(self, since)
    }

    fn model_import(&mut self, bytes: &[u8]) -> Result<(), String> {
        KitSessionDocument::import_changes(self, bytes).map(|_| ())
    }

    fn model_command(
        &mut self,
        command: &str,
        params: Value,
        origin: &Origin,
    ) -> Result<Value, String> {
        self.execute_as(command, params, origin)
    }

    fn model_commands(&self) -> Vec<KitCommand> {
        self.commands()
    }

    /// A shared page with no file yet: the replica alone, failing rather than falling back to an
    /// empty page.
    fn model_open_replica(_name: &str, snapshot: &[u8], blobs: &[u8]) -> Result<Self, String> {
        check_no_blobs(blobs)?;
        let session = KitSession::<HtmlModel>::open_snapshot(
            snapshot,
            Box::new(MemoryBlobs::default()),
            ActorId::random(),
        )?;
        Ok(Self::from_session(session))
    }
}

kition_kit::export_kit_document!(HtmlDocument as "HtmlDocument");
kition_kit::export_kit_session!(HtmlDocument as "HtmlDocument");

#[cfg(target_arch = "wasm32")]
mod web {
    use crate::__kition_kit_export::WebDocument;
    use kition_kit::wasm_bindgen::prelude::*;

    /// Adds a font file for every page (Kition platform E9; kit-editor calls it for each of its
    /// `fonts` before a page opens); answers how many faces it added.
    #[wasm_bindgen(js_name = addFont, wasm_bindgen = kition_kit::wasm_bindgen)]
    pub fn add_font(bytes: Vec<u8>) -> usize {
        crate::add_font(&bytes)
    }
    #[wasm_bindgen(js_class="HtmlDocument",wasm_bindgen=kition_kit::wasm_bindgen)]
    impl WebDocument {
        #[wasm_bindgen(js_name=setResource)]
        pub fn set_resource(&mut self, path: String, bytes: Vec<u8>) -> Result<(), JsError> {
            self.0
                .set_resource(path, bytes)
                .map_err(|e| JsError::new(&e))
        }
        #[wasm_bindgen(js_name=setViewport)]
        pub fn set_viewport(&mut self, width: u32, height: u32, dark: bool) -> Result<(), JsError> {
            self.0
                .set_viewport(width, height, dark)
                .map_err(|e| JsError::new(&e))
        }
        #[wasm_bindgen(js_name=renderPng)]
        pub fn render_png(&self) -> Result<Vec<u8>, JsError> {
            self.0.render_png().map_err(|e| JsError::new(&e))
        }
        /// Opens a file with the replica the host stored for it (engine contract 2).
        #[wasm_bindgen(js_name=openWithReplica)]
        pub fn open_with_replica(
            name: &str,
            bytes: &[u8],
            snapshot: &[u8],
            blobs: &[u8],
        ) -> Result<WebDocument, JsError> {
            crate::HtmlDocument::open_with_replica(name, bytes, snapshot, blobs)
                .map(WebDocument)
                .map_err(|e| JsError::new(&e))
        }
        #[wasm_bindgen(getter, js_name=hasReplica)]
        pub fn has_replica(&self) -> bool {
            true
        }
        #[wasm_bindgen(getter, js_name=docId)]
        pub fn doc_id(&self) -> String {
            self.0.doc_id()
        }
        pub fn blobs(&self) -> Vec<u8> {
            self.0.blobs()
        }
        #[wasm_bindgen(js_name=blobsSince)]
        pub fn blobs_since(&self, _since: &[u8]) -> Vec<u8> {
            self.0.blobs()
        }
        #[wasm_bindgen(js_name=addBlobs)]
        pub fn add_blobs(&mut self, bytes: &[u8]) -> Result<(), JsError> {
            self.0.add_blobs(bytes).map_err(|e| JsError::new(&e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_command_says_what_it_does() -> Result<(), String> {
        let doc = HtmlDocument::create("page.html", 320, 240)?;
        kition_kit::testing::check_commands(&doc.commands(), &[])
            .map_err(|problems| problems.join("\n"))
    }

    #[test]
    fn script_state_survives_frames_without_changing_saved_source() -> Result<(), String> {
        let source = b"<!doctype html><style>body{margin:0}div{width:64px;height:64px;background:red}</style><div></div><script>let n=0;document.querySelector('div').onclick=()=>{n++;document.querySelector('div').style.background=n===2?'#5645d4':'blue'}</script>";
        let mut doc = HtmlDocument::open("script.html", source)?;
        doc.execute("preview.click", json!({"x":4,"y":4}))?;
        doc.render(0.0, 0.0, 2.0, 32, 32)?;
        doc.execute("preview.click", json!({"x":4,"y":4}))?;
        assert_eq!(
            &doc.render(0.0, 0.0, 1.0, 32, 32)?[..4],
            &[86, 69, 212, 255]
        );
        assert_eq!(doc.save(None)?, source);
        assert!(!doc.dirty());
        assert_eq!(doc.revision(), 0);
        Ok(())
    }

    #[test]
    fn source_editing_can_clear_and_replace_a_document() -> Result<(), String> {
        let mut doc = HtmlDocument::open("PAGE.HTML", b"<p>Original</p>")?;
        doc.execute("file.setSource", json!({"source": ""}))?;
        assert_eq!(doc.save(Some("PAGE.HTML"))?, b"");
        doc.execute(
            "file.setSource",
            json!({"source": "<p>Replacement HTML</p>"}),
        )?;
        assert_eq!(doc.save(Some("PAGE.HTM"))?, b"<p>Replacement HTML</p>");
        doc.execute("edit.undo", json!({}))?;
        assert_eq!(doc.save(None)?, b"");
        doc.execute("edit.undo", json!({}))?;
        assert_eq!(doc.save(None)?, b"<p>Original</p>");
        Ok(())
    }

    #[test]
    fn keeps_the_kit_document_contract() -> Result<(), String> {
        kition_kit::testing::check_contract::<HtmlDocument>(
            b"<!doctype html><style>body{margin:0;background:white}</style><div></div>",
            "fixture.html",
            |doc| {
                doc.execute("file.setSource",json!({"source":"<!doctype html><style>body{margin:0;background:#5645d4}</style><div></div>"})).map(|_|())
            },
        )
    }
    #[test]
    fn sdk_and_native_defaults_render_text_identically() -> Result<(), String> {
        let source = "<!doctype html><style>body{margin:0;background:white;color:black;font:24px/32px Inter}</style>Shared default text";
        let mut doc = HtmlDocument::open("text.html", source.as_bytes())?;
        doc.set_viewport(300, 64, false)?;
        let native = kition_html_engine::render(
            source,
            default_resources(),
            &RenderOptions {
                width: 300,
                height: 64,
                ..Default::default()
            },
        )?;
        assert_eq!(doc.render_png()?, native.png()?);
        assert!(
            native
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|pixel| pixel[0] < 200)
                .count()
                > 300
        );
        assert!(native.diagnostics.is_empty());
        Ok(())
    }
    #[test]
    fn saved_bytes_and_new_edits_have_distinct_dirty_states() -> Result<(), String> {
        let mut doc = HtmlDocument::open("fixture.html", b"<div>One</div>")?;
        doc.execute("file.setSource", json!({"source":"<div>Two</div>"}))?;
        doc.save(None)?;
        doc.execute("file.setSource", json!({"source":"<div>Three</div>"}))?;
        doc.mark_saved();
        assert!(doc.dirty());
        doc.execute("edit.undo", json!({}))?;
        assert!(!doc.dirty());
        doc.execute("edit.redo", json!({}))?;
        assert!(doc.dirty());
        Ok(())
    }
}
