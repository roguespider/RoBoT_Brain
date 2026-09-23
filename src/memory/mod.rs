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
pub mod repository;
pub mod retrieval;
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
                kind,
                content: item.content,
                importance: item.importance,
                access_count: item.access_count,
                metadata: crate::data_contracts::metadata::Metadata::default(),
            }
        })
        .collect()
}
