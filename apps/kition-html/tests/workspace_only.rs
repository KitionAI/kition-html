//! `kition-html --kition-kit` reads nothing outside its workspace and connects to nothing (KHP
//! A29, A37, A42, section 10.6): a page is the agent's data, and whatever it links (images,
//! stylesheets, scripts, frames, fonts, a `<base>`, `file:` URLs, absolute and `../` paths, SVG
//! pictures, links inside an SVG written into a `data:` URL, and what its scripts fetch, import
//! or add), the preview loads nothing but the page's own resources: no file of the machine is
//! opened, no server asked, and each picture not drawn is reported (`html.resource-unavailable`).
//! A workspace-escape audit found by reading code that an SVG `<image href>` was read by path
//! (usvg's default resolver, relative to the process's folder), so `html.setSource` and a preview
//! showed a picture from anywhere.
//!
//! The test runs itself again as a child process whose current folder, `HOME` and `TMPDIR` are
//! empty scratch folders, beside a planted `outside/` folder with a picture in a colour nothing
//! else uses and named pipes (opening one to read is seen even when nothing shows it), and a
//! localhost server.

#![cfg(unix)]

use kition_headless::{Headless, HostFolder, KitInfo};
use kition_html_kit::HtmlDocument;
use kition_kit_server::protocol::{Origin, ScreenshotScope};
use kition_kit_server::{KitBackend, OpenRequest, ScreenshotRequest};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Set in the child: the folder holding `workspace/`, `outside/`, and the scratch folders.
const CHILD: &str = "KITION_HTML_TEST_SANDBOX";
/// Set in the child: the port of the server no page may reach.
const PORT: &str = "KITION_HTML_TEST_PORT";
/// The colour of the picture outside the workspace.
const OUTSIDE: [u8; 3] = [23, 211, 97];
/// Named pipes outside the workspace that the page names instead of a file.
const PIPES: [&str; 3] = ["pipe.png", "pipe.svg", "pipe-nested.png"];

/// Every file under `dir`, relative to it, with its bytes and modification time.
fn tree(dir: &Path) -> BTreeMap<String, (Vec<u8>, Option<std::time::SystemTime>)> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        for entry in std::fs::read_dir(&next).into_iter().flatten().flatten() {
            let path = entry.path();
            let Ok(meta) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.is_dir() {
                stack.push(path.clone());
            }
            let bytes = if meta.is_file() {
                std::fs::read(&path).unwrap_or_default()
            } else {
                Vec::new()
            };
            let name = path
                .strip_prefix(dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            out.insert(name, (bytes, meta.modified().ok()));
        }
    }
    out
}

fn at(dir: &Path, name: &str) -> String {
    dir.join(name).to_string_lossy().into_owned()
}

/// A one-colour 8 x 8 PNG.
fn png(rgb: [u8; 3]) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, 8, 8);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&rgb.repeat(64))?;
    }
    Ok(bytes)
}

/// Whether a PNG shows a pixel within 3 of `rgb`.
fn shows(bytes: &[u8], rgb: [u8; 3]) -> Result<bool> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info()?;
    let mut buffer = vec![0; reader.output_buffer_size().ok_or("no buffer size")?];
    let info = reader.next_frame(&mut buffer)?;
    let channels = info.color_type.samples();
    let pixels = buffer.get(..info.buffer_size()).ok_or("short frame")?;
    Ok(pixels
        .chunks(channels)
        .any(|px| px.len() >= 3 && (0..3).all(|c| px[c].abs_diff(rgb[c]) <= 3)))
}

/// `text` as base64.
fn base64(text: &str) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in text.as_bytes().chunks(3) {
        let at = |i: usize| u32::from(chunk.get(i).copied().unwrap_or(0));
        let n = (at(0) << 16) | (at(1) << 8) | at(2);
        for k in 0..4 {
            let digit = TABLE[((n >> (18 - 6 * k)) & 63) as usize];
            out.push(if k <= chunk.len() {
                char::from(digit)
            } else {
                '='
            });
        }
    }
    out
}

/// A page whose every kind of link names a file outside the workspace or the server at `port`.
fn hostile(outside: &Path, port: u16) -> String {
    let file = |name: &str| at(outside, name);
    let server = format!("http://127.0.0.1:{port}");
    let image = |x: u32, href: &str| {
        format!(r#"<image x="{x}" y="0" width="20" height="20" href="{href}"/>"#)
    };
    let nested = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20">{}</svg>"#,
        image(0, &file("pipe-nested.png"))
    );
    let pictures = [
        image(0, &file("secret.png")),
        image(20, "../outside/pipe.png"),
        image(40, &format!("file://{}", file("secret.png"))),
        image(60, &format!("{server}/svg.png")),
        format!(
            r#"<image x="80" y="0" width="20" height="20" href="data:image/svg+xml;base64,{}"/>"#,
            base64(&nested)
        ),
    ]
    .concat();
    let script = format!(
        "fetch('{server}/fetch').catch(() => {{}}); import('{server}/module.js').catch(() => {{}}); \
         const picture = new Image(); picture.src = '{server}/image.png'; \
         document.getElementById('added').innerHTML = '<svg xmlns=\"http://www.w3.org/2000/svg\" \
         width=\"20\" height=\"20\"><image width=\"20\" height=\"20\" href=\"{}\"/></svg>';",
        file("pipe.svg")
    );
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><base href="{server}/">
<link rel="stylesheet" href="{server}/a.css"><link rel="stylesheet" href="file://{css}">
<style>@import url("{server}/b.css"); @font-face {{ font-family: Far; src: url("{server}/f.woff2"), url("file://{font}"); }}
body {{ background: url("file://{secret}") }} p {{ font-family: Far }}</style>
<script src="{server}/a.js"></script></head><body><main id="stage">
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="20">{pictures}</svg>
<img src="{secret}" width="20" height="20"><img src="file://{secret}" width="20" height="20">
<img srcset="{server}/s.png 1x, file://{secret} 2x" width="20" height="20">
<img src="data:image/svg+xml;base64,{nested64}" width="20" height="20">
<iframe src="{server}/frame.html"></iframe><object data="file://{secret}"></object><embed src="{server}/e">
<div id="added"></div><p>Text</p></main><script>{script}</script></body></html>"#,
        css = file("secret.css"),
        font = file("secret.ttf"),
        secret = file("secret.png"),
        nested64 = base64(&nested),
    )
}

/// Watches the [`PIPES`]: a reader that opens one is let through (the pipe is opened to write and
/// closed, so the reader reads an empty file and goes on) and recorded.
struct PipeWatcher {
    stop: Arc<std::sync::atomic::AtomicBool>,
    thread: std::thread::JoinHandle<Vec<String>>,
}

impl PipeWatcher {
    fn start(outside: &Path) -> Self {
        use std::os::unix::fs::OpenOptionsExt;
        #[cfg(target_os = "macos")]
        const O_NONBLOCK: i32 = 0x0004;
        #[cfg(not(target_os = "macos"))]
        const O_NONBLOCK: i32 = 0o4000;
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let pipes: Vec<PathBuf> = PIPES.iter().map(|pipe| outside.join(pipe)).collect();
        let stopped = Arc::clone(&stop);
        let thread = std::thread::spawn(move || {
            let mut opened = Vec::new();
            while !stopped.load(std::sync::atomic::Ordering::SeqCst) {
                for pipe in &pipes {
                    // Opening a pipe to write without waiting succeeds only while a reader has it.
                    let mut options = std::fs::OpenOptions::new();
                    options.write(true).custom_flags(O_NONBLOCK);
                    if options.open(pipe).is_ok() {
                        let name = pipe
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned());
                        opened.push(name.unwrap_or_default());
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            opened
        });
        Self { stop, thread }
    }

    /// The pipes a reader opened, once each time.
    fn stop(self) -> Vec<String> {
        self.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        self.thread.join().unwrap_or_default()
    }
}

fn params(value: Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}

/// What the agent does in the child: puts the hostile page in, previews and checks it.
fn act(root: &Path) -> Result<Vec<String>> {
    let workspace = root.join("workspace");
    let outside = root.join("outside");
    let port: u16 = std::env::var(PORT)?.parse()?;
    const MANIFEST: &str = include_str!("../../../kit.json");
    let info = KitInfo::from_manifest(MANIFEST, &[], &["width", "height"])?;
    let mut kit = Headless::<HtmlDocument>::with_replica_port(info);
    let folder = HostFolder::new(&workspace).map_err(|e| format!("{e:?}"))?;
    kit.set_host(folder.host());
    let file = folder.file("page.html").map_err(|e| format!("{e:?}"))?;
    let request = OpenRequest {
        file: &file,
        read_only: false,
        create: None,
    };
    let id = kit.open(request).map_err(|e| format!("{e:?}"))?.id;
    let mut wrong = Vec::new();

    let source = json!({ "source": hostile(&outside, port) });
    let set = kit.execute(&id, "html.setSource", params(source.clone()), Origin::Agent);
    if let Err(error) = set {
        wrong.push(format!("html.setSource refused the page: {error:?}"));
    }
    let again = format!(
        "{}<!-- again -->",
        source["source"].as_str().unwrap_or_default()
    );
    let op = json!({ "op": "command", "id": "html.setSource", "params": { "source": again } });
    let batch = json!({ "documentId": id, "ops": [op] });
    let _ = kit.model("model.batch", batch);
    for millis in [16, 500, 1000] {
        let step = params(json!({ "millis": millis }));
        let _ = kit.execute(&id, "html.advance", step, Origin::Agent);
    }
    let shot = kit
        .screenshot(
            &id,
            &ScreenshotRequest {
                scope: ScreenshotScope::Canvas,
                max_size: 1024,
                scale: 1.0,
                target: None,
            },
        )
        .map_err(|e| format!("{e:?}"))?;
    if shows(&shot.png, OUTSIDE)? {
        wrong.push("the preview shows the picture outside the workspace".into());
    }
    let checked = kit.check(&id).map_err(|e| format!("{e:?}"))?.to_string();
    // The SVG pictures the page links by path are reported as resources it cannot load.
    for href in [at(&outside, "secret.png"), "../outside/pipe.png".to_owned()] {
        if !checked.contains(&href) {
            wrong.push(format!(
                "the check does not report `{href}` as unavailable: {checked}"
            ));
        }
    }
    let _ = kit.save(&id, None, None);
    Ok(wrong)
}

#[test]
fn a_page_reads_and_fetches_nothing_outside_the_workspace() -> Result {
    if let Some(root) = std::env::var_os(CHILD) {
        let wrong = act(&PathBuf::from(root))?;
        assert!(wrong.is_empty(), "{wrong:#?}");
        return Ok(());
    }
    let name = format!("kition-html-workspace-only-{}", std::process::id());
    let root = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&root);
    for dir in ["workspace", "cwd", "home", "tmp", "outside"] {
        std::fs::create_dir_all(root.join(dir))?;
    }
    let root = std::fs::canonicalize(root)?;
    let (workspace, outside) = (root.join("workspace"), root.join("outside"));
    let server = std::net::TcpListener::bind("127.0.0.1:0")?;
    server.set_nonblocking(true)?;
    let port = server.local_addr()?.port();
    std::fs::write(outside.join("secret.png"), png(OUTSIDE)?)?;
    std::fs::write(
        outside.join("secret.css"),
        "body { background: rgb(23, 211, 97) }",
    )?;
    for pipe in PIPES {
        let made = std::process::Command::new("mkfifo")
            .arg(outside.join(pipe))
            .status()?;
        assert!(made.success(), "mkfifo {pipe}");
    }
    std::fs::write(workspace.join("page.html"), "<!doctype html><p>Start</p>")?;

    let planted = tree(&outside);
    let watcher = PipeWatcher::start(&outside);
    let status = std::process::Command::new(std::env::current_exe()?)
        .args([
            "--exact",
            "a_page_reads_and_fetches_nothing_outside_the_workspace",
        ])
        .args(["--nocapture", "--test-threads=1"])
        .current_dir(root.join("cwd"))
        .env(CHILD, &root)
        .env(PORT, port.to_string())
        .env("HOME", root.join("home"))
        .env("TMPDIR", root.join("tmp"))
        .env("XDG_CONFIG_HOME", root.join("home/.config"))
        .env("XDG_DATA_HOME", root.join("home/.local/share"))
        .env("XDG_CACHE_HOME", root.join("home/.cache"))
        .status()?;
    let mut wrong = Vec::new();
    let opened = watcher.stop();
    if !opened.is_empty() {
        wrong.push(format!(
            "the kit opened files outside the workspace to read them: {opened:?}"
        ));
    }
    if !status.success() {
        wrong.push(format!(
            "the agent's session reached outside the workspace: {status}"
        ));
    }
    for dir in ["cwd", "home", "tmp"] {
        let found: Vec<String> = tree(&root.join(dir)).into_keys().collect();
        if !found.is_empty() {
            wrong.push(format!("the kit wrote into its {dir} folder: {found:?}"));
        }
    }
    if tree(&outside) != planted {
        wrong.push("the folder outside the workspace changed".into());
    }
    let found: Vec<String> = tree(&workspace).into_keys().collect();
    if found != ["page.html"] {
        wrong.push(format!(
            "the workspace holds {found:?}, not only the page the host saved"
        ));
    }
    let asked = server.accept();
    if !matches!(&asked, Err(error) if error.kind() == std::io::ErrorKind::WouldBlock) {
        wrong.push(format!(
            "the kit connected to a server the page named (KHP A37): {asked:?}"
        ));
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
