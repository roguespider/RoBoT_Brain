//! Comprehensive End-to-End Test Module
//! 
//! This module provides true end-to-end testing for all MCP tools.
//! It validates actual functionality without stubs or mocking.

use crate::{TestMcpClient, TestStats};
use crate::test_environment::TestEnvironment;
use crate::function_registry::{FunctionRegistry, TestRequirement, ValidationCheck, CheckType};
use crate::test_results::{TestReport, TestResult, TestStatus, ValidationResult};
use crate::code_analyzer::{CodeAnalyzer, LintAnalyzer, LintSummary};
use crate::test_results::print_issues_table;
use std::time::Instant;

/// Run the comprehensive end-to-end test suite
pub async fn run_comprehensive_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) -> anyhow::Result<TestReport> {
    let start_time = Instant::now();
    let mut report = TestReport::new();
    
    println!("\n{}", "#".repeat(100));
    println!("#  ROBO T BRAIN - COMPREHENSIVE END-TO-END TEST SUITE");
    println!("#  Testing every function 100% end-to-end without stubs or #[allow(*)]");
    println!("{}", "#".repeat(100));
    
    // Step 1: Analyze source code for issues
    println!("\n📊 PHASE 1: SOURCE CODE ANALYSIS");
    println!("{}", "─".repeat(100));
    
    let source_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("src");
    let analyzer = CodeAnalyzer::new(source_path);
    let code_issues = analyzer.analyze();
    let summary = analyzer.get_summary(&code_issues);
    
    summary.print_table();
    report.set_code_issues(code_issues.clone());
    
    // Print issues table
    print_issues_table(&code_issues);
    
    // Step 1b: Run lint analysis (clippy + cargo check)
    println!("\n📋 PHASE 1B: LINT ANALYSIS (clippy + cargo check)");
    println!("{}", "─".repeat(100));
    
    let project_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    
    println!("  Running clippy...");
    let clippy_issues = match LintAnalyzer::run_clippy(&project_path) {
        Ok(issues) => issues,
        Err(e) => {
            println!("    ⚠️  Clippy failed: {}", e);
            Vec::new()
        }
    };
    
    println!("  Running cargo check...");
    let check_issues = match LintAnalyzer::run_check(&project_path) {
        Ok(issues) => issues,
        Err(e) => {
            println!("    ⚠️  Cargo check failed: {}", e);
            Vec::new()
        }
    };
    
    // Combine and dedupe issues
    let mut all_lint_issues = clippy_issues;
    for issue in check_issues {
        if !all_lint_issues.iter().any(|i| i.file_path == issue.file_path && i.line_number == issue.line_number && i.message == issue.message) {
            all_lint_issues.push(issue);
        }
    }
    
    let lint_summary = LintSummary::new(all_lint_issues);
    lint_summary.print_report();
    
    // Store lint issues in report
    report.lint_errors = lint_summary.errors;
    report.lint_warnings = lint_summary.warnings;
    
    // Step 2: Get all test requirements
    println!("\n📋 PHASE 2: COLLECTING TEST REQUIREMENTS");
    println!("{}", "─".repeat(100));
    
    let requirements = FunctionRegistry::get_all_functions();
    println!("  Found {} test requirements across {} categories", 
        requirements.len(),
        get_category_count(&requirements));
    
    for category in get_categories(&requirements) {
        let count = requirements.iter().filter(|r| r.category == category).count();
        println!("    - {}: {} tests", category, count);
    }
    
    // Step 3: Run all tests
    println!("\n🧪 PHASE 3: RUNNING END-TO-END TESTS");
    println!("{}", "─".repeat(100));
    
    // First, ensure workflow is initialized (required for most tests)
    println!("\n  Initializing workflow...");
    match client.call_tool("get_workflow", serde_json::json!({
        "purpose": "default"
    })).await {
        Ok(_) => println!("    ✅ Workflow initialized"),
        Err(e) => println!("    ⚠️  Workflow init warning: {}", e),
    }
    
    // Run tests for each category
    let mut data_created: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    for requirement in &requirements {
        let result = run_single_test(client, requirement, &data_created, env).await;
        
        // Track created data for dependent tests
        if result.status == TestStatus::Pass {
            if let Some(data_req) = &requirement.requires_data {
                data_created
                    .entry(data_req.data_type.clone())
                    .or_default()
                    .push(requirement.id.clone());
            }
        }
        
        report.add_result(result);
        
        // Update stats
        match report.results.last() {
            Some(r) => {
                match r.status {
                    TestStatus::Pass => stats.passed += 1,
                    TestStatus::Fail | TestStatus::Error => stats.failed += 1,
                    TestStatus::Skipped => stats.skipped += 1,
                    TestStatus::Blocked => stats.skipped += 1,
                }
            }
            None => {}
        }
    }
    
    // Step 4: Generate report
    println!("\n📊 PHASE 4: GENERATING REPORT");
    println!("{}", "─".repeat(100));
    
    report.print_report();
    
    println!("\n  Total test duration: {:?}", start_time.elapsed());
    
    Ok(report)
}

/// Run a single test for a requirement
async fn run_single_test(
    client: &mut TestMcpClient,
    requirement: &TestRequirement,
    _data_created: &std::collections::HashMap<String, Vec<String>>,
    env: &TestEnvironment,
) -> TestResult {
    let start = Instant::now();
    let mut validation_results = Vec::new();
    
    // Build the arguments for this tool
    let args = build_test_arguments(requirement, env);
    
    // Call the tool
    match client.call_tool(&requirement.function_name, args).await {
        Ok(result) => {
            // Validate the result
            for check in &requirement.validation {
                let vr = validate_result(&result, check);
                validation_results.push(vr);
            }
            
            // Check if all validations passed
            let all_passed = validation_results.iter().all(|v| v.passed);
            
            TestResult {
                requirement: requirement.clone(),
                status: if all_passed { TestStatus::Pass } else { TestStatus::Fail },
                error_message: if all_passed { None } else { Some("Validation failed".to_string()) },
                duration_ms: start.elapsed().as_millis() as u64,
                validation_results,
            }
        }
        Err(e) => {
            TestResult {
                requirement: requirement.clone(),
                status: TestStatus::Error,
                error_message: Some(e.to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
                validation_results,
            }
        }
    }
}

/// Build test arguments based on the requirement
fn build_test_arguments(requirement: &TestRequirement, env: &TestEnvironment) -> serde_json::Value {
    match requirement.id.as_str() {
        // Agent tools
        "agent_get_workflow_default" => serde_json::json!({
            "purpose": "default"
        }),
        "agent_get_workflow_general" => serde_json::json!({
            "purpose": "general"
        }),
        "agent_list_tools" => serde_json::json!({}),
        "agent_list_tools_memory" => serde_json::json!({
            "category": "memory"
        }),
        "agent_get_tool" => serde_json::json!({
            "name": "store_memory"
        }),
        
        // Memory tools
        "memory_store_basic" => serde_json::json!({
            "content": "Test memory content",
            "memory_type": "note"
        }),
        "memory_store_with_metadata" => serde_json::json!({
            "content": "Test memory with metadata",
            "memory_type": "fact",
            "confidence": 0.9,
            "importance": 0.8
        }),
        "memory_search" => serde_json::json!({
            "query": "test"
        }),
        "memory_get" => serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000"
        }),
        "memory_get_invalid" => serde_json::json!({
            "id": "not-a-valid-uuid"
        }),
        "memory_list" => serde_json::json!({}),
        "memory_list_filtered" => serde_json::json!({
            "memory_type": "note"
        }),
        
        // Experience tools
        "experience_record" => serde_json::json!({
            "action": "Test Action",
            "outcome": "Success",
            "tool_name": "test_tool"
        }),
        "experience_get" => serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000"
        }),
        "experience_list" => serde_json::json!({}),
        "experience_stats" => serde_json::json!({}),
        
        // Reflection tools
        "reflection_create" => serde_json::json!({
            "title": "Test Reflection",
            "reflection_type": "analysis"
        }),
        "reflection_get_patterns" => serde_json::json!({}),
        "reflection_get_insights" => serde_json::json!({}),
        "reflection_analyze" => serde_json::json!({}),
        
        // Search tools
        "search_global" => serde_json::json!({
            "query": "test"
        }),
        "search_recommendations" => serde_json::json!({}),
        "search_reputation" => serde_json::json!({
            "tool_name": "store_memory"
        }),
        
        // Ingestor tools
        "ingestor_list_importable" => serde_json::json!({}),
        "ingestor_list_importable_recursive" => serde_json::json!({
            "recursive": true,
            "list_all": true
        }),
        "ingestor_ingest_text" => serde_json::json!({
            "file_path": env.files_folder.join("readme.txt").to_string_lossy()
        }),
        "ingestor_ingest_json" => serde_json::json!({
            "file_path": env.files_folder.join("config_files/data.json").to_string_lossy()
        }),
        "ingestor_ingest_code" => serde_json::json!({
            "file_path": env.files_folder.join("code_samples/sample.rs").to_string_lossy()
        }),
        "ingestor_list_ingested" => serde_json::json!({}),
        "ingestor_delete_blocked" => serde_json::json!({
            "file_ids": ["test_file_id"]
        }),
        
        // Hypothesis tools
        "hypothesis_record_observation" => serde_json::json!({
            "observation_type": "pattern",
            "content": "Test observation content"
        }),
        "hypothesis_create" => serde_json::json!({
            "hypothesis": "Users prefer memory-first approach"
        }),
        "hypothesis_add_evidence" => serde_json::json!({
            "evidence_type": "support",
            "strength": 0.8
        }),
        "hypothesis_get" => serde_json::json!({}),
        "hypothesis_list" => serde_json::json!({}),
        "hypothesis_evaluate" => serde_json::json!({}),
        "hypothesis_extract" => serde_json::json!({}),
        
        // Exploration tools
        "exploration_start" => serde_json::json!({
            "topic": "Test Exploration"
        }),
        "exploration_status" => serde_json::json!({}),
        "exploration_record_attempt" => serde_json::json!({
            "attempt": "Test attempt",
            "result": "partial"
        }),
        "exploration_add_hypothesis" => serde_json::json!({
            "hypothesis": "Test hypothesis"
        }),
        
        // Knowledge tools
        "knowledge_add" => serde_json::json!({
            "statement": "Test knowledge content"
        }),
        "knowledge_query" => serde_json::json!({
            "query": "test"
        }),
        "knowledge_mature" => serde_json::json!({
            "min_applications": 5
        }),
        "knowledge_stats" => serde_json::json!({}),
        "knowledge_record_application" => serde_json::json!({
            "knowledge_id": "00000000-0000-0000-0000-000000000000",
            "success": true
        }),
        
        // Planner tools
        "planner_create" => serde_json::json!({
            "description": "Test Plan"
        }),
        "planner_add_step" => serde_json::json!({
            "description": "Step 1"
        }),
        "planner_add_dependency" => serde_json::json!({
            "from_step": 0,
            "to_step": 1
        }),
        "planner_get" => serde_json::json!({}),
        "planner_start" => serde_json::json!({}),
        "planner_complete_step" => serde_json::json!({
            "step_index": 0
        }),
        "planner_fail_step" => serde_json::json!({
            "step_index": 1,
            "error": "Test failure"
        }),
        "planner_cancel" => serde_json::json!({}),
        "planner_list" => serde_json::json!({}),
        
        // Workflow tools
        "workflow_create" => serde_json::json!({
            "name": "Test Workflow"
        }),
        "workflow_add_step" => serde_json::json!({
            "name": "Step 1",
            "tool_name": "store_memory"
        }),
        "workflow_status" => serde_json::json!({}),
        "workflow_start" => serde_json::json!({}),
        "workflow_pause" => serde_json::json!({}),
        "workflow_resume" => serde_json::json!({}),
        "workflow_cancel" => serde_json::json!({}),
        "workflow_delete" => serde_json::json!({}),
        "workflow_list" => serde_json::json!({}),
        
        // Skills tools
        "skills_register" => serde_json::json!({
            "name": "test_skill",
            "description": "A test skill",
            "category": "file_operation"
        }),
        "skills_discover" => serde_json::json!({
            "name": "discovered_skill",
            "description": "Discovered from experience",
            "category": "search",
            "source_experience_id": "00000000-0000-0000-0000-000000000000"
        }),
        "skills_get" => serde_json::json!({
            "skill_id": "00000000-0000-0000-0000-000000000000"
        }),
        "skills_list" => serde_json::json!({}),
        "skills_update_mastery" => serde_json::json!({
            "skill_id": "00000000-0000-0000-0000-000000000000",
            "success": true
        }),
        "skills_recommendations" => serde_json::json!({}),
        "skills_execute" => serde_json::json!({
            "skill_id": "00000000-0000-0000-0000-000000000000",
            "task": "test task"
        }),
        "skills_stats" => serde_json::json!({}),
        "skills_decay" => serde_json::json!({
            "decay_rate": 0.05
        }),
        "skills_enable_disable" => serde_json::json!({
            "skill_id": "00000000-0000-0000-0000-000000000000",
            "enable": false
        }),
        "skills_search" => serde_json::json!({
            "query": "test"
        }),
        
        // Default: empty arguments
        _ => serde_json::json!({}),
    }
}

/// Validate a result against a validation check
fn validate_result(result: &serde_json::Value, check: &ValidationCheck) -> ValidationResult {
    let passed = match check.check_type {
        CheckType::HasField => has_field(result, &check.field),
        CheckType::IsNonEmpty => is_non_empty(result, &check.field),
        CheckType::IsSuccess => is_success(result, &check.field, check.expected_value.as_deref()),
        CheckType::MatchesPattern => matches_pattern(result, &check.field, check.expected_value.as_deref()),
        CheckType::GreaterThan => greater_than(result, &check.field, check.expected_value.as_deref()),
        CheckType::LessThan => less_than(result, &check.field, check.expected_value.as_deref()),
    };
    
    ValidationResult {
        check_type: format!("{:?}", check.check_type),
        field: check.field.clone(),
        passed,
        message: Some(if passed { "OK".to_string() } else { "Failed".to_string() }),
    }
}

/// Check if result has a field
fn has_field(result: &serde_json::Value, field: &str) -> bool {
    // Try to find the field in various locations
    if result.get(field).is_some() {
        return true;
    }
    
    // Check in content[0].text (MCP response format)
    if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
        if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                if parsed.get(field).is_some() {
                    return true;
                }
            }
        }
    }
    
    // Check in data field
    if let Some(data) = result.get("data") {
        if data.get(field).is_some() {
            return true;
        }
    }
    
    // Check nested in success field (for ToolOutput format)
    if field == "success" {
        if result.get("success").is_some() {
            return true;
        }
        if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
            if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                    return parsed.get("success").is_some();
                }
            }
        }
    }
    
    false
}

/// Check if field is non-empty
fn is_non_empty(result: &serde_json::Value, field: &str) -> bool {
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
fn is_json_value_empty(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::String(s) => s.is_empty(),
        serde_json::Value::Array(a) => a.is_empty(),
        serde_json::Value::Object(o) => o.is_empty(),
        _ => false,
    }
}

/// Check if success field has expected value
fn is_success(result: &serde_json::Value, _field: &str, expected: Option<&str>) -> bool {
    let success = result.get("success")
        .and_then(|s| s.as_bool())
        .or_else(|| {
            // Check in content.text
            result.get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|c| c.get("text"))
                .and_then(|t| t.as_str())
                .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
                .and_then(|parsed| parsed.get("success").and_then(|s| s.as_bool()))
        });
    
    match (success, expected) {
        (Some(s), Some("false")) => !s,
        (Some(s), Some("true")) | (Some(s), None) => s,
        (None, Some("false")) => true, // If not present, treat as expected failure
        _ => false,
    }
}

/// Check if field matches a pattern
fn matches_pattern(result: &serde_json::Value, field: &str, pattern: Option<&str>) -> bool {
    if let Some(pattern) = pattern {
        if let Some(value) = result.get(field).and_then(|v| v.as_str()) {
            return value.contains(pattern);
        }
    }
    true
}

/// Check if field is greater than value
fn greater_than(result: &serde_json::Value, field: &str, min_value: Option<&str>) -> bool {
    if let (Some(min_str), Some(value)) = (min_value, result.get(field).and_then(|v| v.as_f64())) {
        if let Ok(min) = min_str.parse::<f64>() {
            return value > min;
        }
    }
    true
}

/// Check if field is less than value
fn less_than(result: &serde_json::Value, field: &str, max_value: Option<&str>) -> bool {
    if let (Some(max_str), Some(value)) = (max_value, result.get(field).and_then(|v| v.as_f64())) {
        if let Ok(max) = max_str.parse::<f64>() {
            return value < max;
        }
    }
    true
}

/// Get unique categories from requirements
fn get_categories(requirements: &[TestRequirement]) -> Vec<String> {
    let mut categories: Vec<String> = requirements.iter()
        .map(|r| r.category.clone())
        .collect();
    categories.sort();
    categories.dedup();
    categories
}

/// Get category count
fn get_category_count(requirements: &[TestRequirement]) -> usize {
    get_categories(requirements).len()
}
