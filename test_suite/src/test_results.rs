//! Test Results Module
//! 
//! Provides comprehensive table-based reporting for test results.
//! Shows pass/fail status for each function with detailed information.

use crate::function_registry::TestRequirement;
use crate::code_analyzer::{CodeIssue, LintIssue, LintLevel};

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
    #[allow(dead_code)]
    pub check_type: String,
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
    
    /// Set lint issues (compiler errors and warnings)
    pub fn set_lint_issues(&mut self, issues: Vec<LintIssue>) {
        self.lint_errors = issues.iter().filter(|i| i.level == LintLevel::Error).count();
        self.lint_warnings = issues.iter().filter(|i| i.level == LintLevel::Warning).count();
        self.lint_issues = issues;
    }
    
    /// Get pass count
    pub fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.status == TestStatus::Pass).count()
    }
    
    /// Get fail count
    pub fn failed_count(&self) -> usize {
        self.results.iter().filter(|r| r.status == TestStatus::Fail).count()
    }
    
    /// Get error count
    pub fn error_count(&self) -> usize {
        self.results.iter().filter(|r| r.status == TestStatus::Error).count()
    }
    
    /// Get skipped count
    pub fn skipped_count(&self) -> usize {
        self.results.iter().filter(|r| r.status == TestStatus::Skipped).count()
    }
    
    /// Get blocked count
    pub fn blocked_count(&self) -> usize {
        self.results.iter().filter(|r| r.status == TestStatus::Blocked).count()
    }
    
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.status == TestStatus::Pass)
    }
    
    /// Check if there are any issues (failed tests, code issues, etc.)
    pub fn has_issues(&self) -> bool {
        !self.failed_results().is_empty() || !self.error_results().is_empty() || !self.code_issues.is_empty()
    }
    
    /// Get all failed results
    pub fn failed_results(&self) -> Vec<&TestResult> {
        self.results.iter().filter(|r| r.status == TestStatus::Fail).collect()
    }
    
    /// Get all error results
    pub fn error_results(&self) -> Vec<&TestResult> {
        self.results.iter().filter(|r| r.status == TestStatus::Error).collect()
    }
    
    /// Print the full report in table format
    pub fn print_report(&self) {
        // Header
        crate::teeprintln!("\n{}", "═".repeat(100));
        crate::teeprintln!("  ROBO T BRAIN - COMPREHENSIVE END-TO-END TEST REPORT");
        crate::teeprintln!("{}", "═".repeat(100));
        
        // Summary statistics
        self.print_summary();
        
        // Code quality issues section
        if !self.code_issues.is_empty() {
            self.print_code_issues();
        }
        
        // Compiler errors and warnings section
        self.print_lint_issues();
        
        // Failed tests section
        if !self.failed_results().is_empty() {
            self.print_failed_tests();
        }
        
        // Error tests section
        if !self.error_results().is_empty() {
            self.print_error_tests();
        }
        
        // Full test table
        self.print_full_table();
        
        // Final verdict
        self.print_verdict();
        
        crate::teeprintln!("{}", "═".repeat(100));
    }
    
    /// Print summary statistics
    fn print_summary(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "SUMMARY");
        crate::teeprintln!("├{:─<98}┤", "");
        
        let total = self.results.len();
        let passed = self.passed_count();
        let failed = self.failed_count();
        let errors = self.error_count();
        let skipped = self.skipped_count();
        let blocked = self.blocked_count();
        let code_issues = self.code_issues.len();
        
        // Calculate pass rate
        let pass_rate = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        crate::teeprintln!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Total Tests:", total, "", "", "");
        crate::teeprintln!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Passed:", passed, "", "", "");
        crate::teeprintln!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Failed:", failed, "", "", "");
        crate::teeprintln!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Errors:", errors, "", "", "");
        crate::teeprintln!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Skipped:", skipped, "", "", "");
        crate::teeprintln!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Blocked:", blocked, "", "", "");
        crate::teeprintln!("│  {:<30} {:>15.1}% {:>15} {:>15} {:>15} │", 
            "Pass Rate:", pass_rate, "", "", "");
        crate::teeprintln!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Code Issues:", code_issues, "", "", "");
        
        // Compiler errors and warnings
        crate::teeprintln!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Compiler Errors:", self.lint_errors, "", "", "");
        crate::teeprintln!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Compiler Warnings:", self.lint_warnings, "", "", "");
        
        crate::teeprintln!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Duration:", format!("{}ms", self.total_duration_ms), "", "", "");
        
        crate::teeprintln!("└{:─<98}┘", "");
    }
    
    /// Print code issues table
    fn print_code_issues(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "⚠️  CODE QUALITY ISSUES DETECTED");
        crate::teeprintln!("├{:─<96}:┤", "─");
        crate::teeprintln!("│  The following issues were found in the source code:");
        crate::teeprintln!("│  These may indicate incomplete implementations or technical debt:");
        crate::teeprintln!("│{:─<97}│", "");
        
        // Group issues by type
        let mut issues_by_type: std::collections::HashMap<String, Vec<&CodeIssue>> = std::collections::HashMap::new();
        for issue in &self.code_issues {
            let key = issue.issue_type.to_string();
            issues_by_type.entry(key).or_default().push(issue);
        }
        
        for (issue_type, issues) in &issues_by_type {
            crate::teeprintln!("│");
            crate::teeprintln!("│  Issue Type: {}", issue_type);
            crate::teeprintln!("│  Count: {}", issues.len());
            crate::teeprintln!("│  ├── Files affected: {}", issues.iter()
                .map(|i| i.file_path.file_name().unwrap_or_default().to_string_lossy().to_string())
                .collect::<std::collections::HashSet<_>>()
                .len());
            
            // Show first few examples
            for issue in issues.iter().take(3) {
                let file_name = issue.file_path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                crate::teeprintln!("│  ├── Line {}: {} - {}", 
                    issue.line_number,
                    file_name,
                    issue.description.chars().take(50).collect::<String>()
                );
            }
            if issues.len() > 3 {
                crate::teeprintln!("│  └── ... and {} more", issues.len() - 3);
            }
        }
        
        crate::teeprintln!("│{:─<97}│", "");
        crate::teeprintln!("└{:─<97}┘", "");
    }
    
    /// Print compiler errors and warnings table
    fn print_lint_issues(&self) {
        // Filter for errors and warnings only
        let error_warnings: Vec<&LintIssue> = self.lint_issues.iter()
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
        crate::teeprintln!("│ {:^6} │ {:^4} │ {:^10} │ {:^66} │", 
            "Level", "Line", "Code", "Message");
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
            
            crate::teeprintln!("│ {} {:^4} │ {:>4} │ {:^10} │ {:.<66} │", 
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
        let mut by_file: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
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
            crate::teeprintln!("│    {} {}{} - {} errors, {} warnings", 
                file, err_icon, warn_icon, errs, warns);
        }
        
        crate::teeprintln!("│{:─<97}│", "");
        crate::teeprintln!("└{:─<97}┘", "");
    }
    
    /// Print failed tests details
    fn print_failed_tests(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "❌ FAILED TESTS");
        crate::teeprintln!("├{:─<96}:┤", "─");
        
        for result in self.failed_results() {
            crate::teeprintln!("│");
            crate::teeprintln!("│  Test ID: {}", result.requirement.id);
            crate::teeprintln!("│  Function: {}.{}", result.requirement.category, result.requirement.function_name);
            crate::teeprintln!("│  Expected: {}", result.requirement.expected_behavior);
            crate::teeprintln!("│  Error: {}", result.error_message.as_deref().unwrap_or("Unknown error"));
            
            if !result.validation_results.is_empty() {
                crate::teeprintln!("│  Validation Results:");
                for vr in &result.validation_results {
                    let status = if vr.passed { "✓" } else { "✗" };
                    crate::teeprintln!("│    {} {} - {}", status, vr.field, vr.message.as_deref().unwrap_or(""));
                }
            }
        }
        
        crate::teeprintln!("│{:─<97}│", "");
        crate::teeprintln!("└{:─<97}┘", "");
    }
    
    /// Print error tests details
    fn print_error_tests(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "💥 ERROR TESTS");
        crate::teeprintln!("├{:─<96}:┤", "─");
        
        for result in self.error_results() {
            crate::teeprintln!("│");
            crate::teeprintln!("│  Test ID: {}", result.requirement.id);
            crate::teeprintln!("│  Function: {}.{}", result.requirement.category, result.requirement.function_name);
            crate::teeprintln!("│  Error: {}", result.error_message.as_deref().unwrap_or("Unknown error"));
        }
        
        crate::teeprintln!("│{:─<97}│", "");
        crate::teeprintln!("└{:─<97}┘", "");
    }
    
    /// Print full test table
    fn print_full_table(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "📋 FULL TEST RESULTS TABLE");
        crate::teeprintln!("├{:─<6}├{:─<20}├{:─<25}├{:─<8}├{:─<10}├{:─<25}┤", "─", "─", "─", "─", "─", "─");
        crate::teeprintln!("│ {:^4} │ {:^18} │ {:^23} │ {:^6} │ {:^8} │ {:^23} │", 
            "#", "Category", "Function", "Status", "Priority", "Result");
        crate::teeprintln!("├{:─<6}┼{:─<20}┼{:─<25}┼{:─<8}┼{:─<10}┼{:─<25}┤", "─", "─", "─", "─", "─", "─");
        
        let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        
        for (idx, result) in self.results.iter().enumerate() {
            let cat_count = categories.entry(result.requirement.category.clone()).or_insert(0);
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
            
            crate::teeprintln!("│ {:>3} │ {:<20} │ {:<25} │ {} {:<4} │ {:<10} │ {:<25} │", 
                idx + 1,
                truncate(&result.requirement.category, 20),
                truncate(&result.requirement.function_name, 25),
                status_icon,
                status_text,
                priority_text,
                truncate(&result_text, 25)
            );
        }
        
        crate::teeprintln!("└{:─<6}┴{:─<20}┴{:─<25}┴{:─<8}┴{:─<10}┴{:─<25}┘", "─", "─", "─", "─", "─", "─");
        
        // Print category summary
        crate::teeprintln!("\n  Category Summary:");
        for (cat, count) in categories {
            let passed = self.results.iter()
                .filter(|r| r.requirement.category == cat && r.status == TestStatus::Pass)
                .count();
            crate::teeprintln!("    {:<20} {}/{} passed", cat, passed, count);
        }
    }
    
    /// Print final verdict
    fn print_verdict(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        
        let all_passed = self.all_passed();
        let no_code_issues = self.code_issues.is_empty();
        let no_lint_issues = self.lint_errors == 0 && self.lint_warnings == 0;
        let overall_success = all_passed && no_code_issues && no_lint_issues;
        
        if overall_success {
            crate::teeprintln!("│ {:^96} │", "🎉 VERDICT: ALL TESTS PASSED - READY FOR PRODUCTION");
            crate::teeprintln!("├{:─<98}┤", "");
            crate::teeprintln!("│");
            crate::teeprintln!("│  ✅ All {} functions tested and passed", self.results.len());
            crate::teeprintln!("│  ✅ No stub patterns or partial implementations detected");
            crate::teeprintln!("│  ✅ No #[allow(*)] annotations that hide issues");
            crate::teeprintln!("│  ✅ All sub-functions complete and working");
            crate::teeprintln!("│  ✅ No compiler errors or warnings");
            crate::teeprintln!("│");
        } else {
            crate::teeprintln!("│ {:^96} │", "⚠️  VERDICT: TESTS HAVE ISSUES - REVIEW REQUIRED");
            crate::teeprintln!("├{:─<98}┤", "");
            crate::teeprintln!("│");
            
            if !all_passed {
                let failed = self.failed_count();
                let errors = self.error_count();
                crate::teeprintln!("│  ❌ {} tests failed, {} errors", failed, errors);
            }
            
            if !no_code_issues {
                crate::teeprintln!("│  ⚠️  {} code quality issues detected", self.code_issues.len());
                crate::teeprintln!("│     See code issues section above for details");
            }
            
            if !no_lint_issues {
                crate::teeprintln!("│  ⚠️  {} compiler errors, {} warnings", self.lint_errors, self.lint_warnings);
                crate::teeprintln!("│     See compiler errors & warnings section above");
            }
            
            crate::teeprintln!("│");
            crate::teeprintln!("│  Required actions:");
            let mut action_num = 1;
            if !all_passed {
                crate::teeprintln!("│    {}. Fix all failing tests", action_num);
                action_num += 1;
                crate::teeprintln!("│    {}. Ensure functions work end-to-end", action_num);
                action_num += 1;
            }
            if !no_code_issues {
                crate::teeprintln!("│    {}. Remove stub patterns and #[allow(*)] annotations", action_num);
                action_num += 1;
                crate::teeprintln!("│    {}. Implement missing functionality", action_num);
                action_num += 1;
            }
            if !no_lint_issues {
                crate::teeprintln!("│    {}. Fix compiler errors and warnings", action_num);
            }
            crate::teeprintln!("│");
        }
        
        crate::teeprintln!("└{:─<98}┘", "");
    }
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Print issues table (standalone function for early reporting)
pub fn print_issues_table(issues: &[CodeIssue]) {
    if issues.is_empty() {
        crate::teeprintln!("\n  ✅ No code quality issues detected in source!");
        return;
    }
    
    crate::teeprintln!("\n┌{:─<98}┐", "");
    crate::teeprintln!("│ {:^96} │", "⚠️  CODE QUALITY ISSUES TABLE");
    crate::teeprintln!("├{:─<10}├{:─<40}├{:─<10}├{:─<35}┤", "─", "─", "─", "─");
    crate::teeprintln!("│ {:^8} │ {:^38} │ {:^8} │ {:^33} │", 
        "Line", "File", "Type", "Description");
    crate::teeprintln!("├{:─<10}┼{:─<40}┼{:─<10}┼{:─<35}┤", "─", "─", "─", "─");
    
    for issue in issues {
        let file_name = issue.file_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        let issue_type = issue.issue_type.to_string();
        let description = truncate(&issue.description, 33);
        
        crate::teeprintln!("│ {:>8} │ {:<40} │ {:<10} │ {:<35} │", 
            issue.line_number,
            truncate(&file_name, 40),
            truncate(&issue_type, 10),
            description
        );
    }
    
    crate::teeprintln!("└{:─<10}┴{:─<40}┴{:─<10}┴{:─<35}┘", "─", "─", "─", "─");
}
