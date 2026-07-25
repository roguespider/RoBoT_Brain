// src/tools/exploration_tools.rs
// Exploration MCP tools - wiring up exploration types from experience::exploration

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::experience::exploration::{
    Exploration, ExplorationAttempt, ExplorationFinding, Hypothesis, HypothesisResult,
};
use crate::experience::types::ExperienceContext;
use crate::tools::ToolOutput;

// ============================================================================
// TOOL INPUT/OUTPUT TYPES
// ============================================================================

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
    pub result: String, // supported, partially_supported, rejected, unknown
}

/// Promote a finding to knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PromoteFindingInput {
    pub exploration_id: String,
    pub finding_id: String,
}

// ============================================================================
// TOOL DEFINITIONS
// ============================================================================

pub mod definitions {
    pub const START_EXPLORATION: &str = "start_exploration";
    pub const PAUSE_EXPLORATION: &str = "pause_exploration";
    pub const RESUME_EXPLORATION: &str = "resume_exploration";
    pub const GET_EXPLORATION_STATUS: &str = "get_exploration_status";
    pub const COMPLETE_EXPLORATION: &str = "complete_exploration";
    pub const RECORD_ATTEMPT: &str = "record_attempt";
    pub const ADD_HYPOTHESIS: &str = "add_hypothesis";
    pub const EVALUATE_HYPOTHESIS: &str = "evaluate_exploration_hypothesis";
    pub const PROMOTE_FINDING: &str = "promote_finding";
    pub const ABANDON_EXPLORATION: &str = "abandon_exploration";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: START_EXPLORATION.to_string(),
                description: "Start a new exploration session. Explorations allow RoBoT to actively investigate topics and test hypotheses.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Human-readable title for this exploration"
                        },
                        "purpose": {
                            "type": "string",
                            "description": "Why this exploration is being conducted"
                        }
                    },
                    "required": ["title", "purpose"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: PAUSE_EXPLORATION.to_string(),
                description: "Pause an active exploration. The exploration can be resumed later.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": {
                            "type": "string",
                            "description": "The exploration ID to pause"
                        }
                    },
                    "required": ["exploration_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: RESUME_EXPLORATION.to_string(),
                description: "Resume a paused exploration.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": {
                            "type": "string",
                            "description": "The exploration ID to resume"
                        }
                    },
                    "required": ["exploration_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_EXPLORATION_STATUS.to_string(),
                description: "Get the current status of an exploration including hypotheses, attempts, and findings.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": {
                            "type": "string",
                            "description": "The exploration ID to check"
                        }
                    },
                    "required": ["exploration_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COMPLETE_EXPLORATION.to_string(),
                description: "Mark an exploration as completed with findings. Findings represent discoveries made during the exploration.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": {
                            "type": "string",
                            "description": "The exploration ID to complete"
                        },
                        "findings": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "description": {
                                        "type": "string",
                                        "description": "Description of what was discovered"
                                    },
                                    "confidence": {
                                        "type": "number",
                                        "description": "Confidence in this finding (0.0-1.0)"
                                    }
                                },
                                "required": ["description"]
                            },
                            "description": "Findings from this exploration"
                        }
                    },
                    "required": ["exploration_id", "findings"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: RECORD_ATTEMPT.to_string(),
                description: "Record an attempt made during exploration. Tracks what was tried and what happened.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": {
                            "type": "string",
                            "description": "The exploration ID"
                        },
                        "action": {
                            "type": "string",
                            "description": "What action was taken"
                        },
                        "expected_result": {
                            "type": "string",
                            "description": "What result was expected (optional)"
                        },
                        "actual_result": {
                            "type": "string",
                            "description": "What actually happened (optional)"
                        }
                    },
                    "required": ["exploration_id", "action"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: ADD_HYPOTHESIS.to_string(),
                description: "Add a testable hypothesis to an exploration. Hypotheses guide what to investigate.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": {
                            "type": "string",
                            "description": "The exploration ID"
                        },
                        "statement": {
                            "type": "string",
                            "description": "The hypothesis statement to test"
                        },
                        "initial_confidence": {
                            "type": "number",
                            "description": "Initial confidence in this hypothesis (0.0-1.0)",
                            "default": 0.5
                        }
                    },
                    "required": ["exploration_id", "statement"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: EVALUATE_HYPOTHESIS.to_string(),
                description: "Set the result for a hypothesis based on evidence gathered during exploration.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": {
                            "type": "string",
                            "description": "The exploration ID"
                        },
                        "hypothesis_id": {
                            "type": "string",
                            "description": "The hypothesis ID to evaluate"
                        },
                        "result": {
                            "type": "string",
                            "description": "Evaluation result: supported, partially_supported, rejected, unknown"
                        }
                    },
                    "required": ["exploration_id", "hypothesis_id", "result"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: PROMOTE_FINDING.to_string(),
                description: "Promote a finding from an exploration to reusable knowledge.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": {
                            "type": "string",
                            "description": "The exploration ID"
                        },
                        "finding_id": {
                            "type": "string",
                            "description": "The finding ID to promote"
                        }
                    },
                    "required": ["exploration_id", "finding_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: ABANDON_EXPLORATION.to_string(),
                description: "Abandon an exploration without completing it. The exploration record is kept for analysis.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": {
                            "type": "string",
                            "description": "The exploration ID to abandon"
                        }
                    },
                    "required": ["exploration_id"]
                }),
            },
        ]
    }
}

// ============================================================================
// IN-MEMORY EXPLORATION STORAGE (simple implementation)
// ============================================================================

use std::collections::HashMap;
use std::sync::RwLock;

lazy_static::lazy_static! {
    static ref EXPLORATION_STORE: RwLock<HashMap<String, Exploration>> = RwLock::new(HashMap::new());
}

// ============================================================================
// TOOL IMPLEMENTATIONS
// ============================================================================

/// Start a new exploration
pub fn execute_start_exploration(input: StartExplorationInput) -> ToolOutput {
    let id = Uuid::new_v4().to_string();
    let context = ExperienceContext::default();
    
    let mut exploration = Exploration::new(
        id.clone(),
        input.title,
        input.purpose,
        context,
    );
    
    // Start the exploration immediately
    exploration.start();
    
    let mut store = EXPLORATION_STORE.write().unwrap();
    store.insert(id.clone(), exploration);
    
    ToolOutput::success(serde_json::json!({
        "exploration_id": id,
        "status": "active",
        "message": "Exploration started. Use get_exploration_status to monitor progress."
    }))
}

/// Pause an active exploration
pub fn execute_pause_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            if exp.is_active() {
                exp.pause();
                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "status": "paused",
                    "message": "Exploration paused."
                }))
            } else {
                ToolOutput::error(format!("Exploration is not active (current status: {:?})", exp.status))
            }
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

/// Resume a paused exploration
pub fn execute_resume_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            use crate::experience::exploration::ExplorationStatus;
            if exp.status == ExplorationStatus::Paused {
                exp.start();
                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "status": "active",
                    "message": "Exploration resumed."
                }))
            } else {
                ToolOutput::error(format!("Exploration is not paused (current status: {:?})", exp.status))
            }
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

/// Promote a finding to reusable knowledge
pub fn execute_promote_finding(input: PromoteFindingInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            // Find the finding by ID
            let finding = exp.findings.iter_mut().find(|f| f.id == input.finding_id);
            
            match finding {
                Some(f) => {
                    f.promote();
                    ToolOutput::success(serde_json::json!({
                        "finding_id": f.id,
                        "promoted": true,
                        "message": format!("Finding '{}' has been promoted to reusable knowledge.", f.description)
                    }))
                }
                None => ToolOutput::error(format!("Finding not found: {}", input.finding_id)),
            }
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

/// Get exploration status
pub fn execute_get_exploration_status(input: GetExplorationStatusInput) -> ToolOutput {
    let store = EXPLORATION_STORE.read().unwrap();
    
    match store.get(&input.exploration_id) {
        Some(exp) => {
            let hypotheses_json: Vec<_> = exp.hypotheses.iter().map(|h| serde_json::json!({
                "id": h.id,
                "statement": h.statement,
                "confidence": h.confidence,
                "result": h.result.as_ref().map(|r| format!("{:?}", r))
            })).collect();
            
            let attempts_json: Vec<_> = exp.attempts.iter().map(|a| serde_json::json!({
                "id": a.id,
                "action": a.action,
                "expected_result": a.expected_result,
                "actual_result": a.actual_result,
                "success": a.success
            })).collect();
            
            let findings_json: Vec<_> = exp.findings.iter().map(|f| serde_json::json!({
                "id": f.id,
                "description": f.description,
                "confidence": f.confidence,
                "promoted": f.promoted
            })).collect();
            
            ToolOutput::success(serde_json::json!({
                "exploration_id": exp.id,
                "title": exp.title,
                "purpose": exp.purpose,
                "status": format!("{:?}", exp.status),
                "started_at": exp.started_at.to_rfc3339(),
                "completed_at": exp.completed_at.map(|t| t.to_rfc3339()),
                "is_active": exp.is_active(),
                "is_complete": exp.is_complete(),
                "hypotheses": hypotheses_json,
                "attempts": attempts_json,
                "findings": findings_json
            }))
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

/// Complete an exploration with findings
pub fn execute_complete_exploration(input: CompleteExplorationInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            for finding_input in input.findings {
                let finding = ExplorationFinding::new(
                    Uuid::new_v4().to_string(),
                    finding_input.description,
                    finding_input.confidence,
                );
                exp.add_finding(finding);
            }
            
            exp.complete();
            
            let finding_count = exp.findings.len();
            ToolOutput::success(serde_json::json!({
                "exploration_id": exp.id,
                "status": "completed",
                "finding_count": finding_count,
                "message": format!("Exploration completed with {} findings.", finding_count)
            }))
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

/// Abandon an exploration
pub fn execute_abandon_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            exp.abandon();
            ToolOutput::success(serde_json::json!({
                "exploration_id": exp.id,
                "status": "abandoned",
                "message": "Exploration abandoned."
            }))
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

/// Record an attempt during exploration
pub fn execute_record_attempt(input: RecordAttemptInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            let mut attempt = ExplorationAttempt::new(
                Uuid::new_v4().to_string(),
                input.action,
            );
            
            if let Some(expected) = input.expected_result {
                attempt = attempt.with_expected_result(expected);
            }
            if let Some(actual) = input.actual_result {
                attempt = attempt.with_actual_result(actual);
            }
            
            exp.add_attempt(attempt);
            
            ToolOutput::success(serde_json::json!({
                "exploration_id": exp.id,
                "attempt_count": exp.attempts.len(),
                "message": "Attempt recorded."
            }))
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

/// Add a hypothesis to an exploration
pub fn execute_add_hypothesis(input: AddHypothesisInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            let hypothesis = Hypothesis::new(
                Uuid::new_v4().to_string(),
                input.statement,
                input.initial_confidence.unwrap_or(0.5),
            );
            
            exp.add_hypothesis(hypothesis);
            
            ToolOutput::success(serde_json::json!({
                "exploration_id": exp.id,
                "hypothesis_count": exp.hypotheses.len(),
                "message": "Hypothesis added to exploration."
            }))
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

/// Evaluate a hypothesis in an exploration
pub fn execute_evaluate_hypothesis(input: EvaluateHypothesisInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            let result = match input.result.to_lowercase().as_str() {
                "supported" => HypothesisResult::Supported,
                "partially_supported" => HypothesisResult::PartiallySupported,
                "rejected" => HypothesisResult::Rejected,
                _ => HypothesisResult::Unknown,
            };
            
            let new_confidence = match result {
                HypothesisResult::Supported => 0.9,
                HypothesisResult::PartiallySupported => 0.6,
                HypothesisResult::Rejected => 0.1,
                HypothesisResult::Unknown => 0.5,
            };
            
            if let Some(hypothesis) = exp.hypotheses.iter_mut().find(|h| h.id == input.hypothesis_id) {
                hypothesis.set_result(result.clone());
                hypothesis.update_confidence(new_confidence);
                
                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "hypothesis_id": hypothesis.id,
                    "result": format!("{:?}", result),
                    "confidence": hypothesis.confidence,
                    "message": "Hypothesis evaluated."
                }))
            } else {
                ToolOutput::error(format!("Hypothesis not found: {}", input.hypothesis_id))
            }
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}
