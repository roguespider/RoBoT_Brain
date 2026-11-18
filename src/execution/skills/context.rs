// src/skills/registry/context.rs
//! Execution context for skills

use serde::{Deserialize, Serialize};

/// Context for skill execution
/// Per Architecture §15: "Skill::execute(&context)"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub task: String,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
    pub working_memory: std::collections::HashMap<String, serde_json::Value>,
    pub knowledge_context: Vec<String>,
    pub time_limit_secs: Option<u64>,
}

impl ExecutionContext {
    pub fn new(task: String) -> Self {
        Self {
            task,
            parameters: std::collections::HashMap::new(),
            working_memory: std::collections::HashMap::new(),
            knowledge_context: Vec::new(),
            time_limit_secs: None,
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.parameters.insert(key.into(), value);
        self
    }

    pub fn with_memory(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.working_memory.insert(key.into(), value);
        self
    }

    pub fn with_knowledge_context(mut self, knowledge_ids: Vec<String>) -> Self {
        self.knowledge_context = knowledge_ids;
        self
    }

    pub fn with_time_limit(mut self, secs: u64) -> Self {
        self.time_limit_secs = Some(secs);
        self
    }
}
