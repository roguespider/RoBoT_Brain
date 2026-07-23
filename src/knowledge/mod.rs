// src/knowledge/mod.rs
//! Knowledge System - Manages information that has gained sufficient confidence
//! to influence reasoning.
//!
//! Per architecture #2.3:
//! - Maintain trusted information
//! - Track confidence
//! - Store relationships
//! - Manage knowledge evolution
//! - Connect concepts together
//!
//! Knowledge is not static - it changes as new evidence appears.

#![allow(dead_code, unused_imports)]

pub mod types;
pub mod store;
pub mod query;

pub use types::KnowledgeItem;
pub use store::KnowledgeStore;
pub use query::{KnowledgeQuery, KnowledgeResult, apply_query, rank_items};
