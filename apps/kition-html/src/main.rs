//! Native boundary: render explicitly supplied resources to a PNG or terminal.
use clap::{Args, Parser, Subcommand, ValueEnum};
use kition_html_engine::{RenderOptions, Rendered, Resources, default_resources, render};
use std::{
    fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(version, about = "Kition HTML browser-free rendering prototype")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

mod preview;

#[derive(Args)]
struct RenderInput {
    /// HTML file, or - to read UTF-8 HTML from standard input.
    input: PathBuf,
    /// Directory exposed as /assets; no other external resources are read.
    #[arg(long)]
    assets: Option<PathBuf>,
    #[arg(long, default_value_t = 640)]
    width: u32,
    #[arg(long, default_value_t = 480)]
    height: u32,
    #[arg(long, default_value_t = 1.0)]
    scale: f64,
    #[arg(long)]
    dark: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
enum Command {
    /// Render a static HTML snapshot to PNG without a browser or GPU.
    Render {
        #[command(flatten)]
        input: RenderInput,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Preview HTML directly in a terminal using color half-blocks or ASCII.
    Preview {
        #[command(flatten)]
        input: RenderInput,
        /// Maximum terminal columns; the image aspect ratio is preserved.
        #[arg(long, default_value_t = 80, value_parser = clap::value_parser!(u32).range(1..=512))]
        columns: u32,
        #[arg(long, default_value_t = 60, value_parser = clap::value_parser!(u32).range(1..=256))]
        max_rows: u32,
        /// Auto enables 24-bit color only on a terminal without NO_COLOR.
        #[arg(long, value_enum, default_value_t = ColorChoice::Auto)]
        color: ColorChoice,
    },
}

fn read_html(reader: impl Read) -> Result<String, String> {
    const LIMIT: u64 = 16 * 1024 * 1024;
    let mut bytes = Vec::new();
    reader
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err("HTML exceeds 16 MiB".into());
    }
    String::from_utf8(bytes).map_err(|_| "HTML must be valid UTF-8".into())
}

fn render_input(input: RenderInput) -> Result<Rendered, String> {
    let html = if input.input == Path::new("-") {
        read_html(io::stdin().lock())?
    } else {
        read_html(fs::File::open(input.input).map_err(|e| e.to_string())?)?
    };
    let mut resources = default_resources();
    if let Some(root) = input.assets {
        collect_assets(&root, &root, &mut resources, &mut 0)?;
    }
    render(
        &html,
        resources,
        &RenderOptions {
            width: input.width,
            height: input.height,
            scale: input.scale,
            dark: input.dark,
        },
    )
}

fn collect_assets(
    root: &Path,
    folder: &Path,
    resources: &mut Resources,
    total: &mut u64,
) -> Result<(), String> {
    for item in fs::read_dir(folder).map_err(|e| e.to_string())? {
        let item = item.map_err(|e| e.to_string())?;
        let kind = item.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err("asset bundles must not contain symbolic links".into());
        }
        if kind.is_dir() {
            collect_assets(root, &item.path(), resources, total)?;
        } else if kind.is_file() {
            *total += item.metadata().map_err(|e| e.to_string())?.len();
            if *total > 64 * 1024 * 1024 {
                return Err("asset bundle exceeds 64 MiB".into());
            }
            let path = item.path();
            let relative = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            resources.insert(
                format!("/assets/{relative}"),
                fs::read(path).map_err(|e| e.to_string())?,
            );
        }
    }
    Ok(())
}

fn run() -> Result<(), String> {
    match Cli::parse().command {
        Command::Render { input, output } => {
            let rendered = render_input(input)?;
            let bytes = rendered.png()?;
            // Exports never replace existing files.
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)
                .map_err(|e| e.to_string())?;
            file.write_all(&bytes).map_err(|e| e.to_string())?;
            println!(
                "{}",
                serde_json::json!({
                    "width":rendered.width, "height":rendered.height, "geometry":rendered.geometry,
                    "engineRevision":kition_html_engine::ENGINE_REVISION, "diagnostics":rendered.diagnostics
                })
            );
        }
        Command::Preview {
            input,
            columns,
            max_rows,
            color,
        } => {
            let rendered = render_input(input)?;
            for diagnostic in &rendered.diagnostics {
                eprintln!("{diagnostic}");
            }
            let stdout = io::stdout();
            let color = match color {
                ColorChoice::Auto => stdout.is_terminal() && std::env::var_os("NO_COLOR").is_none(),
                ColorChoice::Always => true,
                ColorChoice::Never => false,
            };
            let mut output = io::BufWriter::new(stdout.lock());
            let result = preview::write(&mut output, &rendered, columns, max_rows, color)
                .and_then(|()| output.flush());
            if let Err(error) = result
                && error.kind() != io::ErrorKind::BrokenPipe
            {
                return Err(error.to_string());
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    if std::env::args().nth(1).as_deref() == Some("--kition-kit") {
        return kition_headless::run_manifest_model::<kition_html_kit::HtmlDocument>(
            include_str!("../../../kit.json"),
            &[],
            kition_html_kit::EDITOR_STATE,
        );
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_is_bounded_and_requires_utf8() -> Result<(), String> {
        assert!(read_html(io::repeat(b'x').take(16 * 1024 * 1024 + 1)).is_err());
        assert!(read_html(&[0xff][..]).is_err());
        assert_eq!(read_html(&b"<p>Hello</p>"[..])?, "<p>Hello</p>");
        Ok(())
    }
}
