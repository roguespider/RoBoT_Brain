//! Machine-readable JSON report output.
//!
//! Serializes the full `TestReport` to `test_suite_report.json` alongside the
//! text report. JSON enables: run-to-run diffing, CI gating, and tooling to
//! filter/group results (e.g. "show only newly failing tests", "new warnings
//! since last run").

use serde::Serialize;

use super::{CoverageReport, TestReport, TestStatus};

/// A single consolidated issue entry for the JSON "issues" array.
///
/// Groups every kind of problem (failing test, untested tool, lint warning,
/// code-quality issue, server-log warning) into one uniform structure so
/// downstream tooling can sort/filter uniformly.
#[derive(Debug, Serialize)]
pub struct IssueEntry {
    pub kind: IssueKind,
    pub severity: Severity,
    pub category: String,
    pub tool: Option<String>,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub message: String,
    pub suggested_action: String,
    /// Server log context (for failing tests).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub server_logs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueKind {
    FailingTest,
    ErrorTest,
    UntestedTool,
    PhantomTool,
    CompilerWarning,
    CompilerError,
    CodeQualityIssue,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

/// Top-level JSON report structure.
#[derive(Debug, Serialize)]
pub struct JsonReport {
    pub summary: JsonSummary,
    pub coverage: CoverageReport,
    pub issues: Vec<IssueEntry>,
    pub results: Vec<super::TestResult>,
    pub lint_issues: Vec<crate::code_analyzer::LintIssue>,
    pub code_issues: Vec<crate::code_analyzer::CodeIssue>,
}

#[derive(Debug, Serialize)]
pub struct JsonSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub skipped: usize,
    pub blocked: usize,
    pub pass_rate_percent: f64,
    pub code_issues: usize,
    pub compiler_errors: usize,
    pub compiler_warnings: usize,
    pub tool_coverage_percent: f64,
    pub untested_tools: usize,
    pub mcp_protocol_ok: bool,
    pub duration_ms: u64,
    pub overall_success: bool,
}

impl TestReport {
    /// Build the consolidated issue list from all problem sources.
    pub fn consolidated_issues(&self) -> Vec<IssueEntry> {
        let mut issues: Vec<IssueEntry> = Vec::new();

        // Failing tests
        for r in self.results.iter().filter(|r| r.status == TestStatus::Fail) {
            issues.push(IssueEntry {
                kind: IssueKind::FailingTest,
                severity: Severity::Critical,
                category: r.requirement.category.clone(),
                tool: Some(r.requirement.function_name.clone()),
                file: None,
                line: None,
                message: r.error_message.clone().unwrap_or_else(|| "validation failed".to_string()),
                suggested_action: format!(
                    "Fix {} so it satisfies its validation checks",
                    r.requirement.function_name
                ),
                server_logs: r.server_logs.clone(),
            });
        }

        // Error tests
        for r in self.results.iter().filter(|r| r.status == TestStatus::Error) {
            issues.push(IssueEntry {
                kind: IssueKind::ErrorTest,
                severity: Severity::Critical,
                category: r.requirement.category.clone(),
                tool: Some(r.requirement.function_name.clone()),
                file: None,
                line: None,
                message: r.error_message.clone().unwrap_or_else(|| "MCP error".to_string()),
                suggested_action: format!(
                    "Investigate MCP protocol error for {} (check server logs)",
                    r.requirement.function_name
                ),
                server_logs: r.server_logs.clone(),
            });
        }

        // Untested server tools
        for tool in &self.coverage.untested_tools {
            issues.push(IssueEntry {
                kind: IssueKind::UntestedTool,
                severity: Severity::High,
                category: "Coverage".to_string(),
                tool: Some(tool.clone()),
                file: Some("test_suite/src/function_registry/".to_string()),
                line: None,
                message: format!("Server exposes '{}' but no test requirement exercises it", tool),
                suggested_action: format!(
                    "Add a TestRequirement for '{}' in the function registry",
                    tool
                ),
                server_logs: Vec::new(),
            });
        }

        // Phantom tools (tested but not exposed by server)
        for tool in &self.coverage.phantom_tools {
            issues.push(IssueEntry {
                kind: IssueKind::PhantomTool,
                severity: Severity::Low,
                category: "Coverage".to_string(),
                tool: Some(tool.clone()),
                file: Some("test_suite/src/function_registry/".to_string()),
                line: None,
                message: format!(
                    "Registry tests '{}' but the server does not expose it",
                    tool
                ),
                suggested_action: format!(
                    "Remove the stale test for '{}' or register the tool in robot_brain",
                    tool
                ),
                server_logs: Vec::new(),
            });
        }

        // Compiler errors
        for lint in self.lint_issues.iter().filter(|l| l.level == crate::code_analyzer::LintLevel::Error) {
            issues.push(IssueEntry {
                kind: IssueKind::CompilerError,
                severity: Severity::Critical,
                category: "Compiler".to_string(),
                tool: None,
                file: Some(lint.file_path.clone()),
                line: Some(lint.line_number),
                message: lint.message.clone(),
                suggested_action: "Fix the compiler error".to_string(),
                server_logs: Vec::new(),
            });
        }

        // Compiler warnings
        for lint in self.lint_issues.iter().filter(|l| l.level == crate::code_analyzer::LintLevel::Warning) {
            issues.push(IssueEntry {
                kind: IssueKind::CompilerWarning,
                severity: Severity::Medium,
                category: "Compiler".to_string(),
                tool: None,
                file: Some(lint.file_path.clone()),
                line: Some(lint.line_number),
                message: lint.message.clone(),
                suggested_action: "Resolve the dead code or wire it up per the Dead Code Resolution Protocol".to_string(),
                server_logs: Vec::new(),
            });
        }

        // Code-quality issues
        for issue in &self.code_issues {
            let file = self
                .source_path
                .as_ref()
                .map(|base| issue.relative_path(base))
                .unwrap_or_else(|| issue.file_path.to_string_lossy().to_string());
            issues.push(IssueEntry {
                kind: IssueKind::CodeQualityIssue,
                severity: Severity::High,
                category: "Code Quality".to_string(),
                tool: None,
                file: Some(file),
                line: Some(issue.line_number),
                message: issue.description.clone(),
                suggested_action: format!(
                    "Resolve {} per the AGENTS.md coding standards",
                    issue.issue_type
                ),
                server_logs: Vec::new(),
            });
        }

        issues
    }

    /// Serialize the full report to a JSON string (pretty-printed).
    pub fn to_json_pretty(&self) -> anyhow::Result<String> {
        let total = self.results.len();
        let summary = JsonSummary {
            total_tests: total,
            passed: self.passed_count(),
            failed: self.failed_count(),
            errors: self.error_count(),
            skipped: self.skipped_count(),
            blocked: self.blocked_count(),
            pass_rate_percent: if total > 0 {
                (self.passed_count() as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            code_issues: self.code_issues.len(),
            compiler_errors: self.lint_errors,
            compiler_warnings: self.lint_warnings,
            tool_coverage_percent: self.coverage.coverage_percent(),
            untested_tools: self.coverage.untested_count(),
            mcp_protocol_ok: self.mcp_protocol_ok,
            duration_ms: self.total_duration_ms,
            overall_success: !self.has_issues(),
        };

        let report = JsonReport {
            summary,
            coverage: self.coverage.clone(),
            issues: self.consolidated_issues(),
            results: self.results.clone(),
            lint_issues: self.lint_issues.clone(),
            code_issues: self.code_issues.clone(),
        };

        Ok(serde_json::to_string_pretty(&report)?)
    }

    /// Write the JSON report to `path`.
    pub fn write_json(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let json = self.to_json_pretty()?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
