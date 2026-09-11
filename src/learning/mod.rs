// src/learning/mod.rs

//! Learning module for experience-based learning
//!
//! Per Architecture §9 - Learning Pipeline:
//! Input → Observation → Memory → Experience → Knowledge → Planning → Decision → Action → Reflection

pub mod candidates;
pub mod hypothesis;
pub mod lineage;
pub mod memory_state;
pub mod pipeline;
pub mod promotion;
pub mod working_memory;
