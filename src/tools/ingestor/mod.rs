

// src/tools/ingestor/mod.rs
// Ingestor module - file ingestion for short-term memory

pub mod archive_handler;
pub mod audio_transcriber;
pub mod core;
pub mod definitions;
pub mod file_collector;
pub mod json_importer;
pub mod semantic_chunker;
pub mod text_extractor;
pub mod workflow;

// Re-export main types and functions
pub use core::{
    execute_delete_ingested_files, execute_list_importable,
    execute_list_ingested_files, ingest_file,
    IngestFilesInput, ListImportableInput,
    DeleteIngestedFilesInput, ListIngestedFilesInput,
    can_delete_files, clear_ingest_tracker, can_verify_deletion,
    TranscribeAudioInput,
};

// Re-export JSON importer

// Re-export semantic chunker
