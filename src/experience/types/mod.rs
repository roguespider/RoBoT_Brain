// /src/experience/types.rs
//! Core types for the Experience Engine (Architecture Chapter 07)
//!
//! Design Invariants (Architecture §07):
//! - Every experience originates from one or more observations.
//! - Experiences are immutable once committed.
//! - Confidence is updated through evidence, never manually.
//! - Reflection creates new experiences rather than modifying old ones.
//! - Promotion to Knowledge requires validation.
//! - Historical data is never destroyed, only archived.

//! NOTE: This module is implemented but not yet fully integrated.

#![allow(dead_code)]

pub mod context;
pub mod encounter;
pub mod evidence;
pub mod experience;
pub mod maturity;
pub mod outcome;
pub mod reputation;
pub mod score;

// Re-export all types for backwards compatibility
pub use context::ExperienceContext;
pub use encounter::{Encounter, EncounterResult};
pub use experience::{Experience, ExperienceType};
pub use outcome::{ExperienceOutcome, OutcomeKind};
pub use score::ExperienceScore;
