//! Browser-free HTML/CSS rasterization shared by native and wasm32 builds.
//! The caller supplies every resource. This crate never reads files or the network: the page's
//! resources come from [`Resources`] (`data:` URLs and `https://kit.invalid/...`), scripts fetch
//! and import only through the same map, and an SVG image draws only pictures written into it
//! (`blitz_dom::util::take_unread_svg_images` lists the others, reported as unavailable).

use anyrender::{PaintScene as _, render_to_buffer};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::{Document, DocumentConfig, util::Color};
use blitz_traits::net::{Bytes, NetHandler, NetProvider, Request};
use blitz_traits::shell::{ColorScheme, Viewport};
use blitz_vibey_script::{FetchError, ScriptDocument, ScriptFetcher};
use parley::FontContext;
use parley::fontique::{Blob, Collection, CollectionOptions, GenericFamily, SourceCache};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};

mod blur;
mod gradients;
mod opacity;
mod paths;
mod scene;
mod session_handle;
pub use session_handle::SessionHandle;

pub const ENGINE_REVISION: &str = "21edc92d90d8f8bedadd3f9ecb14b1ad950952eb";
pub type Resources = BTreeMap<String, Vec<u8>>;

/// Portable defaults shared by the CLI and SDK. Callers may replace this font
/// or add resources before rendering; no operating-system fonts are consulted.
/// The bundled Inter font is distributed under conformance/assets/Inter-OFL.txt.
pub fn default_resources() -> Resources {
    Resources::from([(
        "/assets/Inter-Variable.ttf".into(),
        include_bytes!("../../../conformance/assets/Inter-Variable.ttf").to_vec(),
    )])
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RenderOptions {
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub dark: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            scale: 1.0,
            dark: false,
        }
    }
}

pub struct Rendered {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub diagnostics: Vec<String>,
    pub geometry: Vec<serde_json::Value>,
}

impl Rendered {
    pub fn png(&self) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, self.width, self.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
            writer
                .write_image_data(&self.rgba)
                .map_err(|e| e.to_string())?;
        }
        Ok(bytes)
    }
}

struct ResourceProvider {
    resources: Resources,
    requests: AtomicUsize,
    diagnostics: Mutex<Vec<String>>,
}

impl NetProvider for ResourceProvider {
    fn fetch(&self, _document: usize, request: Request, handler: Box<dyn NetHandler>) {
        self.requests.fetch_add(1, Ordering::SeqCst);
        let url = request.url.to_string();
        let data = if request.url.scheme() == "data" {
            data_url::DataUrl::process(&url)
                .ok()
                .and_then(|value| value.decode_to_vec().ok().map(|(bytes, _)| bytes))
        } else if request.url.scheme() == "https" && request.url.host_str() == Some("kit.invalid") {
            self.resources.get(request.url.path()).cloned()
        } else {
            None
        };
        match data {
            Some(bytes) => handler.bytes(url, Bytes::from(bytes)),
            None => {
                if let Ok(mut diagnostics) = self.diagnostics.lock() {
                    diagnostics.push(format!("resource-unavailable: {url}"));
                }
                // Complete the request, including failed linked stylesheets, so layout cannot hang.
                handler.bytes(url, Bytes::new());
            }
        }
    }
}

/// Fonts a host added for every page of the process ([`add_font`]), in the order added.
static ADDED_FONTS: Mutex<Vec<Arc<Vec<u8>>>> = Mutex::new(Vec::new());
/// How many times [`ADDED_FONTS`] changed: an open page draws again with the new set.
static FONTS_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Adds a font file (TrueType, OpenType or a collection; WOFF when built with it) for every page
/// of the process, as a host hands an engine its faces (Kition platform E9). Answers how many
/// faces it added: 0 for bytes that are no font, or a file added before. The CSS generic families
/// take the faces their family names say ([`FaceRole`]): a serif face serves `serif`, a monospace
/// one `monospace`, an emoji face emoji, and a Chinese, Japanese or Korean face the characters the
/// others lack.
pub fn add_font(bytes: &[u8]) -> usize {
    let bytes = blitz_dom::util::decode_font_bytes(bytes).into_owned();
    let faces = match skrifa::raw::FileRef::new(&bytes) {
        Ok(file) => file.fonts().filter(Result::is_ok).count(),
        Err(_) => 0,
    };
    if faces == 0 {
        return 0;
    }
    let Ok(mut added) = ADDED_FONTS.lock() else {
        return 0;
    };
    if added.iter().any(|known| **known == bytes) {
        return 0;
    }
    added.push(Arc::new(bytes));
    FONTS_GENERATION.fetch_add(1, Ordering::SeqCst);
    faces
}

/// Changes whenever [`add_font`] adds a font.
pub fn fonts_generation() -> u64 {
    FONTS_GENERATION.load(Ordering::SeqCst)
}

/// The fonts a page draws with: its resources' font files (the bundled Inter among them), then
/// the fonts the host added.
fn page_fonts(resources: &Resources) -> Vec<Arc<Vec<u8>>> {
    let mut fonts: Vec<Arc<Vec<u8>>> = resources
        .iter()
        .filter(|(path, _)| {
            path.ends_with(".ttf") || path.ends_with(".otf") || path.ends_with(".ttc")
        })
        .map(|(_, bytes)| Arc::new(bytes.clone()))
        .collect();
    if let Ok(added) = ADDED_FONTS.lock() {
        fonts.extend(added.iter().cloned());
    }
    fonts
}

/// What a face is for, by its family name: the CSS generic families it serves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceRole {
    Sans,
    Serif,
    Mono,
    Emoji,
    /// Chinese, Japanese and Korean: the characters the other faces lack.
    Cjk,
}

/// The role of a face named `family`.
pub fn face_role(family: &str) -> FaceRole {
    let name = family.to_ascii_lowercase();
    let has = |words: &[&str]| words.iter().any(|word| name.contains(word));
    if has(&["emoji"]) {
        FaceRole::Emoji
    } else if has(&["mono", "code", "courier", "cousine", "consol", "menlo"]) {
        FaceRole::Mono
    } else if has(&[
        "fallback", "cjk", " sc", " tc", " jp", " kr", "hei", "song", "ming", "gothic", "mincho",
    ]) {
        FaceRole::Cjk
    } else if has(&["sans"]) {
        FaceRole::Sans
    } else if has(&[
        "serif",
        "lora",
        "tinos",
        "caladea",
        "times",
        "georgia",
        "garamond",
        "merriweather",
        "playfair",
        "baskerville",
    ]) {
        FaceRole::Serif
    } else {
        FaceRole::Sans
    }
}

/// The scripts a Chinese, Japanese or Korean face is the fallback for.
const CJK_SCRIPTS: [[u8; 4]; 5] = [*b"Hani", *b"Hira", *b"Kana", *b"Hang", *b"Bopo"];

fn font_context(resources: &Resources) -> FontContext {
    let mut context = FontContext {
        source_cache: SourceCache::new_shared(),
        collection: Collection::new(CollectionOptions {
            shared: false,
            system_fonts: false,
        }),
    };
    let mut cjk = Vec::new();
    for bytes in page_fonts(resources) {
        let registered = context.collection.register_fonts(Blob::new(bytes), None);
        for (id, _) in registered {
            let role = context
                .collection
                .family_name(id)
                .map_or(FaceRole::Sans, face_role);
            let generics: &[GenericFamily] = match role {
                FaceRole::Sans => &[
                    GenericFamily::SansSerif,
                    GenericFamily::SystemUi,
                    GenericFamily::UiSansSerif,
                ],
                FaceRole::Serif => &[GenericFamily::Serif, GenericFamily::UiSerif],
                FaceRole::Mono => &[GenericFamily::Monospace, GenericFamily::UiMonospace],
                FaceRole::Emoji => &[GenericFamily::Emoji],
                FaceRole::Cjk => {
                    cjk.push(id);
                    &[]
                }
            };
            for generic in generics {
                context
                    .collection
                    .append_generic_families(*generic, std::iter::once(id));
            }
        }
    }
    // The Chinese, Japanese and Korean faces follow every generic family's own faces, and are the
    // fallback of their scripts for named families that lack them.
    for generic in [
        GenericFamily::SansSerif,
        GenericFamily::SystemUi,
        GenericFamily::UiSansSerif,
        GenericFamily::Serif,
        GenericFamily::UiSerif,
        GenericFamily::Monospace,
        GenericFamily::UiMonospace,
    ] {
        context
            .collection
            .append_generic_families(generic, cjk.iter().copied());
    }
    for script in CJK_SCRIPTS {
        for locale in [
            None,
            Some("zh-CN"),
            Some("zh-TW"),
            Some("zh-HK"),
            Some("ja"),
            Some("ko"),
        ] {
            let script = parley::fontique::Script::from_bytes(script);
            let key = match locale {
                Some(locale) => parley::fontique::FallbackKey::from((script, locale)),
                None => parley::fontique::FallbackKey::from(script),
            };
            context
                .collection
                .append_fallbacks(key, cjk.iter().copied());
        }
    }
    context
}

/// The fonts SVG text draws with: the page's ([`page_fonts`]), never the machine's, its generic
/// families the faces that serve them in HTML text ([`face_role`]; Inter for any without one).
fn svg_fonts(resources: &Resources) -> Arc<usvg::fontdb::Database> {
    let mut db = usvg::fontdb::Database::new();
    for bytes in page_fonts(resources) {
        db.load_font_source(usvg::fontdb::Source::Binary(bytes));
    }
    let mut first: BTreeMap<&'static str, String> = BTreeMap::new();
    for face in db.faces() {
        let Some((family, _)) = face.families.first() else {
            continue;
        };
        let role = match face_role(family) {
            FaceRole::Serif => "serif",
            FaceRole::Mono => "monospace",
            FaceRole::Sans => "sans-serif",
            FaceRole::Emoji | FaceRole::Cjk => continue,
        };
        first.entry(role).or_insert_with(|| family.clone());
    }
    let sans = first
        .get("sans-serif")
        .cloned()
        .unwrap_or_else(|| "Inter".to_owned());
    db.set_sans_serif_family(sans.clone());
    db.set_serif_family(first.get("serif").cloned().unwrap_or_else(|| sans.clone()));
    db.set_monospace_family(
        first
            .get("monospace")
            .cloned()
            .unwrap_or_else(|| sans.clone()),
    );
    db.set_cursive_family(sans.clone());
    db.set_fantasy_family(sans);
    Arc::new(db)
}

/// Render the initial frame, including scripts and the first animation frame.
/// Source bytes are never rewritten by JavaScript DOM mutations.
pub fn render(
    html: &str,
    resources: Resources,
    options: &RenderOptions,
) -> Result<Rendered, String> {
    Session::new(html, resources, options)?.frame(options.scale)
}

fn dimensions(options: &RenderOptions) -> Result<(u32, u32), String> {
    if options.width == 0
        || options.height == 0
        || !options.scale.is_finite()
        || options.scale <= 0.0
        || options.scale > 4.0
    {
        return Err("invalid viewport or scale (expected 0 < scale <= 4)".into());
    }
    let width = (f64::from(options.width) * options.scale).round();
    let height = (f64::from(options.height) * options.scale).round();
    if width < 1.0
        || height < 1.0
        || width > 16384.0
        || height > 16384.0
        || width * height > 16_777_216.0
    {
        return Err("render target must contain 1 to 16,777,216 pixels".into());
    }
    Ok((width as u32, height as u32))
}

/// A live DOM and script context. It stays on its creating thread.
pub struct Session {
    document: ScriptDocument,
    provider: Arc<ResourceProvider>,
    options: RenderOptions,
    diagnostics: Vec<String>,
    timers_paused: bool,
}

impl Session {
    pub fn new(html: &str, resources: Resources, options: &RenderOptions) -> Result<Self, String> {
        if html.len() > 16 * 1024 * 1024 {
            return Err("HTML exceeds the 16 MiB input limit".into());
        }
        let (width, height) = dimensions(options)?;
        // Enable the selector implementation before parsing author stylesheets.
        style_config::set_pref!("layout.css.has-selector.enabled", true);
        style_config::set_pref!("layout.css.appearance.enabled", true);
        style_config::set_pref!("layout.css.font-synthesis.enabled", true);
        style_config::set_pref!("layout.css.writing-mode.sideways.enabled", true);
        style_config::set_pref!("layout.css.container-queries.enabled", true);
        style_config::set_pref!("layout.css.grid-template-subgrid-value.enabled", true);
        style_config::set_pref!("layout.css.columns.enabled", true);
        let fonts = font_context(&resources);
        let svg_fonts = svg_fonts(&resources);
        let provider = Arc::new(ResourceProvider {
            resources,
            requests: AtomicUsize::new(0),
            diagnostics: Mutex::new(Vec::new()),
        });
        let document = ScriptDocument::from_html(
            html,
            DocumentConfig {
                base_url: Some("https://kit.invalid/cases/document.html".into()),
                viewport: Some(Viewport::new(
                    width,
                    height,
                    options.scale as f32,
                    if options.dark {
                        ColorScheme::Dark
                    } else {
                        ColorScheme::Light
                    },
                )),
                font_ctx: Some(fonts),
                svg_fonts: Some(svg_fonts),
                net_provider: Some(provider.clone()),
                ..Default::default()
            },
        );
        let document = document
            .without_timer_thread()
            .with_canvas_scene_preparer(|scene, width, height| {
                opacity::prepare(scene);
                paths::prepare(scene);
                gradients::prepare_gradients(scene, width, height);
            })
            .with_virtual_time()
            .with_fetcher(ScriptResources(provider.clone()));
        let mut session = Self {
            document,
            provider,
            options: options.clone(),
            diagnostics: Vec::new(),
            timers_paused: false,
        };
        session.settle()?;
        session.document.execute_scripts();
        session.advance(16)?;
        Ok(session)
    }

    fn settle(&mut self) -> Result<(), String> {
        let provider = &self.provider;
        let document = &mut self.document;
        let mut settled = false;
        for _ in 0..32 {
            let before = provider.requests.load(Ordering::SeqCst);
            document.inner_mut().resolve(0.0);
            if before == provider.requests.load(Ordering::SeqCst) {
                // Consume responses from style-driven requests before painting.
                document.inner_mut().resolve(0.0);
                if before == provider.requests.load(Ordering::SeqCst) {
                    settled = true;
                    break;
                }
            }
        }
        if !settled {
            return Err("resource resolution exceeded 32 passes".into());
        }
        Ok(())
    }

    /// Drive timers in deadline order without wall-clock sleeps. Each call is bounded.
    pub fn advance(&mut self, millis: u64) -> Result<bool, String> {
        if millis > 1000 {
            return Err("time step must not exceed 1000 ms".into());
        }
        if self.timers_paused {
            return Ok(false);
        }
        let target = self.document.clock_now() + std::time::Duration::from_millis(millis);
        let mut changed = false;
        for _ in 0..128 {
            let Some(deadline) = self
                .document
                .next_timer_deadline()
                .filter(|time| *time <= target)
            else {
                self.document.advance_clock_to(target);
                return Ok(changed);
            };
            self.document.advance_clock_to(deadline);
            changed |= self.document.poll(None);
        }
        self.timers_paused = true;
        self.diagnostics
            .push("html.javascript-timer-limit: paused after 128 timer batches".into());
        Ok(changed)
    }

    pub fn click(&mut self, x: f32, y: f32) -> Result<bool, String> {
        if !x.is_finite() || !y.is_finite() {
            return Err("invalid click coordinates".into());
        }
        self.settle()?;
        let event = {
            let doc = self.document.inner();
            doc.element_from_point(x, y)
                .and_then(|id| doc.get_node(id))
                .map(|node| {
                    let mut event = node.synthetic_click_event_data(Default::default());
                    event.coords.client_x = x;
                    event.coords.client_y = y;
                    event.coords.page_x = x;
                    event.coords.page_y = y;
                    event.coords.screen_x = x;
                    event.coords.screen_y = y;
                    blitz_traits::events::DomEvent::new(
                        node.id,
                        blitz_traits::events::DomEventData::Click(event),
                    )
                })
        };
        if let Some(event) = event {
            self.document.dispatch_dom_event(event);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn frame(&mut self, scale: f64) -> Result<Rendered, String> {
        let options = RenderOptions {
            scale,
            ..self.options.clone()
        };
        let (width, height) = dimensions(&options)?;
        self.document.inner_mut().set_viewport(Viewport::new(
            width,
            height,
            scale as f32,
            if options.dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
        ));
        self.settle()?;
        self.diagnostics.extend(
            self.document
                .take_js_errors()
                .into_iter()
                .map(|error| format!("html.javascript-error: {error}")),
        );
        // Pictures SVG images link by anything but a `data:` URL are never read (KHP A29, A37):
        // like any resource the page cannot load, they are reported.
        self.diagnostics.extend(
            blitz_dom::util::take_unread_svg_images()
                .into_iter()
                .map(|href| format!("resource-unavailable: {href}")),
        );
        self.diagnostics.sort();
        self.diagnostics.dedup();
        self.diagnostics.truncate(256);
        let document = &mut self.document;
        let provider = &self.provider;
        let geometry = document
        .inner()
        .query_selector_all("#stage *")
        .map_err(|_| "geometry query failed")?
        .into_iter()
        .filter_map(|id| {
            let dom = document.inner();
            let element = dom.get_node(id)?.element_data()?;
            let tag = element.name.local.to_string().to_uppercase();
            if matches!(tag.as_str(), "SCRIPT" | "STYLE" | "DEFS") {
                return None;
            }
            let rect = dom.get_client_bounding_rect(id)?;
            Some(
                serde_json::json!({"tag":tag,
                    "id":element.attrs.iter().find(|attr|attr.name.local.to_string()=="id").map(|attr|attr.value.to_string()),
                    "class":element.attrs.iter().find(|attr|attr.name.local.to_string()=="class").map(|attr|attr.value.to_string()),
                "x":rect.x,"y":rect.y,"width":rect.width,"height":rect.height}),
            )
        })
        .collect();
        let mut recording = anyrender::Scene::new();
        recording.fill(
            peniko::Fill::NonZero,
            Default::default(),
            Color::WHITE,
            Default::default(),
            &peniko::kurbo::Rect::new(0.0, 0.0, f64::from(width), f64::from(height)),
        );
        let paint_diagnostics = blitz_paint::paint_scene_with_diagnostics(
            &mut recording,
            &mut document.inner_mut(),
            options.scale,
            width,
            height,
            0,
            0,
        );
        let mut diagnostics = provider
            .diagnostics
            .lock()
            .map_err(|_| "resource diagnostics unavailable")?
            .clone();
        diagnostics.extend(paint_diagnostics);
        if document.inner().layout_dependencies_unstable() {
            // Size containment keeps container queries from cycling, so no page within the limits
            // gets here; if the engine does, the render fails rather than draw a layout it knows
            // is unsettled.
            return Err("layout did not settle: container sizes or inherited grid tracks changed in every one of 32 style/layout passes".into());
        }
        diagnostics.extend(self.diagnostics.clone());
        opacity::prepare(&mut recording);
        paths::prepare(&mut recording);
        gradients::prepare_gradients(&mut recording, width, height);
        scene::prepare_filters(&mut recording, width, height, &mut diagnostics)?;
        let rgba = render_to_buffer::<VelloCpuImageRenderer, _>(
            |scene| scene.append_scene(recording, peniko::kurbo::Affine::IDENTITY),
            width,
            height,
        );
        diagnostics.sort();
        diagnostics.dedup();
        Ok(Rendered {
            width,
            height,
            rgba,
            diagnostics,
            geometry,
        })
    }
}

struct ScriptResources(Arc<ResourceProvider>);
impl ScriptFetcher for ScriptResources {
    fn fetch(&self, url: &url::Url) -> Result<String, FetchError> {
        let bytes = if url.scheme() == "data" {
            data_url::DataUrl::process(url.as_str())
                .ok()
                .and_then(|data| data.decode_to_vec().ok().map(|(bytes, _)| bytes))
        } else if url.scheme() == "https"
            && url.host_str() == Some("kit.invalid")
            && url.port().is_none()
            && url.username().is_empty()
            && url.password().is_none()
        {
            self.0.resources.get(url.path()).cloned()
        } else {
            None
        };
        let bytes =
            bytes.ok_or_else(|| FetchError::InvalidData(format!("resource-unavailable: {url}")))?;
        if bytes.len() > 1024 * 1024 {
            return Err(FetchError::InvalidData(
                "script resource exceeds 1 MiB".into(),
            ));
        }
        String::from_utf8(bytes)
            .map_err(|_| FetchError::InvalidData("script resource must be UTF-8".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_invalid_and_excessive_render_targets() {
        for options in [
            RenderOptions {
                width: 0,
                ..Default::default()
            },
            RenderOptions {
                scale: f64::NAN,
                ..Default::default()
            },
            RenderOptions {
                width: u32::MAX,
                ..Default::default()
            },
        ] {
            assert!(render("<div></div>", Resources::new(), &options).is_err());
        }
    }

    #[test]
    fn paints_geometry_without_browser_or_gpu() -> Result<(), String> {
        let output = render(
            "<!doctype html><style>body{margin:0}div{width:20px;height:10px;background:#5645d4}</style><div></div>",
            Resources::new(),
            &RenderOptions {
                width: 32,
                height: 24,
                ..Default::default()
            },
        )?;
        assert_eq!(&output.rgba[0..4], &[86, 69, 212, 255]);
        assert_eq!(&output.rgba[31 * 4..32 * 4], &[255, 255, 255, 255]);
        assert!(output.png()?.starts_with(b"\x89PNG\r\n\x1a\n"));
        Ok(())
    }
}
