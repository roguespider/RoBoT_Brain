

// src/tools/hypothesis/mod.rs
// Hypothesis Engine: Observation -> Hypothesis -> Test -> Evidence -> Knowledge

mod db;
mod execute;

use serde::{Deserialize, Serialize};

// ============================================================================
// TOOL INPUT/OUTPUT TYPES
// ============================================================================

/// Record an observation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecordObservationInput {
    pub content: String,
    pub context: String,
    pub observation_type: String, // success, failure, pattern, anomaly
}

/// Create a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateHypothesisInput {
    pub statement: String,
    pub domain: String,
    pub source_observations: Vec<String>,
}

/// Add evidence to a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddEvidenceInput {
    pub hypothesis_id: String,
    pub content: String,
    pub evidence_type: String, // success, failure, correlation, anomaly
    pub direction: String,     // support, contradict
    pub strength: f32,
}

/// Get hypothesis details
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetHypothesisInput {
    pub hypothesis_id: String,
}

/// Get observation by ID
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetObservationInput {
    pub observation_id: String,
}

/// List hypotheses with optional filter
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListHypothesesInput {
    pub domain: Option<String>,
    pub status: Option<String>,
    pub limit: Option<usize>,
}

/// List observations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListObservationsInput {
    pub observation_type: Option<String>,
    pub limit: Option<usize>,
}

/// Get learned knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetKnowledgeInput {
    pub domain: Option<String>,
    pub limit: Option<usize>,
}

/// Evaluate and update hypothesis status
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EvaluateHypothesisInput {
    pub hypothesis_id: String,
}

/// Convert supported hypothesis to knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExtractKnowledgeInput {
    pub hypothesis_id: String,
    pub knowledge_content: String,
}

// ============================================================================
// TOOL DEFINITIONS
// ============================================================================

pub mod definitions {
    pub const RECORD_OBSERVATION: &str = "record_observation";
    pub const CREATE_HYPOTHESIS: &str = "create_hypothesis";
    pub const ADD_EVIDENCE: &str = "add_evidence";
    pub const GET_HYPOTHESIS: &str = "get_hypothesis";
    pub const GET_OBSERVATION: &str = "get_observation";
    pub const LIST_HYPOTHESES: &str = "list_hypotheses";
    pub const LIST_OBSERVATIONS: &str = "list_observations";
    pub const EVALUATE_HYPOTHESIS: &str = "evaluate_hypothesis";
    pub const GET_KNOWLEDGE: &str = "get_knowledge";
    pub const EXTRACT_KNOWLEDGE: &str = "extract_knowledge";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: RECORD_OBSERVATION.to_string(),
                description: "Record an observation. Observations are the starting point for learning - record successes, failures, patterns, or anomalies.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "What was observed"
                        },
                        "context": {
                            "type": "string",
                            "description": "Context or circumstances of the observation"
                        },
                        "observation_type": {
                            "type": "string",
                            "description": "Type: success, failure, pattern, anomaly"
                        }
                    },
                    "required": ["content", "observation_type"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: CREATE_HYPOTHESIS.to_string(),
                description: "Create a testable hypothesis from observations. A hypothesis is a statement that can be tested with evidence.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "statement": {
                            "type": "string",
                            "description": "The hypothesis statement (e.g., 'Using X approach improves Y outcome')"
                        },
                        "domain": {
                            "type": "string",
                            "description": "Domain/category (e.g., workflow, tool, pattern)"
                        },
                        "source_observations": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "IDs of observations that led to this hypothesis"
                        }
                    },
                    "required": ["statement", "domain"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: ADD_EVIDENCE.to_string(),
                description: "Add evidence to a hypothesis. Evidence can support or contradict the hypothesis.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": {
                            "type": "string",
                            "description": "ID of the hypothesis"
                        },
                        "content": {
                            "type": "string",
                            "description": "Description of the evidence"
                        },
                        "evidence_type": {
                            "type": "string",
                            "description": "Type: success, failure, correlation, anomaly"
                        },
                        "direction": {
                            "type": "string",
                            "description": "support or contradict"
                        },
                        "strength": {
                            "type": "number",
                            "description": "Strength of evidence 0.0-1.0"
                        }
                    },
                    "required": ["hypothesis_id", "content", "direction"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_HYPOTHESIS.to_string(),
                description: "Get details of a specific hypothesis including all its evidence.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": {
                            "type": "string",
                            "description": "ID of the hypothesis"
                        }
                    },
                    "required": ["hypothesis_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_OBSERVATION.to_string(),
                description: "Get a specific observation by its ID. Useful for examining individual observations that contributed to hypotheses.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "observation_id": {
                            "type": "string",
                            "description": "ID of the observation to retrieve"
                        }
                    },
                    "required": ["observation_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_HYPOTHESES.to_string(),
                description: "List all hypotheses with optional filters.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "domain": {
                            "type": "string",
                            "description": "Filter by domain/category"
                        },
                        "status": {
                            "type": "string",
                            "description": "Filter by status: testing, supported, refuted, inconclusive, superseded"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results (default: 10)"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_OBSERVATIONS.to_string(),
                description: "List recorded observations.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "observation_type": {
                            "type": "string",
                            "description": "Filter by type: success, failure, pattern, anomaly"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results (default: 10)"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: EVALUATE_HYPOTHESIS.to_string(),
                description: "Evaluate a hypothesis based on accumulated evidence. Determines if hypothesis is supported, refuted, or needs more testing.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": {
                            "type": "string",
                            "description": "ID of the hypothesis to evaluate"
                        }
                    },
                    "required": ["hypothesis_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_KNOWLEDGE.to_string(),
                description: "Get learned knowledge that can inform future decisions.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "domain": {
                            "type": "string",
                            "description": "Filter by domain/category"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results (default: 10)"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: EXTRACT_KNOWLEDGE.to_string(),
                description: "Extract supported hypothesis as reusable knowledge. Only supported hypotheses can be extracted.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": {
                            "type": "string",
                            "description": "ID of the supported hypothesis"
                        },
                        "knowledge_content": {
                            "type": "string",
                            "description": "The knowledge content to extract from this hypothesis"
                        }
                    },
                    "required": ["hypothesis_id", "knowledge_content"]
                }),
            },
        ]
    }
}

// Re-export database functions

// Re-export execution functions
#[allow(unused_imports)]
pub use execute::{
    execute_record_observation, execute_create_hypothesis, execute_add_evidence,
    execute_get_hypothesis, execute_get_observation, execute_list_hypotheses, 
    execute_list_observations, execute_evaluate_hypothesis, execute_get_knowledge, 
    execute_extract_knowledge,
};
