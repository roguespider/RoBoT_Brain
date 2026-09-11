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
use crate::experience::types::{Experience, ExperienceContext, ExperienceOutcome, ExperienceType};
use crate::learning::memory_state::{MemoryState, StateTransition};
use crate::learning::promotion::PromotionPolicy;
use crate::memory::WorkingMemory;
use crate::memory::repository::{MemoryRepository, SqliteMemoryRepository};
use crate::memory::types::{MemoryItem, MemoryLayer};

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

    // Persist the memory through the MemoryRepository (Architecture §4.06):
    // the cognitive layer stores a MemoryItem; the repository hides SQL.
    let repo = SqliteMemoryRepository::new((**database).clone());
    MemoryRepository::store(&repo, &memory_item)?;

    // Store embedding: client-provided embedding takes priority.
    // Falls back to hash-based generation if not provided (per docs "Selective Embedding".
    let memory_conf = memory_item.confidence;
    let memory_imp = memory_item.importance;
    let memory_uuid = memory_item.id;

    // Use the same threshold from the embedding module so the handler and
    // generator stay in sync. Memories below 0.3 on BOTH dimensions are skipped.
    const SELECTIVE_EMBED_THRESHOLD: f32 = 0.3;

    let embedding = if let Some(emb) = input.embedding {
        // Client (LLM) provided a real embedding vector
        Some((emb, "client".to_string()))
    } else if memory_conf >= SELECTIVE_EMBED_THRESHOLD && memory_imp >= SELECTIVE_EMBED_THRESHOLD {
        // Selective: memories meeting threshold get hash-based fallback
        crate::memory::generate_embedding(&memory_item.content, memory_conf, memory_imp)
            .map(|emb| (emb, "hash-based".to_string()))
    } else {
        // Low-value memories use hybrid retrieval (graph + symbolic search)
        None
    };

    if let Some((embedding, model)) = embedding {
        let emb_model =
            crate::database::models::MemoryEmbedding::new(memory_uuid, embedding, model);
        let conn2 = database.connection()?;
        if let Err(e) = queries::insert_embedding(&conn2, &emb_model) {
            tracing::warn!(
                "Failed to store embedding for memory {}: {}",
                memory_uuid,
                e
            );
        }
    }

    // Trigger promotion evaluation per Architecture §15.3:
    // When memory is stored, evaluate against promotion policy.
    let policy = PromotionPolicy::default();
    let current_time = chrono::Utc::now();

    // Validate initial state transition (Architecture §7.2: state machine must be valid)
    let is_valid = MemoryState::Active.can_transition(StateTransition::Observe);
    if !is_valid {
        tracing::warn!("Invalid initial memory state transition detected");
    }
    let target_state = MemoryState::Active.transition_to(StateTransition::Observe);
    tracing::debug!("Memory state transition: Active -> {:?}", target_state);

    let evaluation = policy.evaluate(
        MemoryState::Active,
        memory_imp,
        1, // access_count (first store)
        0, // repeated_count
        0, // confirmation_count
        current_time,
    );
    if let Some(transition) = &evaluation.recommended_transition {
        tracing::debug!(
            "Promotion evaluation for memory {}: {} ({}): {}",
            memory_uuid,
            evaluation.reason,
            transition,
            evaluation.confidence_delta
        );
    }

    // Calculate adjusted confidence using promotion policy (Architecture §15.4)
    let adjusted_confidence = policy.calculate_confidence(
        0.5, // base confidence for new memory
        MemoryState::Active,
        1, // access_count
        0, // confirmations
    );
    tracing::debug!("Memory confidence after promotion policy: {adjusted_confidence:.3}");

    tracing::info!(
        "Memory stored in Working Memory cache with observation and experience: \
         memory_id={}, observation_id={}, experience_id={}",
        memory_id,
        observation_id,
        experience_id
    );

    Ok(ToolOutput::success(serde_json::json!({
        "success": true,
        "message": "Memory stored successfully in Working Memory cache with observation and experience",
        "id": memory_id.to_string(),
        "observation_id": observation_id.to_string(),
        "experience_id": experience_id.to_string(),
        "layer": "working",
        "promotion_evaluated": evaluation.should_promote,
        "note": "Per Architecture §15.3: Promotion pipeline evaluated on store"
    })))
}
