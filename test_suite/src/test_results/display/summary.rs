//! Summary statistics and final verdict display.
//! Contains `print_summary` and `print_verdict` methods.

use super::super::TestReport;

impl TestReport {
    /// Print summary statistics
    pub fn print_summary(&self) {
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

        crate::teeprintln!(
            "│  {:<30} {:>15} {:>15} {:>15} {:>15} │",
            "Total Tests:",
            total,
            "",
            "",
            ""
        );
        crate::teeprintln!(
            "│  {:<30} {:>15} {:>15} {:>15} {:>15} │",
            "Passed:",
            passed,
            "",
            "",
            ""
        );
        crate::teeprintln!(
            "│  {:<30} {:>15} {:>15} {:>15} {:>15} │",
            "Failed:",
            failed,
            "",
            "",
            ""
        );
        crate::teeprintln!(
            "│  {:<30} {:>15} {:>15} {:>15} {:>15} │",
            "Errors:",
            errors,
            "",
            "",
            ""
        );
        crate::teeprintln!(
            "│  {:<30} {:>15} {:>15} {:>15} {:>15} │",
            "Skipped:",
            skipped,
            "",
            "",
            ""
        );
        crate::teeprintln!(
            "│  {:<30} {:>15} {:>15} {:>15} {:>15} │",
            "Blocked:",
            blocked,
            "",
            "",
            ""
        );
        crate::teeprintln!(
            "│  {:<30} {:>15.1}% {:>15} {:>15} {:>15} │",
            "Pass Rate:",
            pass_rate,
            "",
            "",
            ""
        );
        crate::teeprintln!(
            "│  {:<30} {:>15} {:>15} {:>15} {:>15} │",
            "Code Issues:",
            code_issues,
            "",
            "",
            ""
        );

        // Compiler errors and warnings
        crate::teeprintln!(
            "│  {:<30} {:>15} {:>15} {:>15} {:>15} │",
            "Compiler Errors:",
            self.lint_errors,
            "",
            "",
            ""
        );
        crate::teeprintln!(
            "│  {:<30} {:>15} {:>15} {:>15} {:>15} │",
            "Compiler Warnings:",
            self.lint_warnings,
            "",
            "",
            ""
        );

        crate::teeprintln!(
            "│  {:<30} {:>15} {:>15} {:>15} {:>15} │",
            "Duration:",
            format!("{}ms", self.total_duration_ms),
            "",
            "",
            ""
        );

        crate::teeprintln!("└{:─<98}┘", "");
    }

    /// Print final verdict
    pub fn print_verdict(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");

        let all_passed = self.all_passed();
        let no_code_issues = self.code_issues.is_empty();
        let no_lint_issues = self.lint_errors == 0 && self.lint_warnings == 0;
        let overall_success = all_passed && no_code_issues && no_lint_issues;

        if overall_success {
            crate::teeprintln!(
                "│ {:^96} │",
                "🎉 VERDICT: ALL TESTS PASSED - READY FOR PRODUCTION"
            );
            crate::teeprintln!("├{:─<98}┤", "");
            crate::teeprintln!("│");
            crate::teeprintln!(
                "│  ✅ All {} functions tested and passed",
                self.results.len()
            );
            crate::teeprintln!("│  ✅ No stub patterns or partial implementations detected");
            crate::teeprintln!("│  ✅ No #[allow(*)] annotations that hide issues");
            crate::teeprintln!("│  ✅ All sub-functions complete and working");
            crate::teeprintln!("│  ✅ No compiler errors or warnings");
            crate::teeprintln!("│");
        } else {
            crate::teeprintln!(
                "│ {:^96} │",
                "⚠️  VERDICT: TESTS HAVE ISSUES - REVIEW REQUIRED"
            );
            crate::teeprintln!("├{:─<98}┤", "");
            crate::teeprintln!("│");

            if !all_passed {
                let failed = self.failed_count();
                let errors = self.error_count();
                crate::teeprintln!("│  ❌ {} tests failed, {} errors", failed, errors);
            }

            if !no_code_issues {
                crate::teeprintln!(
                    "│  ⚠️  {} code quality issues detected",
                    self.code_issues.len()
                );
                crate::teeprintln!("│     See code issues section above for details");
            }

            if !no_lint_issues {
                crate::teeprintln!(
                    "│  ⚠️  {} compiler errors, {} warnings",
                    self.lint_errors,
                    self.lint_warnings
                );
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
                crate::teeprintln!(
                    "│    {}. Remove stub patterns and #[allow(*)] annotations",
                    action_num
                );
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
