//! Archive memory — per Architecture §14 (Memory Hierarchy).
//! Stores archived (cold) memory records.

use serde_json;

/// Archive memory store.
#[derive(Default)]
pub struct ArchiveMemory {
    archives: Vec<(String, serde_json::Value)>,
}

impl ArchiveMemory {
    /// Create a new archive memory store.
    pub fn new() -> Self {
        Self {
            archives: Vec::new(),
        }
    }

    /// Store an archived record.
    pub fn store(&mut self, key: String, data: serde_json::Value) {
        self.archives.push((key, data));
    }

    /// Retrieve an archived record by key.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.archives.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Get the number of archived records.
    pub fn len(&self) -> usize {
        self.archives.len()
    }

    /// Check if the archive is empty.
    pub fn is_empty(&self) -> bool {
        self.archives.is_empty()
    }
}
