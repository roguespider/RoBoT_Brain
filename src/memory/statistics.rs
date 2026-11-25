//! Statistics — per Architecture §14 (Memory Hierarchy).
//! Statistics for memory records.

use serde_json;

/// Statistics for memory records.
#[derive(Default)]
pub struct Statistics {
    total_records: u64,
    total_accesses: u64,
}

impl Statistics {
    /// Create a new statistics tracker.
    pub fn new() -> Self {
        Self {
            total_records: 0,
            total_accesses: 0,
        }
    }

    /// Record a record access.
    pub fn record_access(&mut self) {
        self.total_accesses += 1;
    }

    /// Set the total number of records.
    pub fn set_record_count(&mut self, count: u64) {
        self.total_records = count;
    }

    /// Get the total number of records.
    pub fn record_count(&self) -> u64 {
        self.total_records
    }

    /// Get the total number of accesses.
    pub fn access_count(&self) -> u64 {
        self.total_accesses
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "statistics",
            "adapted": true
        })
    }
}
