//! Table formatting for full results and lint issues.
//! Contains `print_full_table` and `print_lint_issues` methods.

use std::path::Path;

use crate::code_analyzer::{LintIssue, LintLevel};

use super::super::{truncate, TestReport, TestStatus};

/// Make `file_path` relative to `base` for display. Falls back to the
/// original path if stripping the prefix fails.
fn relative_path(base: &Path, file_path: &str) -> String {
    std::path::Path::new(file_path)
        .strip_prefix(base)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| file_path.to_string())
}

impl TestReport {
    /// Print full test table
    pub fn print_full_table(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "[INFO] FULL TEST RESULTS TABLE");
        crate::teeprintln!(
            "├{:─<6}├{:─<20}├{:─<25}├{:─<8}├{:─<10}├{:─<25}┤",
            "─",
            "─",
            "─",
            "─",
            "─",
            "─"
        );

        crate::teeprintln!(
            "│ {:^4} │ {:^18} │ {:^23} │ {:^6} │ {:^8} │ {:^23} │",
            "#",
            "Category",
            "Function",
            "Status",
            "Priority",
            "Result"
        );
        crate::teeprintln!(
            "├{:─<6}┼{:─<20}┼{:─<25}┼{:─<8}┼{:─<10}┼{:─<25}┤",
            "─",
            "─",
            "─",
            "─",
            "─",
            "─"
        );

        let mut categories: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for (idx, result) in self.results.iter().enumerate() {
            let cat_count = categories
                .entry(result.requirement.category.clone())
                .or_insert(0);
            *cat_count += 1;

            let status_icon = match result.status {
                TestStatus::Pass => "[OK]",
                TestStatus::Fail => "[FAIL]",
                TestStatus::Error => "[FAIL]",
                TestStatus::Skipped => "[SKIP]",
                TestStatus::Blocked => "[BLOCKED]",
            };

            let status_text = result.status.to_string();

            let priority_text = match result.requirement.priority {
                1 => "CRITICAL",
                2 => "HIGH",
                3 => "MEDIUM",
                _ => "LOW",
            };

            let result_text = if let Some(err) = &result.error_message {
                if err.len() > 23 {
                    format!("{}...", &err[..20])
                } else {
                    err.clone()
                }
            } else {
                "OK".to_string()
            };

            crate::teeprintln!(
                "│ {:>3} │ {:<20} │ {:<25} │ {} {:<4} │ {:<10} │ {:<25} │",
                idx + 1,
                truncate(&result.requirement.category, 20),
                truncate(&result.requirement.function_name, 25),
                status_icon,
                status_text,
                priority_text,
                truncate(&result_text, 25)
            );
        }

        crate::teeprintln!(
            "└{:─<6}┴{:─<20}┴{:─<25}┴{:─<8}┴{:─<10}┴{:─<25}┘",
            "─",
            "─",
            "─",
            "─",
            "─",
            "─"
        );

        // Print category summary
        crate::teeprintln!("\n  Category Summary:");
        for (cat, count) in categories {
            let passed = self
                .results
                .iter()
                .filter(|r| r.requirement.category == cat && r.status == TestStatus::Pass)
                .count();
            crate::teeprintln!("    {:<20} {}/{} passed", cat, passed, count);
        }
    }

    /// Print compiler errors and warnings.
    ///
    /// Each warning/error is printed as a full multi-line entry grouped under
    /// its file, showing the complete message and line number — no truncation,
    /// no dot-padding, no count-only summaries.
    pub fn print_lint_issues(&self) {
        let error_warnings: Vec<&LintIssue> = self
            .lint_issues
            .iter()
            .filter(|i| i.level == LintLevel::Error || i.level == LintLevel::Warning)
            .collect();

        if error_warnings.is_empty() {
            crate::teeprintln!("\n┌{:─<98}┐", "");
            crate::teeprintln!("│ {:^96} │", "[INFO]  COMPILER ERRORS & WARNINGS");
            crate::teeprintln!("├{:─<98}┤", "");
            crate::teeprintln!("│");
            crate::teeprintln!("│  [OK] No compiler errors or warnings!");
            crate::teeprintln!("│");
            crate::teeprintln!("└{:─<98}┘", "");
            return;
        }

        let error_count = error_warnings
            .iter()
            .filter(|i| i.level == LintLevel::Error)
            .count();
        let warning_count = error_warnings
            .iter()
            .filter(|i| i.level == LintLevel::Warning)
            .count();

        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "[INFO]  COMPILER ERRORS & WARNINGS");
        crate::teeprintln!("├{:─<98}┤", "");
        crate::teeprintln!(
            "│  {} error(s), {} warning(s):",
            error_count,
            warning_count
        );
        crate::teeprintln!("│");

        // Group by file so all warnings for a file are together
        let mut by_file: std::collections::BTreeMap<String, Vec<&LintIssue>> =
            std::collections::BTreeMap::new();
        for issue in &error_warnings {
            let file_name = self
                .source_path
                .as_ref()
                .map(|base| relative_path(base, &issue.file_path))
                .unwrap_or_else(|| issue.file_path.clone());
            by_file.entry(file_name).or_default().push(issue);
        }

        for (file, issues) in &by_file {
            crate::teeprintln!("│  [INFO] {}", file);
            for issue in issues {
                let level_str = match issue.level {
                    LintLevel::Error => "[FAIL] ERROR",
                    LintLevel::Warning => "[WARN]  WARN ",
                    _ => continue,
                };
                crate::teeprintln!("│    {} line {} [{}]: {}", level_str, issue.line_number, issue.code, issue.message);
            }
            crate::teeprintln!("│");
        }

        crate::teeprintln!("└{:─<98}┘", "");
    }
}
