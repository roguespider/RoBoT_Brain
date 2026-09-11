// src/knowledge/mod.rs
//! Knowledge System - Manages information that has gained sufficient confidence
//! to influence reasoning.
//!
//! Per architecture #2.3:
//! - Maintain trusted information
//! - Track confidence
//! - Store relationships
//! - Manage knowledge evolution
//! - Connect concepts together
//!
//! Knowledge is not static - it changes as new evidence appears.

use crate::research::Finding;

pub mod query;
pub mod store;
pub mod types;

pub use query::{KnowledgeQuery, KnowledgeResult, apply_query, rank_items};
pub use store::KnowledgeStore;
pub use types::KnowledgeItem;

/// Promote high-confidence research findings to the knowledge store.
/// Gate: confidence >= 0.7 (per architecture).
/// Each Finding becomes a KnowledgeItem with source="research".
pub async fn promote_research_findings(
    store: &KnowledgeStore,
    findings: &[Finding],
    query: &str,
) -> usize {
    let mut promoted = 0usize;
    for finding in findings {
        if finding.confidence >= 0.7 {
            let item = KnowledgeItem::new(
                String::new(),
                types::KnowledgeType::Fact,
                finding.statement.clone(),
                finding.confidence,
                types::KnowledgeSource::External(format!("research:{}", finding.source_url)),
                vec!["research".to_string(), "promoted".to_string()],
            );
            let id = store.add(item).await;
            tracing::info!(
                knowledge_id = %id,
                confidence = finding.confidence,
                "Research finding promoted to knowledge"
            );
            promoted += 1;
        }
    }
    tracing::info!(
        findings_processed = findings.len(),
        promoted = promoted,
        query = query,
        "Research promotion complete"
    );
    promoted
}
