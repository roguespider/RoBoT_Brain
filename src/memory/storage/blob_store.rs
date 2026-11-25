//! Blob store — per Architecture §14 (Memory Hierarchy).
//! Provides blob (binary large object) storage for memory records.

use std::collections::HashMap;

/// Blob store for storing binary large objects.
#[derive(Default)]
pub struct BlobStore {
    blobs: HashMap<String, Vec<u8>>,
}

impl BlobStore {
    /// Create a new blob store.
    pub fn new() -> Self {
        Self {
            blobs: HashMap::new(),
        }
    }

    /// Store a blob with the given key.
    pub fn store(&mut self, key: String, data: Vec<u8>) {
        self.blobs.insert(key, data);
    }

    /// Retrieve a blob by key.
    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.blobs.get(key)
    }

    /// Delete a blob by key.
    pub fn delete(&mut self, key: &str) -> bool {
        self.blobs.remove(key).is_some()
    }

    /// Get the number of stored blobs.
    pub fn len(&self) -> usize {
        self.blobs.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.blobs.is_empty()
    }
}
