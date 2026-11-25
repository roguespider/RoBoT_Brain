//! Confidence System — Confidence scoring, propagation, thresholds, audit (Chapter 19).
//!
//! Per Architecture §19.1-19.7:
//! - Confidence scoring for facts, relationships, experiences, skills, workflows, tools, strategies
//! - Confidence propagation through the cognitive pipeline
//! - Confidence thresholds for decision making
//! - Confidence audit trail
//! - Wiring: confidence/ -> memory/ (retention) -> knowledge/ (promotion) ->
//!   retrieval/ (ranking) -> execution/ (budget/approval) -> database/ (confidence table)

pub mod action;
pub mod system;
pub mod types;

pub use action::*;
pub use system::ConfidenceSystem;
pub use types::*;
