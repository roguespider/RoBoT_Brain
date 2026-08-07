//! Helper Functions Module
//!
//! Contains utility functions for the test suite.

use crate::function_registry::TestRequirement;

/// Get unique categories from requirements
pub fn get_categories(requirements: &[TestRequirement]) -> Vec<String> {
    let mut categories: Vec<String> = requirements.iter().map(|r| r.category.clone()).collect();
    categories.sort();
    categories.dedup();
    categories
}

/// Get category count
pub fn get_category_count(requirements: &[TestRequirement]) -> usize {
    get_categories(requirements).len()
}

/// Print test result in table format
pub fn format_test_result(test_num: usize, requirement: &TestRequirement, status: &str, details: &str) -> String {
    // Truncate for table
    let cat_str = if requirement.category.len() > 18 {
        format!("{}...", &requirement.category[..15])
    } else {
        requirement.category.clone()
    };
    let name_str = if requirement.function_name.len() > 28 {
        format!("{}...", &requirement.function_name[..25])
    } else {
        requirement.function_name.clone()
    };
    let detail_str = if details.len() > 48 {
        format!("{}...", &details[..45])
    } else {
        details.to_string()
    };

    format!(
        "  │ {:>3} │ {:<18} │ {:<28} │ {} │ {:<48} │",
        test_num,
        cat_str,
        name_str,
        status,
        detail_str
    )
}

/// Get status icon for test result
pub fn get_status_icon(status: &crate::test_results::TestStatus) -> &'static str {
    match status {
        crate::test_results::TestStatus::Pass => "✅ PASS",
        crate::test_results::TestStatus::Fail => "❌ FAIL",
        crate::test_results::TestStatus::Error => "💥 ERROR",
        crate::test_results::TestStatus::Skipped => "⏭️  SKIP",
        crate::test_results::TestStatus::Blocked => "🚫 BLOCK",
    }
}
