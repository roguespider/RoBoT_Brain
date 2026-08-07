//! Lint analyzer for running clippy and cargo check

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use super::types::{LintIssue, LintLevel};

/// Analyzer for running external linters (clippy, cargo check)
pub struct LintAnalyzer;

impl LintAnalyzer {
    /// Run clippy linter
    pub fn run_clippy(project_path: &Path) -> anyhow::Result<Vec<LintIssue>> {
        let output = Command::new("cargo")
            .args(["clippy", "--message-format=short"])
            .current_dir(project_path)
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}\n{}", stdout, stderr);

        Ok(Self::parse_lint_output(&combined))
    }

    /// Run cargo check
    pub fn run_check(project_path: &Path) -> anyhow::Result<Vec<LintIssue>> {
        let output = Command::new("cargo")
            .args(["check", "--message-format=short"])
            .current_dir(project_path)
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}\n{}", stdout, stderr);

        Ok(Self::parse_lint_output(&combined))
    }

    /// Parse lint output into structured issues
    pub fn parse_lint_output(output: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Pattern for rustc/clipp output: file:line:col: level: code (message)
        let re = match regex::Regex::new(
            r"^(.+?):(\d+):(\d+):\s*((?:error|warning|help|note)+(?:\[\w+\])?):\s*((?:\w+)+)\s*(.*)$"
        ) {
            Ok(r) => r,
            Err(_) => {
                // If pattern is invalid (should never happen), return empty results
                return issues;
            }
        };

        for line in output.lines() {
            // Try main pattern first
            if let Some(caps) = re.captures(line) {
                let file = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                let line_num: usize = caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                let col: usize = caps
                    .get(3)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                let level_str = caps.get(4).map(|m| m.as_str()).unwrap_or("warning");
                let code = caps.get(5).map(|m| m.as_str()).unwrap_or("").to_string();
                let message = caps
                    .get(6)
                    .map(|m| m.as_str().trim())
                    .unwrap_or("")
                    .to_string();

                let level = match level_str.trim() {
                    s if s.contains("error") => LintLevel::Error,
                    s if s.contains("warning") => LintLevel::Warning,
                    s if s.contains("help") => LintLevel::Help,
                    _ => LintLevel::Note,
                };

                // Skip empty messages and notes/help without code
                if !message.is_empty()
                    && (level == LintLevel::Error
                        || level == LintLevel::Warning
                        || !code.is_empty())
                {
                    issues.push(LintIssue {
                        file_path: file,
                        line_number: line_num,
                        column: col,
                        level,
                        code,
                        message,
                    });
                }
            }
        }

        // Deduplicate
        issues.sort_by(|a, b| {
            let file_cmp = a.file_path.cmp(&b.file_path);
            if file_cmp == std::cmp::Ordering::Equal {
                a.line_number.cmp(&b.line_number)
            } else {
                file_cmp
            }
        });
        issues.dedup();

        issues
    }
}

/// Summary of lint analysis
#[derive(Debug, Default)]
pub struct LintSummary {
    pub errors: usize,
    pub warnings: usize,
    pub helps: usize,
    pub notes: usize,
    pub issues: Vec<LintIssue>,
    pub issues_by_file: HashMap<String, usize>,
}

impl LintSummary {
    /// Create new summary from issues
    pub fn new(issues: Vec<LintIssue>) -> Self {
        let mut summary = Self {
            issues,
            ..Default::default()
        };

        for issue in &summary.issues {
            match issue.level {
                LintLevel::Error => summary.errors += 1,
                LintLevel::Warning => summary.warnings += 1,
                LintLevel::Help => summary.helps += 1,
                LintLevel::Note => summary.notes += 1,
            }

            let file_key = issue.file_path.clone();
            *summary.issues_by_file.entry(file_key).or_insert(0) += 1;
        }

        summary
    }

    /// Print lint summary and issues table
    pub fn print_report(&self) {
        crate::teeprintln!("\n{}", "═".repeat(100));
        crate::teeprintln!("  LINT ANALYSIS SUMMARY (clippy + cargo check)");
        crate::teeprintln!("{}", "═".repeat(100));
        crate::teeprintln!("");
        crate::teeprintln!("  {:<20} {:>10}", "Lint Level", "Count");
        crate::teeprintln!("  {}", "─".repeat(33));
        if self.errors > 0 {
            crate::teeprintln!("  {:<20} {:>10}", "Errors", self.errors);
        }
        if self.warnings > 0 {
            crate::teeprintln!("  {:<20} {:>10}", "Warnings", self.warnings);
        }
        if self.helps > 0 {
            crate::teeprintln!("  {:<20} {:>10}", "Help suggestions", self.helps);
        }
        if self.notes > 0 {
            crate::teeprintln!("  {:<20} {:>10}", "Notes", self.notes);
        }
        crate::teeprintln!("  {}", "─".repeat(33));
        crate::teeprintln!(
            "  {:<20} {:>10}",
            "TOTAL (E+W)",
            self.errors + self.warnings
        );
        crate::teeprintln!("");

        if !self.issues_by_file.is_empty() && (self.errors > 0 || self.warnings > 0) {
            crate::teeprintln!("  Lint issues by File:");
            crate::teeprintln!("  {}", "─".repeat(33));
            let mut files: Vec<_> = self.issues_by_file.iter().collect();
            files.sort_by(|a, b| b.1.cmp(a.1));
            for (file, count) in files.iter().take(10) {
                let relative = file.split("src/").last().unwrap_or(file);
                crate::teeprintln!("    {:.<40} {:>6}", relative, count);
            }
        }

        // Print detailed issues table if there are errors or warnings
        if !self.issues.is_empty()
            && self
                .issues
                .iter()
                .any(|i| i.level == LintLevel::Error || i.level == LintLevel::Warning)
        {
            crate::teeprintln!("");
            crate::teeprintln!("{}", "─".repeat(100));
            crate::teeprintln!("  DETAILED LINT ISSUES TABLE");
            crate::teeprintln!("{}", "─".repeat(100));
            crate::teeprintln!("");
            crate::teeprintln!("┌{:─<8}┬{:─<6}┬{:─<50}┬{:─<30}┐", "", "", "", "");
            crate::teeprintln!(
                "│{:^8}│{:^6}│{:^50}│{:^30}│",
                "Level",
                "Line",
                "File",
                "Message"
            );
            crate::teeprintln!("├{:─<8}┼{:─<6}┼{:─<50}┼{:─<30}┤", "", "", "", "");

            for issue in &self.issues {
                if issue.level == LintLevel::Error || issue.level == LintLevel::Warning {
                    let file_short = if issue.file_path.len() > 48 {
                        format!("...{}", &issue.file_path[issue.file_path.len() - 45..])
                    } else {
                        issue.file_path.clone()
                    };

                    let msg_short = if issue.message.len() > 28 {
                        format!("{}...", &issue.message[..25])
                    } else {
                        issue.message.clone()
                    };

                    let level_str = match issue.level {
                        LintLevel::Error => "ERROR",
                        LintLevel::Warning => "WARN",
                        LintLevel::Help => "HELP",
                        LintLevel::Note => "NOTE",
                    };

                    crate::teeprintln!(
                        "│{:^8}│{:^6}│{:.<50}│{:.<30}│",
                        level_str,
                        issue.line_number,
                        file_short,
                        msg_short
                    );
                }
            }

            crate::teeprintln!("└{:─<8}┴{:─<6}┴{:─<50}┴{:─<30}┘", "", "", "", "");
        }

        crate::teeprintln!("{}", "═".repeat(100));
    }
}
