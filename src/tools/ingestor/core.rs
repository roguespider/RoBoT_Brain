// src/tools/ingestor/core.rs
// Core file ingestion logic

#![allow(dead_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::time;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::database::models::{MemoryCard, MemoryType};
use crate::database::sqlite::SqliteDatabase;
use crate::memory::pipeline::MemoryPipeline;
use crate::memory::types::MemoryItem;
use crate::memory::WorkingMemory;
use crate::tools::ToolOutput;
use crate::tools::ingestor::archive_handler::{
    create_archive_temp_dir, delete_empty_folders, process_archive,
};
use crate::tools::ingestor::file_collector::{collect_all_files_recursive, collect_importable_files, collect_importable_files_with_recursive, get_import_folder, is_supported_extension, normalize_path, AUDIO_EXTENSIONS, JSON_EXTENSIONS, ARCHIVE_EXTENSIONS, TEXT_EXTENSIONS, IMAGE_EXTENSIONS};
use crate::tools::ingestor::text_extractor::{extract_text, extract_image_metadata, validate_text_quality};
use crate::tools::ingestor::semantic_chunker::{parse_document, get_file_type};
use crate::tools::ingestor::json_importer::{import_json_file, ExtractedJsonData};
use crate::tools::ingestor::audio_transcriber::{
    self, is_audio_file, store_transcription_as_memory,
};

// Re-export for convenience (via parent module)
pub use super::workflow::{
    execute_delete_ingested_files, execute_list_importable, execute_list_ingested_files,
    find_empty_folders_after_deletion,
};

/// Default chunk size for text splitting
pub const DEFAULT_CHUNK_SIZE: usize = 1000;

/// Default overlap between chunks
pub const DEFAULT_CHUNK_OVERLAP: usize = 100;

/// Check if a file is an archive based on its extension
fn is_archive_file(path: &Path) -> bool {
    is_supported_extension(path, ARCHIVE_EXTENSIONS)
}

/// Get file size as a human-readable string
fn file_info_size(path: &str) -> String {
    std::fs::metadata(path)
        .map(|m| format_size(m.len()))
        .unwrap_or_else(|_| "unknown size".to_string())
}

/// Format bytes as human-readable string
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

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
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let import_folder = exe_dir.join("files_to_import");
                if let Ok(file_path_buf) = Path::new(file_path).canonicalize() {
                    if let Ok(import_canonical) = import_folder.canonicalize() {
                        // Normalize both paths to handle Windows extended-length paths
                        let file_normalized = normalize_path(file_path_buf);
                        let import_normalized = normalize_path(import_canonical);
                        return file_normalized.starts_with(import_normalized);
                    }
                }
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

// Global ingest tracker
static INGEST_TRACKER: std::sync::OnceLock<tokio::sync::Mutex<IngestTracker>> = std::sync::OnceLock::new();

fn get_ingest_tracker() -> &'static tokio::sync::Mutex<IngestTracker> {
    INGEST_TRACKER.get_or_init(|| tokio::sync::Mutex::new(IngestTracker::new()))
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

// ============================================================================
// INPUT/OUTPUT TYPES
// ============================================================================

/// Default timeout for ingestion operations (60 seconds)
pub const DEFAULT_INGEST_TIMEOUT_SECS: u64 = 60;

/// Tool: Ingest files from import folder
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct IngestFilesInput {
    pub folder: Option<String>,
    pub file_path: Option<String>,
    /// Alias for file_path for backwards compatibility
    #[serde(rename = "file_paths", alias = "file_paths")]
    pub file_paths_alias: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub chunk_size: Option<usize>,
    pub memory_type: Option<String>,
    /// Timeout in seconds for the entire ingestion operation (default: 60)
    /// Increase this value for large files or slow storage
    pub timeout_seconds: Option<u64>,
    /// Search subfolders recursively (default: true)
    pub recursive: Option<bool>,
    /// Force re-ingestion of already-ingested files (default: false)
    /// Use this when user confirms they want to add a file again
    pub force: Option<bool>,
    /// Return a compact summary instead of full verbose output (default: false)
    /// Set to true for cleaner output when using interactively
    pub summary_only: Option<bool>,
}

impl IngestFilesInput {
    /// Get the first file path from either file_path or file_paths
    pub fn get_file_path(&self) -> Option<&str> {
        if let Some(ref fp) = self.file_path {
            return Some(fp);
        }
        if let Some(ref fps) = self.file_paths_alias {
            if let Some(first) = fps.first() {
                return Some(first);
            }
        }
        None
    }
}

/// Tool: List files ready for import
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListImportableInput {
    pub folder: Option<String>,
    pub limit: Option<usize>,
    /// Search subfolders recursively (default: true)
    pub recursive: Option<bool>,
    /// List all files without limit (default: false)
    /// Set to true to show all files instead of just the first few
    pub list_all: Option<bool>,
}

/// Tool: Transcribe an audio file
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TranscribeAudioInput {
    pub path: String,
    /// Whether to store the transcription as memory (default: true)
    pub store_as_memory: Option<bool>,
}

/// Tool: Delete successfully imported files
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteIngestedFilesInput {
    pub files: Vec<String>,
    pub confirmation: String,
}

/// Tool: List files that were successfully ingested
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListIngestedFilesInput {
    pub folder: Option<String>,
    pub limit: Option<usize>,
    /// Search subfolders recursively (default: true) - matches ingest_files behavior
    pub recursive: Option<bool>,
}

/// Result of ingesting a single file
#[derive(Debug, Clone, Serialize)]
pub struct IngestResult {
    pub filename: String,
    pub file_path: String,
    pub success: bool,
    pub chunks_created: usize,
    pub chunk_size_used: usize,
    pub memory_ids: Vec<String>,
    pub error: Option<String>,
    pub remaining_count: usize,
}

/// Summary of ingestion operation
#[derive(Debug, Clone, Serialize)]
pub struct IngestSummary {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_chunks: usize,
    pub results: Vec<IngestResult>,
}

// ============================================================================
// MAIN INGESTION FUNCTIONS
// ============================================================================

/// Ingest files into memory
/// Per Architecture §6.3: Stores memories in Working Memory cache (fast, volatile, in-memory)
/// Also persists to SQLite for recovery via checkpoint
pub async fn ingest_file(
    input: IngestFilesInput,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<ToolOutput> {
    let folder = get_import_folder(input.folder.as_deref());
    let file_path = input.get_file_path();
    let limit = input.limit.unwrap_or(1);
    let chunk_size = input.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
    let memory_type = parse_memory_type(input.memory_type.as_deref().unwrap_or("file"));
    let timeout_secs = input.timeout_seconds.unwrap_or(DEFAULT_INGEST_TIMEOUT_SECS);
    let recursive = input.recursive.unwrap_or(true);
    let force = input.force.unwrap_or(false);
    let summary_only = input.summary_only.unwrap_or(false);

    tracing::info!("Starting file ingestion: limit={}, chunk_size={}, timeout={}s, recursive={}, force={}, summary_only={}", 
                   limit, chunk_size, timeout_secs, recursive, force, summary_only);

    // Helper function to ingest a single file (handles both regular and archive files)
    async fn ingest_path(
        path: &Path,
        chunk_size: usize,
        memory_type: MemoryType,
        db: Arc<SqliteDatabase>,
        working_memory: Arc<WorkingMemory>,
    ) -> Result<IngestResult> {
        if is_archive_file(path) {
            ingest_archive(path, chunk_size, memory_type, db, working_memory).await
        } else {
            ingest_single_file(path, chunk_size, memory_type, db, working_memory).await
        }
    }

    // Wrapper for backward compatibility
    async fn ingest_path_with_memory(
        path: &Path,
        chunk_size: usize,
        memory_type: MemoryType,
        db: Arc<SqliteDatabase>,
        working_memory: Arc<WorkingMemory>,
    ) -> Result<IngestResult> {
        ingest_path(path, chunk_size, memory_type, db, working_memory).await
    }

    // Check if ingesting a specific file or from folder
    if let Some(file_path) = file_path {
        let path = Path::new(file_path);
        if path.exists() {
            let result = time::timeout(
                Duration::from_secs(timeout_secs),
                ingest_path_with_memory(path, chunk_size, memory_type, db, working_memory)
            ).await;
            
            match result {
                Ok(Ok(ingest_result)) => {
                    tracing::info!("Ingested file successfully: {} chunks", ingest_result.chunks_created);
                    Ok(ToolOutput::success(serde_json::to_value(ingest_result)?))
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to ingest file: {}", e);
                    Ok(ToolOutput::error(format!("Failed to ingest file: {}", e)))
                }
                Err(_) => {
                    tracing::error!("Ingestion timed out after {} seconds", timeout_secs);
                    Ok(ToolOutput::error(format!(
                        "Ingestion timed out after {} seconds. Try increasing timeout_seconds for large files.",
                        timeout_secs
                    )))
                }
            }
        } else {
            // Try relative to folder
            let path = folder.join(file_path);
            if path.exists() {
                let result = time::timeout(
                    Duration::from_secs(timeout_secs),
                    ingest_path_with_memory(&path, chunk_size, memory_type, db, working_memory)
                ).await;
                
                match result {
                    Ok(Ok(ingest_result)) => {
                        tracing::info!("Ingested file successfully: {} chunks", ingest_result.chunks_created);
                        Ok(ToolOutput::success(serde_json::to_value(ingest_result)?))
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Failed to ingest file: {}", e);
                        Ok(ToolOutput::error(format!("Failed to ingest file: {}", e)))
                    }
                    Err(_) => {
                        tracing::error!("Ingestion timed out after {} seconds", timeout_secs);
                        Ok(ToolOutput::error(format!(
                            "Ingestion timed out after {} seconds. Try increasing timeout_seconds for large files.",
                            timeout_secs
                        )))
                    }
                }
            } else {
                Ok(ToolOutput::error(format!("File not found: {}", file_path)))
            }
        }
    } else {
        // Ingest from folder
        if !folder.exists() {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_string_lossy().to_string()))
                .unwrap_or_else(|| "robot_brain.exe directory".to_string());
            
            return Ok(ToolOutput::error(format!(
                "Import folder does not exist: {}\n\
                \n\
                The 'files_to_import' folder should be in: {}\n\
                \n\
                Create the folder and add files there, or put files_to_import next to robot_brain.exe",
                folder.display(),
                exe_dir
            )));
        }

        // If folder is a file, ingest it directly
        if folder.is_file() {
            let result = time::timeout(
                Duration::from_secs(timeout_secs),
                ingest_path_with_memory(&folder, chunk_size, memory_type, db, working_memory)
            ).await;
            
            return match result {
                Ok(Ok(ingest_result)) => {
                    tracing::info!("Ingested file successfully: {} chunks", ingest_result.chunks_created);
                    Ok(ToolOutput::success(serde_json::to_value(ingest_result)?))
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to ingest file: {}", e);
                    Ok(ToolOutput::error(format!("Failed to ingest file: {}", e)))
                }
                Err(_) => {
                    tracing::error!("Ingestion timed out after {} seconds", timeout_secs);
                    Ok(ToolOutput::error(format!(
                        "Ingestion timed out after {} seconds. Try increasing timeout_seconds for large files.",
                        timeout_secs
                    )))
                }
            };
        }

        // Collect files from folder (with optional recursive search)
        let all_files = if recursive {
            collect_importable_files_with_recursive(&folder, true)?
        } else {
            collect_importable_files(&folder)?
        };
        
        // Filter out files with skip reasons
        let (skipped_files, files_to_check): (Vec<_>, Vec<_>) = all_files
            .into_iter()
            .partition(|f| f.skip_reason.is_some());
        
        // Check for already-ingested files and separate them
        let ingest_tracker = get_ingest_tracker().try_lock().ok();
        let (already_ingested, files_to_process): (Vec<_>, Vec<_>) = files_to_check
            .into_iter()
            .partition(|f| {
                if !force {
                    // Only check tracker if not forcing
                    if let Some(ref tracker) = ingest_tracker {
                        return tracker.was_recently_ingested(&f.path);
                    }
                }
                false
            });
        
        let files_to_process: Vec<_> = files_to_process.into_iter().take(limit).collect();

        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;
        let mut total_chunks = 0;
        let mut timeout_occurred = false;

        for file_info in files_to_process {
            let path = Path::new(&file_info.path);
            let filename = file_info.filename.clone();

            // Check if it's an archive
            if file_info.file_type == "archive" {
                let result = time::timeout(
                    Duration::from_secs(timeout_secs),
                    ingest_archive(path, chunk_size, memory_type.clone(), db.clone(), working_memory.clone())
                ).await;
                
                match result {
                    Ok(Ok(result)) => {
                        results.push(result);
                        successful += 1;
                    }
                    Ok(Err(e)) => {
                        failed += 1;
                        results.push(IngestResult {
                            filename,
                            file_path: file_info.path.clone(),
                            success: false,
                            chunks_created: 0,
                            chunk_size_used: chunk_size,
                            memory_ids: vec![],
                            error: Some(e.to_string()),
                            remaining_count: 0,
                        });
                    }
                    Err(_) => {
                        timeout_occurred = true;
                        failed += 1;
                        tracing::error!("Archive ingestion timed out for: {}", file_info.filename);
                        results.push(IngestResult {
                            filename,
                            file_path: file_info.path.clone(),
                            success: false,
                            chunks_created: 0,
                            chunk_size_used: chunk_size,
                            memory_ids: vec![],
                            error: Some(format!("Ingestion timed out after {} seconds. Try increasing timeout_seconds.", timeout_secs)),
                            remaining_count: 0,
                        });
                        break; // Stop processing on timeout
                    }
                }
            } else {
                let result = time::timeout(
                    Duration::from_secs(timeout_secs),
                    ingest_single_file(path, chunk_size, memory_type.clone(), db.clone(), working_memory.clone())
                ).await;
                
                match result {
                    Ok(Ok(result)) => {
                        let chunks = result.chunks_created;
                        let success = result.success;
                        results.push(result);
                        if success {
                            successful += 1;
                            total_chunks += chunks;
                        } else {
                            failed += 1;
                        }
                    }
                    Ok(Err(e)) => {
                        failed += 1;
                        results.push(IngestResult {
                            filename,
                            file_path: file_info.path.clone(),
                            success: false,
                            chunks_created: 0,
                            chunk_size_used: chunk_size,
                            memory_ids: vec![],
                            error: Some(e.to_string()),
                            remaining_count: 0,
                        });
                    }
                    Err(_) => {
                        timeout_occurred = true;
                        failed += 1;
                        tracing::error!("File ingestion timed out for: {}", file_info.filename);
                        results.push(IngestResult {
                            filename,
                            file_path: file_info.path.clone(),
                            success: false,
                            chunks_created: 0,
                            chunk_size_used: chunk_size,
                            memory_ids: vec![],
                            error: Some(format!("Ingestion timed out after {} seconds. Try increasing timeout_seconds.", timeout_secs)),
                            remaining_count: 0,
                        });
                        break; // Stop processing on timeout
                    }
                }
            }
        }

        let total_files = results.len();
        let successfully_ingested: Vec<String> = results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.file_path.clone())
            .collect();

        let _remaining_count: usize = results.iter().map(|r| r.remaining_count).sum();

        // RECORD INGESTED FILES for deletion tracking
        // This enables the delete_ingested_files tool to verify files were actually ingested
        if !successfully_ingested.is_empty() {
            record_ingested_files(successfully_ingested.clone()).await;
        }

        // CLEANUP WAL FILES after batch operations
        // This checkpoints the WAL and cleans up the -wal and -shm files
        if let Err(e) = db.cleanup_wal_files() {
            tracing::warn!("Failed to cleanup WAL files: {}", e);
        }

        	        // Get folder path for reference
        let _folder_display = folder.to_string_lossy().to_string();
        let _exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        // Build detailed file info for the summary BEFORE moving results
        let ingested_file_details: Vec<serde_json::Value> = results
            .iter()
            .filter(|r| r.success)
            .map(|r| {
                let filename = r.filename.clone();
                let file_size = file_info_size(&r.file_path);
                serde_json::json!({
                    "filename": filename,
                    "file_path": r.file_path,
                    "file_size": file_size,
                    "chunks": r.chunks_created,
                    "memory_ids": r.memory_ids,
                    "content_preview": format!("{} chunks added to memory", r.chunks_created)
                })
            })
            .collect();

        // Check if the ingested file had an error (before moving results)
        let failed_file = if failed > 0 {
            results.iter().find(|r| !r.success).map(|r| {
                serde_json::json!({
                    "filename": r.filename,
                    "error": r.error
                })
            })
        } else {
            None
        };

        let _summary = IngestSummary {
            total_files,
            successful,
            failed,
            total_chunks,
            results,
        };

        // Format already ingested files for display
        let already_ingested_filenames: Vec<String> = already_ingested.iter().map(|f| f.filename.clone()).collect();
        
        // Check if this file was skipped (has issues before ingestion)
        let skipped_file = if !skipped_files.is_empty() {
            Some(serde_json::json!({
                "filename": skipped_files[0].filename,
                "size": format_size(skipped_files[0].size),
                "reason": skipped_files[0].skip_reason
            }))
        } else {
            None
        };
        
        // Generate user-facing message based on what happened
        let _user_message = if let Some(ref skipped) = skipped_file {
            format!("Can't ingest '{}': {}", skipped["filename"], skipped["reason"])
        } else if let Some(ref failed) = failed_file {
            format!("'{}' failed: {}. Do you want to retry?", 
                failed["filename"], failed["error"])
        } else if !already_ingested.is_empty() && successfully_ingested.is_empty() {
            format!("'{}' was already ingested. Do you want to ingest it again?", already_ingested_filenames[0])
        } else if successfully_ingested.is_empty() && already_ingested.is_empty() {
            "No files to ingest.".to_string()
        } else if successfully_ingested.len() == 1 {
            let filename = successfully_ingested[0].rsplit('/').next_back().unwrap_or(&successfully_ingested[0]).rsplit('\\').next_back().unwrap_or(&successfully_ingested[0]);
            format!("Successfully ingested: {}", filename)
        } else {
            format!("Successfully ingested {} files.", successfully_ingested.len())
        };
        
        // Return compact response if summary_only is true
        if summary_only {
            let empty_folders = find_empty_folders_after_deletion(&successfully_ingested);
            
            return Ok(ToolOutput::success(serde_json::json!({
                "success": successful > 0,
                "total_chunks": total_chunks,
                
                // Files ingested
                "files": ingested_file_details.iter().map(|d| {
                    serde_json::json!({
                        "filename": d.get("filename").and_then(|v| v.as_str()).unwrap_or("?"),
                        "size": d.get("file_size").and_then(|v| v.as_str()).unwrap_or("?"),
                        "chunks": d.get("chunks").and_then(|v| v.as_u64()).unwrap_or(0)
                    })
                }).collect::<Vec<_>>(),
                
                // Summary of what was added to memory
                "summary": if successful == 1 {
                    format!("Added {} chunks to memory from '{}'", 
                        total_chunks,
                        ingested_file_details.first().and_then(|d| d.get("filename")).and_then(|v| v.as_str()).unwrap_or("file"))
                } else {
                    format!("Added {} chunks to memory from {} files", total_chunks, successful)
                },
                
                // Ask user about deleting the file(s)
                "ask_delete_file": if successful > 0 {
                    serde_json::json!("Can I delete the original file to save space?")
                } else {
                    serde_json::Value::Null
                },
                "deletion_candidates": successfully_ingested,
                
                // Check for empty folders
                "empty_folders": empty_folders.iter().map(|p| {
                    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.to_string_lossy().to_string())
                }).collect::<Vec<_>>(),
                "ask_delete_folders": if !empty_folders.is_empty() {
                    serde_json::json!("Some folders are now empty. Can I delete them too?")
                } else {
                    serde_json::Value::Null
                },
                
                // Skipped or failed file info
                "can't_ingest": skipped_file.clone(),
                "ask_skip": if skipped_file.is_some() {
                    serde_json::json!("Skip this file and move to the next one?")
                } else if failed_file.is_some() {
                    serde_json::json!("Do you want to retry ingesting this file?")
                } else {
                    serde_json::Value::Null
                },
                
                // Already ingested
                "already_ingested": already_ingested_filenames,
                "ask_reingest": if !already_ingested.is_empty() && successful == 0 {
                    serde_json::json!("This file was already ingested. Do you want to ingest it again?")
                } else {
                    serde_json::Value::Null
                },
                
                "timeout_occurred": timeout_occurred
            })));
        }
        
        Ok(ToolOutput::success(serde_json::json!({
            // Success status
            "success": successful > 0,
            "total_chunks": total_chunks,
            
            // Files ingested (one at a time)
            "files": ingested_file_details.iter().map(|d| {
                serde_json::json!({
                    "filename": d.get("filename").and_then(|v| v.as_str()).unwrap_or("?"),
                    "size": d.get("file_size").and_then(|v| v.as_str()).unwrap_or("?"),
                    "chunks": d.get("chunks").and_then(|v| v.as_u64()).unwrap_or(0)
                })
            }).collect::<Vec<_>>(),
            
            // Summary of what was added to memory
            "summary": if successful == 1 {
                format!("Added {} chunks to memory from '{}'", 
                    total_chunks,
                    ingested_file_details.first().and_then(|d| d.get("filename")).and_then(|v| v.as_str()).unwrap_or("file"))
            } else {
                format!("Added {} chunks to memory from {} files", total_chunks, successful)
            },
            
            // Ask user about deleting the file(s)
            "ask_delete_file": if successful > 0 {
                serde_json::json!("Can I delete the original file to save space?")
            } else {
                serde_json::Value::Null
            },
            "deletion_candidates": successfully_ingested,
            
            // Check for empty folders after deletion
            "empty_folders": find_empty_folders_after_deletion(&successfully_ingested).iter().map(|p| {
                p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.to_string_lossy().to_string())
            }).collect::<Vec<_>>(),
            
            // Ask user about deleting empty folders
            "ask_delete_folders": if !find_empty_folders_after_deletion(&successfully_ingested).is_empty() {
                serde_json::json!("Some folders are now empty. Can I delete them too?")
            } else {
                serde_json::Value::Null
            },
            
            // Skipped or failed file info
            "can't_ingest": skipped_file.clone(),
            "ask_skip": if skipped_file.is_some() {
                serde_json::json!("Skip this file and move to the next one?")
            } else if failed_file.is_some() {
                serde_json::json!("Do you want to retry ingesting this file?")
            } else {
                serde_json::Value::Null
            },
            
            // Already ingested
            "already_ingested": already_ingested_filenames,
            "ask_reingest": if !already_ingested.is_empty() && successful == 0 {
                serde_json::json!("This file was already ingested. Do you want to ingest it again?")
            } else {
                serde_json::Value::Null
            },
            
            "timeout_occurred": timeout_occurred,
            "timeout_help": if timeout_occurred {
                serde_json::json!("A timeout occurred. Call ingest_files again with a higher timeout_seconds value (e.g., 300).")
            } else {
                serde_json::Value::Null
            }
        })))
    }
}

/// Ingest an archive file
/// Per Architecture §6.3: Stores in Working Memory cache
async fn ingest_archive(
    path: &Path,
    chunk_size: usize,
    memory_type: MemoryType,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<IngestResult> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive")
        .to_string();

    // Create temp directory for extraction
    let temp_dir = create_archive_temp_dir(&filename);
    std::fs::create_dir_all(&temp_dir)?;

    // Process archive
    let files = process_archive(path, &temp_dir)?;

    if files.is_empty() {
        return Ok(IngestResult {
            filename,
            file_path: path.to_string_lossy().to_string(),
            success: false,
            chunks_created: 0,
            chunk_size_used: chunk_size,
            memory_ids: vec![],
            error: Some("Archive is empty".to_string()),
            remaining_count: 0,
        });
    }

    // Filter to only text-based files (skip images, binaries, etc.)
    let all_files = files.clone();
    let text_files: Vec<PathBuf> = files
        .into_iter()
        .filter(|f| {
            let ext = f.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            TEXT_EXTENSIONS.contains(&ext.as_str()) || 
            JSON_EXTENSIONS.contains(&ext.as_str()) ||
            ext == "txt" || ext == "md" || ext == "html" || ext == "xml"
        })
        .collect();

    if text_files.is_empty() {
        // Clean up the temp directory
        for file_path in all_files {
            let _ = std::fs::remove_file(&file_path);
        }
        delete_empty_folders(&temp_dir);
        
        return Ok(IngestResult {
            filename,
            file_path: path.to_string_lossy().to_string(),
            success: false,
            chunks_created: 0,
            chunk_size_used: chunk_size,
            memory_ids: vec![],
            error: Some("Archive contains no text-based files (only images, binaries, etc.)".to_string()),
            remaining_count: 0,
        });
    }

    // Ingest the first text file
    let first_file = &text_files[0];
    let result = ingest_single_file(first_file, chunk_size, memory_type, db, working_memory).await?;

    // Count remaining files BEFORE cleanup
    let remaining_files = collect_all_files_recursive(&temp_dir)?;
    let remaining_count = remaining_files.len();

    // Delete ALL remaining files in temp directory (including transcribed audio files)
    for file_path in &remaining_files {
        let _ = std::fs::remove_file(file_path);
    }

    // Clean up empty subfolders
    delete_empty_folders(&temp_dir);

    Ok(IngestResult {
        filename,
        file_path: path.to_string_lossy().to_string(),
        success: result.success,
        chunks_created: result.chunks_created,
        chunk_size_used: result.chunk_size_used,
        memory_ids: result.memory_ids,
        error: result.error,
        remaining_count,
    })
}

/// Ingest a single file into memory using semantic hierarchical chunking
/// Per Architecture §6.3: Stores in Working Memory cache
async fn ingest_single_file(
    path: &Path,
    _chunk_size: usize,
    memory_type: MemoryType,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<IngestResult> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Check if this is an image file - handle separately
    if is_supported_extension(path, IMAGE_EXTENSIONS) {
        return ingest_image_file(path, 0, memory_type, db, working_memory).await;
    }

    // Check if this is a JSON file - use smart JSON importer
    if is_supported_extension(path, JSON_EXTENSIONS) {
        return ingest_json_file(path, 0, memory_type, db, working_memory).await;
    }

    // Check if this is an audio file - use Whisper transcription
    if is_supported_extension(path, AUDIO_EXTENSIONS) {
        return ingest_audio_file(path, 0, memory_type, db, working_memory).await;
    }

    // Extract text content for other file types
    let text = extract_text(path)
        .with_context(|| format!("Failed to extract text from {}", filename))?;

    if text.trim().is_empty() {
        return Ok(IngestResult {
            filename,
            file_path: path.to_string_lossy().to_string(),
            success: false,
            chunks_created: 0,
            chunk_size_used: 0,
            memory_ids: vec![],
            error: Some("File contains no text".to_string()),
            remaining_count: 0,
        });
    }

    // Validate text quality - reject binary garbage
    let (is_valid, quality_reason) = validate_text_quality(&text);
    if !is_valid {
        return Ok(IngestResult {
            filename,
            file_path: path.to_string_lossy().to_string(),
            success: false,
            chunks_created: 0,
            chunk_size_used: 0,
            memory_ids: vec![],
            error: Some(format!("Content is not readable text: {}", quality_reason)),
            remaining_count: 0,
        });
    }

    // Get file extension for semantic parsing
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("txt");
    let file_type = get_file_type(extension);
    
    // Parse document into hierarchy tree
    let hierarchy = parse_document(&text, &filename, file_type);
    
    // Flatten tree into memories
    let mut memories = hierarchy.flatten();
    
    // Update file_source for all memories
    let file_source = path.to_string_lossy().to_string();
    for memory in &mut memories {
        memory.file_source = Some(file_source.clone());
    }
    
    let total_memories = memories.len();

    // Store memories using MemoryPipeline (stores in Working layer for consolidation)
    // Also store directly in Working Memory cache (Architecture §6.3)
    let pipeline = MemoryPipeline::new(db.clone());
    let mut memory_ids = Vec::new();

    for memory in &memories {
        // Store in pipeline (for SQLite persistence)
        if let Err(e) = pipeline.store_working(memory) {
            tracing::warn!("Failed to store memory chunk via pipeline: {}", e);
        }
        
        // Store in Working Memory cache (fast, in-memory) - Architecture §6.3
        let memory_item = MemoryItem::from(memory);
        working_memory.store(memory_item).await;
        memory_ids.push(memory.id.to_string());
    }

    Ok(IngestResult {
        filename,
        file_path: path.to_string_lossy().to_string(),
        success: true,
        chunks_created: total_memories,
        chunk_size_used: 0,  // Semantic chunking doesn't use fixed sizes
        memory_ids,
        error: None,
        remaining_count: 0,
    })
}

/// Ingest a JSON file using smart structured extraction
/// Per Architecture §6.3: Stores in Working Memory cache
async fn ingest_json_file(
    path: &Path,
    chunk_size: usize,
    memory_type: MemoryType,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<IngestResult> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Use smart JSON importer
    let result = match import_json_file(path, None) {
        Ok(r) => r,
        Err(e) => {
            return Ok(IngestResult {
                filename,
                file_path: path.to_string_lossy().to_string(),
                success: false,
                chunks_created: 0,
                chunk_size_used: chunk_size,
                memory_ids: vec![],
                error: Some(format!("Failed to parse JSON: {}", e)),
                remaining_count: 0,
            });
        }
    };

    // Even if items is empty, we try to read the raw file content as fallback
    let items_to_store = if result.items.is_empty() {
        // Try to read raw JSON content as a single fallback item
        if let Ok(raw_content) = std::fs::read_to_string(path) {
            tracing::info!("JSON file had no structured items, storing raw content ({} chars)", raw_content.len());
            vec![ExtractedJsonData {
                content: raw_content,
                json_path: "root".to_string(),
                field_name: "raw".to_string(),
                sibling_context: String::new(),
                data_type: "raw.json".to_string(),
                raw_value: serde_json::Value::String("raw file content".to_string()),
            }]
        } else {
            return Ok(IngestResult {
                filename,
                file_path: path.to_string_lossy().to_string(),
                success: false,
                chunks_created: 0,
                chunk_size_used: chunk_size,
                memory_ids: vec![],
                error: Some("JSON file contains no extractable content".to_string()),
                remaining_count: 0,
            });
        }
    } else {
        result.items
    };

    // Store each extracted item as a memory with hierarchy using MemoryPipeline
    let pipeline = MemoryPipeline::new(db.clone());
    let mut memory_ids = Vec::new();
    
    let file_source = path.to_string_lossy().to_string();
    
    for (idx, item) in items_to_store.iter().enumerate() {
        let content = item.to_memory_content();
        
        // Use semantic chunking for long content
        if content.len() > 1000 {
            // Parse as JSON to get structure
            let hierarchy = parse_document(&content, &format!("{}[{}]", filename, idx), "json");
            let mut memories = hierarchy.flatten();
            
            for memory in &mut memories {
                memory.file_source = Some(file_source.clone());
            }
            
            for memory in &memories {
                // Store in pipeline (for SQLite persistence)
                if let Err(e) = pipeline.store_working(memory) {
                    tracing::warn!("Failed to store JSON memory chunk via pipeline: {}", e);
                }
                
                // Store in Working Memory cache (Architecture §6.3)
                let memory_item = MemoryItem::from(memory);
                working_memory.store(memory_item).await;
                memory_ids.push(memory.id.to_string());
            }
        } else {
            // Short content - store as single memory
            let memory = MemoryCard::new_hierarchical(
                content,
                memory_type.clone(),
                None,  // Top level
                crate::database::models::HierarchyLevel::Section,
                idx,
                format!("{}/item[{}]", filename, idx),
                Some(file_source.clone()),
            );
            
            // Store in pipeline (for SQLite persistence)
            if let Err(e) = pipeline.store_working(&memory) {
                tracing::warn!("Failed to store JSON memory via pipeline: {}", e);
            }
            
            // Store in Working Memory cache (Architecture §6.3)
            let memory_item = MemoryItem::from(&memory);
            working_memory.store(memory_item).await;
            memory_ids.push(memory.id.to_string());
        }
    }

    Ok(IngestResult {
        filename,
        file_path: path.to_string_lossy().to_string(),
        success: true,
        chunks_created: memory_ids.len(),
        chunk_size_used: 0,
        memory_ids,
        error: None,
        remaining_count: 0,
    })
}

/// Ingest an image file - store only metadata, not image content
/// Per Architecture §6.3: Stores in Working Memory cache
async fn ingest_image_file(
    path: &Path,
    chunk_size: usize,
    memory_type: MemoryType,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<IngestResult> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Extract image metadata
    let metadata = match extract_image_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            return Ok(IngestResult {
                filename,
                file_path: path.to_string_lossy().to_string(),
                success: false,
                chunks_created: 0,
                chunk_size_used: chunk_size,
                memory_ids: vec![],
                error: Some(format!("Failed to extract image metadata: {}", e)),
                remaining_count: 0,
            });
        }
    };

    // Convert metadata to memory content
    let content = metadata.to_memory_content();

    // Store as a single chunk using MemoryPipeline
    // Also store in Working Memory cache (Architecture §6.3)
    let memory = MemoryCard::new(content, memory_type);
    let pipeline = MemoryPipeline::new(db.clone());
    
    // Store in pipeline (for SQLite persistence)
    if let Err(e) = pipeline.store_working(&memory) {
        return Ok(IngestResult {
            filename,
            file_path: path.to_string_lossy().to_string(),
            success: false,
            chunks_created: 0,
            chunk_size_used: chunk_size,
            memory_ids: vec![],
            error: Some(format!("Failed to store image memory: {}", e)),
            remaining_count: 0,
        });
    }
    
    // Store in Working Memory cache (Architecture §6.3)
    let memory_item = MemoryItem::from(&memory);
    working_memory.store(memory_item).await;

    Ok(IngestResult {
        filename,
        file_path: path.to_string_lossy().to_string(),
        success: true,
        chunks_created: 1,
        chunk_size_used: chunk_size,
        memory_ids: vec![memory.id.to_string()],
        error: None,
        remaining_count: 0,
    })
}

fn parse_memory_type(s: &str) -> MemoryType {
    match s.to_lowercase().as_str() {
        "file" => MemoryType::File,
        "conversation" => MemoryType::Conversation,
        "code" => MemoryType::Code,
        "note" => MemoryType::Note,
        _ => MemoryType::File,
    }
}

/// Transcribe an audio file and store as memory
async fn ingest_audio_file(
    path: &Path,
    _chunk_size: usize,
    _memory_type: MemoryType,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<IngestResult> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let file_path_str = path.to_string_lossy().to_string();

    tracing::info!("Transcribing audio file: {}", filename);

    // Transcribe the audio
    let transcription = match audio_transcriber::transcribe_audio(path) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to transcribe audio: {}", e);
            return Ok(IngestResult {
                filename,
                file_path: file_path_str,
                success: false,
                chunks_created: 0,
                chunk_size_used: 0,
                memory_ids: vec![],
                error: Some(format!("Transcription failed: {}", e)),
                remaining_count: 0,
            });
        }
    };

    // Store the transcription as memory
    let memory_ids = match store_transcription_as_memory(
        &transcription,
        &filename,
        &file_path_str,
        db,
        working_memory,
    )
    .await
    {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("Failed to store transcription as memory: {}", e);
            return Ok(IngestResult {
                filename,
                file_path: file_path_str,
                success: false,
                chunks_created: 0,
                chunk_size_used: 0,
                memory_ids: vec![],
                error: Some(format!("Failed to store memory: {}", e)),
                remaining_count: 0,
            });
        }
    };

    Ok(IngestResult {
        filename,
        file_path: file_path_str,
        success: true,
        chunks_created: memory_ids.len(),
        chunk_size_used: 0,
        memory_ids,
        error: None,
        remaining_count: 0,
    })
}

/// Tool: Transcribe an audio file
pub async fn execute_transcribe_audio(
    input: TranscribeAudioInput,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<ToolOutput> {
    let path = Path::new(&input.path);

    if !path.exists() {
        return Ok(ToolOutput::error(format!(
            "Audio file not found: {}",
            input.path
        )));
    }

    if !is_audio_file(path) {
        return Ok(ToolOutput::error(format!(
            "Not a supported audio file: {}",
            input.path
        )));
    }

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    tracing::info!("Transcribing audio file: {}", filename);

    // Transcribe the audio (audio analysis works without Whisper model)
    // Full Whisper transcription requires enabling Candle dependencies
    let transcription = match audio_transcriber::transcribe_audio(path) {
        Ok(t) => t,
        Err(e) => {
            return Ok(ToolOutput::error(format!(
                "Transcription failed: {}",
                e
            )));
        }
    };

    // Store as memory if requested
    let memory_ids = if input.store_as_memory.unwrap_or(true) {
        match store_transcription_as_memory(
            &transcription,
            &filename,
            &input.path,
            db,
            working_memory,
        )
        .await
        {
            Ok(ids) => ids,
            Err(e) => {
                tracing::warn!("Failed to store as memory: {}", e);
                vec![]
            }
        }
    } else {
        vec![]
    };

    Ok(ToolOutput::success(serde_json::json!({
        "success": true,
        "filename": filename,
        "path": input.path,
        "language": transcription.language,
        "duration_seconds": transcription.duration_seconds,
        "text": transcription.text,
        "segments": transcription.segments.iter().map(|s| {
            serde_json::json!({
                "text": s.text,
                "start": s.start,
                "end": s.end
            })
        }).collect::<Vec<_>>(),
        "memory_ids": memory_ids,
        "message": format!(
            "Successfully transcribed {:.1}s audio to {} characters",
            transcription.duration_seconds,
            transcription.text.len()
        )
    })))
}
