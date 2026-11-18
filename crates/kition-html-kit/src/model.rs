//! The HTML kit's document on the Kit Data Model: schema `kit.html` v1 (`model/schema.json`).
//!
//! A page is one `Page` object whose `source` text is the whole HTML file. The file is the text:
//! export writes the stored characters back, nothing is regenerated from a parse, so a round trip
//! is byte-identical by construction (a byte order mark and CRLF line ends included).

use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};

use kition_kit::kdm::{ActorId, DocId, ObjectId, Origin, Replica, Schema, Tx};
use kition_kit::session::{BlobSink, BlobSource, Damage, KitError, KitModel, Result, write_text};

/// The kit id the schema names.
pub const KIT: &str = "kit.html";
/// The root type and its one field.
pub const PAGE: &str = "Page";
pub const SOURCE: &str = "source";

const SCHEMA: &str = include_str!("../model/schema.json");

/// The `Page` object's id: one per document, written by the replica that creates it, so every
/// reader finds it without an index.
pub fn root_id() -> ObjectId {
    ObjectId::from_string("page")
}

/// The kit's schema.
pub fn schema() -> Arc<Schema> {
    static PARSED: OnceLock<Arc<Schema>> = OnceLock::new();
    PARSED
        .get_or_init(|| {
            #[allow(clippy::expect_used)]
            Arc::new(Schema::parse(SCHEMA).expect("model/schema.json is a fixed, tested file"))
        })
        .clone()
}

/// The materialised view: the page's HTML source.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HtmlModel {
    source: String,
}

impl HtmlModel {
    /// The HTML source as the replica holds it.
    pub fn source(&self) -> &str {
        &self.source
    }

    fn read(replica: &Replica) -> Result<Self> {
        Ok(Self {
            source: replica.text(&root_id(), SOURCE)?,
        })
    }
}

/// The largest page the kit opens or keeps: 8 MiB of markup. A real web page of a few megabytes
/// opens and renders well within the platform's three-second budget for a large file in the
/// release engine; a page of 12 MB takes longer than that.
pub const MAX_SOURCE_BYTES: usize = 8 * 1024 * 1024;

/// Why `source` cannot be a page: too large, not markup, or control characters. A page over
/// [`MAX_SOURCE_BYTES`] is refused with the `too-large:` reason, so a host can say so plainly
/// instead of calling the file damaged.
pub fn validate_source(source: &str) -> std::result::Result<(), String> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(format!(
            "too-large: the page is {:.1} MB; the HTML kit opens pages of at most {} MB",
            source.len() as f64 / (1024.0 * 1024.0),
            MAX_SOURCE_BYTES / (1024 * 1024)
        ));
    }
    let markup = source.trim_start_matches('\u{feff}').trim_start();
    if !markup.is_empty() && !markup.starts_with('<') {
        return Err("expected HTML markup or an empty document".into());
    }
    if source
        .chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
    {
        return Err("HTML contains unsupported control characters".into());
    }
    Ok(())
}

/// A new page replica holding `source`, written in the creating commit (not an undo step).
pub fn create_replica(source: &str, actor: ActorId, origin: &Origin) -> Result<Replica> {
    let (replica, ()) = Replica::create_with(schema(), DocId::new(), actor, origin, |tx| {
        tx.create_with_id(PAGE, &root_id(), serde_json::Map::new())?;
        if !source.is_empty() {
            tx.text_insert(&root_id(), SOURCE, 0, source)?;
        }
        Ok(())
    })?;
    Ok(replica)
}

/// Writes the difference from `current` to `target` as small text edits, so concurrent edits to
/// other parts of the page survive. Answers whether the source changed.
pub fn write_source(tx: &mut Tx<'_>, current: &str, target: &str) -> Result<bool> {
    Ok(!write_text(tx, &root_id(), SOURCE, current, target)?.is_empty())
}

impl KitModel for HtmlModel {
    fn schema() -> Arc<Schema> {
        schema()
    }

    fn load(replica: &Replica, _blobs: &dyn BlobSource) -> Result<Self> {
        Self::read(replica)
    }

    fn apply(
        &mut self,
        replica: &Replica,
        _changed: &BTreeSet<ObjectId>,
        _blobs: &dyn BlobSource,
    ) -> Result<Damage> {
        *self = Self::read(replica)?;
        Ok(Damage::Full)
    }

    /// One text field: nothing a merge can leave inconsistent.
    fn validate(_tx: &mut Tx<'_>, _changed: &BTreeSet<ObjectId>) -> Result<bool> {
        Ok(false)
    }

    /// A new page from an HTML file: UTF-8 markup of at most [`MAX_SOURCE_BYTES`].
    fn import(
        _name: &str,
        bytes: &[u8],
        _blobs: &mut dyn BlobSink,
        actor: ActorId,
    ) -> Result<Replica> {
        let source = std::str::from_utf8(bytes).map_err(|_| KitError::new("HTML must be UTF-8"))?;
        validate_source(source).map_err(KitError::new)?;
        create_replica(source, actor, &Origin::system("import"))
    }

    fn export(&self, _replica: &Replica, _name: &str, _blobs: &dyn BlobSource) -> Result<Vec<u8>> {
        Ok(self.source.as_bytes().to_vec())
    }
}
