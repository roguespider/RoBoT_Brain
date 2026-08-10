//! Store-memory tool handler.
//!
//! Wires the MCP store request through the cognitive pipeline: an observation
//! is recorded, an experience is derived, and the resulting memory is cached
//! in Working Memory (Architecture §6.3) and checkpointed to the database.

use std::sync::Arc;

use anyhow::Result;

use crate::bridge::tools::ToolOutput;
use crate::database::models::{MemoryCard, Observation};
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::types::{
    Experience, ExperienceContext, ExperienceOutcome, ExperienceType,
};
use crate::memory::types::{MemoryItem, MemoryLayer};
use crate::memory::WorkingMemory;

use super::super::helpers::{convert_memory_type_to_memory, parse_memory_type};
use super::super::types::StoreMemoryInput;

/// Execute store memory tool
/// Per Architecture §07: Every experience originates from observations
/// Per Architecture §6.3: Stores in Working Memory (fast, volatile, in-memory cache)
pub async fn execute_store_memory(
    input: StoreMemoryInput,
    database: &Arc<SqliteDatabase>,
    working_memory: &Arc<WorkingMemory>,
) -> Result<ToolOutput> {
    let conn = database.connection()?;

    let memory_type = parse_memory_type(&input.memory_type);

    // Step 1: Create an Observation (Per Architecture §07 invariant)
    let content_preview = if input.content.len() > 100 {
        format!("{}...", &input.content[..100])
    } else {
        input.content.clone()
    };
    let observation = Observation::new(
        content_preview.clone(),
        format!("memory_type={}", input.memory_type),
        "memory_store".to_string(),
    );
    let observation_id = observation.id;

    // Step 2: Create an Experience with observation origin (Per Architecture §07)
    let mut experience = Experience::new(
        format!("Memory stored: {}", input.memory_type),
        format!("Stored {} memory: {}", input.memory_type, content_preview),
        ExperienceType::MemoryStore,
        vec![observation_id],
    );
    experience.context = ExperienceContext {
        memory_type: Some(input.memory_type.clone()),
        content_length: Some(input.content.len()),
        source: Some("store_memory_tool".to_string()),
        ..Default::default()
    };
    experience.outcome = ExperienceOutcome::success();
    experience.tags = vec!["memory".to_string(), memory_type.to_string()];

    // Step 3: Create the MemoryItem for Working Memory cache (Architecture §6.3)
    let mut memory_item = MemoryItem::new(
        MemoryLayer::Working,
        convert_memory_type_to_memory(memory_type.clone()),
        input.content.clone(),
        "store_memory_tool".to_string(),
    );
    memory_item.confidence = input.confidence.unwrap_or(0.5);
    memory_item.importance = input.importance.unwrap_or(0.5);
    if let Some(tags) = input.tags {
        for tag in tags {
            memory_item.add_tag(tag);
        }
    }

    let memory_id = memory_item.id;
    let experience_id = experience.id;

    // Store in Working Memory cache (Architecture §6.3)
    working_memory.store(memory_item.clone()).await;

    // Store observation first (per Architecture §07)
    queries::insert_observation(&conn, &observation)?;

    if let Err(e) = experience.commit() {
        tracing::warn!("Experience already committed: {}", e);
    }
    let memory_from_exp = MemoryCard::from_experience(&experience);
    queries::insert_memory(&conn, &memory_from_exp)?;

    let memory_card: MemoryCard = memory_item.into();
    queries::insert_memory(&conn, &memory_card)?;

    tracing::info!(
        "Memory stored in Working Memory cache with observation and experience: \
         memory_id={}, observation_id={}, experience_id={}",
        memory_id, observation_id, experience_id
    );

    Ok(ToolOutput::success(serde_json::json!({
        "success": true,
        "message": "Memory stored successfully in Working Memory cache with observation and experience",
        "id": memory_id.to_string(),
        "observation_id": observation_id.to_string(),
        "experience_id": experience_id.to_string(),
        "layer": "working",
        "note": "Per Architecture §9: Memory will be evaluated before promotion to Permanent layer"
    })))
}
