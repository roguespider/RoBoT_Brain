//! Test Runner Module
//!
//! Contains the single test runner logic.

use std::collections::HashMap;
use std::time::Instant;

use crate::function_registry::{TestRequirement};
use crate::test_environment::TestEnvironment;
use crate::test_results::{TestResult, TestStatus};
use crate::TestMcpClient;

use super::argument_builder;
use super::validation;

/// Run a single test for a requirement
pub async fn run_single_test(
    client: &mut TestMcpClient,
    requirement: &TestRequirement,
    data_created: &HashMap<String, Vec<String>>,
    env: &TestEnvironment,
) -> TestResult {
    let start = Instant::now();
    let mut validation_results = Vec::new();

    // Debug: Print test start with full details
    eprintln!("[TEST] Starting: {} ({})", requirement.function_name, requirement.id);
    eprintln!("[TEST] Category: {}, Priority: {}", requirement.category, requirement.priority);
    eprintln!("[TEST] Expected: {}", requirement.expected_behavior);

    // Log available data dependencies (if any were created by prior tests)
    if let Some(data_req) = &requirement.requires_data
        && let Some(ids) = data_created.get(&data_req.data_type) {
            eprintln!("[TEST] Dependent on {} ({} items available)", data_req.data_type, ids.len());
        }

    // Build the arguments for this tool
    let args = argument_builder::build_test_arguments(requirement, env);
    eprintln!("[TEST] Args: {:?}", args);

    // Call the tool
    let tool_result = client.call_tool(&requirement.function_name, args).await;
    
    // Debug: Print raw result
    match &tool_result {
        Ok(result) => {
            eprintln!("[TEST] Raw result: {:?}", result);
        }
        Err(e) => {
            eprintln!("[TEST] Raw error: {}", e);
        }
    }

    // Run validation even if there's an error (for tests that expect errors)
    // Note: We need to distinguish between:
    // 1. Tool returned success=false (expected error) - validation should check this
    // 2. MCP protocol error (isError: true) - validation should treat as success=false
    // 3. MCP call failed (connection error, etc.) - this is a real error
    let (result_for_validation, is_tool_error) = match &tool_result {
        Ok(result) => {
            // Check if the result indicates an error (isError: true in MCP response)
            let has_is_error = result
                .get("isError")
                .and_then(|e| e.as_bool())
                .unwrap_or(false);
            
            if has_is_error {
                // Tool returned an error response (isError: true)
                // Parse the tool's response from the content array
                if let Some(content) = result
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                {
                    if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                            // Return the parsed tool response with isError flag
                            let mut tool_response = parsed;
                            tool_response["isError"] = serde_json::json!(true);
                            (tool_response, true)
                        } else {
                            // Fallback: create mock result
                            (serde_json::json!({"success": false, "isError": true}), true)
                        }
                    } else {
                        // Fallback: create mock result
                        (serde_json::json!({"success": false, "isError": true}), true)
                    }
                } else {
                    // Fallback: create mock result
                    (serde_json::json!({"success": false, "isError": true}), true)
                }
            } else {
                // Tool succeeded - use actual result
                (result.clone(), false)
            }
        }
        Err(e) => {
            // MCP call failed entirely - this is a real protocol error
            // Extract error message and create result for validation
            let error_msg = e.to_string();
            (serde_json::json!({
                "success": false,
                "isError": true,
                "error": error_msg
            }), true)
        }
    };

    for check in &requirement.validation {
        let vr = validation::validate_result(&result_for_validation, check);
        validation_results.push(vr);
    }

    // Debug: Print validation results
    eprintln!("[TEST] Validation Results:");
    for vr in &validation_results {
        let icon = if vr.passed { "[OK]" } else { "[FAIL]" };
        let msg = vr.message.as_deref().unwrap_or("no message");
        eprintln!("[TEST]   {} {} - {}", icon, vr.field, msg);
    }

    let all_passed = validation_results.iter().all(|v| v.passed);

    // Determine the appropriate status and message
    // We need to handle:
    // 1. is_tool_error=true, all_passed=true -> This is CORRECT behavior (tool returned error, test expected error)
    // 2. is_tool_error=true, all_passed=false -> Tool returned error but test didn't expect it (FAIL)
    // 3. is_tool_error=false, all_passed=true -> Tool succeeded, test expected success (PASS)
    // 4. is_tool_error=false, all_passed=false -> Tool succeeded but test expected different result (FAIL)
    // 5. MCP call failed entirely (Err) -> Real error, report it
    
    let (status, error_message) = if is_tool_error && all_passed {
        // Case 1: Tool returned error (isError: true) and test expected error -> PASS
        (TestStatus::Pass, None)
    } else if is_tool_error && !all_passed {
        // Case 2: Tool returned error but test didn't expect it
        let failed_checks: Vec<_> = validation_results
            .iter()
            .filter(|v| !v.passed)
            .map(|v| v.message.as_deref().unwrap_or("validation failed"))
            .collect();
        (TestStatus::Fail, Some(format!("Tool returned error but validation failed: {}", failed_checks.join("; "))))
    } else if !is_tool_error && all_passed {
        // Case 3: Tool succeeded and test expected success -> PASS
        (TestStatus::Pass, None)
    } else if !is_tool_error && !all_passed {
        // Case 4: Tool succeeded but validation failed
        let failed_checks: Vec<_> = validation_results
            .iter()
            .filter(|v| !v.passed)
            .map(|v| v.message.as_deref().unwrap_or("validation failed"))
            .collect();
        (TestStatus::Fail, Some(format!("Validation failed: {}", failed_checks.join("; "))))
    } else {
        // Case 5: MCP call failed entirely
        let err_msg = match &tool_result {
            Err(e) => e.to_string(),
            _ => "Unknown error".to_string(),
        };
        (TestStatus::Error, Some(format!("MCP protocol error: {}", err_msg)))
    };

    // Attach recent server logs to non-passing results for diagnosis.
    // Passes don't need logs (kept empty to minimize noise/report size).
    let server_logs = if status == TestStatus::Pass {
        Vec::new()
    } else {
        // Pull recent lines plus any lines mentioning this tool name.
        let mut logs = client.recent_server_logs(15).await;
        let matching = client
            .server_logs_matching(&requirement.function_name)
            .await;
        for m in matching {
            if !logs.contains(&m) {
                logs.push(m);
            }
        }
        logs
    };

    TestResult {
        requirement: requirement.clone(),
        status,
        error_message,
        duration_ms: start.elapsed().as_millis() as u64,
        validation_results,
        server_logs,
    }
}

/// Track created data for dependent tests
pub fn track_data_creation(
    result_status: TestStatus,
    requirement: &TestRequirement,
    mut data_created: HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    if result_status == TestStatus::Pass
        && let Some(data_req) = &requirement.requires_data {
            data_created
                .entry(data_req.data_type.clone())
                .or_default()
                .push(requirement.id.clone());
        }
    data_created
}
