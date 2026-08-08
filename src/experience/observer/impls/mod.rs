// src/experience/observer/impls/mod.rs
//! Observer implementations for the learning subsystems
//!
//! Per Architecture §22 - Background Workers:
//! Each learning subsystem has a dedicated worker that processes events
//! relevant to its domain.
//!
//! Event Flow:
//! ExperienceRecorded → Reflection → Hypothesis → Knowledge → Reputation
//!                 ↓           ↓           ↓           ↓
//!            Exploration  Evolution   Memory      Sources

pub mod hypothesis;
pub mod metrics;
pub mod reputation;

pub use hypothesis::HypothesisObserver;
pub use metrics::MetricsObserver;
pub use reputation::ReputationObserver;
