// src/database/queries/tests.rs
//! Unit tests for database query functions

use uuid::Uuid;

use crate::database::models::MemoryEmbedding;

use super::helpers::{bytes_to_embedding, embedding_to_bytes};

#[test]
fn test_embedding_bytes_conversion() {
    let original = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let bytes = embedding_to_bytes(&original);
    let recovered = bytes_to_embedding(&bytes);
    assert_eq!(original.len(), recovered.len());
    for (o, r) in original.iter().zip(recovered.iter()) {
        assert!((o - r).abs() < f32::EPSILON);
    }
}

#[test]
fn test_memory_embedding_cosine_similarity() {
    let e1 = MemoryEmbedding::new(
        Uuid::new_v4(),
        vec![1.0, 0.0, 0.0],
        "test".to_string(),
    );
    let e2 = MemoryEmbedding::new(
        Uuid::new_v4(),
        vec![1.0, 0.0, 0.0],
        "test".to_string(),
    );
    let e3 = MemoryEmbedding::new(
        Uuid::new_v4(),
        vec![0.0, 1.0, 0.0],
        "test".to_string(),
    );

    assert!((e1.cosine_similarity(&e2) - 1.0).abs() < f32::EPSILON);
    assert!((e1.cosine_similarity(&e3) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_embedding_dimension() {
    let e = MemoryEmbedding::new(
        Uuid::new_v4(),
        vec![1.0, 2.0, 3.0, 4.0, 5.0],
        "test".to_string(),
    );
    assert_eq!(e.dimension(), 5);
}
