// src/tools/ingestor.rs
// File ingestion tool for importing documents into short-term memory

use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use tar::Archive;
use uuid::Uuid;
use zip::ZipArchive;

use crate::database::models::{MemoryCard, MemoryType};
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;

/// Default folder name for files to import
pub const DEFAULT_IMPORT_FOLDER: &str = "files_to_import";

/// Default chunk size for text splitting
pub const DEFAULT_CHUNK_SIZE: usize = 1000;

/// Default overlap between chunks
pub const DEFAULT_CHUNK_OVERLAP: usize = 100;

/// Supported file extensions
const TEXT_EXTENSIONS: &[&str] = &["txt", "md", "rst", "csv", "log", "xml", "html", "htm"];
const JSON_EXTENSIONS: &[&str] = &["json", "jsonl"];
const PDF_EXTENSIONS: &[&str] = &["pdf"];
const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "tar", "gz", "tgz", "tar.gz"];
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "m4a", "flac", "ogg", "aac"];

/// Tool: Ingest files from import folder
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct IngestFilesInput {
    /// Path to the folder containing files to import (defaults to 'files_to_import')
    pub folder: Option<String>,
    /// Maximum number of files to process
    pub limit: Option<usize>,
    /// Chunk size for text splitting
    pub chunk_size: Option<usize>,
    /// Whether to delete files after successful ingestion
    pub delete_after: Option<bool>,
    /// Memory type to use for ingested content
    pub memory_type: Option<String>,
}

/// Tool: List files ready for import
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListImportableInput {
    /// Path to the folder to check (defaults to 'files_to_import')
    pub folder: Option<String>,
}

/// Tool: Transcribe an audio file
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TranscribeAudioInput {
    /// Path to the audio file
    pub path: String,
    /// Output path for the transcription JSON file
    pub output: Option<String>,
}

/// Result of ingesting a single file
#[derive(Debug, Clone, Serialize)]
pub struct IngestResult {
    pub filename: String,
    pub success: bool,
    pub chunks_created: usize,
    pub memory_ids: Vec<String>,
    pub error: Option<String>,
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

/// File info for importable files
#[derive(Debug, Clone, Serialize)]
pub struct ImportableFile {
    pub path: String,
    pub filename: String,
    pub size: u64,
    pub file_type: String,
}

/// Tool definitions
pub mod definitions {
    pub const INGEST_FILES: &str = "ingest_files";
    pub const LIST_IMPORTABLE: &str = "list_importable";
    pub const TRANSCRIBE_AUDIO: &str = "transcribe_audio";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: INGEST_FILES.to_string(),
                description: "Ingest files from a folder into short-term memory. Handles zip, tar, pdf, txt, json, and audio files.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "folder": {
                            "type": "string",
                            "description": "Path to folder containing files to import (defaults to 'files_to_import')"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of files to process"
                        },
                        "chunk_size": {
                            "type": "number",
                            "description": "Size of text chunks (default: 1000 characters)"
                        },
                        "delete_after": {
                            "type": "boolean",
                            "description": "Delete files after successful ingestion (default: false)"
                        },
                        "memory_type": {
                            "type": "string",
                            "description": "Memory type for ingested content: note, file, conversation, code",
                            "enum": ["note", "file", "conversation", "code"]
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_IMPORTABLE.to_string(),
                description: "List all files ready for import in the import folder".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "folder": {
                            "type": "string",
                            "description": "Path to folder to check (defaults to 'files_to_import')"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: TRANSCRIBE_AUDIO.to_string(),
                description: "Transcribe an audio file to text (requires whisper or similar)".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the audio file"
                        },
                        "output": {
                            "type": "string",
                            "description": "Output path for transcription JSON file"
                        }
                    },
                    "required": ["path"]
                }),
            },
        ]
    }
}

/// Detect file type from extension
fn detect_file_type(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if TEXT_EXTENSIONS.contains(&ext.as_str()) {
        "text".to_string()
    } else if JSON_EXTENSIONS.contains(&ext.as_str()) {
        "json".to_string()
    } else if PDF_EXTENSIONS.contains(&ext.as_str()) {
        "pdf".to_string()
    } else if ARCHIVE_EXTENSIONS.contains(&ext.as_str()) {
        "archive".to_string()
    } else if AUDIO_EXTENSIONS.contains(&ext.as_str()) {
        "audio".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Extract text content from a file
fn extract_text(path: &Path) -> Result<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" | "md" | "rst" | "csv" | "log" | "xml" | "html" | "htm" => {
            fs::read_to_string(path).context("Failed to read text file")
        }
        "json" | "jsonl" => {
            let content = fs::read_to_string(path).context("Failed to read JSON file")?;
            // Pretty print JSON for better searchability
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(v) => Ok(serde_json::to_string_pretty(&v).unwrap_or(content)),
                Err(_) => Ok(content), // Return raw content if not valid JSON
            }
        }
        "pdf" => extract_pdf_text(path),
        _ => Err(anyhow::anyhow!("Unsupported file type: {}", ext)),
    }
}

/// Extract text from PDF file (basic implementation)
fn extract_pdf_text(path: &Path) -> Result<String> {
    let file = File::open(path).context("Failed to open PDF file")?;
    let mut reader = BufReader::new(file);
    
    // Simple PDF text extraction - reads raw PDF content
    // For production, use pdf-extract or lopdf crate
    let mut content = String::new();
    let mut buffer = [0u8; 8192];
    
    while let Ok(n) = reader.read(&mut buffer) {
        if n == 0 {
            break;
        }
        // Extract printable ASCII characters from PDF
        for &byte in &buffer[..n] {
            if byte >= 32 && byte <= 126 || byte == b'\n' || byte == b'\r' || byte == b'\t' {
                content.push(byte as char);
            }
        }
    }
    
    // Clean up the extracted content
    let cleaned: String = content
        .lines()
        .filter(|line| {
            !line.is_empty() && 
            !line.starts_with('%') && 
            !line.starts_with("<<") &&
            !line.starts_with(">>")
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    if cleaned.trim().is_empty() {
        Err(anyhow::anyhow!("No readable text found in PDF"))
    } else {
        Ok(cleaned)
    }
}

/// Chunk text into smaller pieces with overlap
fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < text.len() {
        let end = (start + chunk_size).min(text.len());
        let chunk = text[start..end].to_string();
        chunks.push(chunk);

        if end >= text.len() {
            break;
        }
        start = end - overlap.min(end);
    }

    chunks
}

/// Process an archive file and return contained files
fn process_archive(archive_path: &Path, temp_dir: &Path) -> Result<Vec<PathBuf>> {
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut extracted_files = Vec::new();

    match ext.as_str() {
        "zip" => {
            let file = File::open(archive_path).context("Failed to open zip file")?;
            let mut archive = ZipArchive::new(BufReader::new(file))?;
            
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let outpath = match file.enclosed_name() {
                    Some(path) => temp_dir.join(path),
                    None => continue,
                };

                if file.is_dir() {
                    fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let mut outfile = File::create(&outpath)?;
                    io::copy(&mut file, &mut outfile)?;
                    extracted_files.push(outpath);
                }
            }
        }
        "tar" => {
            let file = File::open(archive_path).context("Failed to open tar file")?;
            let mut archive = Archive::new(file);
            
            for entry in archive.entries()? {
                let mut entry = entry?;
                let outpath = temp_dir.join(entry.path()?);
                
                if entry.header().entry_type().is_dir() {
                    fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    entry.unpack(&outpath)?;
                    extracted_files.push(outpath);
                }
            }
        }
        "gz" | "tgz" => {
            // Handle .gz and .tar.gz files
            if archive_path.to_string_lossy().ends_with(".tar.gz") || ext == "tgz" {
                let file = File::open(archive_path).context("Failed to open tar.gz file")?;
                let mut archive = Archive::new(GzDecoder::new(file));
                
                for entry in archive.entries()? {
                    let mut entry = entry?;
                    let outpath = temp_dir.join(entry.path()?);
                    
                    if entry.header().entry_type().is_dir() {
                        fs::create_dir_all(&outpath)?;
                    } else {
                        if let Some(parent) = outpath.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        entry.unpack(&outpath)?;
                        extracted_files.push(outpath);
                    }
                }
            } else {
                // Single gzipped file
                let file = File::open(archive_path)?;
                let mut decoder = GzDecoder::new(file);
                let mut content = String::new();
                decoder.read_to_string(&mut content)?;
                
                let outpath = temp_dir.join(
                    archive_path.file_stem().unwrap_or_default().to_string_lossy().as_ref()
                );
                fs::write(&outpath, content)?;
                extracted_files.push(outpath);
            }
        }
        _ => return Err(anyhow::anyhow!("Unsupported archive format: {}", ext)),
    }

    Ok(extracted_files)
}

/// Extract text from audio file (placeholder - requires external transcription service)
fn extract_audio_text(path: &Path, output_path: Option<&Path>) -> Result<String> {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    
    // Placeholder implementation
    // In production, integrate with:
    // - OpenAI Whisper API
    // - Local Whisper model
    // - Azure Speech Services
    // - Google Speech-to-Text
    
    let placeholder_text = format!(
        "[Audio file: {}]\n\
        This audio file requires transcription using an external service.\n\
        Supported transcription services:\n\
        - OpenAI Whisper API\n\
        - Local Whisper model\n\
        - Azure Speech-to-Text\n\
        - Google Speech-to-Text\n\
        \n\
        To transcribe this file, run the transcribe_audio tool with the file path.",
        filename
    );
    
    // If output path specified, write placeholder JSON
    if let Some(output) = output_path {
        let transcription = serde_json::json!({
            "file": path.to_string_lossy(),
            "transcription": placeholder_text,
            "status": "pending",
            "message": "Audio transcription requires external service"
        });
        
        let json = serde_json::to_string_pretty(&transcription)?;
        fs::write(output, json)?;
    }
    
    Ok(placeholder_text)
}

/// Ingest a single file into memory
fn ingest_file(
    path: &Path,
    database: &Arc<SqliteDatabase>,
    chunk_size: usize,
    overlap: usize,
    memory_type: &str,
) -> Result<IngestResult> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let file_type = detect_file_type(path);
    
    // Extract text content
    let text = match file_type.as_str() {
        "audio" => extract_audio_text(path, None)?,
        _ => extract_text(path)?,
    };

    // Chunk the text
    let chunks = chunk_text(&text, chunk_size, overlap);
    
    // Store chunks in memory
    let mut memory_ids = Vec::new();
    let conn = database.connection()?;
    
    for (i, chunk) in chunks.iter().enumerate() {
        let memory = MemoryCard {
            id: Uuid::new_v4(),
            content: format!(
                "[File: {}] [Chunk {}/{}]\n\n{}",
                filename,
                i + 1,
                chunks.len(),
                chunk
            ),
            memory_type: parse_memory_type(memory_type),
            confidence: 0.9, // Ingested files get high confidence
            importance: 0.7,  // Default importance
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        queries::insert_memory(&conn, &memory)?;
        memory_ids.push(memory.id.to_string());
    }

    Ok(IngestResult {
        filename,
        success: true,
        chunks_created: chunks.len(),
        memory_ids,
        error: None,
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

/// Get the import folder path
fn get_import_folder(folder: Option<&str>) -> PathBuf {
    folder
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_IMPORT_FOLDER))
}

/// Collect all files from import folder (including from archives)
fn collect_importable_files(folder: &Path) -> Result<Vec<ImportableFile>> {
    let mut files = Vec::new();

    if !folder.exists() {
        return Ok(files);
    }

    // Create temp directory for archive extraction
    let temp_dir = folder.join(".temp_extract");
    fs::create_dir_all(&temp_dir)?;

    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        
        // Skip temp directory
        if path.to_string_lossy().contains(".temp_extract") {
            continue;
        }

        let file_type = detect_file_type(&path);
        
        if file_type == "archive" {
            // Process archive and add contained files
            match process_archive(&path, &temp_dir) {
                Ok(extracted) => {
                    for extracted_path in extracted {
                        if extracted_path.is_file() {
                            let metadata = fs::metadata(&extracted_path)?;
                            files.push(ImportableFile {
                                path: extracted_path.to_string_lossy().to_string(),
                                filename: extracted_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                size: metadata.len(),
                                file_type: detect_file_type(&extracted_path),
                            });
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to extract archive {:?}: {}", path, e);
                }
            }
        } else if path.is_file() {
            let metadata = fs::metadata(&path)?;
            files.push(ImportableFile {
                path: path.to_string_lossy().to_string(),
                filename: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                size: metadata.len(),
                file_type,
            });
        }
    }

    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir);

    Ok(files)
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Execute ingest files tool
pub async fn execute_ingest_files(
    input: IngestFilesInput,
    database: &Arc<SqliteDatabase>,
) -> Result<serde_json::Value> {
    let folder = get_import_folder(input.folder.as_deref());
    let chunk_size = input.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
    let overlap = DEFAULT_CHUNK_OVERLAP;
    let delete_after = input.delete_after.unwrap_or(false);
    let memory_type = input.memory_type.unwrap_or_else(|| "file".to_string());
    let limit = input.limit;

    tracing::info!("Starting file ingestion from {:?}", folder);

    // Collect all importable files
    let files = collect_importable_files(&folder)?;
    let files_to_process: Vec<_> = if let Some(lim) = limit {
        files.into_iter().take(lim).collect()
    } else {
        files
    };

    let total_files = files_to_process.len();
    let mut results = Vec::new();
    let mut total_chunks = 0;
    let mut successful = 0;
    let mut failed = 0;

    for file_info in files_to_process {
        let path = Path::new(&file_info.path);
        
        match ingest_file(path, database, chunk_size, overlap, &memory_type) {
            Ok(result) => {
                total_chunks += result.chunks_created;
                successful += 1;
                
                // Delete original file if requested
                if delete_after {
                    if let Err(e) = fs::remove_file(path) {
                        tracing::warn!("Failed to delete {:?}: {}", path, e);
                    }
                }
                
                results.push(result);
            }
            Err(e) => {
                failed += 1;
                results.push(IngestResult {
                    filename: file_info.filename,
                    success: false,
                    chunks_created: 0,
                    memory_ids: vec![],
                    error: Some(e.to_string()),
                });
            }
        }
    }

    let summary = IngestSummary {
        total_files,
        successful,
        failed,
        total_chunks,
        results,
    };

    Ok(serde_json::to_value(summary)?)
}

/// Execute list importable tool
pub async fn execute_list_importable(
    input: ListImportableInput,
) -> Result<serde_json::Value> {
    let folder = get_import_folder(input.folder.as_deref());
    
    let files = collect_importable_files(&folder)?;
    
    Ok(serde_json::json!({
        "folder": folder.to_string_lossy(),
        "count": files.len(),
        "files": files
    }))
}

/// Execute transcribe audio tool
pub async fn execute_transcribe_audio(
    input: TranscribeAudioInput,
) -> Result<serde_json::Value> {
    let audio_path = Path::new(&input.path);
    
    if !audio_path.exists() {
        return Err(anyhow::anyhow!("Audio file not found: {}", input.path));
    }
    
    let file_type = detect_file_type(audio_path);
    if file_type != "audio" {
        return Err(anyhow::anyhow!("Not an audio file: {}", input.path));
    }
    
    let output_path = input.output
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("transcription.json"));
    
    let text = extract_audio_text(audio_path, Some(&output_path))?;
    
    Ok(serde_json::json!({
        "success": true,
        "audio_file": input.path,
        "transcription_file": output_path.to_string_lossy(),
        "transcription": text,
        "note": "This is a placeholder. Connect to Whisper API or local model for actual transcription."
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_type() {
        assert_eq!(detect_file_type(Path::new("test.txt")), "text");
        assert_eq!(detect_file_type(Path::new("test.json")), "json");
        assert_eq!(detect_file_type(Path::new("test.pdf")), "pdf");
        assert_eq!(detect_file_type(Path::new("test.zip")), "archive");
        assert_eq!(detect_file_type(Path::new("test.mp3")), "audio");
        assert_eq!(detect_file_type(Path::new("test.unknown")), "unknown");
    }

    #[test]
    fn test_chunk_text() {
        let text = "Hello world! This is a test. ".repeat(100);
        let chunks = chunk_text(&text, 50, 10);
        
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|c| c.len() <= 50 || c.len() < 50));
    }

    #[test]
    fn test_chunk_small_text() {
        let text = "Short text";
        let chunks = chunk_text(text, 1000, 100);
        
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }
}
