// src/vector/mod.rs

//! Vector Index Operations
//!
//! Per Architecture: Provides vector storage, similarity search, and embedding
//! operations for semantic memory and knowledge retrieval.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Embedding vector with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    /// Unique identifier
    pub id: String,
    
    /// Vector data (normalized to unit length)
    pub vector: Vec<f32>,
    
    /// Original text or content
    pub content: String,
    
    /// Embedding model used
    pub model: String,
    
    /// Dimension of the vector
    pub dimension: usize,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Search result with similarity score
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Embedding ID
    pub id: String,
    
    /// Content
    pub content: String,
    
    /// Similarity score (0.0 - 1.0)
    pub score: f32,
    
    /// Distance (lower is more similar)
    pub distance: f32,
}

/// Vector index for semantic search
pub struct VectorIndex {
    /// Embeddings stored by ID
    embeddings: HashMap<String, Embedding>,
    
    /// Dimension size
    dimension: usize,
    
    /// Embedding model name
    model: String,
    
    /// Index statistics
    stats: VectorIndexStats,
}

/// Vector index statistics
#[derive(Debug, Clone, Default)]
pub struct VectorIndexStats {
    pub total_embeddings: usize,
    pub total_dimension: usize,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

impl VectorIndex {
    /// Create a new vector index
    pub fn new(dimension: usize, model: &str) -> Self {
        Self {
            embeddings: HashMap::new(),
            dimension,
            model: model.to_string(),
            stats: VectorIndexStats {
                total_embeddings: 0,
                total_dimension: dimension,
                last_updated: Some(chrono::Utc::now()),
            },
        }
    }
    
    /// Add an embedding to the index
    pub fn add(&mut self, embedding: Embedding) -> Result<(), String> {
        // Validate dimension
        if embedding.vector.len() != self.dimension {
            return Err(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.vector.len()
            ));
        }
        
        let id = embedding.id.clone();
        self.embeddings.insert(id, embedding);
        self.stats.total_embeddings = self.embeddings.len();
        self.stats.last_updated = Some(chrono::Utc::now());
        
        Ok(())
    }
    
    /// Add embedding from text (placeholder - would use actual embedding model)
    pub fn add_text(&mut self, id: String, content: String, metadata: HashMap<String, String>) -> Result<Embedding, String> {
        // Generate a simple hash-based embedding (placeholder for actual embedding)
        let vector = self.generate_text_embedding(&content);
        
        let embedding = Embedding {
            id: id.clone(),
            vector,
            content,
            model: self.model.clone(),
            dimension: self.dimension,
            metadata,
        };
        
        self.add(embedding.clone())?;
        Ok(embedding)
    }
    
    /// Generate embedding for text (simplified - would use actual ML model)
    fn generate_text_embedding(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0.0f32; self.dimension];
        
        // Simple hash-based embedding for demonstration
        // Real implementation would use transformer models
        let words: Vec<&str> = text.split_whitespace().collect();
        for word in &words {
            let hash = self.simple_hash(word);
            let idx = hash % self.dimension;
            vector[idx] += 1.0;
        }
        
        // Normalize
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in &mut vector {
                *v /= magnitude;
            }
        }
        
        vector
    }
    
    /// Simple string hash
    fn simple_hash(&self, s: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish() as usize
    }
    
    /// Search for similar embeddings
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_vector = self.generate_text_embedding(query);
        self.search_by_vector(&query_vector, limit)
    }
    
    /// Search by vector
    pub fn search_by_vector(&self, query_vector: &[f32], limit: usize) -> Vec<SearchResult> {
        if query_vector.len() != self.dimension || self.embeddings.is_empty() {
            return Vec::new();
        }
        
        let mut results: Vec<SearchResult> = self.embeddings
            .values()
            .map(|embedding| {
                let distance = self.cosine_distance(&embedding.vector, query_vector);
                let score = 1.0 - distance;
                
                SearchResult {
                    id: embedding.id.clone(),
                    content: embedding.content.clone(),
                    score,
                    distance,
                }
            })
            .collect();
        
        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        
        results
    }
    
    /// Search by vector ID
    pub fn search_by_id(&self, query_id: &str, limit: usize) -> Vec<SearchResult> {
        if let Some(query_embedding) = self.embeddings.get(query_id) {
            self.search_by_vector(&query_embedding.vector, limit)
        } else {
            Vec::new()
        }
    }
    
    /// Calculate cosine distance between two vectors
    fn cosine_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        1.0 - dot_product.clamp(-1.0, 1.0)
    }
    
    /// Get embedding by ID
    pub fn get(&self, id: &str) -> Option<&Embedding> {
        self.embeddings.get(id)
    }
    
    /// Delete embedding
    pub fn delete(&mut self, id: &str) -> bool {
        if self.embeddings.remove(id).is_some() {
            self.stats.total_embeddings = self.embeddings.len();
            self.stats.last_updated = Some(chrono::Utc::now());
            true
        } else {
            false
        }
    }
    
    /// Get all embeddings
    pub fn get_all(&self) -> Vec<&Embedding> {
        self.embeddings.values().collect()
    }
    
    /// Get index statistics
    pub fn stats(&self) -> &VectorIndexStats {
        &self.stats
    }
    
    /// Clear all embeddings
    pub fn clear(&mut self) {
        self.embeddings.clear();
        self.stats.total_embeddings = 0;
        self.stats.last_updated = Some(chrono::Utc::now());
    }
    
    /// Batch add embeddings
    pub fn add_batch(&mut self, embeddings: Vec<Embedding>) -> Result<usize, String> {
        let mut added = 0;
        for embedding in embeddings {
            if self.add(embedding).is_ok() {
                added += 1;
            }
        }
        Ok(added)
    }
    
    /// Get embeddings filtered by metadata
    pub fn get_by_metadata(&self, key: &str, value: &str) -> Vec<&Embedding> {
        self.embeddings
            .values()
            .filter(|e| e.metadata.get(key) == Some(&value.to_string()))
            .collect()
    }
    
    /// Find nearest neighbors in a cluster
    pub fn cluster_search(&self, query: &str, cluster_id: &str, limit: usize) -> Vec<SearchResult> {
        // First get embeddings from the cluster
        let cluster_embeddings: Vec<_> = self
            .embeddings
            .values()
            .filter(|e| e.metadata.get("cluster") == Some(&cluster_id.to_string()))
            .collect();
        
        if cluster_embeddings.is_empty() {
            return Vec::new();
        }
        
        let query_vector = self.generate_text_embedding(query);
        
        let mut results: Vec<SearchResult> = cluster_embeddings
            .iter()
            .map(|embedding| {
                let distance = self.cosine_distance(&embedding.vector, &query_vector);
                let score = 1.0 - distance;
                
                SearchResult {
                    id: embedding.id.clone(),
                    content: embedding.content.clone(),
                    score,
                    distance,
                }
            })
            .collect();
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        
        results
    }
}

/// Vector operations utilities
pub mod utils {
    /// Compute cosine similarity between two vectors
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        
        dot / (mag_a * mag_b)
    }
    
    /// Compute Euclidean distance
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::MAX;
        }
        
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
    
    /// Normalize a vector to unit length
    pub fn normalize(vector: &mut [f32]) {
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in vector.iter_mut() {
                *v /= magnitude;
            }
        }
    }
    
    /// Compute centroid of multiple vectors
    pub fn compute_centroid(vectors: &[Vec<f32>]) -> Option<Vec<f32>> {
        if vectors.is_empty() {
            return None;
        }
        
        let dim = vectors[0].len();
        let mut centroid = vec![0.0f32; dim];
        
        for vector in vectors {
            if vector.len() != dim {
                return None;
            }
            for (i, v) in vector.iter().enumerate() {
                centroid[i] += v;
            }
        }
        
        let count = vectors.len() as f32;
        for c in &mut centroid {
            *c /= count;
        }
        
        normalize(&mut centroid);
        Some(centroid)
    }
}
