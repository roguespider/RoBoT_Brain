// src/memory/embedding.rs

//! Selective embedding generation - Per Documentation "Selective Embedding" principle.
//!
//! "Score content on ingestion. Only embed high-value architectural decisions,
//! not logs/temp data/repeated discussions."
//!
//! Only memories with confidence >= EMBED_THRESHOLD and importance >= EMBED_THRESHOLD
//! receive embeddings. The rest use the hybrid retrieval system (graph + symbolic
//! search) which the architecture intentionally relies on more than embeddings alone.

/// Minimum confidence and importance for a memory to receive an embedding.
/// Set to 0.3 so default store_memory (0.5/0.5) qualifies.
/// Memories below this on both dimensions are skipped (truly low-value).
const EMBED_THRESHOLD: f32 = 0.3;

/// Dimension for the seeded embedding vectors.
const EMBED_DIM: usize = 128;

/// Generate a deterministic embedding vector from content.
///
/// Uses FNV-1a hashing of content chunks to produce a fixed-dimensional vector.
/// Same content always produces the same vector (deterministic).
///
/// Per docs: embeddings are one signal in hybrid retrieval, not the sole signal.
/// Per docs: provenance is preserved via the memory item tags (prov: url, provider, timestamp, query).
/// Embeddings are generated from content only; provenance stays in the memory layer.
pub fn generate_embedding(content: &str, confidence: f32, importance: f32) -> Option<Vec<f32>> {
    // Selective embedding: only high-value content (per docs)
    if confidence < EMBED_THRESHOLD || importance < EMBED_THRESHOLD {
        return None;
    }

    let mut embedding = Vec::with_capacity(EMBED_DIM);
    let content_bytes = content.as_bytes();
    let chunk_size = content_bytes.len().div_ceil(EMBED_DIM);

    for i in 0..EMBED_DIM {
        let chunk_start = i * chunk_size;
        if chunk_start >= content_bytes.len() {
            // Past the end of content: use a zero vector slot.
            embedding.push(0.0);
            continue;
        }
        let chunk_end = (chunk_start + chunk_size).min(content_bytes.len());
        let chunk = &content_bytes[chunk_start..chunk_end];

        // FNV-1a hash of this chunk
        let hash = fnv1a_64(chunk);

        // Normalize hash to [0, 1]
        let normalized = (hash as f32) / (u64::MAX as f32);

        // Scale by average of confidence and importance
        let weighted = normalized * ((confidence + importance) / 2.0);
        embedding.push(weighted);
    }

    Some(embedding)
}

/// FNV-1a 64-bit hash.
/// Simple, fast, deterministic hash for embedding generation.
fn fnv1a_64(data: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

// Tests for generate_embedding live in .agents/scripts/test_suite2/
