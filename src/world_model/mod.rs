// src/world_model/mod.rs
//! World Model (Architecture §14).
//!
//! "Memory stores facts. World Model stores understanding."
//!
//! The World Model is a typed entity-relationship graph that represents *how
//! the world works*: objects, places, people, events, time, goals, resources,
//! and the relationships between them. Where memory items are raw facts, a
//! world-model entity carries properties, a confidence, salience, and typed
//! links to other entities — enabling graph-based reasoning such as "what
//! blocks this goal?" or "what does this action depend on?".
//!
//! This module is one of the pieces called out in the v2.0 roadmap
//! (TASK-V2-06) as "one of the biggest missing pieces." It is wired into the
//! App at startup and exercised by world-model MCP tools.

pub mod store;
pub mod types;

// Re-export only what is consumed via the `crate::world_model::` path. Other
// types are accessed via their module path to avoid dead re-export warnings.
pub use store::WorldModel;
