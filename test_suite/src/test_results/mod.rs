//! Test Results Module
//!
//! Provides comprehensive table-based reporting for test results.
//! Shows pass/fail status for each function with detailed information.

pub mod display;

use crate::code_analyzer::{CodeIssue, LintIssue, LintLevel};
use crate::function_registry::TestRequirement;

use std::path::PathBuf;

/// Represents the result of a single test
#[derive(Debug, Clone)]
pub struct TestResult {
    pub requirement: TestRequirement,
    pub status: TestStatus,
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub validation_results: Vec<ValidationResult>,
}

/// Status of a test
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub field: String,
    pub passed: bool,
    pub message: Option<String>,
}

/// Comprehensive test report
#[derive(Debug, Default)]
pub struct TestReport {
    pub results: Vec<TestResult>,
    pub code_issues: Vec<CodeIssue>,
    pub total_duration_ms: u64,
    pub lint_errors: usize,
    pub lint_warnings: usize,
    pub lint_issues: Vec<LintIssue>,
    pub source_path: Option<PathBuf>,
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

    /// Check if there are any issues (failed tests, code issues, etc.)
    pub fn has_issues(&self) -> bool {
        !self.failed_results().is_empty()
            || !self.error_results().is_empty()
            || !self.code_issues.is_empty()
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
