use std::path::PathBuf;
use std::{env, fs};

use anyhow::{anyhow, Context, Result};
use fbink_rs::{FbInk, FbInkConfig};
use resvg::tiny_skia;
use resvg::usvg::{self, fontdb};

const USAGE: &str = "Usage: ./print-svg <SVG_FILE> <FONT_DIR>";

// Displays an SVG file on screen, searching the current or provided directory for fonts
// build with `cross -v build --profile release-minsized --target armv7-unknown-linux-musleabihf`
fn main() -> Result<()> {
    let (svg_path, font_dir) = parse_args()?;
    let svg_data = fs::read(&svg_path)
        .with_context(|| format!("Failed to read SVG file {}", svg_path.display()))?;
    let config = FbInkConfig {
        is_flashing: true,
        is_cleared: true,
        ..Default::default()
    };
    let fbink = FbInk::new(config).context("Failed to initialize FBInk")?;
    let state = fbink.state();
    let width = state.view_width;
    let height = state.view_height;

    let tree = {
        let mut fontdb = fontdb::Database::new();
        fontdb.load_fonts_dir(&font_dir);
        if fontdb.is_empty() {
            eprintln!("Warning: no fonts present in {}", font_dir.display());
        }
        let opts = usvg::Options {
            fontdb: std::sync::Arc::new(fontdb),
            ..Default::default()
        };
        usvg::Tree::from_data(&svg_data, &opts).context("Failed to create SVG tree")?
    };

    let mut pixmap = tiny_skia::Pixmap::new(width, height).context("Failed to create pixmap")?;
    resvg::render(&tree, Default::default(), &mut pixmap.as_mut());
    fbink
        .print_raw_data(pixmap.data(), width as i32, height as i32, 0, 0)
        .context("FBInk failed to print data to screen")?;
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf)> {
    let mut args = env::args_os().skip(1);

    let svg_path = match args.next() {
        Some(p) => {
            let path = PathBuf::from(p);
            if !path.is_file() {
                return Err(anyhow!("First argument must be a file. {USAGE}"));
            }
            path
        }
        None => return Err(anyhow!("No file was provided. {USAGE}")),
    };
    let font_dir = match args.next() {
        Some(p) => {
            let path = PathBuf::from(p);
            if !path.is_dir() {
                return Err(anyhow!("Second argument must be a directory. {USAGE}"));
            }
            path
        }
        None => {
            eprintln!("No font directory provided. Using current directory");
            env::current_dir().context("Failed to get current directory")?
        }
    };
    Ok((svg_path, font_dir))
}
