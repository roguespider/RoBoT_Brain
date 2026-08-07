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

    // Build the arguments for this tool
    let args = argument_builder::build_test_arguments(requirement, env);

    // Call the tool
    let tool_result = client.call_tool(&requirement.function_name, args).await;

    // Run validation even if there's an error (for tests that expect errors)
    let result_for_validation = match &tool_result {
        Ok(result) => result.clone(),
        Err(_) => {
            // Create a mock result for validation that indicates error
            // The validation can check for expected error conditions
            serde_json::json!({
                "success": false,
                "error": tool_result.as_ref().err().map(|e| e.to_string())
            })
        }
    };

    for check in &requirement.validation {
        let vr = validation::validate_result(&result_for_validation, check);
        validation_results.push(vr);
    }

    match tool_result {
        Ok(result) => {
            // Check if all validations passed
            let all_passed = validation_results.iter().all(|v| v.passed);

            TestResult {
                requirement: requirement.clone(),
                status: if all_passed {
                    TestStatus::Pass
                } else {
                    TestStatus::Fail
                },
                error_message: if all_passed {
                    None
                } else {
                    Some("Validation failed".to_string())
                },
                duration_ms: start.elapsed().as_millis() as u64,
                validation_results,
            }
        }
        Err(e) => {
            // MCP error occurred - ALL validations should fail
            // (even if they expect success=false, we want to know MCP is broken)
            let all_passed = validation_results.iter().all(|v| v.passed);

            TestResult {
                requirement: requirement.clone(),
                status: if all_passed {
                    // This shouldn't happen if validation properly checks for errors
                    TestStatus::Pass
                } else {
                    TestStatus::Error
                },
                error_message: Some("MCP protocol error - tools/call method not found".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
                validation_results,
            }
        }
    }
}

/// Track created data for dependent tests
pub fn track_data_creation(
    result_status: TestStatus,
    requirement: &TestRequirement,
    mut data_created: HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    if result_status == TestStatus::Pass {
        if let Some(data_req) = &requirement.requires_data {
            data_created
                .entry(data_req.data_type.clone())
                .or_default()
                .push(requirement.id.clone());
        }
    }
    data_created
}
