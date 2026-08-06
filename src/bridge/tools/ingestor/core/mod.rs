// src/tools/ingestor/core/mod.rs
//! Core file ingestion module

pub mod execute;
pub mod helpers;
pub mod ingestion;
pub mod tracker;
pub mod types;

// Re-exports for types (transitive - used by ingestor/mod.rs)
pub use types::{
    DeleteIngestedFilesInput, IngestFilesInput, ListImportableInput, ListIngestedFilesInput,
    TranscribeAudioInput,
};
