// src/memory/consolidation.rs
//! Memory consolidation - Per Architecture Chapter 08 & 17
//!
//! Handles merging duplicate memories and summarizing low-confidence items.

use std::collections::HashSet;

use crate::memory::types::MemoryItem;

/// Configures how memory entries are merged and filtered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoryConsolidator {
    /// Minimum similarity (0.0..=1.0) required to merge two memories.
    pub similarity_threshold: f32,
    /// Minimum confidence (0.0..=1.0) required to retain a memory.
    pub confidence_threshold: f32,
}

impl MemoryConsolidator {
    /// Creates a new consolidator with validation.
    ///
    /// # Errors
    /// Returns error string if thresholds are outside `[0.0, 1.0]` or NaN.
    pub fn new(similarity_threshold: f32, confidence_threshold: f32) -> Result<Self, String> {
        if similarity_threshold.is_nan() || confidence_threshold.is_nan() {
            return Err("threshold values must not be NaN".into());
        }
        if !(0.0..=1.0).contains(&similarity_threshold) {
            return Err(format!(
                "similarity_threshold out of range [0.0, 1.0]: {}",
                similarity_threshold
            ));
        }
        if !(0.0..=1.0).contains(&confidence_threshold) {
            return Err(format!(
                "confidence_threshold out of range [0.0, 1.0]: {}",
                confidence_threshold
            ));
        }
        Ok(Self {
            similarity_threshold,
            confidence_threshold,
        })
    }

    /// Default consolidator with conservative thresholds.
    pub fn default_config() -> Self {
        Self {
            similarity_threshold: 0.85,
            confidence_threshold: 0.7,
        }
    }

    /// Returns true if both scores meet the consolidation thresholds.
    pub fn should_consolidate(&self, similarity: f32, confidence: f32) -> bool {
        similarity >= self.similarity_threshold && confidence >= self.confidence_threshold
    }

    /// Merge duplicate memories based on Jaccard word-set similarity.
    ///
    /// Memories with word-set Jaccard similarity >= `similarity_threshold`
    /// are merged into a single entry.
    pub fn merge_duplicates(&self, items: &[MemoryItem]) -> Vec<MemoryItem> {
        let threshold = self.similarity_threshold;
        let mut groups: Vec<(MemoryItem, HashSet<String>)> = Vec::new();

        for item in items {
            let words = extract_words(&item.content);
            let mut merged_idx: Option<usize> = None;

            for (idx, (existing_item, existing_words)) in groups.iter_mut().enumerate() {
                let sim = jaccard_similarity(&words, existing_words);
                if sim >= threshold {
                    merged_idx = Some(idx);
                    // Collect data before mutating
                    let new_content = format!("{} {}", existing_item.content, item.content);
                    let new_confidence = existing_item.confidence.max(item.confidence);
                    let new_ids = item.id;
                    // Apply updates
                    existing_item.content = new_content;
                    existing_item.confidence = new_confidence;
                    existing_item.related_ids.push(new_ids);
                    break;
                }
            }

            if merged_idx.is_none() {
                groups.push((item.clone(), words));
            }
        }

        groups.into_iter().map(|(item, _)| item).collect()
    }

    /// Summarize low-confidence memories into a single summary entry.
    ///
    /// Memories with confidence below `confidence_threshold` are replaced
    /// by a single summary entry with average confidence.
    pub fn summarize_low_confidence(&self, items: &[MemoryItem]) -> Vec<MemoryItem> {
        let low: Vec<&MemoryItem> = items
            .iter()
            .filter(|item| item.confidence < self.confidence_threshold)
            .collect();

        if low.is_empty() {
            return items.to_vec();
        }

        // Keep high-confidence items as-is
        let mut result: Vec<MemoryItem> = items
            .iter()
            .filter(|item| item.confidence >= self.confidence_threshold)
            .cloned()
            .collect();

        // Create summary of low-confidence items
        let avg_confidence: f32 =
            low.iter().map(|item| item.confidence).sum::<f32>() / low.len() as f32;

        let contents: Vec<String> = low.iter().map(|item| item.content.clone()).collect();
        let summary_content = format!(
            "Summarized {} low-confidence memories: {}",
            low.len(),
            contents.join(" | ")
        );

        // Create summary memory with unique ID
        let summary_id = uuid::Uuid::new_v4();

        result.push(MemoryItem {
            id: summary_id,
            content: summary_content,
            confidence: avg_confidence,
            ..low[0].clone()
        });

        result
    }
}

/// Extract unique lowercase words from text.
fn extract_words(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Jaccard similarity between two word sets.
fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let inter = a.intersection(b).count() as f32;
    let union = a.union(b).count() as f32;
    if union == 0.0 { 0.0 } else { inter / union }
}
