//! Search Tools Module
//!
//! Defines test requirements for Search-related MCP tools.

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

/// Returns test requirements for Search tools
pub fn search_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "search_global".to_string(),
            function_name: "global_search".to_string(),
            category: "Search".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Performs global search across all data".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "results".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "search_recommendations".to_string(),
            function_name: "get_recommendations".to_string(),
            category: "Search".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns tool recommendations".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "recommendations".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "search_reputation".to_string(),
            function_name: "get_reputation".to_string(),
            category: "Search".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns reputation data for a tool".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "tool_name".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
    ]
}
