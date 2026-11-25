//! Sensory buffer — per Architecture §14 (Memory Hierarchy).
//! Temporary buffer for sensory input before processing.

use serde_json;

/// Sensory buffer for temporary sensory input.
#[derive(Default)]
pub struct SensoryBuffer {
    buffer: Vec<(String, serde_json::Value)>,
}

impl SensoryBuffer {
    /// Create a new sensory buffer.
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Add sensory input to the buffer.
    pub fn add(&mut self, key: String, data: serde_json::Value) {
        self.buffer.push((key, data));
    }

    /// Retrieve sensory input by key.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.buffer.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get the number of buffered items.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
