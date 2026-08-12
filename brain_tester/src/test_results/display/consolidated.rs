//! Consolidated issues view.
//!
//! Renders every kind of problem (failing tests, untested tools, compiler
//! warnings, code-quality issues) in a single grouped table so diagnosis
//! doesn't require hunting through separate sections of a long report.

use super::super::{TestReport};

impl TestReport {
    /// Print a single consolidated table of all issues, grouped by kind.
    pub fn print_consolidated_issues(&self) {
        let issues = self.consolidated_issues();
        if issues.is_empty() {
            return;
        }

        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!(
            "│ {:^96} │",
            "📋 CONSOLIDATED ISSUES (all problems in one place)"
        );
        crate::teeprintln!("├{:─<98}┤", "");
        crate::teeprintln!(
            "│  Total issues: {}",
            issues.len()
        );
        crate::teeprintln!("├{:─<10}┼{:─<10}┼{:─<14}┼{:─<22}┼{:─<34}┤",
            "─", "─", "─", "─", "─");
        crate::teeprintln!(
            "│ {:^8} │ {:^8} │ {:^12} │ {:^20} │ {:^32} │",
            "Kind", "Sev", "Category", "Tool / File", "Message"
        );
        crate::teeprintln!("├{:─<10}┼{:─<10}┼{:─<14}┼{:─<22}┼{:─<34}┤",
            "─", "─", "─", "─", "─");

        for issue in &issues {
            let kind = issue_kind_label(&issue.kind);
            let sev = severity_label(&issue.severity);
            let category = crate::test_results::truncate(&issue.category, 12);
            let location = issue_location(issue);
            let message = crate::test_results::truncate(&issue.message, 32);
            crate::teeprintln!(
                "│ {:<8} │ {:<8} │ {:<12} │ {:<20} │ {:<32} │",
                kind, sev, category, location, message
            );
        }
        crate::teeprintln!("└{:─<10}┴{:─<10}┴{:─<14}┴{:─<22}┴{:─<34}┘",
            "─", "─", "─", "─", "─");

        // Detailed suggested actions
        crate::teeprintln!("\n  Suggested actions:");
        for (i, issue) in issues.iter().enumerate() {
            let loc = issue_location(issue);
            crate::teeprintln!(
                "    {}. [{}] {} — {}",
                i + 1,
                issue_kind_label(&issue.kind),
                loc,
                issue.suggested_action
            );
            if !issue.server_logs.is_empty() {
                crate::teeprintln!("       server logs:");
                for line in &issue.server_logs {
                    crate::teeprintln!("         {}", crate::test_results::truncate(line, 88));
                }
            }
        }
    }
}

fn issue_kind_label(kind: &crate::test_results::json_report::IssueKind) -> &'static str {
    use crate::test_results::json_report::IssueKind;
    match kind {
        IssueKind::FailingTest => "FAILTEST",
        IssueKind::ErrorTest => "ERRTEST",
        IssueKind::UntestedTool => "UNTESTED",
        IssueKind::PhantomTool => "PHANTOM",
        IssueKind::CompilerWarning => "CWARN",
        IssueKind::CompilerError => "CERR",
        IssueKind::CodeQualityIssue => "CODEQ",
    }
}

fn severity_label(sev: &crate::test_results::json_report::Severity) -> &'static str {
    use crate::test_results::json_report::Severity;
    match sev {
        Severity::Critical => "CRIT",
        Severity::High => "HIGH",
        Severity::Medium => "MED",
        Severity::Low => "LOW",
    }
}

fn issue_location(issue: &crate::test_results::json_report::IssueEntry) -> String {
    if let Some(tool) = &issue.tool {
        if let Some(file) = &issue.file {
            if let Some(line) = issue.line {
                format!("{} ({}:{})", tool, file, line)
            } else {
                format!("{} ({})", tool, file)
            }
        } else {
            tool.clone()
        }
    } else if let Some(file) = &issue.file {
        if let Some(line) = issue.line {
            format!("{}:{}", file, line)
        } else {
            file.clone()
        }
    } else {
        "-".to_string()
    }
}
