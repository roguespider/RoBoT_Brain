

// src/tools/ingestor/mod.rs
// Ingestor module - file ingestion for short-term memory


pub mod archive_handler;
#[cfg(feature = "audio")]
pub mod audio_transcriber;
pub mod core;
pub mod definitions;
pub mod file_collector;
pub mod json_importer;
pub mod semantic_chunker;
pub mod text_extractor;
pub mod workflow;

// Re-export types from core
pub use core::{
    IngestFilesInput, ListImportableInput, ListIngestedFilesInput, DeleteIngestedFilesInput,
    TranscribeAudioInput,
};

// Re-export workflow functions (these take single input argument)
pub use workflow::{
    execute_delete_ingested_files, execute_list_ingested_files, execute_list_importable,
};

// Re-export execute_transcribe_audio (ingest_file is aliased below)
#[cfg(feature = "audio")]
pub use core::execute::execute_transcribe_audio;

// Alias ingest_file to execute_ingest_files for backward compatibility
pub use core::execute::execute_ingest_files as ingest_file;

// Re-export tracker functions
pub use core::tracker::{can_delete_files, can_verify_deletion, clear_ingest_tracker};

// Re-export JSON importer

// Re-export semantic chunker
