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
            format!("Failed: expected {} for field '{}'", 
                match check.check_type {
                    CheckType::IsSuccess => format!("success={}", check.expected_value.as_deref().unwrap_or("true")),
                    CheckType::HasField => "field present".to_string(),
                    CheckType::IsNonEmpty => "non-empty value".to_string(),
                    CheckType::MatchesPattern => format!("pattern '{}'", check.expected_value.as_deref().unwrap_or("")),
                    CheckType::GreaterThan => format!("> {}", check.expected_value.as_deref().unwrap_or("0")),
                    CheckType::LessThan => format!("< {}", check.expected_value.as_deref().unwrap_or("0")),
                },
                check.field)
        }),
    }
}

/// Check if result has a field (supports dot notation for nested fields)
pub fn has_field(result: &serde_json::Value, field: &str) -> bool {
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
        && let Some(text) = content.get("text").and_then(|t| t.as_str())
            && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text)
                && has_nested_field(&parsed, field) {
                    return true;
                }

    // Check in data field
    if let Some(data) = result.get("data")
        && has_nested_field(data, field) {
            return true;
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
    if let Some(value) = result.get(field) {
        return !value.is_null() && !is_json_value_empty(value);
    }

    // Check in data field
    if let Some(data) = result.get("data")
        && let Some(value) = data.get(field) {
            return !value.is_null() && !is_json_value_empty(value);
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
    // Check for isError field (MCP response format)
    // isError: true means the tool returned an error response
    let is_error = result
        .get("isError")
        .and_then(|e| e.as_bool())
        .unwrap_or(false);

    // Get the success value from the specified field
    let success_value = result.get(field);

    // Try to parse as bool
    let content_success = success_value.and_then(|s| s.as_bool());

    // Determine actual success:
    // - If isError is true, treat as success=false (the tool returned an error)
    // - Otherwise check the content's success field
    // - Default to true if no success field present (tool succeeded without explicit success)
    let actual_success = if is_error {
        false
    } else {
        content_success.unwrap_or(true)
    };

    // Validate against expected value
    match (actual_success, expected) {
        (false, Some("false")) => true,  // Got false, expected false -> PASS
        (true, Some("true")) | (true, None) => true,  // Got true, expected true or nothing -> PASS
        (false, Some("true")) => false,  // Got false, expected true -> FAIL
        (true, Some("false")) => false,  // Got true, expected false -> FAIL (shouldn't happen normally)
        (false, None) => false,  // Got false, expected nothing (default true) -> FAIL
        _ => false,
    }
}

/// Check if field matches a pattern
pub fn matches_pattern(result: &serde_json::Value, field: &str, pattern: Option<&str>) -> bool {
    if let Some(pattern) = pattern
        && let Some(value) = result.get(field).and_then(|v| v.as_str()) {
            return value.contains(pattern);
        }
    true
}

/// Check if field is greater than value
pub fn greater_than(result: &serde_json::Value, field: &str, min_value: Option<&str>) -> bool {
    if let (Some(min_str), Some(value)) = (min_value, result.get(field).and_then(|v| v.as_f64()))
        && let Ok(min) = min_str.parse::<f64>() {
            return value > min;
        }
    true
}

/// Check if field is less than value
pub fn less_than(result: &serde_json::Value, field: &str, max_value: Option<&str>) -> bool {
    if let (Some(max_str), Some(value)) = (max_value, result.get(field).and_then(|v| v.as_f64()))
        && let Ok(max) = max_str.parse::<f64>() {
            return value < max;
        }
    true
}
