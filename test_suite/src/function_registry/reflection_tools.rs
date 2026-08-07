//! Reflection Tools Module
//!
//! Defines test requirements for Reflection-related MCP tools.

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

/// Returns test requirements for Reflection tools
pub fn reflection_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "reflection_create".to_string(),
            function_name: "create_reflection".to_string(),
            category: "Reflection".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Creates a new reflection".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "reflection_get_patterns".to_string(),
            function_name: "get_patterns".to_string(),
            category: "Reflection".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns learned patterns".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "patterns".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "reflection_get_insights".to_string(),
            function_name: "get_insights".to_string(),
            category: "Reflection".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns insights from analysis".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "insights".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "reflection_analyze".to_string(),
            function_name: "analyze_patterns".to_string(),
            category: "Reflection".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Performs pattern analysis".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
    ]
}
