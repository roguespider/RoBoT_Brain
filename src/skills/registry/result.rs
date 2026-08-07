// src/skills/registry/result.rs
//! Execution result for skills

use serde::{Deserialize, Serialize};

/// Result of skill execution
/// Per Architecture §15: "Skill::execute(&context)"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub skill_id: String,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub mastery_at_execution: f32,
    pub mastery_delta: f32,
    pub new_mastery: f32,
}

impl ExecutionResult {
    pub fn success(
        skill_id: String,
        output: serde_json::Value,
        duration_ms: u64,
        mastery_before: f32,
        mastery_delta: f32,
    ) -> Self {
        Self {
            skill_id,
            success: true,
            output: Some(output),
            error: None,
            duration_ms,
            mastery_at_execution: mastery_before,
            mastery_delta,
            new_mastery: mastery_before + mastery_delta,
        }
    }

    pub fn failure(
        skill_id: String,
        error: String,
        duration_ms: u64,
        mastery_before: f32,
        mastery_delta: f32,
    ) -> Self {
        Self {
            skill_id,
            success: false,
            output: None,
            error: Some(error),
            duration_ms,
            mastery_at_execution: mastery_before,
            mastery_delta,
            new_mastery: (mastery_before + mastery_delta).max(0.0),
        }
    }
}
