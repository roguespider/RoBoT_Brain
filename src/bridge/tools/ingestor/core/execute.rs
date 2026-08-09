// src/tools/ingestor/core/execute.rs
//! Main execution functions for file ingestion tools

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::time;

use crate::database::models::MemoryType;
use crate::database::sqlite::SqliteDatabase;
use crate::memory::WorkingMemory;
use crate::bridge::tools::ToolOutput;
use crate::bridge::tools::ingestor::file_collector::{
    collect_importable_files, collect_importable_files_with_recursive, get_import_folder,
};
use crate::bridge::tools::ingestor::workflow::find_empty_folders_after_deletion;

use super::helpers::{file_info_size, format_size, is_archive_file};
use super::ingestion::{ingest_archive, ingest_single_file};
use super::tracker::record_ingested_files;
use super::types::{
    IngestFilesInput, IngestResult, TranscribeAudioInput,
    DEFAULT_INGEST_TIMEOUT_SECS,
};

/// Tool: Ingest files from import folder
pub async fn execute_ingest_files(
    input: IngestFilesInput,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<ToolOutput> {
    // Get the import folder
    let folder: PathBuf = input.folder.clone()
        .map(PathBuf::from)
        .unwrap_or_else(|| get_import_folder(input.folder.as_deref()));
    let chunk_size = input.chunk_size.unwrap_or(1000);
    let memory_type = input.memory_type.clone().unwrap_or_else(|| "file".to_string());
    let force = input.force.unwrap_or(false);
    let summary_only = input.summary_only.unwrap_or(false);
    let timeout_secs = input.timeout_seconds.unwrap_or(DEFAULT_INGEST_TIMEOUT_SECS);
    let recursive = input.recursive.unwrap_or(true);

    // Handle single file path
    if let Some(file_path) = input.get_file_path() {
        let path = Path::new(file_path);

        if !path.exists() {
            return Ok(ToolOutput::error(format!("File not found: {}", file_path)));
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Resolve path
        let resolved_path = match super::helpers::resolve_path(file_path) {
            Ok(p) => p,
            Err(e) => {
                return Ok(ToolOutput::error(format!(
                    "Failed to resolve path: {}",
                    e
                )));
            }
        };

        // Check if it's an archive and handle accordingly
        let result = if is_archive_file(&resolved_path) {
            time::timeout(
                Duration::from_secs(timeout_secs),
                ingest_archive(
                    &resolved_path,
                    chunk_size,
                    super::ingestion::parse_memory_type(&memory_type),
                    db.clone(),
                    working_memory.clone(),
                ),
            )
            .await
            .map(|r| r.map(|res| vec![res]))
        } else {
            time::timeout(
                Duration::from_secs(timeout_secs),
                async { Ok(vec![ingest_single_file(
                    &resolved_path,
                    chunk_size,
                    super::ingestion::parse_memory_type(&memory_type),
                    db.clone(),
                    working_memory.clone(),
                ).await?]) }
            )
            .await
        };

        match result {
            Ok(Ok(results)) => {
                // Get the first result for summary
                let first = results.first();
                
                // Record all ingested files
                for r in &results {
                    if r.success {
                        record_ingested_files(vec![r.file_path.clone()]).await;
                    }
                }

                // Cleanup WAL
                if let Err(e) = db.cleanup_wal_files() {
                    tracing::warn!("Failed to cleanup WAL files: {}", e);
                }

                // Calculate totals
                let total_chunks: usize = results.iter().map(|r| r.chunks_created).sum();
                let all_success = results.iter().all(|r| r.success);
                let first_error = results.iter().find(|r| !r.success).and_then(|r| r.error.clone());

                return Ok(ToolOutput::success(serde_json::json!({
                    "success": all_success,
                    "filename": first.map(|r| r.filename.clone()).unwrap_or(filename.clone()),
                    "file_path": first.map(|r| r.file_path.clone()).unwrap_or_default(),
                    "file_size": first.as_ref().map(|r| file_info_size(&r.file_path)).unwrap_or_default(),
                    "chunks_created": total_chunks,
                    "memory_ids": results.iter().flat_map(|r| r.memory_ids.clone()).collect::<Vec<_>>(),
                    "error": first_error,
                    "message": if all_success {
                        format!("Added {} chunks from '{}'", total_chunks, filename)
                    } else {
                        results.iter().find(|r| !r.success)
                            .and_then(|r| r.error.clone())
                            .unwrap_or_else(|| "Some files failed".to_string())
                    },
                    "ask_delete_file": if all_success {
                        serde_json::json!("Can I delete the original file to save space?")
                    } else {
                        serde_json::Value::Null
                    },
                    "deletion_candidates": if all_success {
                        results.iter().map(|r| r.file_path.clone()).collect::<Vec<_>>()
                    } else {
                        vec![]
                    },
                    "timeout_occurred": false
                })));
            }
            Ok(Err(e)) => {
                return Ok(ToolOutput::error(format!(
                    "Failed to ingest file: {}",
                    e
                )));
            }
            Err(_) => {
                return Ok(ToolOutput::error(format!(
                    "Ingestion timed out after {} seconds. Try increasing timeout_seconds.",
                    timeout_secs
                )));
            }
        }
    }

    // Handle folder ingestion
    let limit = input.limit.unwrap_or(1);

    // Collect files
    let all_files = if recursive {
        collect_importable_files_with_recursive(&folder, recursive)?
    } else {
        collect_importable_files(&folder)?
    };

    if all_files.is_empty() {
        return Ok(ToolOutput::success(serde_json::json!({
            "success": true,
            "message": "No files found to ingest",
            "files": Vec::<String>::new(),
            "total_chunks": 0
        })));
    }

    // Filter out files with skip reasons
    let (skipped_files, files_to_check): (Vec<_>, Vec<_>) = all_files
        .into_iter()
        .partition(|f| f.skip_reason.is_some());

    // Check for already-ingested files and separate them
    let ingest_tracker = super::tracker::get_ingest_tracker_public().try_lock().ok();
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
        let filename = file_info.filename.clone();

        // Resolve the path to ensure it works on all platforms (handles Windows \\?\ prefix)
        let path = match super::helpers::resolve_path(&file_info.path) {
            Ok(p) => p,
            Err(e) => {
                failed += 1;
                results.push(IngestResult {
                    filename,
                    file_path: file_info.path.clone(),
                    success: false,
                    chunks_created: 0,
                    chunk_size_used: chunk_size,
                    memory_ids: vec![],
                    error: Some(format!("Failed to resolve path: {}", e)),
                    remaining_count: 0,
                });
                continue;
            }
        };

        // Check if it's an archive
        if file_info.file_type == "archive" {
            let result = time::timeout(
                Duration::from_secs(timeout_secs),
                ingest_archive(&path, chunk_size, super::ingestion::parse_memory_type(&memory_type), db.clone(), working_memory.clone())
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
                ingest_single_file(&path, chunk_size, super::ingestion::parse_memory_type(&memory_type), db.clone(), working_memory.clone())
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

    let successfully_ingested: Vec<String> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.file_path.clone())
        .collect();

    // RECORD INGESTED FILES for deletion tracking
    if !successfully_ingested.is_empty() {
        record_ingested_files(successfully_ingested.clone()).await;
    }

    // CLEANUP WAL FILES after batch operations
    if let Err(e) = db.cleanup_wal_files() {
        tracing::warn!("Failed to cleanup WAL files: {}", e);
    }

    // Build detailed file info for the summary
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

    let already_ingested_filenames: Vec<String> = already_ingested.iter().map(|f| f.filename.clone()).collect();

    let skipped_file = if !skipped_files.is_empty() {
        Some(serde_json::json!({
            "filename": skipped_files[0].filename,
            "size": format_size(skipped_files[0].size),
            "reason": skipped_files[0].skip_reason
        }))
    } else {
        None
    };

    // Return compact response if summary_only is true
    if summary_only {
        let empty_folders = find_empty_folders_after_deletion(&successfully_ingested);

        return Ok(ToolOutput::success(serde_json::json!({
            "success": successful > 0,
            "total_chunks": total_chunks,
            "files": ingested_file_details.iter().map(|d| {
                serde_json::json!({
                    "filename": d.get("filename").and_then(|v| v.as_str()).unwrap_or("?"),
                    "size": d.get("file_size").and_then(|v| v.as_str()).unwrap_or("?"),
                    "chunks": d.get("chunks").and_then(|v| v.as_u64()).unwrap_or(0)
                })
            }).collect::<Vec<_>>(),
            "summary": if successful == 1 {
                format!("Added {} chunks to memory from '{}'",
                    total_chunks,
                    ingested_file_details.first().and_then(|d| d.get("filename")).and_then(|v| v.as_str()).unwrap_or("file"))
            } else {
                format!("Added {} chunks to memory from {} files", total_chunks, successful)
            },
            "ask_delete_file": if successful > 0 {
                serde_json::json!("Can I delete the original file to save space?")
            } else {
                serde_json::Value::Null
            },
            "deletion_candidates": successfully_ingested,
            "empty_folders": empty_folders.iter().map(|p| {
                p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.to_string_lossy().to_string())
            }).collect::<Vec<_>>(),
            "ask_delete_folders": if !empty_folders.is_empty() {
                serde_json::json!("Some folders are now empty. Can I delete them too?")
            } else {
                serde_json::Value::Null
            },
            "can't_ingest": skipped_file.clone(),
            "ask_skip": if skipped_file.is_some() {
                serde_json::json!("Skip this file and move to the next one?")
            } else if failed_file.is_some() {
                serde_json::json!("Do you want to retry ingesting this file?")
            } else {
                serde_json::Value::Null
            },
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
        "success": successful > 0,
        "total_chunks": total_chunks,
        "files": ingested_file_details.iter().map(|d| {
            serde_json::json!({
                "filename": d.get("filename").and_then(|v| v.as_str()).unwrap_or("?"),
                "size": d.get("file_size").and_then(|v| v.as_str()).unwrap_or("?"),
                "chunks": d.get("chunks").and_then(|v| v.as_u64()).unwrap_or(0)
            })
        }).collect::<Vec<_>>(),
        "summary": if successful == 1 {
            format!("Added {} chunks to memory from '{}'",
                total_chunks,
                ingested_file_details.first().and_then(|d| d.get("filename")).and_then(|v| v.as_str()).unwrap_or("file"))
        } else {
            format!("Added {} chunks to memory from {} files", total_chunks, successful)
        },
        "ask_delete_file": if successful > 0 {
            serde_json::json!("Can I delete the original file to save space?")
        } else {
            serde_json::Value::Null
        },
        "deletion_candidates": successfully_ingested,
        "empty_folders": find_empty_folders_after_deletion(&successfully_ingested).iter().map(|p| {
            p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.to_string_lossy().to_string())
        }).collect::<Vec<_>>(),
        "ask_delete_folders": if !find_empty_folders_after_deletion(&successfully_ingested).is_empty() {
            serde_json::json!("Some folders are now empty. Can I delete them too?")
        } else {
            serde_json::Value::Null
        },
        "can't_ingest": skipped_file.clone(),
        "ask_skip": if skipped_file.is_some() {
            serde_json::json!("Skip this file and move to the next one?")
        } else if failed_file.is_some() {
            serde_json::json!("Do you want to retry ingesting this file?")
        } else {
            serde_json::Value::Null
        },
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

/// Tool: Transcribe an audio file
#[cfg(feature = "audio")]
pub async fn execute_transcribe_audio(
    input: TranscribeAudioInput,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<ToolOutput> {
    use crate::bridge::tools::ingestor::audio_transcriber::{is_audio_file, store_transcription_as_memory, transcribe_audio};
    
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
    let transcription = match transcribe_audio(path) {
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
            MemoryType::File,
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

/// Tool: Transcribe an audio file (stub when audio feature is disabled)
#[cfg(not(feature = "audio"))]
pub async fn execute_transcribe_audio(
    _: TranscribeAudioInput,
    _: Arc<SqliteDatabase>,
    _: Arc<WorkingMemory>,
) -> Result<ToolOutput> {
    Ok(ToolOutput::error(
        "Audio transcription is not available. Enable the 'audio' feature to use this tool.".to_string()
    ))
}
