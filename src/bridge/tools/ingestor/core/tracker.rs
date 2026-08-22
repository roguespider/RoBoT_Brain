// src/tools/ingestor/core/tracker.rs
//! Ingest tracker for deletion verification

use std::collections::HashSet;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::bridge::tools::ingestor::file_collector::normalize_path;

// Global ingest tracker
static INGEST_TRACKER: std::sync::OnceLock<tokio::sync::Mutex<IngestTracker>> = std::sync::OnceLock::new();

/// Tracks recently ingested files for deletion verification
/// This prevents agents from deleting files without proper ingestion
pub struct IngestTracker {
    recently_ingested: HashSet<String>,
    last_ingest_time: Option<Instant>,
}

impl IngestTracker {
    pub fn new() -> Self {
        Self {
            recently_ingested: HashSet::new(),
            last_ingest_time: None,
        }
    }

    /// Record that files were ingested
    pub fn record_ingestion(&mut self, file_paths: Vec<String>) {
        for path in file_paths {
            self.recently_ingested.insert(path);
        }
        self.last_ingest_time = Some(Instant::now());
    }

    /// Check if a file was recently ingested
    pub fn was_recently_ingested(&self, file_path: &str) -> bool {
        // Normalize path for comparison
        let normalized = Path::new(file_path)
            .to_path_buf()
            .to_string_lossy()
            .to_lowercase();

        // Check exact match
        if self.recently_ingested.iter().any(|p| {
            Path::new(p).to_path_buf().to_string_lossy().to_lowercase() == normalized
        }) {
            return true;
        }

        // Check if it's in files_to_import (allow deletion of any file from import folder)
        if let Ok(exe_path) = std::env::current_exe()
            && let Some(exe_dir) = exe_path.parent() {
                let import_folder = exe_dir.join("files_to_import");
                if let Ok(file_path_buf) = Path::new(file_path).canonicalize()
                    && let Ok(import_canonical) = import_folder.canonicalize() {
                        // Normalize both paths to handle Windows extended-length paths
                        let file_normalized = normalize_path(file_path_buf);
                        let import_normalized = normalize_path(import_canonical);
                        return file_normalized.starts_with(import_normalized);
                    }
            }

        false
    }

    /// Check if we can verify deletion (means a recent ingest happened)
    pub fn can_verify_deletion(&self) -> bool {
        match self.last_ingest_time {
            Some(time) => time.elapsed() < Duration::from_secs(300), // 5 minute window
            None => false,
        }
    }

    /// Clear the tracker (after successful deletion or timeout)
    pub fn clear(&mut self) {
        self.recently_ingested.clear();
        self.last_ingest_time = None;
    }
}

impl Default for IngestTracker {
    fn default() -> Self {
        Self::new()
    }
}

fn get_ingest_tracker() -> &'static tokio::sync::Mutex<IngestTracker> {
    INGEST_TRACKER.get_or_init(|| tokio::sync::Mutex::new(IngestTracker::new()))
}

/// Get a reference to the ingest tracker (public version)
pub fn get_ingest_tracker_public() -> &'static tokio::sync::Mutex<IngestTracker> {
    get_ingest_tracker()
}

/// Record files as ingested (call after successful ingest)
pub async fn record_ingested_files(file_paths: Vec<String>) {
    if let Ok(mut tracker) = get_ingest_tracker().try_lock() {
        tracker.record_ingestion(file_paths);
    }
}

/// Check if files can be deleted
pub async fn can_delete_files(file_paths: &[String]) -> (bool, Vec<String>) {
    if let Ok(tracker) = get_ingest_tracker().try_lock() {
        let unverified: Vec<String> = file_paths
            .iter()
            .filter(|p| !tracker.was_recently_ingested(p))
            .cloned()
            .collect();

        let all_verified = unverified.is_empty();
        (all_verified, unverified)
    } else {
        (true, vec![]) // If can't lock, allow (fail open for now)
    }
}

/// Clear the ingest tracker
pub async fn clear_ingest_tracker() {
    if let Ok(mut tracker) = get_ingest_tracker().try_lock() {
        tracker.clear();
    }
}

/// Check if we can verify deletions (recent ingest happened)
pub async fn can_verify_deletion() -> bool {
    if let Ok(tracker) = get_ingest_tracker().try_lock() {
        tracker.can_verify_deletion()
    } else {
        false
    }
}
