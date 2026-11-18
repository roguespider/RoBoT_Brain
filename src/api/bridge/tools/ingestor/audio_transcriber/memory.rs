// src/api/bridge/tools/ingestor/audio_transcriber/memory.rs
//! Store transcription as memory

use std::sync::Arc;

use anyhow::Result;

use crate::database::models::{MemoryCard, MemoryType};
use crate::database::sqlite::SqliteDatabase;
use crate::memory::WorkingMemory;
use crate::memory::pipeline::MemoryPipeline;
use crate::memory::types::MemoryItem;

use super::format::format_transcription_as_memory;
use super::types::TranscriptionResult;

/// Store transcribed audio as memory
pub async fn store_transcription_as_memory(
    transcription: &TranscriptionResult,
    filename: &str,
    source_path: &str,
    memory_type: MemoryType,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<Vec<String>> {
    let content = format_transcription_as_memory(transcription, filename, source_path);

    let mut memory = MemoryCard::new(content, memory_type);
    memory.file_source = Some(source_path.to_string());

    let pipeline = MemoryPipeline::new(db.clone());
    pipeline.store_working(&memory)?;

    let memory_item = MemoryItem::from(&memory);
    working_memory.store(memory_item).await;

    Ok(vec![memory.id.to_string()])
}
