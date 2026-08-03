// src/experience/observer/mod.rs
//! Observer system for the Experience module.
//!
//! Defines the [`ExperienceObserver`] trait and provides concrete
//! implementations for the learning subsystems.

mod experience;
mod impls;

pub use self::experience::ExperienceObserver;
pub use self::impls::{HypothesisObserver, MetricsObserver, ReputationObserver};
