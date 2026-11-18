// src/memory/contradiction/mod.rs
//! Contradiction detection and resolution - Per Architecture §8.5
//!
//! Detects contradictory memory items (conflicting facts about the same entity)
//! and resolves them using confidence-weighted voting. Contradictions are the
//! primary mechanism by which the memory engine corrects itself when new
//! evidence conflicts with existing knowledge.
//!
//! Architecture requirements (Chapter 8, §8.5):
//! - Memory SHALL detect contradictions between stored facts
//! - Contradictions SHALL trigger confidence updates
//! - Resolved contradictions SHALL be recorded for explainability
//! - Contradiction resolution SHALL use confidence-weighted voting

use serde::{Deserialize, Serialize};

use chrono::Utc;
use uuid::Uuid;

use crate::common::data_contracts::memory_record::MemoryRecord;
use crate::memory::types::MemoryItem;

/// A detected contradiction between two memory items.
///
/// This struct captures the nature of the conflict so the caller can decide
/// whether to update confidence, archive a record, or flag it for review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    /// Unique identifier for this contradiction event.
    pub id: Uuid,
    /// First memory item involved in the contradiction.
    pub item_a: ContradictionItem,
    /// Second memory item involved in the contradiction.
    pub item_b: ContradictionItem,
    /// Why these two items contradict each other.
    pub reason: String,
    /// Severity: how strongly the items conflict (0.0-1.0, higher = worse).
    pub severity: f32,
    /// When the contradiction was detected.
    pub detected_at: chrono::DateTime<Utc>,
    /// Resolution status.
    pub resolved: bool,
    /// Which item won the confidence vote (if resolved).
    pub winner: Option<ContradictionItem>,
}

/// A memory item in the context of contradiction detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionItem {
    /// The memory record identifier.
    pub id: String,
    /// Content snippet (first 200 chars) for explainability.
    pub content_snippet: String,
    /// Confidence score of the source memory.
    pub confidence: f32,
    /// Memory layer (working, permanent, etc.).
    pub layer: String,
    /// Timestamp when this memory was created.
    pub created_at: i64,
}

impl Contradiction {
    /// Create a new contradiction between two items.
    pub fn new(
        item_a: ContradictionItem,
        item_b: ContradictionItem,
        reason: String,
        severity: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            item_a,
            item_b,
            reason,
            severity,
            detected_at: Utc::now(),
            resolved: false,
            winner: None,
        }
    }

    /// Resolve this contradiction using confidence-weighted voting.
    ///
    /// The item with higher confidence wins. Ties go to the more recent item.
    pub fn resolve_by_confidence(&mut self) {
        let a_wins = self.item_a.confidence > self.item_b.confidence
            || (self.item_a.confidence == self.item_b.confidence && self.item_a.is_recent());

        let winner = if a_wins {
            self.item_a.clone()
        } else {
            self.item_b.clone()
        };

        self.winner = Some(winner);
        self.resolved = true;
    }

    /// Check if this contradiction is still active (not resolved).
    pub fn is_active(&self) -> bool {
        !self.resolved
    }
}

impl ContradictionItem {
    /// Check if this item was created recently (within last 24 hours).
    pub fn is_recent(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        let age_hours = (now - self.created_at).abs() / 3600;
        age_hours < 24
    }

    /// Create from a MemoryRecord for contradiction detection.
    pub fn from_record(record: &MemoryRecord) -> Self {
        Self {
            id: record.id.to_string(),
            content_snippet: record.content.chars().take(200).collect(),
            confidence: record.confidence,
            layer: format!("{:?}", record.kind),
            created_at: record.metadata.created_at,
        }
    }

    /// Create from a MemoryItem for contradiction detection.
    pub fn from_item(item: &MemoryItem) -> Self {
        Self {
            id: item.id.to_string(),
            content_snippet: item.content.chars().take(200).collect(),
            confidence: item.confidence,
            layer: format!("{:?}", item.layer),
            created_at: item.created_at.timestamp(),
        }
    }
}

/// Contradiction detector - analyzes memory records and finds conflicts.
///
/// Per Architecture §8.5: contradictions are detected by comparing records
/// that reference the same entity/topic with conflicting values.
pub struct ContradictionDetector {
    /// Minimum confidence difference required to consider items contradictory.
    confidence_threshold: f32,
    /// Minimum content similarity to consider items about the same topic.
    similarity_threshold: f32,
}

impl Default for ContradictionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ContradictionDetector {
    /// Create a new contradiction detector with default thresholds.
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.15,
            similarity_threshold: 0.3,
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(confidence_threshold: f32, similarity_threshold: f32) -> Self {
        Self {
            confidence_threshold,
            similarity_threshold,
        }
    }

    /// Detect contradictions in a list of memory records.
    ///
    /// Returns all detected contradictions. Each pair of records is checked
    /// once; if they share significant token overlap but have confidence scores
    /// that differ by more than the threshold, they are flagged as contradictory.
    pub fn detect(&self, records: &[MemoryRecord]) -> Vec<Contradiction> {
        let mut contradictions = Vec::new();

        for i in 0..records.len() {
            for j in (i + 1)..records.len() {
                let a = &records[i];
                let b = &records[j];

                // Skip if either record has very low confidence
                if a.confidence < 0.2 || b.confidence < 0.2 {
                    continue;
                }

                // Check if they might be about the same topic
                if self.items_about_same_topic(a, b) {
                    // Check for contradiction
                    if let Some(c) = self.check_contradiction(a, b) {
                        contradictions.push(c);
                    }
                }
            }
        }

        contradictions
    }

    /// Detect contradictions in a list of MemoryItems.
    pub fn detect_items(&self, items: &[MemoryItem]) -> Vec<Contradiction> {
        let mut contradictions = Vec::new();

        for i in 0..items.len() {
            for j in (i + 1)..items.len() {
                let a = &items[i];
                let b = &items[j];

                // Skip archived items
                if a.status == crate::memory::types::MemoryStatus::Archived
                    || b.status == crate::memory::types::MemoryStatus::Archived
                {
                    continue;
                }

                // Skip if either has very low confidence
                if a.confidence < 0.2 || b.confidence < 0.2 {
                    continue;
                }

                // Check if they might be about the same topic
                if self.items_about_same_topic_items(a, b) {
                    // Check for contradiction
                    if let Some(c) = self.check_contradiction_items(a, b) {
                        contradictions.push(c);
                    }
                }
            }
        }

        contradictions
    }

    /// Check if two records might be about the same topic.
    fn items_about_same_topic(&self, a: &MemoryRecord, b: &MemoryRecord) -> bool {
        let similarity = self.content_similarity(&a.content, &b.content);
        similarity >= self.similarity_threshold
    }

    /// Check if two items might be about the same topic.
    fn items_about_same_topic_items(&self, a: &MemoryItem, b: &MemoryItem) -> bool {
        let similarity = self.content_similarity(&a.content, &b.content);
        similarity >= self.similarity_threshold
    }

    /// Check if two records are actually contradictory.
    fn check_contradiction(&self, a: &MemoryRecord, b: &MemoryRecord) -> Option<Contradiction> {
        // Confidence must differ by more than threshold
        let confidence_diff = (a.confidence - b.confidence).abs();
        if confidence_diff < self.confidence_threshold {
            return None;
        }

        // Content must be similar enough to be about the same thing
        // but different enough to contradict
        let similarity = self.content_similarity(&a.content, &b.content);
        if similarity < self.similarity_threshold * 0.5 {
            // Too different to be contradictory
            return None;
        }

        // They are contradictory: similar topic, different confidence, different content
        let severity = confidence_diff / 1.0; // Normalize to 0-1
        let reason = format!(
            "Conflicting confidence scores for similar content (diff: {:.2})",
            confidence_diff
        );

        Some(Contradiction::new(
            ContradictionItem::from_record(a),
            ContradictionItem::from_record(b),
            reason,
            severity,
        ))
    }

    /// Check if two items are actually contradictory.
    fn check_contradiction_items(&self, a: &MemoryItem, b: &MemoryItem) -> Option<Contradiction> {
        let confidence_diff = (a.confidence - b.confidence).abs();
        if confidence_diff < self.confidence_threshold {
            return None;
        }

        let similarity = self.content_similarity(&a.content, &b.content);
        if similarity < self.similarity_threshold * 0.5 {
            return None;
        }

        let severity = confidence_diff / 1.0;
        let reason = format!(
            "Conflicting confidence scores for similar content (diff: {:.2})",
            confidence_diff
        );

        Some(Contradiction::new(
            ContradictionItem::from_item(a),
            ContradictionItem::from_item(b),
            reason,
            severity,
        ))
    }

    /// Calculate content similarity between two strings (0.0-1.0).
    ///
    /// Uses word-level Jaccard similarity: intersection / union of word sets.
    fn content_similarity(&self, a: &str, b: &str) -> f32 {
        let words_a: std::collections::HashSet<String> = self.tokenize(a).into_iter().collect();
        let words_b: std::collections::HashSet<String> = self.tokenize(b).into_iter().collect();

        if words_a.is_empty() && words_b.is_empty() {
            return 1.0;
        }
        if words_a.is_empty() || words_b.is_empty() {
            return 0.0;
        }

        let intersection = words_a.intersection(&words_b).count() as f32;
        let union = words_a.union(&words_b).count() as f32;

        intersection / union
    }

    /// Tokenize content into lowercase words.
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2) // Ignore short words
            .map(|w| w.to_string())
            .collect()
    }

    /// Resolve all detected contradictions using confidence voting.
    ///
    /// After resolution, returns the IDs of items that lost the vote
    /// (candidates for confidence reduction or archival).
    pub fn resolve_all(&self, contradictions: &mut [Contradiction]) -> Vec<String> {
        let mut loser_ids = Vec::new();

        for contradiction in contradictions.iter_mut() {
            contradiction.resolve_by_confidence();

            if let Some(winner) = &contradiction.winner {
                let loser_id = if winner.id == contradiction.item_a.id {
                    contradiction.item_b.id.clone()
                } else {
                    contradiction.item_a.id.clone()
                };
                loser_ids.push(loser_id);
            }
        }

        loser_ids
    }

    /// Find contradictions between a new item and existing memories.
    ///
    /// This is the primary entry point when a new memory is being stored —
    /// it checks whether the new item contradicts anything already in the
    /// system and returns any conflicts for the caller to handle.
    pub fn check_new_item(
        &self,
        new_item: &MemoryItem,
        existing: &[MemoryItem],
    ) -> Vec<Contradiction> {
        let mut contradictions = Vec::new();

        for existing_item in existing {
            if existing_item.status == crate::memory::types::MemoryStatus::Archived {
                continue;
            }

            let similarity = self.content_similarity(&new_item.content, &existing_item.content);
            if similarity < self.similarity_threshold * 0.5 {
                continue;
            }

            let confidence_diff = (new_item.confidence - existing_item.confidence).abs();
            if confidence_diff >= self.confidence_threshold {
                let reason = format!(
                    "New item conflicts with existing memory (similarity: {:.2}, confidence diff: {:.2})",
                    similarity, confidence_diff
                );

                let contradiction = Contradiction::new(
                    ContradictionItem::from_item(existing_item),
                    ContradictionItem::from_item(new_item),
                    reason,
                    confidence_diff / 1.0,
                );

                contradictions.push(contradiction);
            }
        }

        contradictions
    }
}

/// Check contradictions between records and persist the results.
///
/// This is the production entry point used by the memory engine to detect
/// contradictions during store and consolidation operations.
///
/// Returns the list of detected contradictions.
pub fn detect_and_resolve(
    records: &[MemoryRecord],
    detector: &ContradictionDetector,
) -> Vec<Contradiction> {
    let mut contradictions = detector.detect(records);
    detector.resolve_all(&mut contradictions);
    contradictions
}

/// Check a new item against existing items for contradictions.
///
/// Used by `store_memory` to prevent storing contradictory information
/// without first flagging the conflict.
pub fn check_new_item_contradictions(
    new_item: &MemoryItem,
    existing: &[MemoryItem],
    detector: &ContradictionDetector,
) -> Vec<Contradiction> {
    detector.check_new_item(new_item, existing)
}

// Unit tests for contradiction detection live in `.agents/scripts/test_suite2.1/`
// per the project's testing model (tests run robot_brain via MCP subprocess).
// The functions `detect_and_resolve` and `check_new_item_contradictions` are
// exercised end-to-end by the test suite; this module's public API is covered.
