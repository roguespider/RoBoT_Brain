// src/workflows/engine/experience.rs
//! Experience recording helpers

use std::collections::HashMap;

use crate::tools::ToolOutput;
use crate::workflows::engine::types::ExperienceRecord;

/// Build search query from action and parameters
pub fn build_search_query(action: &str, params: &HashMap<String, String>) -> String {
    let mut parts = vec![action.replace('_', " ")];

    for (_key, value) in params.iter() {
        if !value.is_empty() && value.len() < 100 {
            let normalized_value = value.replace(['[', ']', '{', '}'], "").trim().to_string();
            if !normalized_value.is_empty() {
                parts.push(normalized_value);
            }
        }
    }

    let query = parts.join(" ");
    if query.len() > 200 {
        query[..200].to_string()
    } else {
        query
    }
}

/// Build experience description following architecture #5, #10
/// Architecture #5: Separate Observation From Interpretation
/// Architecture #10: Reflection asks what happened, why, expected, what changes
pub fn build_experience_description(
    action: &str,
    params: &HashMap<String, String>,
    result: &ToolOutput,
) -> String {
    let record = ExperienceRecord::new(action, params, result);
    record.to_description()
}

/// Extract a brief summary from the result
pub fn extract_result_summary(result: &ToolOutput) -> String {
    result
        .data
        .get("message")
        .or_else(|| result.data.get("summary"))
        .or_else(|| result.data.get("content"))
        .and_then(|v| v.as_str())
        .map(|s| {
            if s.len() > 100 {
                format!("{}...", &s[..100])
            } else {
                s.to_string()
            }
        })
        .unwrap_or_else(|| "No details".to_string())
}

/// Map workflow action names to ExperienceType
pub fn map_action_to_experience_type(action: &str) -> String {
    match action {
        // File operations
        "create_file" | "write_file" | "edit_file" | "delete_file" => "tool_execution".to_string(),

        // Command execution
        "run_command" | "execute_command" | "bash" => "tool_execution".to_string(),

        // Memory operations
        "store_memory" | "write_memory" => "memory_store".to_string(),
        "search_memory" | "read_memory" | "get_memory" => "memory_lookup".to_string(),

        // Workflow operations
        "create_workflow" | "start_workflow" | "execute_workflow" => "workflow".to_string(),

        // Reflection
        "create_reflection" | "reflect" => "reflection".to_string(),

        // File ingestion
        "ingest_files" | "import_files" => "tool_execution".to_string(),

        // Experience recording
        "record_experience" => "learning".to_string(),

        // Generic fallback
        _ => "system".to_string(),
    }
}

impl ExperienceRecord {
    /// Create a new experience record with separated observation/interpretation
    pub fn new(action: &str, params: &HashMap<String, String>, result: &ToolOutput) -> Self {
        let observation = Self::build_observation(action, params);
        let outcome = Self::build_outcome(result);
        let search_query = build_search_query(action, params);
        let title = format!("Workflow: {}", params.get("title").or(params.get("name")).cloned().unwrap_or_else(|| action.replace('_', " ")));
        let outcome_kind = if result.success { "success" } else { "failure" }.to_string();

        let reflection_questions = vec![
            "Why did this happen?".to_string(),
            "Was the outcome expected?".to_string(),
            "What should change?".to_string(),
            "What should be attempted next?".to_string(),
        ];

        Self {
            action: action.to_string(),
            observation,
            outcome,
            outcome_kind,
            search_query,
            title,
            interpretation: None,
            reflection_questions,
        }
    }

    /// Build raw observation - observable facts only
    fn build_observation(action: &str, _params: &HashMap<String, String>) -> String {
        match action {
            "create_file" | "write_file" => format!("File operation: create/write"),
            "edit_file" => format!("File operation: edit"),
            "delete_file" => format!("File operation: delete"),
            "run_command" | "execute_command" | "bash" => format!("Command executed"),
            "create_reflection" => format!("Reflection created"),
            "ingest_files" | "import_files" => format!("Files ingested"),
            "record_experience" => format!("Experience recorded"),
            "search_memory" | "get_memory" => format!("Memory accessed"),
            "add_knowledge" => format!("Knowledge added"),
            _ => format!("Action: {}", action),
        }
    }

    /// Build outcome description
    fn build_outcome(result: &ToolOutput) -> String {
        let status = if result.success { "success" } else { "failure" };
        let summary = extract_result_summary(result);
        format!("{}: {}", status, summary)
    }

    /// Convert to description string for storage
    /// Separates observation from interpretation for architecture #5
    pub fn to_description(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("[OBSERVATION] {}", self.observation));
        parts.push(format!("[OUTCOME] {}", self.outcome));

        if let Some(ref interp) = self.interpretation {
            parts.push(format!("[INTERPRETATION] {}", interp));
        } else {
            parts.push("[INTERPRETATION] Pending reflection".to_string());
        }

        parts.push("[REFLECTION QUESTIONS]".to_string());
        for q in &self.reflection_questions {
            parts.push(format!("  - {}", q));
        }

        parts.join("\n")
    }
}
