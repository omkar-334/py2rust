//! A tool to extract and organise highlights and notes from the "My Clippings.txt"
//! file on a Kindle e-reader.

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use kindle_clippings_extractor::{run, Config};
use std::path::PathBuf;

/// Extract and organise Kindle highlights and notes from "My Clippings.txt".
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "A Rust-based tool to extract and organise highlights and notes from the 'My Clippings.txt' file on a Kindle e-reader."
)]
struct Cli {
    /// Path to the "My Clippings.txt" file.
    /// If not provided, it will search for the Kindle mount point on Linux.
    #[arg(value_name = "CLIPPINGS_FILE")]
    input_file: Option<PathBuf>,

    /// Directory to save the extracted `.rst` files.
    #[arg(value_name = "OUTPUT_DIRECTORY", default_value = "clippings")]
    output_dir: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine the input file path, using defaults if necessary.
    let input_file = cli.input_file.or_else(find_default_clippings_file).ok_or_else(|| {
        anyhow!(
            "Could not find 'My Clippings.txt'. Please provide the file location as an argument.\n\nUSAGE:\n    kindle-clippings-extractor <CLIPPINGS_FILE> [OUTPUT_DIRECTORY]"
        )
    })?;

    if !input_file.is_file() {
        return Err(anyhow!(
            "Input file not found at: {}",
            input_file.display()
        ));
    }

    let config = Config {
        input_file,
        output_dir: cli.output_dir,
    };

    run(config).context("Failed to process clippings")?;

    println!("\nExtraction complete!");
    Ok(())
}

/// Tries to find "My Clippings.txt" in the default Kindle location on Linux.
#[cfg(target_os = "linux")]
fn find_default_clippings_file() -> Option<PathBuf> {
    let username = users::get_current_username()?.into_string().ok()?;
    let path = PathBuf::from(format!(
        "/media/{}/Kindle/documents/My Clippings.txt",
        username
    ));
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

/// Fallback for non-Linux systems where a default path is not implemented.
#[cfg(not(target_os = "linux"))]
fn find_default_clippings_file() -> Option<PathBuf> {
    None
}