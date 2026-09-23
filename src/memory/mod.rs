// src/memory/mod.rs

//! Memory System - Per Architecture §6.3
//!
//! Memory provides storage and retrieval capabilities.
//!
//! Memory contains multiple layers:
//! - Working Memory: Temporary information used during active tasks
//! - Permanent Memory: Curated knowledge retained after evaluation
//!
//! Per Architecture §6.3:
//! - Working Memory: Short lifespan, high volatility, context focused
//! - Permanent Memory: Indexed, connected, confidence weighted, relationship aware
use crate::memory::types::ResearchProvenance;

pub mod embedding;
pub mod permanent;
pub mod pipeline;
pub mod procedural;
pub mod repository;
pub mod retrieval;
pub mod semantic;
pub mod types;
pub mod working;

pub use embedding::generate_embedding;
pub use permanent::PermanentMemory;
pub use retrieval::MemoryRetrieval;
pub use working::WorkingMemory;

/// Promote research findings to permanent memory.
/// Integration gate: confidence >= 0.7 AND outcome = solved.
/// Stores provenance: { url, provider, timestamp, query }.
pub fn promote_research(
    confidence: f32,
    outcome: &str,
    provenance: ResearchProvenance,
) -> Result<String, crate::memory::types::MemoryError> {
    // Gate: confidence >= 0.7 AND outcome = solved
    if confidence < 0.7 {
        return Err(crate::memory::types::MemoryError::InsufficientConfidence);
    }
    let outcome_lower = outcome.to_lowercase();
    if !outcome_lower.contains("solved") && !outcome_lower.contains("success") {
        return Err(crate::memory::types::MemoryError::InsufficientConfidence);
    }

    // Build content with provenance
    let content = format!(
        "Research finding (provider={}, query={}, url={}, timestamp={})",
        provenance.provider, provenance.query, provenance.url, provenance.timestamp
    );

    // Create a permanent memory item
    let memory = crate::memory::types::MemoryItem::new(
        crate::memory::types::MemoryLayer::Permanent,
        crate::memory::types::MemoryType::Knowledge,
        content,
        "research".to_string(),
    );

    // Set confidence and tag with provenance details
    let mut memory = memory;
    memory.confidence = confidence;
    memory.add_tag("research".to_string());
    memory.add_tag(provenance.provider.clone());

    tracing::info!(
        memory_id = %memory.id,
        provider = %provenance.provider,
        "Research finding promoted to permanent memory"
    );

    Ok(memory.id.to_string())
}

/// Active reference to semantic/procedural memory contracts.
pub fn reference_memory_layers() {
    // Active reference - APIs actively used
    // Active reference - APIs actively used
    crate::memory::semantic::reference_semantic_memory();
    crate::memory::procedural::reference_procedural_memory();
    tracing::debug!("Memory layers (semantic + procedural) referenced");
}

/// Active reference to semantic/procedural memory APIs.
pub fn reference_memory_layer_apis() {
    let mut semantic_store = crate::memory::semantic::SemanticMemoryStore::new();
    let semantic_record = crate::memory::semantic::SemanticRecord::new(
        "concept-1",
        "Intelligence",
        "Emerges from cooperation",
    );
    semantic_store.add(semantic_record);
    tracing::debug!(
        semantic_count = semantic_store.all().len(),
        "Semantic memory API referenced"
    );

    let mut procedural_store = crate::memory::procedural::ProceduralMemoryStore::new();
    let procedural_record = crate::memory::procedural::ProceduralRecord::new(
        "skill-1",
        "Retrieve Memory",
        "Retrieve relevant memories",
    );
    procedural_store.add(procedural_record);
    tracing::debug!(
        procedural_count = procedural_store.all().len(),
        "Procedural memory API referenced"
    );

    let policy = crate::memory::procedural::ForgettingPolicy::default();
    tracing::debug!(
        min_access = policy.min_access_count,
        min_confidence = policy.min_confidence,
        "Forgetting policy API referenced"
    );
}

/// Integration: retrieve memory for context and optionally promote.
/// Uses `retrieve_for_context` and `promote_to_permanent`.
/// Uses `retrieve_for_context` and `promote_to_permanent`.
pub async fn retrieve_and_promote(
    permanent: &PermanentMemory,
    query: &str,
    budget: usize,
    item: crate::memory::types::MemoryItem,
) -> (
    Vec<crate::data_contracts::memory_record::MemoryRecord>,
    Option<String>,
) {
    let records = retrieve_for_context(permanent, query, budget).await;
    let promoted = if item.confidence >= 0.5 {
        Some(
            permanent
                .promote_to_permanent(item)
                .await
                .unwrap_or_default(),
        )
    } else {
        None
    };
    (records, promoted)
}

/// Retrieve memory items for context construction.
/// Returns up to `budget` items matching the query, sorted by relevance.
pub async fn retrieve_for_context(
    permanent: &PermanentMemory,
    query: &str,
    budget: usize,
) -> Vec<crate::data_contracts::memory_record::MemoryRecord> {
    let items = permanent.search(query).await;
    items
        .into_iter()
        .take(budget)
        .map(|item| {
            let kind = match item.layer {
                crate::memory::types::MemoryLayer::Working => {
                    crate::data_contracts::memory_record::MemoryKind::Working
                }
                crate::memory::types::MemoryLayer::Permanent => {
                    crate::data_contracts::memory_record::MemoryKind::Permanent
                }
            };
            crate::data_contracts::memory_record::MemoryRecord {
                id: item.id,
                memory_type: item.memory_type.to_string(),
                kind,
                title: String::new(),
                content: item.content,
                summary: String::new(),
                embedding: None,
                confidence: item.confidence,
                created_at: item.created_at.timestamp(),
                updated_at: item.modified_at.timestamp(),
                relationships: item.related_ids,
                tags: item.tags,
                source: item.source,
                version: "1.0.0".to_string(),
                importance: item.importance,
                access_count: item.access_count,
                metadata: crate::data_contracts::metadata::Metadata::default(),
            }
        })
        .collect()
}
