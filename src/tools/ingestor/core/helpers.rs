// src/tools/ingestor/core/helpers.rs
//! Helper functions for file ingestion

use std::path::{Path, PathBuf};

use crate::tools::ingestor::file_collector::{
    is_supported_extension, normalize_path, ARCHIVE_EXTENSIONS,
};

/// Default chunk size for text splitting
#[allow(dead_code)]
pub const DEFAULT_CHUNK_SIZE: usize = 1000;

/// Resolve a file path string to an actual PathBuf that can be used for file operations.
/// This handles the Windows extended-length path prefix issue where:
/// 1. canonicalize() returns \\?\E:\... style paths on Windows
/// 2. normalize_path() strips this prefix for storage
/// 3. We need to re-canonicalize to ensure the path works for file operations
pub fn resolve_path(path: &str) -> std::io::Result<PathBuf> {
    let p = Path::new(path);

    // First try canonicalize (resolves symlinks, relative paths, etc.)
    if let Ok(canonical) = p.canonicalize() {
        // Canonicalize returns \\?\E:\... on Windows, so normalize it
        let normalized = normalize_path(canonical);
        return Ok(normalized);
    }

    // If canonicalize fails, try the raw path (might exist but have permission issues)
    if p.exists() {
        // Return normalized version of the original path
        return Ok(normalize_path(p.to_path_buf()));
    }

    // Path doesn't exist or can't be accessed
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("Path not found: {}", path)
    ))
}

/// Check if a file is an archive based on its extension
#[allow(dead_code)]
pub fn is_archive_file(path: &Path) -> bool {
    is_supported_extension(path, ARCHIVE_EXTENSIONS)
}

/// Get file size as a human-readable string
pub fn file_info_size(path: &str) -> String {
    std::fs::metadata(path)
        .map(|m| format_size(m.len()))
        .unwrap_or_else(|_| "unknown size".to_string())
}

/// Format bytes as human-readable string
pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024. * 1024.0))
    }
}
