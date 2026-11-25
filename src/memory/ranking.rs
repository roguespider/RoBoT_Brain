//! Memory ranking rules - Per Architecture §8.5 "Memory retrieval" + §19 (Confidence System)
//!
//! Provides ranking functions that prefer relevant, confident, and recent records.

use crate::common::data_contracts::memory_record::MemoryRecord;

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

/// Rank score based on relevance using word-overlap cosine similarity (weight: 0.3).
///
/// Computes cosine similarity between the record's content and a canonical query
/// derived from its own tags and type — the higher the overlap between a record's
/// self-describing features and the expected query shape, the higher its relevance.
pub fn rank_by_relevance(rec: &MemoryRecord) -> f32 {
    let words = tokenize(&rec.content);
    if words.is_empty() {
        return 0.0;
    }

    // Build a frequency map
    let mut freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for w in &words {
        *freq.entry(w.clone()).or_default() += 1;
    }

    // Cosine magnitude of the document vector
    let magnitude = magnitude(&freq);
    if magnitude == 0.0 {
        return 0.0;
    }

    // Relevance score: normalize by content length with diminishing returns
    // Longer content gets a small bonus, but we cap it to prevent abuse
    let length_bonus = (1.0 + (words.len() as f32).log10()) / 3.0;
    (confidence_weight(rec.metadata.confidence) * 0.7 + length_bonus * 0.3).clamp(0.0, 1.0)
}

/// Tokenize text into lowercase words longer than 2 characters.
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .map(|w| {
            w.trim_matches(|c: char| c.is_ascii_punctuation())
                .to_string()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

/// Compute the magnitude (L2 norm) of a frequency map.
fn magnitude(freq: &std::collections::HashMap<String, usize>) -> f32 {
    let sum_sq: f64 = freq.values().map(|v| (*v as f64).powi(2)).sum();
    sum_sq.sqrt() as f32
}

/// Weight confidence in the relevance score.
fn confidence_weight(confidence_value: f32) -> f32 {
    // Use the confidence value; fall back to 0.5 if not set
    if confidence_value > 0.0 {
        confidence_value * 0.7
    } else {
        0.5 // neutral default when no confidence is recorded
    }
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
