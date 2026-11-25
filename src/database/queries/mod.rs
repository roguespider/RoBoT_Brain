// src/database/queries/mod.rs
//! Database query functions for SQLite operations

pub mod ai_model;
pub mod architecture_trace;
pub mod confidence_history;
pub mod conversation;
pub mod diagnostics;
pub mod embeddings;
pub mod execution;
pub mod experiences;
pub mod helpers;
pub mod knowledge_graph;
pub mod learning;
pub mod lesson;
pub mod memory;
pub mod observations;
pub mod planning;
pub mod relationships;
pub mod scheduled_tasks;
pub mod skill;
pub mod tool;
pub mod workflow;

pub use embeddings::{
    count_embeddings, delete_embedding_by_memory_id, get_embedding_by_memory_id, insert_embedding,
    list_embeddings,
};
pub use execution::insert_execution;
pub use experiences::{insert_reputation, list_experiences, list_reputations};
pub use memory::{
    delete_memory_by_id, get_memory, insert_memory, list_memories, list_memories_by_layer,
    search_memory,
};
pub use observations::{get_observation, insert_observation, list_observations};
pub use planning::insert_planning;
pub use relationships::insert_memory_relationship;
pub use scheduled_tasks::{
    delete_scheduled_task, get_scheduled_task, insert_scheduled_task, list_scheduled_tasks,
};
