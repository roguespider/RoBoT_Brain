//! Vector store — per Architecture §14 (Memory Hierarchy).
//! Provides vector similarity search storage for memory records.

use serde_json;

/// Vector store for semantic similarity search.
#[derive(Default)]
pub struct VectorStore {
    items: Vec<(String, serde_json::Value, Vec<f32>)>,
}

impl VectorStore {
    /// Create a new vector store.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Store a vector with its associated data.
    pub fn store(&mut self, key: String, value: serde_json::Value, vector: Vec<f32>) {
        self.items.push((key, value, vector));
    }

    /// Retrieve a vector by key.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.items
            .iter()
            .find(|(k, _, _)| k == key)
            .map(|(_, v, _)| v)
    }

    /// Search for the most similar vectors to the query vector.
    pub fn search(&self, query: &[f32], limit: usize) -> Vec<(String, serde_json::Value, f32)> {
        let mut scored: Vec<(String, serde_json::Value, f32)> = self
            .items
            .iter()
            .map(|(k, v, v_vec)| {
                let sim = cosine_similarity(query, v_vec);
                (k.clone(), v.clone(), sim)
            })
            .collect();
        scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        scored
    }

    /// Get the number of stored vectors.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}
