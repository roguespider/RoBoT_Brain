//! ============================================================================
//! HYPOTHESIS CORE
//! ============================================================================
//!
//! Core domain types for the hypothesis system.
//!
//! These modules define what a hypothesis is, how evidence is represented,
//! how confidence is evaluated, and how hypotheses move through their
//! lifecycle.

pub mod hypothesis;
pub mod evidence;
pub mod evaluator;
pub mod lifecycle;

pub use hypothesis::*;
pub use evidence::*;
pub use evaluator::*;
pub use lifecycle::*;
