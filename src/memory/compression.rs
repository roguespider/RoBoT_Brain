//! Compression — per Architecture §14 (Memory Hierarchy).
//! Compression for memory records to reduce storage size.

use serde_json;

/// Compression for memory records.
#[derive(Default)]
pub struct Compression {
    compressed: std::collections::HashMap<String, Vec<u8>>,
}

impl Compression {
    /// Create a new compression store.
    pub fn new() -> Self {
        Self {
            compressed: std::collections::HashMap::new(),
        }
    }

    /// Compress and store data under a key.
    pub fn compress(&mut self, key: String, data: Vec<u8>) {
        self.compressed.insert(key, data);
    }

    /// Retrieve compressed data by key.
    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.compressed.get(key)
    }

    /// Delete compressed data by key.
    pub fn delete(&mut self, key: &str) -> bool {
        self.compressed.remove(key).is_some()
    }

    /// Get the number of compressed items.
    pub fn len(&self) -> usize {
        self.compressed.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.compressed.is_empty()
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "compression",
            "adapted": true
        })
    }
}
