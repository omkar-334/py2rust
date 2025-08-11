//! Core logic for the Kindle clippings extractor.
//!
//! This library handles parsing the "My Clippings.txt" file, organizing the
//! data into a structured format, and writing the output `.rst` files.

use anyhow::{anyhow, Context, Result};
use chrono::{NaiveDateTime, TimeZone, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;
use walkdir::WalkDir;

// --- Data Structures ---

/// Represents a single parsed clipping (a highlight or a note).
#[derive(Debug, Clone)]
pub struct Clipping {
    pub text: String,
    pub hash: String,
    pub note_type: String,
    pub location: String,
    pub date_str: String,
    pub date: Option<NaiveDateTime>,
}

/// Represents a book, containing its metadata and all associated clippings.
#[derive(Debug, Default)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub clippings: Vec<Clipping>,
}

/// A map from a book's original title line to the `Book` struct.
type BookMap = HashMap<String, Book>;
/// A map from a clipping's hash to the path of the file it's in.
type ExistingHashMap = HashMap<String, PathBuf>;

/// Configuration for the extraction process.
pub struct Config {
    pub input_file: PathBuf,
    pub output_dir: PathBuf,
}

// --- Regex Definitions ---

// Using once_cell::sync::Lazy for one-time compilation of regexes.
static RE_TITLE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(.*)\s*\(([^)]+)\)$").expect("Invalid title regex"));
static RE_INFO: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^- Your (\S+) on page ([\d\-]+) \| Location ([\d\-]+) \| Added on (.+)$")
        .or_else(|_| Regex::new(r"^- Your (\S+) on Location ([\d\-]+) \| Added on (.+)$"))
        .or_else(|_| Regex::new(r"^- Your (\S+) on page ([\d\-]+) \| Added on (.+)$"))
        .or_else(|_| Regex::new(r"^- Your (\S+) \| Location ([\d\-]+) \| Added on (.+)$"))
        .expect("Invalid info regex")
});
static RE_HASHLINE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\.\.\s+([a-fA-F0-9]{8})").expect("Invalid hash regex"));
static RE_INVALID_FILENAME_CHARS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"[^\w\s()'.,?!:-]"#).expect("Invalid filename regex"));

/// Main entry point for the library. Executes the entire extraction process.
pub fn run(config: Config) -> Result<()> {
    println!("Scanning output dir '{}'...", config.output_dir.display());
    if !config.output_dir.exists() {
        fs::create_dir_all(&config.output_dir).context("Failed to create output directory")?;
    }
    let existing_hashes = scan_existing_hashes(&config.output_dir)?;
    println!(
        "Found {} existing note hashes.",
        existing_hashes.len()
    );

    println!("Processing clippings file '{}'...", config.input_file.display());
    let books = parse_clippings_file(&config.input_file)?;
    println!("Parsed {} books from clippings file.", books.len());

    write_all_books(&books, &existing_hashes, &config.output_dir)?;

    Ok(())
}

/// Scans the output directory for `.rst` files and extracts hashes of existing notes.
fn scan_existing_hashes(out_dir: &Path) -> Result<ExistingHashMap> {
    let mut existing_hashes = HashMap::new();
    if !out_dir.exists() {
        return Ok(existing_hashes);
    }

    for entry in WalkDir::new(out_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "rst") {
            let file = File::open(path).context(format!("Failed to open {}", path.display()))?;
            let reader = BufReader::new(file);
            for line in reader.lines().filter_map(|l| l.ok()) {
                if let Some(caps) = RE_HASHLINE.captures(&line) {
                    if let Some(hash) = caps.get(1) {
                        existing_hashes.insert(hash.as_str().to_string(), path.to_path_buf());
                    }
                }
            }
        }
    }
    Ok(existing_hashes)
}

/// Parses the "My Clippings.txt" file into a map of books and their clippings.
fn parse_clippings_file(in_file: &Path) -> Result<BookMap> {
    let content = fs::read_to_string(in_file)
        .context(format!("Failed to read {}", in_file.display()))?;
    let mut books = BookMap::new();

    // Clippings are separated by "==========".
    for entry_str in content.trim_start_matches('\u{feff}').split("==========") {
        let entry_str = entry_str.trim();
        if entry_str.is_empty() {
            continue;
        }

        let mut lines = entry_str.lines();
        let title_line = lines.next().context("Entry missing title line")?;
        let info_line = lines.next().context("Entry missing info line")?;
        // Skip the empty line between info and note text
        lines.next();
        let note_text = lines.collect::<Vec<_>>().join("\n").trim().to_string();

        if note_text.is_empty() {
            continue;
        }

        // --- Parse Title and Author ---
        let (title, author) = if let Some(caps) = RE_TITLE.captures(title_line) {
            (
                caps.get(1).map_or("", |m| m.as_str()).trim().to_string(),
                caps.get(2).map_or("Unknown", |m| m.as_str()).trim().to_string(),
            )
        } else {
            (title_line.trim().to_string(), "Unknown".to_string())
        };

        // --- Parse Info Line ---
        let (note_type, location, date_str) =
            if let Some(caps) = RE_INFO.captures(info_line) {
                // This regex is complex due to multiple optional parts.
                // We check captures by index, which is brittle but mirrors the Python logic.
                // A more robust solution would use named capture groups.
                if caps.len() == 5 { // page and location
                    (
                        caps.get(1).unwrap().as_str().to_string(),
                        format!("p.{}, loc.{}", caps.get(2).unwrap().as_str(), caps.get(3).unwrap().as_str()),
                        caps.get(4).unwrap().as_str().to_string(),
                    )
                } else if caps.get(2).unwrap().as_str().contains('-') { // location only
                     (
                        caps.get(1).unwrap().as_str().to_string(),
                        format!("loc.{}", caps.get(2).unwrap().as_str()),
                        caps.get(3).unwrap().as_str().to_string(),
                    )
                } else { // page only
                    (
                        caps.get(1).unwrap().as_str().to_string(),
                        format!("p.{}", caps.get(2).unwrap().as_str()),
                        caps.get(3).unwrap().as_str().to_string(),
                    )
                }
            } else {
                // Fallback for unrecognized info line format
                ("Unknown".to_string(), "".to_string(), "".to_string())
            };

        // --- Create Clipping ---
        let mut hasher = Sha256::new();
        hasher.update(note_text.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        let short_hash = hash[..8].to_string();

        let date = NaiveDateTime::parse_from_str(&date_str, "%A, %d %B %Y %I:%M:%S %p").ok();

        let clipping = Clipping {
            text: note_text,
            hash: short_hash,
            note_type,
            location,
            date_str,
            date,
        };

        // --- Add to Book ---
        let book_entry = books.entry(title_line.to_string()).or_default();
        book_entry.title = title;
        book_entry.author = author;
        book_entry.clippings.push(clipping);
    }

    Ok(books)
}

/// Writes all parsed books to their respective `.rst` files.
fn write_all_books(
    books: &BookMap,
    existing_hashes: &ExistingHashMap,
    out_dir: &Path,
) -> Result<()> {
    for book in books.values() {
        let new_clippings: Vec<_> = book
            .clippings
            .iter()
            .filter(|c| !existing_hashes.contains_key(&c.hash))
            .cloned()
            .collect();

        if new_clippings.is_empty() {
            continue; // Skip if no new notes for this book
        }

        println!(
            "Found {} new notes for '{}'",
            new_clippings.len(),
            book.title
        );

        let (is_short, filename) = if book.clippings.len() > 2 {
            let short_title = create_short_title(&book.title);
            let fname = format!("{} - {}.rst", book.author, short_title);
            (false, fname)
        } else {
            (true, "short_notes.rst".to_string())
        };

        let valid_filename = get_valid_filename(&filename);
        let out_path = out_dir.join(valid_filename);
        let is_new_file = !out_path.exists();

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&out_path)
            .context(format!("Failed to open or create {}", out_path.display()))?;

        // --- Write File Header ---
        if is_short {
            let title_str = if book.author != "Unknown" {
                format!("{} - {}", book.author, book.title)
            } else {
                book.title.clone()
            };
            writeln!(file, "{}", title_str)?;
            writeln!(file, "{}\n", "-".repeat(title_str.len()))?;
        } else if is_new_file {
            let title_str = format!("Highlights from {}", book.title);
            writeln!(file, "{}", title_str)?;
            writeln!(file, "{}\n", "=".repeat(title_str.len()))?;
            if book.author != "Unknown" {
                writeln!(file, ":authors: {}\n", book.author.replace(';', ", "))?;
            }
        }

        // --- Write New Clippings ---
        for clipping in &new_clippings {
            println!(
                "  Adding new note to {}: {} {} {} {}",
                out_path.display(),
                clipping.hash,
                clipping.note_type,
                clipping.location,
                clipping.date_str
            );

            let mut comment = format!(
                ".. {} ; {} ; {} ; {}",
                clipping.hash, clipping.note_type, clipping.location, clipping.date_str
            );
            if is_short {
                comment.push_str(&format!(" ; {} ; {}", book.author, book.title));
            }

            writeln!(file, "{}\n", comment)?;
            writeln!(file, "{}\n", clipping.text)?;
        }

        // --- Update File Modification Time ---
        if let Some(last_date) = book.clippings.last().and_then(|c| c.date) {
            let timestamp = Utc.from_utc_datetime(&last_date).timestamp();
            let mtime = filetime::FileTime::from_unix_time(timestamp, 0);
            filetime::set_file_mtime(&out_path, mtime)
                .context(format!("Failed to set mtime for {}", out_path.display()))?;
        }
    }
    Ok(())
}

/// Sanitizes a string to be a valid filename.
fn get_valid_filename(filename: &str) -> String {
    let normalized: String = filename.nfkd().collect();
    RE_INVALID_FILENAME_CHARS
        .replace_all(&normalized, "")
        .into_owned()
}

/// Creates a shortened version of a book title for use in filenames.
fn create_short_title(title: &str) -> String {
    let mut short_title = title.split('|').next().unwrap_or("").trim();
    short_title = short_title.split(" - ").next().unwrap_or("").trim();
    short_title = short_title.split(". ").next().unwrap_or("").trim();
    let mut short_title = short_title.replace(['?', ':', '*'], "");
    if short_title.len() > 128 {
        short_title.truncate(127);
    }
    short_title.trim().to_string()
}