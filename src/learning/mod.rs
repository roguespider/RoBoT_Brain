// src/learning/mod.rs

//! Learning module for experience-based learning
//!
//! Per Architecture §9 - Learning Pipeline:
//! Input → Observation → Memory → Experience → Knowledge → Planning → Decision → Action → Reflection

pub mod working_memory;
#[cfg(test)]
pub mod hypothesis;
pub mod candidates;
#[cfg(test)]
pub mod lineage;
#[cfg(test)]
pub mod pipeline;
