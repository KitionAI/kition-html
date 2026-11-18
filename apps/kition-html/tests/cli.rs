//! Exercise the executable boundary, including pipes and export preservation.
use std::{
    fs,
    io::Write,
    process::{Command, Output, Stdio},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn run(args: &[&str], input: &[u8]) -> Result<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kition-html"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing input pipe")?
        .write_all(input)?;
    Ok(child.wait_with_output()?)
}

#[test]
fn stdin_png_exports_preserve_existing_files() -> Result {
    let folder = std::env::temp_dir().join(format!("kition-html-cli-{}", std::process::id()));
    fs::create_dir_all(&folder)?;
    let png = folder.join("stdin.png");
    let args = [
        "render",
        "-",
        "--width",
        "8",
        "--height",
        "4",
        "--output",
        png.to_str().ok_or("non-UTF-8 temporary directory")?,
    ];
    let first = run(&args, b"<style>body{margin:0;background:red}</style>")?;
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&first.stdout)?;
    assert_eq!(
        (report["width"].as_u64(), report["height"].as_u64()),
        (Some(8), Some(4))
    );
    let bytes = fs::read(&png)?;
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    assert!(
        !run(&args, b"<style>body{background:blue}</style>")?
            .status
            .success()
    );
    assert_eq!(fs::read(&png)?, bytes);
    fs::remove_file(png)?;
    fs::remove_dir(folder)?;
    Ok(())
}

#[test]
fn terminal_preview_supports_plain_pipes_color_and_diagnostics() -> Result {
    let html = b"<style>body{margin:0;background:black}</style><script>throw new Error('script fixture')</script>";
    let plain = run(
        &[
            "preview",
            "-",
            "--width",
            "4",
            "--height",
            "4",
            "--columns",
            "2",
        ],
        html,
    )?;
    assert!(
        plain.status.success(),
        "{}",
        String::from_utf8_lossy(&plain.stderr)
    );
    assert_eq!(plain.stdout, b"@@\n");
    assert!(String::from_utf8_lossy(&plain.stderr).contains("script"));
    let color = run(
        &[
            "preview", "-", "--width", "1", "--height", "2", "--color", "always",
        ],
        html,
    )?;
    assert!(color.status.success());
    assert_eq!(
        color.stdout,
        "\x1b[38;2;0;0;0m\x1b[48;2;0;0;0m\u{2580}\x1b[0m\n".as_bytes()
    );
    assert!(
        !run(&["preview", "-", "--columns", "0"], b"")?
            .status
            .success()
    );
    Ok(())
}

#[test]
fn plain_html_has_visible_text_without_an_asset_directory() -> Result {
    let html=b"<style>body{margin:0;font:24px/32px Inter;color:black;background:white}</style>Readable HTML";
    let result = run(
        &[
            "preview",
            "-",
            "--width",
            "240",
            "--height",
            "40",
            "--columns",
            "120",
            "--color",
            "never",
        ],
        html,
    )?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        result
            .stdout
            .iter()
            .filter(|byte| !byte.is_ascii_whitespace())
            .count()
            > 40,
        "default font produced an empty preview"
    );
    assert!(
        result.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
