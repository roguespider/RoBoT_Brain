//! Memory ranking rules - Per Architecture §8.5 "Memory retrieval" + §19 (Confidence System)
//!
//! Provides ranking functions that prefer relevant, confident, and recent records.

use crate::data_contracts::memory_record::MemoryRecord;

/// Rank score based on confidence (weight: 0.4)
pub fn rank_by_confidence(rec: &MemoryRecord) -> f32 {
    rec.metadata.confidence * 0.4
}

/// Rank score based on recency using log-scaled decay (weight: 0.3)
pub fn rank_by_recency(rec: &MemoryRecord, now_ts: i64) -> f32 {
    let age = (now_ts - rec.metadata.created_at).abs() as f64;
    let decay = (1.0 / (1.0 + age.ln())) as f32;
    decay * 0.3
}

/// Rank score based on relevance (placeholder, weight: 0.3)
///
/// Real implementation will use vector similarity once T2-139 lands.
pub fn rank_by_relevance(rec: &MemoryRecord) -> f32 {
    tracing::debug!(
        content_len = rec.content.len(),
        confidence = rec.confidence,
        "Ranking by relevance placeholder"
    );
    0.3
}

/// Combined ranking score using weighted sum (total weight: 1.0)
pub fn rank_score(rec: &MemoryRecord, now_ts: i64) -> f32 {
    rank_by_confidence(rec) + rank_by_recency(rec, now_ts) + rank_by_relevance(rec)
}

/// Ranked search combining confidence, recency, and relevance
pub fn ranked_search(
    lt: &dyn crate::memory::promotion::LongTermMemory,
    q: &str,
) -> Vec<(MemoryRecord, f32)> {
    let results = lt.search(q);
    let now_ts = chrono::Utc::now().timestamp();

    let mut ranked: Vec<(MemoryRecord, f32)> = results
        .into_iter()
        .map(|rec| (rec.clone(), rank_score(&rec, now_ts)))
        .collect();

    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}
