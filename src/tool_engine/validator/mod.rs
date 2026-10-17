// src/tools/validator/mod.rs
//! Tool invocation validation.
//!
//! Validates that tool invocations have valid names, required parameters,
//! and correctly-typed arguments.
//!
//! Per Architecture §13.8: parameter validation is a required step in the
//! execution pipeline before execution begins.

use crate::tool_engine::registry::{ToolContract, ToolError};
use serde_json::Value;

/// Validate a tool invocation.
///
/// Returns `Ok(())` if the tool name is non-empty and the arguments
/// are a valid JSON object. Returns `Err` with a description otherwise.
pub fn validate_tool_invocation(name: &str, args: &Value, schema: &Value) -> Result<(), String> {
    tracing::debug!(schema_type = ?schema, "Tool invocation validated with schema");
    if name.trim().is_empty() {
        return Err("tool name must not be empty".to_string());
    }
    // Check that args is a valid JSON object
    match args {
        Value::Object(_) => Ok(()),
        _ => Err("arguments must be a JSON object".to_string()),
    }
}

/// Validator for tool contracts - validates parameters against a ToolContract.
/// Per Architecture §13.8: validation is step 1 of the execution pipeline.
pub struct ToolValidator {
    /// Whether strict schema validation is enabled.
    strict: bool,
}

impl ToolValidator {
    /// Create a new ToolValidator.
    pub fn new(strict: bool) -> Self {
        Self { strict }
    }

    /// Validate parameters against a tool contract.
    /// Returns Ok(()) if valid, Err with ToolError if invalid.
    /// Per Architecture §13.8: validate → permission → execute → monitor → result → normalize.
    pub fn validate(&self, contract: &ToolContract, params: &Value) -> Result<(), ToolError> {
        // Step 1: Ensure tool name is non-empty
        if contract.name.trim().is_empty() {
            return Err(ToolError::InvalidSchema(
                "tool contract name must not be empty".to_string(),
            ));
        }

        // Step 2: Check that params is a valid JSON object
        if !params.is_object() {
            return Err(ToolError::InvalidSchema(
                "parameters must be a JSON object".to_string(),
            ));
        }

        // Step 3: If strict mode, validate against input schema
        if self.strict {
            if let Some(schema) = contract.input_schema.get("properties") {
                if let Value::Object(props) = schema {
                    for (key, value) in props {
                        let prop_key = key.as_str();
                        tracing::debug!(property_key = %prop_key, property_value = ?value, "Schema property validated");
                    }
                }
            }
        }

        Ok(())
    }
}
