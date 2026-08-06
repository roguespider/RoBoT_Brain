// src/tools/ingestor/core/ingestion.rs
//! Core file ingestion logic

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::database::models::{MemoryCard, MemoryType};
use crate::database::sqlite::SqliteDatabase;
use crate::memory::pipeline::MemoryPipeline;
use crate::memory::types::MemoryItem;
use crate::memory::WorkingMemory;
use crate::tools::ingestor::archive_handler::{
    create_archive_temp_dir, delete_empty_folders, process_archive,
};
use crate::tools::ingestor::audio_transcriber::{store_transcription_as_memory, transcribe_audio};
use crate::tools::ingestor::file_collector::{
    collect_all_files_recursive, AUDIO_EXTENSIONS, IMAGE_EXTENSIONS, JSON_EXTENSIONS,
    TEXT_EXTENSIONS,
};
use crate::tools::ingestor::json_importer::{import_json_file, ExtractedJsonData};
use crate::tools::ingestor::semantic_chunker::{get_file_type, parse_document};
use crate::tools::ingestor::text_extractor::{
    extract_image_metadata, extract_text, validate_text_quality,
};

use super::types::IngestResult;

/// Parse memory type string to MemoryType enum
pub fn parse_memory_type(s: &str) -> MemoryType {
    match s.to_lowercase().as_str() {
        "file" => MemoryType::File,
        "conversation" => MemoryType::Conversation,
        "code" => MemoryType::Code,
        "note" => MemoryType::Note,
        _ => MemoryType::File,
    }
}

/// Ingest an archive file
/// Per Architecture §6.3: Stores in Working Memory cache
pub async fn ingest_archive(
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
            let ext = f
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            TEXT_EXTENSIONS.contains(&ext.as_str())
                || JSON_EXTENSIONS.contains(&ext.as_str())
                || ext == "txt"
                || ext == "md"
                || ext == "html"
                || ext == "xml"
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
            error: Some(
                "Archive contains no text-based files (only images, binaries, etc.)".to_string(),
            ),
            remaining_count: 0,
        });
    }

    // Ingest the first text file
    let first_file = &text_files[0];
    let result =
        ingest_single_file(first_file, chunk_size, memory_type, db, working_memory).await?;

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
pub async fn ingest_single_file(
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
    let file_size = fs::metadata(path)?.len();
    let recommended_chunk_size = if chunk_size == 0 {
        // Auto-detect chunk size based on file size
        if file_size > 1_000_000 {
            8000 // Larger chunks for large files
        } else if file_size > 10_000 {
            2000 // Medium chunks for medium files
        } else {
            1000 // Default for small files
        }
    } else {
        chunk_size
    };

    // Check if this is an image file - handle separately
    if crate::tools::ingestor::file_collector::is_supported_extension(path, IMAGE_EXTENSIONS) {
        return ingest_image_file(
            path,
            recommended_chunk_size,
            memory_type,
            db,
            working_memory,
        )
        .await;
    }

    // Check if this is a JSON file - use smart JSON importer
    if crate::tools::ingestor::file_collector::is_supported_extension(path, JSON_EXTENSIONS) {
        return ingest_json_file(
            path,
            recommended_chunk_size,
            memory_type,
            db,
            working_memory,
        )
        .await;
    }

    // Check if this is an audio file - use Whisper transcription
    if crate::tools::ingestor::file_collector::is_supported_extension(path, AUDIO_EXTENSIONS) {
        return ingest_audio_file(
            path,
            recommended_chunk_size,
            memory_type,
            db,
            working_memory,
        )
        .await;
    }

    // Extract text content for other file types
    let text =
        extract_text(path).with_context(|| format!("Failed to extract text from {}", filename))?;

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
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("txt");
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
        chunk_size_used: 0, // Semantic chunking doesn't use fixed sizes
        memory_ids,
        error: None,
        remaining_count: 0,
    })
}

/// Ingest a JSON file using smart structured extraction
/// Per Architecture §6.3: Stores in Working Memory cache
pub async fn ingest_json_file(
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
            tracing::info!(
                "JSON file had no structured items, storing raw content ({} chars)",
                raw_content.len()
            );
            vec![ExtractedJsonData {
                content: raw_content,
                json_path: "root".to_string(),
                sibling_context: String::new(),
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
                None, // Top level
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
pub async fn ingest_image_file(
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

/// Transcribe an audio file and store as memory
pub async fn ingest_audio_file(
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

    let file_path_str = path.to_string_lossy().to_string();

    tracing::info!("Transcribing audio file: {}", filename);

    // Transcribe the audio
    let transcription = match transcribe_audio(path) {
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
