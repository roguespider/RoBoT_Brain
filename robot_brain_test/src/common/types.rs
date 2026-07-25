//! Common types shared across the test suite

use serde::{Deserialize, Serialize};

/// Result of a single tool test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTestResult {
    pub tool_name: String,
    pub passed: bool,
    pub message: String,
    pub response_time_ms: u64,
}

/// Overall test suite summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub tool_results: Vec<ToolTestResult>,
}

impl TestSuiteSummary {
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            passed: 0,
            failed: 0,
            tool_results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: ToolTestResult) {
        self.total_tests += 1;
        if result.passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
        self.tool_results.push(result);
    }

    pub fn pass_rate(&self) -> f64 {
        if self.total_tests == 0 {
            return 0.0;
        }
        (self.passed as f64 / self.total_tests as f64) * 100.0
    }
}

impl Default for TestSuiteSummary {
    fn default() -> Self {
        Self::new()
    }
}
