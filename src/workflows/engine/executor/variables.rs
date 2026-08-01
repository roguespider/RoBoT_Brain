// src/workflows/engine/executor/variables.rs
//! Variable resolution for workflow steps

use std::collections::HashMap;

use crate::tools::ToolOutput;

/// Replace variables in parameters with their values
pub fn replace_variables(
    params: &HashMap<String, String>,
    workflow_vars: &HashMap<String, String>,
    step_results: &HashMap<String, ToolOutput>,
) -> HashMap<String, String> {
    let mut resolved = params.clone();
    for value in resolved.values_mut() {
        // Replace workflow variables ${var_name}
        for (var_name, var_value) in workflow_vars {
            let placeholder = format!("${{{}}}", var_name);
            *value = value.replace(&placeholder, var_value);
        }
        // Replace step result references ${step_id.output_field}
        for (step_id, result) in step_results {
            if let Some(obj) = result.data.as_object() {
                for (field, field_value) in obj {
                    let placeholder = format!("${{{}.{}}}", step_id, field);
                    *value = value.replace(&placeholder, &field_value.to_string());
                }
            }
        }
    }
    resolved
}
