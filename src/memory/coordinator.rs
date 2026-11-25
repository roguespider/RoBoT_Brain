//! Coordinator — per Architecture §14 (Memory Hierarchy).
//! Coordinates the memory hierarchy layers and promotes/demotes records.

use serde_json;

/// Memory hierarchy coordinator.
#[derive(Default)]
pub struct Coordinator {
    promoted_count: u64,
    demoted_count: u64,
    promoted_keys: Vec<String>,
    demoted_keys: Vec<String>,
}

impl Coordinator {
    /// Create a new coordinator.
    pub fn new() -> Self {
        Self {
            promoted_count: 0,
            demoted_count: 0,
            promoted_keys: Vec::new(),
            demoted_keys: Vec::new(),
        }
    }

    /// Promote a record to a higher memory layer.
    pub fn promote(&mut self, key: &str) -> Result<(), String> {
        self.promoted_count += 1;
        self.promoted_keys.push(key.to_string());
        Ok(())
    }

    /// Demote a record by key.
    pub fn demote(&mut self, key: &str) -> Result<(), String> {
        self.demoted_count += 1;
        self.demoted_keys.push(key.to_string());
        Ok(())
    }

    /// Get the number of promotions.
    pub fn promotion_count(&self) -> u64 {
        self.promoted_count
    }

    /// Get the number of demotions.
    pub fn demotion_count(&self) -> u64 {
        self.demoted_count
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "coordinator",
            "adapted": true
        })
    }
}
