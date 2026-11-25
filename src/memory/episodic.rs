//! Episodic memory — per Architecture §14 (Memory Hierarchy).
//! Stores episodic memories (events and experiences).

use serde_json;

/// Episodic memory store.
#[derive(Default)]
pub struct EpisodicMemory {
    episodes: Vec<(String, serde_json::Value)>,
}

impl EpisodicMemory {
    /// Create a new episodic memory store.
    pub fn new() -> Self {
        Self {
            episodes: Vec::new(),
        }
    }

    /// Store an episodic memory.
    pub fn store(&mut self, key: String, data: serde_json::Value) {
        self.episodes.push((key, data));
    }

    /// Retrieve an episodic memory by key.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.episodes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Get the number of stored episodes.
    pub fn len(&self) -> usize {
        self.episodes.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.episodes.is_empty()
    }
}
