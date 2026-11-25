//! Retrieval caching per Architecture Chapter 16.
//! Caches retrieval results to avoid redundant searches.

use std::collections::HashMap;
use std::time::Instant;

/// A cached retrieval result with expiry.
#[derive(Debug, Clone)]
struct CachedResult {
    result: String,
    cached_at: Instant,
    ttl_seconds: u64,
}

impl CachedResult {
    /// Check if the cache entry has expired.
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed().as_secs() > self.ttl_seconds
    }
}

/// Simple in-memory cache for retrieval results.
pub struct RetrievalCache {
    entries: HashMap<String, CachedResult>,
}

impl RetrievalCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Store a result in the cache with the given TTL.
    pub fn insert(&mut self, key: String, value: String, ttl_seconds: u64) {
        self.entries.insert(
            key,
            CachedResult {
                result: value,
                cached_at: Instant::now(),
                ttl_seconds,
            },
        );
    }

    /// Retrieve a cached result if it exists and hasn't expired.
    pub fn get(&self, key: &str) -> Option<String> {
        self.entries.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.result.clone())
            }
        })
    }

    /// Remove expired entries from the cache.
    pub fn cleanup_expired(&mut self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }

    /// Get the number of entries in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for RetrievalCache {
    fn default() -> Self {
        Self::new()
    }
}
