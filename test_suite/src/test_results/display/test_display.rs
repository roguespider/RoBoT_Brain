//! Failed and error test detail views.
//! Contains `print_failed_tests` and `print_error_tests` methods.

use super::super::TestReport;

impl TestReport {
    /// Print failed tests details
    pub fn print_failed_tests(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "[FAIL] FAILED TESTS");
        crate::teeprintln!("├{:─<96}:┤", "─");

        for result in self.failed_results() {
            crate::teeprintln!("│");
            crate::teeprintln!("│  Test ID: {}", result.requirement.id);
            crate::teeprintln!(
                "│  Function: {}.{}",
                result.requirement.category,
                result.requirement.function_name
            );
            crate::teeprintln!("│  Expected: {}", result.requirement.expected_behavior);
            crate::teeprintln!(
                "│  Error: {}",
                result.error_message.as_deref().unwrap_or("Unknown error")
            );

            if !result.validation_results.is_empty() {
                crate::teeprintln!("│  Validation Results:");
                for vr in &result.validation_results {
                    let status = if vr.passed { "[OK]" } else { "[FAIL]" };
                    crate::teeprintln!(
                        "│    {} {} - {}",
                        status,
                        vr.field,
                        vr.message.as_deref().unwrap_or("")
                    );
                }
            }

            if !result.server_logs.is_empty() {
                crate::teeprintln!("│  Server Logs (recent stderr):");
                for log_line in &result.server_logs {
                    crate::teeprintln!("│    {}", crate::test_results::truncate(log_line, 92));
                }
            }
        }

        crate::teeprintln!("│{:─<97}│", "");
        crate::teeprintln!("└{:─<97}┘", "");
    }

    /// Print error tests details
    pub fn print_error_tests(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "[FAIL] ERROR TESTS");
        crate::teeprintln!("├{:─<96}:┤", "─");

        for result in self.error_results() {
            crate::teeprintln!("│");
            crate::teeprintln!("│  Test ID: {}", result.requirement.id);
            crate::teeprintln!(
                "│  Function: {}.{}",
                result.requirement.category,
                result.requirement.function_name
            );
            crate::teeprintln!(
                "│  Error: {}",
                result.error_message.as_deref().unwrap_or("Unknown error")
            );

            if !result.server_logs.is_empty() {
                crate::teeprintln!("│  Server Logs (recent stderr):");
                for log_line in &result.server_logs {
                    crate::teeprintln!("│    {}", crate::test_results::truncate(log_line, 92));
                }
            }
        }

        crate::teeprintln!("│{:─<97}│", "");
        crate::teeprintln!("└{:─<97}┘", "");
    }
}
