// src/database/queries/mod.rs
//! Database query functions for SQLite operations

pub mod embeddings;
pub mod experiences;
pub mod helpers;
pub mod memory;
pub mod observations;
pub mod relationships;
pub mod scheduled_tasks;
#[cfg(test)]
pub mod tests;

// Re-export public functions for convenience
pub use embeddings::{
    count_embeddings, delete_embedding_by_memory_id, get_embedding_by_memory_id, insert_embedding,
    list_embeddings,
};
pub use experiences::{insert_reputation, list_experiences, list_reputations};
pub use memory::{
    get_memory, insert_memory, list_memories,
    list_memories_by_layer, search_memory,
};
pub use observations::{
    insert_observation, list_observations,
};
pub use relationships::insert_memory_relationship;
pub use scheduled_tasks::{
    delete_scheduled_task, get_scheduled_task, insert_scheduled_task, list_scheduled_tasks,
};
