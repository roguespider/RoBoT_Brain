

// src/tools/exploration_tools.rs
//! Exploration MCP tools - wiring up exploration types from experience::exploration


pub mod definitions;
mod handlers;

pub use handlers::*;

use serde::{Deserialize, Serialize};

/// Start a new exploration
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StartExplorationInput {
    pub title: String,
    pub purpose: String,
}

/// Get exploration status
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetExplorationStatusInput {
    pub exploration_id: String,
}

/// Complete an exploration with findings
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CompleteExplorationInput {
    pub exploration_id: String,
    pub findings: Vec<FindingInput>,
}

/// Input for a finding
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FindingInput {
    pub description: String,
    pub confidence: f32,
}

/// Record an attempt during exploration
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecordAttemptInput {
    pub exploration_id: String,
    pub action: String,
    pub expected_result: Option<String>,
    pub actual_result: Option<String>,
}

/// Add a hypothesis to an exploration
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddHypothesisInput {
    pub exploration_id: String,
    pub statement: String,
    pub initial_confidence: Option<f32>,
}

/// Evaluate a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EvaluateHypothesisInput {
    pub exploration_id: String,
    pub hypothesis_id: String,
    pub result: String,
}

/// Promote a finding to knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PromoteFindingInput {
    pub exploration_id: String,
    pub finding_id: String,
}
