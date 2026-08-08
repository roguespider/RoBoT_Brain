// src/tools/ingestor/core/types.rs
//! Input/Output types for file ingestion

use serde::{Deserialize, Serialize};

/// Default timeout for ingestion operations (60 seconds)
pub const DEFAULT_INGEST_TIMEOUT_SECS: u64 = 60;

/// Tool: Ingest files from import folder
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct ListIngestedFilesInput {
    pub folder: Option<String>,
    pub limit: Option<usize>,
    /// Search subfolders recursively (default: true) - matches ingest_files behavior
    pub recursive: Option<bool>,
}

/// Result of ingesting a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
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
