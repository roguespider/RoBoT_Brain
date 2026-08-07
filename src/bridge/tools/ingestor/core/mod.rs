// src/tools/ingestor/core/mod.rs
//! Core file ingestion module

pub mod execute;
pub mod helpers;
pub mod ingestion;
pub mod tracker;
pub mod types;

// Re-exports for types (TranscribeAudioInput only when audio feature is enabled)
pub use types::{
    DeleteIngestedFilesInput, IngestFilesInput, ListImportableInput, ListIngestedFilesInput,
};
#[cfg(feature = "audio")]
pub use types::TranscribeAudioInput;
