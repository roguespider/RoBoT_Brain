// robot/src/experience/hypothesis/support/mod.rs

//! ============================================================================
//! HYPOTHESIS SUPPORT
//! ============================================================================
//!
//! Extended capabilities for the hypothesis subsystem.
//!
//! These modules provide advanced functionality that builds on top of the
//! core hypothesis model.


pub mod graph;
#[cfg(test)]
pub mod planner;
pub mod simulation;
#[cfg(test)]
pub mod statistics;

