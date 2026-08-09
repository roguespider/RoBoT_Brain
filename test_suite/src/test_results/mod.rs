//! Test Results Module
//!
//! Provides comprehensive table-based reporting for test results.
//! Shows pass/fail status for each function with detailed information.

pub mod display;
pub mod json_report;

use crate::code_analyzer::{CodeIssue, LintIssue, LintLevel};
use crate::function_registry::TestRequirement;

use serde::Serialize;
use std::path::PathBuf;

/// Represents the result of a single test
#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub requirement: TestRequirement,
    pub status: TestStatus,
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub validation_results: Vec<ValidationResult>,
    /// Recent server-side log lines captured around the time this test ran.
    /// Populated for non-passing results to aid diagnosis; empty for passes.
    pub server_logs: Vec<String>,
}

/// Status of a test
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TestStatus {
    Pass,
    Fail,
    Error,
    Skipped,
    Blocked,
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestStatus::Pass => write!(f, "PASS"),
            TestStatus::Fail => write!(f, "FAIL"),
            TestStatus::Error => write!(f, "ERROR"),
            TestStatus::Skipped => write!(f, "SKIP"),
            TestStatus::Blocked => write!(f, "BLOCK"),
        }
    }
}

/// Result of a validation check
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub field: String,
    pub passed: bool,
    pub message: Option<String>,
}

/// Tool coverage analysis: compares the tools the server actually exposes
/// (discovered via `tools/list`) against the tools the `FunctionRegistry`
/// exercises. Any server tool without a corresponding test requirement is a
/// coverage gap — a tool that could be broken without the suite noticing.
#[derive(Debug, Default, Clone, Serialize)]
pub struct CoverageReport {
    /// Tool names the server exposes (from `tools/list`), sorted.
    pub server_tools: Vec<String>,
    /// Tool names the test registry exercises, sorted.
    pub tested_tools: Vec<String>,
    /// Server tools with no matching test requirement (the gap).
    pub untested_tools: Vec<String>,
    /// Test-registry tool names the server does NOT expose (stale/phantom tests).
    pub phantom_tools: Vec<String>,
}

impl CoverageReport {
    /// Build a coverage report by diffing server tools against tested tools.
    pub fn new(server_tools: Vec<String>, tested_tools: Vec<String>) -> Self {
        let mut server_sorted = server_tools.clone();
        server_sorted.sort();
        let mut tested_sorted = tested_tools.clone();
        tested_sorted.sort();

        use std::collections::HashSet;
        let server_set: HashSet<&str> =
            server_sorted.iter().map(|s| s.as_str()).collect();
        let tested_set: HashSet<&str> =
            tested_sorted.iter().map(|s| s.as_str()).collect();

        let mut untested_tools: Vec<String> =
            server_set.difference(&tested_set).map(|s| s.to_string()).collect();
        untested_tools.sort();
        let mut phantom_tools: Vec<String> =
            tested_set.difference(&server_set).map(|s| s.to_string()).collect();
        phantom_tools.sort();

        Self {
            server_tools: server_sorted,
            tested_tools: tested_sorted,
            untested_tools,
            phantom_tools,
        }
    }

    /// Total tools the server exposes.
    pub fn server_tool_count(&self) -> usize {
        self.server_tools.len()
    }

    /// Tools the registry exercises.
    pub fn tested_tool_count(&self) -> usize {
        self.tested_tools.len()
    }

    /// Number of server tools with no test coverage.
    pub fn untested_count(&self) -> usize {
        self.untested_tools.len()
    }

    /// Number of registry entries that don't match a real server tool.
    pub fn phantom_count(&self) -> usize {
        self.phantom_tools.len()
    }

    /// Coverage percentage of server tools that ARE tested.
    pub fn coverage_percent(&self) -> f64 {
        if self.server_tools.is_empty() {
            0.0
        } else {
            let tested = self.server_tool_count() - self.untested_count();
            (tested as f64 / self.server_tool_count() as f64) * 100.0
        }
    }

    /// True if any server tool lacks a test.
    pub fn has_gap(&self) -> bool {
        !self.untested_tools.is_empty()
    }
}

/// Comprehensive test report
#[derive(Debug, Default, Serialize)]
pub struct TestReport {
    pub results: Vec<TestResult>,
    pub code_issues: Vec<CodeIssue>,
    pub total_duration_ms: u64,
    pub lint_errors: usize,
    pub lint_warnings: usize,
    pub lint_issues: Vec<LintIssue>,
    pub source_path: Option<PathBuf>,
    pub mcp_protocol_ok: bool,
    /// Tool coverage gap: tools the server exposes (via `tools/list`) that are
    /// NOT exercised by any `FunctionRegistry` test requirement.
    pub coverage: CoverageReport,
}

impl TestReport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a test result
    pub fn add_result(&mut self, result: TestResult) {
        self.total_duration_ms += result.duration_ms;
        self.results.push(result);
    }

    /// Set code issues
    pub fn set_code_issues(&mut self, issues: Vec<CodeIssue>) {
        self.code_issues = issues;
    }

    /// Set source path for relative path display
    pub fn set_source_path(&mut self, path: PathBuf) {
        self.source_path = Some(path);
    }
    
    /// Set MCP protocol status
    pub fn set_mcp_protocol_ok(&mut self, ok: bool) {
        self.mcp_protocol_ok = ok;
    }

    /// Set the tool coverage analysis (server tools vs tested tools).
    pub fn set_coverage(&mut self, coverage: CoverageReport) {
        self.coverage = coverage;
    }

    /// Set lint issues (compiler errors and warnings)
    pub fn set_lint_issues(&mut self, issues: Vec<LintIssue>) {
        self.lint_errors = issues
            .iter()
            .filter(|i| i.level == LintLevel::Error)
            .count();
        self.lint_warnings = issues
            .iter()
            .filter(|i| i.level == LintLevel::Warning)
            .count();
        self.lint_issues = issues;
    }

    /// Get pass count
    pub fn passed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Pass)
            .count()
    }

    /// Get fail count
    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Fail)
            .count()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Error)
            .count()
    }

    /// Get skipped count
    pub fn skipped_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Skipped)
            .count()
    }

    /// Get blocked count
    pub fn blocked_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Blocked)
            .count()
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.status == TestStatus::Pass)
    }

    /// Check if there are any issues (failed tests, code issues, coverage gaps, etc.)
    pub fn has_issues(&self) -> bool {
        !self.failed_results().is_empty()
            || !self.error_results().is_empty()
            || !self.code_issues.is_empty()
            || self.lint_errors > 0
            || self.lint_warnings > 0
            || self.coverage.has_gap()
            || !self.coverage.phantom_tools.is_empty()
    }

    /// Get all failed results
    pub fn failed_results(&self) -> Vec<&TestResult> {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Fail)
            .collect()
    }

    /// Get all error results
    pub fn error_results(&self) -> Vec<&TestResult> {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Error)
            .collect()
    }
}

/// Truncate a string to a maximum length
pub(crate) fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Print issues table (standalone function for early reporting)
pub fn print_issues_table(issues: &[CodeIssue], source_path: &std::path::Path) {
    if issues.is_empty() {
        crate::teeprintln!("\n  ✅ No code quality issues detected in source!");
        return;
    }

    crate::teeprintln!("\n┌{:─<98}┐", "");
    crate::teeprintln!("│ {:^96} │", "⚠️  CODE QUALITY ISSUES TABLE");
    crate::teeprintln!("├{:─<10}├{:─<40}├{:─<10}├{:─<35}┤", "─", "─", "─", "─");
    crate::teeprintln!(
        "│ {:^8} │ {:^38} │ {:^8} │ {:^33} │",
        "Line",
        "File",
        "Type",
        "Description"
    );
    crate::teeprintln!("├{:─<10}┼{:─<40}┼{:─<10}┼{:─<35}┤", "─", "─", "─", "─");
    for issue in issues {
        let file_name = issue.relative_path(source_path);
        let issue_type = issue.issue_type.to_string();
        let description = truncate(&issue.description, 33);

        crate::teeprintln!(
            "│ {:>8} │ {:<40} │ {:<10} │ {:<35} │",
            issue.line_number,
            truncate(&file_name, 40),
            truncate(&issue_type, 10),
            description
        );
    }

    crate::teeprintln!("└{:─<10}┴{:─<40}┴{:─<10}┴{:─<35}┘", "─", "─", "─", "─");
}
