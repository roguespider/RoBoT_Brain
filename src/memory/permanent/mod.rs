// src/memory/permanent.rs
//! Permanent Memory - Per Architecture §6.3
//!
//! Permanent Memory contains curated knowledge retained after evaluation.
//!
//! Characteristics:
//! - Indexed
//! - Connected
//! - Confidence weighted
//! - Relationship aware

mod store;
#[cfg(test)]
mod tests;

pub use store::PermanentMemory;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Permanent memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentMemoryStats {
    pub total_items: usize,
    pub by_type: HashMap<String, usize>,
    pub avg_confidence: f32,
    pub avg_importance: f32,
}
