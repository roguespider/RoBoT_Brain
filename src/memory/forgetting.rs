//! Forgetting — per Architecture §14 (Memory Hierarchy).
//! Forgetting policy for memory records (archiving low-value, aged memories).

use serde_json;

/// Forgetting policy for memory records.
#[derive(Default)]
pub struct Forgetting {
    archived_count: u64,
    archived_keys: Vec<String>,
}

impl Forgetting {
    /// Create a new forgetting policy.
    pub fn new() -> Self {
        Self {
            archived_count: 0,
            archived_keys: Vec::new(),
        }
    }

    /// Archive (forget) a record by key.
    pub fn forget(&mut self, key: &str) -> bool {
        self.archived_count += 1;
        self.archived_keys.push(key.to_string());
        true
    }

    /// Get the number of archived (forgotten) records.
    pub fn archived_count(&self) -> u64 {
        self.archived_count
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "forgetting",
            "adapted": true
        })
    }
}
