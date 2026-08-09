//! Display methods for TestReport - main orchestrator and module declarations.
//!
//! Splits display logic into focused submodules:
//! - `summary`: Summary statistics and final verdict
//! - `tables`: Table formatting for full results and lint issues
//! - `test_display`: Failed and error test detail views
//! - `code_issues`: Code quality issues display

pub mod code_issues;
pub mod consolidated;
pub mod coverage;
pub mod summary;
pub mod tables;
pub mod test_display;

use super::TestReport;

/// Print the full report in table format - orchestrates all display sections
impl TestReport {
    pub fn print_report(&self) {
        // Header
        crate::teeprintln!("\n{}", "═".repeat(100));
        crate::teeprintln!("  ROBO T BRAIN - COMPREHENSIVE END-TO-END TEST REPORT");
        crate::teeprintln!("{}", "═".repeat(100));

        // Summary statistics
        self.print_summary();

        // Tool coverage section (server tools vs tested tools)
        self.print_coverage();

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

        // Consolidated issues view (all problem kinds in one place)
        if self.has_issues() {
            self.print_consolidated_issues();
        }

        // Final verdict
        self.print_verdict();

        crate::teeprintln!("{}", "═".repeat(100));
    }
}
