//! Validation helpers for test results.
//!
//! Contains functions to validate MCP tool responses against expected criteria.

use crate::function_registry::{CheckType, ValidationCheck};
use crate::test_results::ValidationResult;

/// Validate a result against a validation check
pub fn validate_result(result: &serde_json::Value, check: &ValidationCheck) -> ValidationResult {
    let passed = match check.check_type {
        CheckType::HasField => has_field(result, &check.field),
        CheckType::IsNonEmpty => is_non_empty(result, &check.field),
        CheckType::IsSuccess => is_success(result, &check.field, check.expected_value.as_deref()),
        CheckType::MatchesPattern => {
            matches_pattern(result, &check.field, check.expected_value.as_deref())
        }
        CheckType::GreaterThan => {
            greater_than(result, &check.field, check.expected_value.as_deref())
        }
        CheckType::LessThan => less_than(result, &check.field, check.expected_value.as_deref()),
    };

    ValidationResult {
        field: check.field.clone(),
        passed,
        message: Some(if passed {
            "OK".to_string()
        } else {
            "Failed".to_string()
        }),
    }
}

/// Check if result has a field (supports dot notation for nested fields)
pub fn has_field(result: &serde_json::Value, field: &str) -> bool {
    // If there's an error, the test should fail
    if result.get("error").is_some() || result.get("isError").and_then(|e| e.as_bool()).unwrap_or(false) {
        return false;
    }

    // Support dot notation for nested fields (e.g., "skill.name")
    if field.contains('.') {
        return has_nested_field(result, field);
    }

    // Try to find the field in various locations
    if result.get(field).is_some() {
        return true;
    }

    // Check in content[0].text (MCP response format)
    if let Some(content) = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
    {
        if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                if has_nested_field(&parsed, field) {
                    return true;
                }
            }
        }
    }

    // Check in data field
    if let Some(data) = result.get("data") {
        if has_nested_field(data, field) {
            return true;
        }
    }

    // Check nested in success field (for ToolOutput format)
    if field == "success" && result.get("success").is_some() {
        return true;
    }

    false
}

/// Check for nested field using dot notation
pub fn has_nested_field(result: &serde_json::Value, path: &str) -> bool {
    let parts: Vec<&str> = path.splitn(2, '.').collect();
    if parts.is_empty() {
        return false;
    }

    let first = parts[0];
    if let Some(value) = result.get(first) {
        if parts.len() == 1 {
            return true;
        }
        if parts.len() > 1 {
            return has_nested_field(value, parts[1]);
        }
    }
    false
}

/// Check if field is non-empty
pub fn is_non_empty(result: &serde_json::Value, field: &str) -> bool {
    // If there's an error, the test should fail
    if result.get("error").is_some() || result.get("isError").and_then(|e| e.as_bool()).unwrap_or(false) {
        return false;
    }

    if let Some(value) = result.get(field) {
        return !value.is_null() && !is_json_value_empty(value);
    }

    // Check in data field
    if let Some(data) = result.get("data") {
        if let Some(value) = data.get(field) {
            return !value.is_null() && !is_json_value_empty(value);
        }
    }

    false
}

/// Check if a JSON value is empty
pub fn is_json_value_empty(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::String(s) => s.is_empty(),
        serde_json::Value::Array(a) => a.is_empty(),
        serde_json::Value::Object(o) => o.is_empty(),
        _ => false,
    }
}

/// Check if success field has expected value
pub fn is_success(result: &serde_json::Value, field: &str, expected: Option<&str>) -> bool {
    // Check for error field (indicates MCP error occurred)
    let has_error_field = result.get("error").is_some();
    
    // Check for isError field (MCP response format)
    let is_error = result
        .get("isError")
        .and_then(|e| e.as_bool())
        .unwrap_or(false);

    // If there's an MCP error, ALWAYS fail the test
    // (even if validation expects failure - we want to know the MCP protocol is broken)
    if has_error_field || is_error {
        return false;
    }

    // Get the success value from the specified field
    let success_value = result.get(field);

    // Try to parse as bool
    let content_success = success_value.and_then(|s| s.as_bool());

    // If isError is true, treat as failure regardless of content
    // If isError is not present, check content's success field
    let success = if is_error {
        Some(false)
    } else {
        content_success.or(Some(true))
    };

    match (success, expected) {
        (Some(s), Some("false")) => !s,
        (Some(s), Some("true")) | (Some(s), None) => s,
        (None, Some("false")) => true, // If not present, treat as expected failure
        (None, Some("true")) => false, // Missing success field with "true" expected = fail
        _ => false,
    }
}

/// Check if field matches a pattern
pub fn matches_pattern(result: &serde_json::Value, field: &str, pattern: Option<&str>) -> bool {
    if let Some(pattern) = pattern {
        if let Some(value) = result.get(field).and_then(|v| v.as_str()) {
            return value.contains(pattern);
        }
    }
    true
}

/// Check if field is greater than value
pub fn greater_than(result: &serde_json::Value, field: &str, min_value: Option<&str>) -> bool {
    if let (Some(min_str), Some(value)) = (min_value, result.get(field).and_then(|v| v.as_f64())) {
        if let Ok(min) = min_str.parse::<f64>() {
            return value > min;
        }
    }
    true
}

/// Check if field is less than value
pub fn less_than(result: &serde_json::Value, field: &str, max_value: Option<&str>) -> bool {
    if let (Some(max_str), Some(value)) = (max_value, result.get(field).and_then(|v| v.as_f64())) {
        if let Ok(max) = max_str.parse::<f64>() {
            return value < max;
        }
    }
    true
}
