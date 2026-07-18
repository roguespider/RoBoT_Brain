// robot\src\experience\hypothesis\services\mod.rs

//! ============================================================================
//! HYPOTHESIS SERVICES
//! ============================================================================
//!
//! Service layer for the hypothesis subsystem.
//!
//! These modules provide operational capabilities around hypotheses:
//!
//! - Repository      -> persistence and retrieval
//! - Analytics      -> trend analysis
//! - Generator      -> creating new hypotheses
//! - Matcher        -> finding relevant hypotheses
//! - Validator      -> detecting conflicts and contradictions
//!
//! Core domain models live in the `core` module.
//! Services operate on those models.

pub mod analytics;
pub mod generator;
pub mod matcher;
pub mod repository;
pub mod validator;

// Re-export commonly used services.

pub use analytics::*;
pub use generator::*;
pub use matcher::*;
pub use repository::*;
pub use validator::*;

