// src/experience/integration/mod.rs
//! Integration layer that wires all experience subsystems together
//!
//! Per Architecture §4.04 - Event-Driven Architecture:
//! ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts
//!
//! This module creates the event subscriptions and coordinates the learning pipeline.

pub mod event_subscriber;
pub mod learning_coordinator;
pub mod reflection_pipeline;
pub mod hypothesis_pipeline;

pub use event_subscriber::EventSubscriber;
pub use learning_coordinator::LearningCoordinator;
