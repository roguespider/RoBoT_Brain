//! Comprehensive End-to-End Test Module
//!
//! This module provides true end-to-end testing for all MCP tools.
//! It validates actual functionality without stubs or mocking.

use std::collections::HashMap;
use std::time::Instant;

use crate::code_analyzer::{CodeAnalyzer, LintAnalyzer, LintSummary};
use crate::function_registry::FunctionRegistry;
use crate::paths;
use crate::test_environment::TestEnvironment;
use crate::test_results::print_issues_table;
use crate::test_results::{TestReport, TestStatus};
use crate::{TestMcpClient, TestStats};

pub mod argument_builder;
pub mod helpers;
pub mod protocol;
pub mod runner;
pub mod validation;

pub use helpers::{get_categories, get_category_count};
pub use protocol::test_mcp_basics;
pub use runner::run_single_test;

/// Run the comprehensive end-to-end test suite
pub async fn run_comprehensive_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) -> anyhow::Result<TestReport> {
    let start_time = Instant::now();
    let mut report = TestReport::new();

    crate::teeprintln!("\n{}", "#".repeat(100));
    crate::teeprintln!("#  ROBO T BRAIN - COMPREHENSIVE END-TO-END TEST SUITE");
    crate::teeprintln!("#  Testing every function 100% end-to-end without stubs or #[allow(*)]");
    crate::teeprintln!("{}", "#".repeat(100));

    // Step 0: Test MCP Protocol basics first
    crate::teeprintln!("\n[INFO] PHASE 0: MCP PROTOCOL VALIDATION");
    crate::teeprintln!("{}", "─".repeat(100));
    
    let mcp_ok = test_mcp_basics(client, stats).await;
    report.set_mcp_protocol_ok(mcp_ok);
    
    if !mcp_ok {
        crate::teeprintln!("\n[WARN] MCP Protocol Issue Detected!");
        crate::teeprintln!("    The MCP server must implement ServerHandler trait methods:");
        crate::teeprintln!("    - list_tools() - Returns ListToolsResult with available tools");
        crate::teeprintln!("    - call_tool() - Executes tools and returns CallToolResult");
        crate::teeprintln!("    - get_tool() - Returns Tool definition for a given name");
        crate::teeprintln!("\n    Tests will continue but most will fail until MCP protocol is fixed.");
    }

    // Step 1: Analyze source code for issues
    crate::teeprintln!("\n[INFO] PHASE 1: SOURCE CODE ANALYSIS");
    crate::teeprintln!("{}", "─".repeat(100));

    let source_path = paths::project_root().join("src");
    let analyzer = CodeAnalyzer::new(source_path.clone());
    let mut code_issues = analyzer.analyze();
    // Also enforce the no-emoji rule across test_suite/src/ (AGENTS.md:
    // "No emoji / plain-text markers"). Run emoji-only scan there — the full
    // analyzer is not run on test_suite/src because checks like cfg_test have
    // robot_brain-src-specific semantics that do not apply to the test suite.
    let test_suite_src = paths::project_root().join("test_suite").join("src");
    let emoji_issues = analyzer.analyze_emoji_in_dir(&test_suite_src);
    code_issues.extend(emoji_issues);
    let summary = analyzer.get_summary(&code_issues);

    summary.print_table();
    report.set_code_issues(code_issues.clone());
    // Use the project root as the display base so relative paths render
    // correctly for BOTH robot_brain src/ and test_suite/src/ files.
    report.set_source_path(paths::project_root());

    // Print issues table
    print_issues_table(&code_issues, &paths::project_root());

    // Step 1b: Run lint analysis (clippy + cargo check)
    crate::teeprintln!("\n[INFO] PHASE 1B: LINT ANALYSIS (clippy + cargo check)");
    crate::teeprintln!("{}", "─".repeat(100));

    let project_path = paths::project_root();

    crate::teeprintln!("  Running clippy...");
    let clippy_issues = match LintAnalyzer::run_clippy(&project_path) {
        Ok(issues) => issues,
        Err(e) => {
            crate::teeprintln!("    [WARN] Clippy failed: {}", e);
            Vec::new()
        }
    };

    crate::teeprintln!("  Running cargo check...");
    let check_issues = match LintAnalyzer::run_check(&project_path) {
        Ok(issues) => issues,
        Err(e) => {
            crate::teeprintln!("    [WARN] Cargo check failed: {}", e);
            Vec::new()
        }
    };

    // Combine and dedupe issues
    let mut all_lint_issues = clippy_issues;
    for issue in check_issues {
        if !all_lint_issues.iter().any(|i| {
            i.file_path == issue.file_path
                && i.line_number == issue.line_number
                && i.message == issue.message
        }) {
            all_lint_issues.push(issue);
        }
    }

    let lint_summary = LintSummary::new(all_lint_issues.clone());
    lint_summary.print_report();

    // Store lint issues in report
    report.set_lint_issues(all_lint_issues);

    // Step 2: Get all test requirements
    crate::teeprintln!("\n[INFO] PHASE 2: COLLECTING TEST REQUIREMENTS");
    crate::teeprintln!("{}", "─".repeat(100));

    let requirements = FunctionRegistry::get_all_functions();
    crate::teeprintln!(
        "  Found {} test requirements across {} categories",
        requirements.len(),
        get_category_count(&requirements)
    );

    for category in get_categories(&requirements) {
        let count = requirements
            .iter()
            .filter(|r| r.category == category)
            .count();
        crate::teeprintln!("    - {}: {} tests", category, count);
    }

    // Coverage cross-check: diff the tools the server actually exposes (via
    // tools/list) against the tools the FunctionRegistry exercises. Any server
    // tool with no matching test requirement is a coverage gap — a tool that
    // could break without the suite noticing.
    crate::teeprintln!("\n  [INFO] TOOL COVERAGE ANALYSIS");
    let server_tool_names: Vec<String> = match client.list_tools().await {
        Ok(tools) => tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
            .collect(),
        Err(e) => {
            crate::teeprintln!("    [WARN] Could not retrieve server tool list for coverage check: {}", e);
            Vec::new()
        }
    };
    let tested_tool_names: Vec<String> = requirements
        .iter()
        .map(|r| r.function_name.clone())
        .collect();
    let coverage = crate::test_results::CoverageReport::new(server_tool_names, tested_tool_names);
    crate::teeprintln!(
        "    Server exposes {} tools; registry tests {} (coverage {:.1}%)",
        coverage.server_tool_count(),
        coverage.tested_tool_count(),
        coverage.coverage_percent()
    );
    if coverage.has_gap() {
        crate::teeprintln!(
            "    [WARN] {} server tool(s) have NO test: {}",
            coverage.untested_count(),
            coverage.untested_tools.join(", ")
        );
    }
    if !coverage.phantom_tools.is_empty() {
        crate::teeprintln!(
            "    [INFO] {} registry tool(s) not exposed by server: {}",
            coverage.phantom_count(),
            coverage.phantom_tools.join(", ")
        );
    }
    if !coverage.has_gap() && coverage.phantom_tools.is_empty() {
        crate::teeprintln!("    [OK] Tool coverage is complete — every server tool is tested");
    }
    report.set_coverage(coverage);

    // Step 3: Run all tests
    crate::teeprintln!("\n🧪 PHASE 3: RUNNING END-TO-END TESTS");
    crate::teeprintln!("{}", "─".repeat(100));

    // First, ensure workflow is initialized (required for most tests)
    crate::teeprintln!("\n  Initializing workflow...");
    match client
        .call_tool(
            "get_workflow",
            serde_json::json!({
                "purpose": "default"
            }),
        )
        .await
    {
        Ok(_) => crate::teeprintln!("    [OK] Workflow initialized"),
        Err(e) => crate::teeprintln!("    [WARN] Workflow init warning: {}", e),
    }

    // Print test table header
    crate::teeprintln!(
        "\n  ┌{:─<5}┬{:─<20}┬{:─<30}┬{:─<8}┬{:─<50}┐",
        "",
        "",
        "",
        "",
        ""
    );
    crate::teeprintln!(
        "  │ {:^3} │ {:^18} │ {:^28} │ {:^6} │ {:^48} │",
        "#",
        "Category",
        "Test Name",
        "Status",
        "Details"
    );
    crate::teeprintln!(
        "  ├{:─<5}┼{:─<20}┼{:─<30}┼{:─<8}┼{:─<50}┤",
        "",
        "",
        "",
        "",
        ""
    );

    // CRITICAL: Initialize workflow enforcement by calling get_workflow first
    // This is REQUIRED before any non-exempt tool can be called
    crate::teeprintln!("\n🔒 WORKFLOW ENFORCEMENT: Initializing...");
    match client.call_tool("get_workflow", serde_json::json!({"purpose": "general"})).await {
        Ok(_result) => {
            crate::teeprintln!("  [OK] Workflow retrieved - enforcement active");
        }
        Err(e) => {
            crate::teeprintln!("  [WARN] Failed to retrieve workflow: {}", e);
        }
    }

    // Also call search_memory to satisfy memory search requirement
    crate::teeprintln!("  🔍 Checking memory...");
    match client.call_tool("search_memory", serde_json::json!({"query": "test"})).await {
        Ok(_) => crate::teeprintln!("    [OK] Memory search responded"),
        Err(e) => crate::teeprintln!("    [WARN] Memory search failed: {}", e),
    }
    
    crate::teeprintln!("  [OK] Workflow enforcement satisfied - running tests...\n");

    // Run tests for each category
    let mut data_created: HashMap<String, Vec<String>> = HashMap::new();
    let mut test_num = 0;

    for requirement in &requirements {
        test_num += 1;
        let result = run_single_test(client, requirement, &data_created, env).await;

        // Capture status before moving result into tracking/report
        let status_icon = helpers::get_status_icon(&result.status);
        let is_pass = result.status == TestStatus::Pass;

        // Track created data for dependent tests
        let updated_data = runner::track_data_creation(
            result.status.clone(),
            requirement,
            data_created.clone(),
        );
        data_created = updated_data;

        // Print test result in table format
        let details = if is_pass {
            "OK".to_string()
        } else {
            result
                .error_message
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string())
        };

        let table_row = helpers::format_test_result(
            test_num,
            requirement,
            status_icon,
            &details,
        );
        crate::teeprintln!("{}", table_row);

        report.add_result(result);

        // Update stats
        if let Some(r) = report.results.last() {
            match r.status {
                TestStatus::Pass => stats.passed += 1,
                TestStatus::Fail | TestStatus::Error => stats.failed += 1,
                TestStatus::Skipped => stats.skipped += 1,
                TestStatus::Blocked => stats.skipped += 1,
            }
        }
    }

    crate::teeprintln!(
        "  └{:─<5}┴{:─<20}┴{:─<30}┴{:─<8}┴{:─<50}┘",
        "",
        "",
        "",
        "",
        ""
    );

    // Step 4: Generate report
    crate::teeprintln!("\n[INFO] PHASE 4: GENERATING REPORT");
    crate::teeprintln!("{}", "─".repeat(100));

    report.print_report();

    crate::teeprintln!("\n  Total test duration: {:?}", start_time.elapsed());

    Ok(report)
}
