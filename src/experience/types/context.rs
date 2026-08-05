// src/experience/types/context.rs
// Context types for experiences

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Context surrounding an experience.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExperienceContext {
    pub workflow: Option<WorkflowContext>,
    pub tool: Option<ToolContext>,
    pub model: Option<ModelContext>,

    pub session_id: Option<String>,
    pub parent_experience: Option<Uuid>,
    pub user_query: Option<String>,
    
    // Per Architecture §6.3: Memory-related context
    pub memory_type: Option<String>,
    pub content_length: Option<usize>,
    pub search_query: Option<String>,
    pub results_count: Option<usize>,
    pub source: Option<String>,
    
    // Hypothesis exploration context
    pub related_hypothesis: Option<String>,
}

/// Workflow information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub name: String,
    pub step: Option<String>,
    pub parent_workflow: Option<String>,
}

/// Tool information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    pub name: String,
    pub version: Option<String>,
    pub arguments: HashMap<String, String>,
}

/// Model information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelContext {
    pub name: String,
    pub provider: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}
