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
