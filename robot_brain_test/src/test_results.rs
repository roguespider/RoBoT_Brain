//! Test Results Module
//! 
//! Provides comprehensive table-based reporting for test results.
//! Shows pass/fail status for each function with detailed information.

use crate::function_registry::TestRequirement;
use crate::code_analyzer::{CodeIssue, IssueType};

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
        println!("\n{}", "═".repeat(100));
        println!("  ROBO T BRAIN - COMPREHENSIVE END-TO-END TEST REPORT");
        println!("{}", "═".repeat(100));
        
        // Summary statistics
        self.print_summary();
        
        // Code quality issues section
        if !self.code_issues.is_empty() {
            self.print_code_issues();
        }
        
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
        
        println!("{}", "═".repeat(100));
    }
    
    /// Print summary statistics
    fn print_summary(&self) {
        println!("\n┌{:─<98}┐", "");
        println!("│ {:^96} │", "SUMMARY");
        println!("├{:─<98}┤", "");
        
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
        
        println!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Total Tests:", total, "", "", "");
        println!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Passed:", passed, "", "", "");
        println!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Failed:", failed, "", "", "");
        println!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Errors:", errors, "", "", "");
        println!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Skipped:", skipped, "", "", "");
        println!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Blocked:", blocked, "", "", "");
        println!("│  {:<30} {:>15.1}% {:>15} {:>15} {:>15} │", 
            "Pass Rate:", pass_rate, "", "", "");
        println!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Code Issues:", code_issues, "", "", "");
        println!("│  {:<30} {:>15} {:>15} {:>15} {:>15} │", 
            "Duration:", format!("{}ms", self.total_duration_ms), "", "", "");
        
        println!("└{:─<98}┘", "");
    }
    
    /// Print code issues table
    fn print_code_issues(&self) {
        println!("\n┌{:─<98}┐", "");
        println!("│ {:^96} │", "⚠️  CODE QUALITY ISSUES DETECTED");
        println!("├{:─<96}:┤", "─");
        println!("│  The following issues were found in the source code:");
        println!("│  These may indicate incomplete implementations or technical debt:");
        println!("│{:─<97}│", "");
        
        // Group issues by type
        let mut issues_by_type: std::collections::HashMap<String, Vec<&CodeIssue>> = std::collections::HashMap::new();
        for issue in &self.code_issues {
            let key = issue.issue_type.to_string();
            issues_by_type.entry(key).or_default().push(issue);
        }
        
        for (issue_type, issues) in &issues_by_type {
            println!("│");
            println!("│  Issue Type: {}", issue_type);
            println!("│  Count: {}", issues.len());
            println!("│  ├── Files affected: {}", issues.iter()
                .map(|i| i.file_path.file_name().unwrap_or_default().to_string_lossy().to_string())
                .collect::<std::collections::HashSet<_>>()
                .len());
            
            // Show first few examples
            for issue in issues.iter().take(3) {
                let file_name = issue.file_path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                println!("│  ├── Line {}: {} - {}", 
                    issue.line_number,
                    file_name,
                    issue.description.chars().take(50).collect::<String>()
                );
            }
            if issues.len() > 3 {
                println!("│  └── ... and {} more", issues.len() - 3);
            }
        }
        
        println!("│{:─<97}│", "");
        println!("└{:─<97}┘", "");
    }
    
    /// Print failed tests details
    fn print_failed_tests(&self) {
        println!("\n┌{:─<98}┐", "");
        println!("│ {:^96} │", "❌ FAILED TESTS");
        println!("├{:─<96}:┤", "─");
        
        for result in self.failed_results() {
            println!("│");
            println!("│  Test ID: {}", result.requirement.id);
            println!("│  Function: {}.{}", result.requirement.category, result.requirement.function_name);
            println!("│  Expected: {}", result.requirement.expected_behavior);
            println!("│  Error: {}", result.error_message.as_deref().unwrap_or("Unknown error"));
            
            if !result.validation_results.is_empty() {
                println!("│  Validation Results:");
                for vr in &result.validation_results {
                    let status = if vr.passed { "✓" } else { "✗" };
                    println!("│    {} {} - {}", status, vr.field, vr.message.as_deref().unwrap_or(""));
                }
            }
        }
        
        println!("│{:─<97}│", "");
        println!("└{:─<97}┘", "");
    }
    
    /// Print error tests details
    fn print_error_tests(&self) {
        println!("\n┌{:─<98}┐", "");
        println!("│ {:^96} │", "💥 ERROR TESTS");
        println!("├{:─<96}:┤", "─");
        
        for result in self.error_results() {
            println!("│");
            println!("│  Test ID: {}", result.requirement.id);
            println!("│  Function: {}.{}", result.requirement.category, result.requirement.function_name);
            println!("│  Error: {}", result.error_message.as_deref().unwrap_or("Unknown error"));
        }
        
        println!("│{:─<97}│", "");
        println!("└{:─<97}┘", "");
    }
    
    /// Print full test table
    fn print_full_table(&self) {
        println!("\n┌{:─<98}┐", "");
        println!("│ {:^96} │", "📋 FULL TEST RESULTS TABLE");
        println!("├{:─<6}├{:─<20}├{:─<25}├{:─<8}├{:─<10}├{:─<25}┤", "─", "─", "─", "─", "─", "─");
        println!("│ {:^4} │ {:^18} │ {:^23} │ {:^6} │ {:^8} │ {:^23} │", 
            "#", "Category", "Function", "Status", "Priority", "Result");
        println!("├{:─<6}┼{:─<20}┼{:─<25}┼{:─<8}┼{:─<10}┼{:─<25}┤", "─", "─", "─", "─", "─", "─");
        
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
            
            println!("│ {:>3} │ {:<20} │ {:<25} │ {} {:<4} │ {:<10} │ {:<25} │", 
                idx + 1,
                truncate(&result.requirement.category, 20),
                truncate(&result.requirement.function_name, 25),
                status_icon,
                status_text,
                priority_text,
                truncate(&result_text, 25)
            );
        }
        
        println!("└{:─<6}┴{:─<20}┴{:─<25}┴{:─<8}┴{:─<10}┴{:─<25}┘", "─", "─", "─", "─", "─", "─");
        
        // Print category summary
        println!("\n  Category Summary:");
        for (cat, count) in categories {
            let passed = self.results.iter()
                .filter(|r| r.requirement.category == cat && r.status == TestStatus::Pass)
                .count();
            println!("    {:<20} {}/{} passed", cat, passed, count);
        }
    }
    
    /// Print final verdict
    fn print_verdict(&self) {
        println!("\n┌{:─<98}┐", "");
        
        let all_passed = self.all_passed();
        let no_code_issues = self.code_issues.is_empty();
        let overall_success = all_passed && no_code_issues;
        
        if overall_success {
            println!("│ {:^96} │", "🎉 VERDICT: ALL TESTS PASSED - READY FOR PRODUCTION");
            println!("├{:─<98}┤", "");
            println!("│");
            println!("│  ✅ All {} functions tested and passed", self.results.len());
            println!("│  ✅ No stub patterns or partial implementations detected");
            println!("│  ✅ No #[allow(*)] annotations that hide issues");
            println!("│  ✅ All sub-functions complete and working");
            println!("│");
        } else {
            println!("│ {:^96} │", "⚠️  VERDICT: TESTS HAVE ISSUES - REVIEW REQUIRED");
            println!("├{:─<98}┤", "");
            println!("│");
            
            if !all_passed {
                let failed = self.failed_count();
                let errors = self.error_count();
                println!("│  ❌ {} tests failed, {} errors", failed, errors);
            }
            
            if !no_code_issues {
                println!("│  ⚠️  {} code quality issues detected", self.code_issues.len());
                println!("│     See code issues section above for details");
            }
            
            println!("│");
            println!("│  Required actions:");
            if !all_passed {
                println!("│    1. Fix all failing tests");
                println!("│    2. Ensure functions work end-to-end");
            }
            if !no_code_issues {
                println!("│    3. Remove stub patterns and #[allow(*)] annotations");
                println!("│    4. Implement missing functionality");
            }
            println!("│");
        }
        
        println!("└{:─<98}┘", "");
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
        println!("\n  ✅ No code quality issues detected in source!");
        return;
    }
    
    println!("\n┌{:─<98}┐", "");
    println!("│ {:^96} │", "⚠️  CODE QUALITY ISSUES TABLE");
    println!("├{:─<10}├{:─<40}├{:─<10}├{:─<35}┤", "─", "─", "─", "─");
    println!("│ {:^8} │ {:^38} │ {:^8} │ {:^33} │", 
        "Line", "File", "Type", "Description");
    println!("├{:─<10}┼{:─<40}┼{:─<10}┼{:─<35}┤", "─", "─", "─", "─");
    
    for issue in issues {
        let file_name = issue.file_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        let issue_type = issue.issue_type.to_string();
        let description = truncate(&issue.description, 33);
        
        println!("│ {:>8} │ {:<40} │ {:<10} │ {:<35} │", 
            issue.line_number,
            truncate(&file_name, 40),
            truncate(&issue_type, 10),
            description
        );
    }
    
    println!("└{:─<10}┴{:─<40}┴{:─<10}┴{:─<35}┘", "─", "─", "─", "─");
}
