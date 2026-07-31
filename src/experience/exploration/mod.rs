//! Exploration module - intentional investigation performed by the system.
//!
//! Per Architecture §2.7, Exploration allows RoBoT to actively seek new information,
//! test hypotheses, and discover opportunities. Exploration prevents the system from
//! becoming passive.
//!
//! ## Module Structure
//! - `exploration.rs` - Main Exploration struct and ExplorationStatus enum
//! - `attempt.rs` - ExplorationAttempt struct for recording attempts
//! - `finding.rs` - ExplorationFinding struct for discoveries
//! - `hypothesis.rs` - Hypothesis struct and HypothesisResult enum
//! - `store.rs` - ExplorationRepository trait for persistence
//!
//! ## Usage
//! ```
//! use robot_brain::experience::exploration::{
//!     Exploration, ExplorationStatus, ExplorationAttempt,
//!     ExplorationFinding, Hypothesis, HypothesisResult,
//!     ExplorationRepository, InMemoryExplorationRepository,
//! };

//! ```

#![allow(clippy::module_inception)]

pub mod attempt;

pub mod exploration;
pub mod finding;
pub mod hypothesis;
pub mod store;

// Re-export all exploration types for convenient access
pub use exploration::{Exploration, ExplorationStatus};
pub use attempt::ExplorationAttempt;
pub use finding::ExplorationFinding;
pub use hypothesis::{Hypothesis, HypothesisResult};
