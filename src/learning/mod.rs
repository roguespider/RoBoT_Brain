// src/learning/mod.rs

#![allow(dead_code)]
//! Learning module for experience-based learning
//!
//! Per Architecture §9 - Learning Pipeline:
//! Input → Observation → Memory → Experience → Knowledge → Planning → Decision → Action → Reflection

pub mod working_memory;
pub mod hypothesis;
pub mod candidates;
pub mod lineage;
pub mod pipeline;

pub use working_memory::WorkingMemory;
pub use lineage::LineageTracker;
