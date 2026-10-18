//! Loop Runner — Main loop controller (Architecture Chapter 23).
//!
//! This module is split into submodules for maintainability:
//! - `loop_runner`: Core LoopRunner struct and lifecycle methods
//! - `objectives`: Objective collection, evaluation, selection, and generation
//! - `planning`: Plan generation
//! - `execution`: Objective execution
//! - `verification`: Outcome verification
//! - `experience`: Experience recording and knowledge updates
//! - `reflection`: Post-cycle reflection
//! - `state_evaluation`: Current state evaluation

pub mod config;
pub mod enums;
pub mod execution;
pub mod experience;
pub mod loop_runner;
pub mod objectives;
pub mod planning;
pub mod reflection;
pub mod state_evaluation;
pub mod verification;

/// Re-export public types and APIs from loop_runner module
pub use enums::*;
pub use loop_runner::*;
