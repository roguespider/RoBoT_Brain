//! Importance — per Architecture §14 (Memory Hierarchy).
//! Importance scoring for memory records.

use serde_json;

/// Importance scorer for memory records.
#[derive(Default)]
pub struct Importance {
    scores: std::collections::HashMap<String, f32>,
}

impl Importance {
    /// Create a new importance scorer.
    pub fn new() -> Self {
        Self {
            scores: std::collections::HashMap::new(),
        }
    }

    /// Set the importance score for a key.
    pub fn set(&mut self, key: String, score: f32) {
        self.scores.insert(key, score);
    }

    /// Get the importance score for a key.
    pub fn get(&self, key: &str) -> Option<f32> {
        self.scores.get(key).copied()
    }

    /// Get the number of tracked importances.
    pub fn len(&self) -> usize {
        self.scores.len()
    }

    /// Check if the scorer is empty.
    pub fn is_empty(&self) -> bool {
        self.scores.is_empty()
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "importance",
            "adapted": true
        })
    }
}
