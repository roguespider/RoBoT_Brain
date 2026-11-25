//! Index cards — per Architecture §14 (Memory Hierarchy).
//! Index cards for quick reference to memory records.

use serde_json;
use std::collections::HashMap;

/// Index cards for quick reference to memory records.
#[derive(Default)]
pub struct IndexCards {
    cards: HashMap<String, serde_json::Value>,
}

impl IndexCards {
    /// Create a new index cards store.
    pub fn new() -> Self {
        Self {
            cards: HashMap::new(),
        }
    }

    /// Store an index card.
    pub fn store(&mut self, key: String, value: serde_json::Value) {
        self.cards.insert(key, value);
    }

    /// Retrieve an index card by key.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.cards.get(key)
    }

    /// Delete an index card by key.
    pub fn delete(&mut self, key: &str) -> bool {
        self.cards.remove(key).is_some()
    }

    /// Get the number of index cards.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}
