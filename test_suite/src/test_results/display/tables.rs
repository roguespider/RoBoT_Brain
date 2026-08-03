//! Table formatting for full results and lint issues.
//! Contains `print_full_table` and `print_lint_issues` methods.

use crate::code_analyzer::{LintIssue, LintLevel};

use super::super::{truncate, TestReport, TestStatus};

impl TestReport {
    /// Print full test table
    pub fn print_full_table(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "📋 FULL TEST RESULTS TABLE");
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
                TestStatus::Pass => "✅",
                TestStatus::Fail => "❌",
                TestStatus::Error => "💥",
                TestStatus::Skipped => "⏭️",
                TestStatus::Blocked => "🚫",
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

    /// Print compiler errors and warnings table
    pub fn print_lint_issues(&self) {
        // Filter for errors and warnings only
        let error_warnings: Vec<&LintIssue> = self
            .lint_issues
            .iter()
            .filter(|i| i.level == LintLevel::Error || i.level == LintLevel::Warning)
            .collect();

        if error_warnings.is_empty() {
            crate::teeprintln!("\n┌{:─<98}┐", "");
            crate::teeprintln!("│ {:^96} │", "🔧  COMPILER ERRORS & WARNINGS");
            crate::teeprintln!("├{:─<98}┤", "");
            crate::teeprintln!("│");
            crate::teeprintln!("│  ✅ No compiler errors or warnings!");
            crate::teeprintln!("│");
            crate::teeprintln!("└{:─<98}┘", "");
            return;
        }

        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "🔧  COMPILER ERRORS & WARNINGS");
        crate::teeprintln!("├{:─<98}┤", "");
        crate::teeprintln!("│");
        crate::teeprintln!("│  Compiler errors and warnings from cargo check / clippy:");
        crate::teeprintln!("│{:─<97}│", "");

        // Table header
        crate::teeprintln!("│");
        crate::teeprintln!("├{:─<8}├{:─<6}├{:─<12}├{:─<68}┤", "─", "─", "─", "─");
        crate::teeprintln!(
            "│ {:^6} │ {:^4} │ {:^10} │ {:^66} │",
            "Level",
            "Line",
            "Code",
            "Message"
        );
        crate::teeprintln!("├{:─<8}┼{:─<6}┼{:─<12}┼{:─<68}┤", "─", "─", "─", "─");

        // Print each error/warning
        for issue in &error_warnings {
            let level_str = match issue.level {
                LintLevel::Error => "ERROR",
                LintLevel::Warning => "WARN",
                _ => continue,
            };

            let level_icon = match issue.level {
                LintLevel::Error => "❌",
                LintLevel::Warning => "⚠️",
                _ => " ",
            };

            let msg_truncated = truncate(&issue.message, 66);

            crate::teeprintln!(
                "│ {} {:^4} │ {:>4} │ {:^10} │ {:.<66} │",
                level_icon,
                level_str,
                issue.line_number,
                truncate(&issue.code, 10),
                msg_truncated
            );
        }

        crate::teeprintln!("├{:─<8}┴{:─<6}┴{:─<12}┴{:─<68}┤", "─", "─", "─", "─");

        // Summary by file
        crate::teeprintln!("│");
        crate::teeprintln!("│  Summary by file:");
        let mut by_file: std::collections::HashMap<String, (usize, usize)> =
            std::collections::HashMap::new();
        for issue in &error_warnings {
            let file_name = std::path::Path::new(&issue.file_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| issue.file_path.clone());

            let entry = by_file.entry(file_name).or_insert((0, 0));
            match issue.level {
                LintLevel::Error => entry.0 += 1,
                LintLevel::Warning => entry.1 += 1,
                _ => {}
            }
        }

        for (file, (errs, warns)) in by_file.iter().take(5) {
            let err_icon = if *errs > 0 { "❌" } else { "" };
            let warn_icon = if *warns > 0 { "⚠️" } else { "" };
            crate::teeprintln!(
                "│    {} {}{} - {} errors, {} warnings",
                file,
                err_icon,
                warn_icon,
                errs,
                warns
            );
        }

        crate::teeprintln!("│{:─<97}│", "");
        crate::teeprintln!("└{:─<97}┘", "");
    }
}
