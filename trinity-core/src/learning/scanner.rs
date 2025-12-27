//! Recursive File Scanner for Trinity Memory
//!
//! Walks directories, filters for relevant files, and extracts text
//! for ingestion into the vector store.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

/// Supported file extensions for indexing
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "rs", "toml", "md", "txt", "json", "js", "html", "css", "py", "sh",
];

/// Configuration for the scanner
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    pub max_file_size: u64,
    pub ignore_hidden: bool,
    pub follow_links: bool,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10 MB
            ignore_hidden: true,
            follow_links: false,
        }
    }
}

/// A scanned document ready for ingestion
#[derive(Debug)]
pub struct ScannedDocument {
    pub path: PathBuf,
    pub content: String,
    pub extension: String,
}

/// The File Scanner
pub struct FileScanner {
    config: ScannerConfig,
}

impl FileScanner {
    pub fn new(config: ScannerConfig) -> Self {
        Self { config }
    }

    /// Recursively scan a directory and return a stream or iterator of documents
    pub fn scan(&self, root: &Path) -> impl Iterator<Item = Result<ScannedDocument>> {
        let config = self.config.clone();

        WalkDir::new(root)
            .follow_links(config.follow_links)
            .into_iter()
            .filter_entry(move |e| !config.ignore_hidden || !is_hidden(e))
            .filter_map(move |entry| match entry {
                Ok(entry) => {
                    if !entry.file_type().is_file() {
                        return None;
                    }

                    let path = entry.path().to_path_buf();

                    // Check extension
                    let ext = path.extension()?.to_str()?.to_lowercase();
                    if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                        return None;
                    }

                    // Check size
                    match entry.metadata() {
                        Ok(meta) => {
                            if meta.len() > config.max_file_size {
                                return None;
                            }
                        }
                        Err(_) => return None,
                    }

                    Some(read_document(path, ext))
                }
                Err(e) => Some(Err(anyhow::anyhow!("Scan error: {}", e))),
            })
    }
}

/// Helper to read a document from disk
fn read_document(path: PathBuf, extension: String) -> Result<ScannedDocument> {
    // TODO: Add PDF support later (needs pdf-extract)

    // For now, assume text/utf8
    let content = fs::read_to_string(&path)?;

    Ok(ScannedDocument {
        path,
        content,
        extension,
    })
}

/// Helper to identify hidden files (starts with .)
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') && s != "." && s != "..")
        .unwrap_or(false)
}
