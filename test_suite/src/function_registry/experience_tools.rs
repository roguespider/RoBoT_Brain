//! Experience Tools Module
//!
//! Defines test requirements for Experience-related MCP tools.

use crate::function_registry::types::{CheckType, DataRequirement, TestRequirement, ValidationCheck};

/// Returns test requirements for Experience tools
pub fn experience_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "experience_record".to_string(),
            function_name: "record_experience".to_string(),
            category: "Experience".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Records a new experience with action, outcome, and tool name"
                .to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "id".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "experience_get".to_string(),
            function_name: "get_experience".to_string(),
            category: "Experience".to_string(),
            requires_workflow: true,
            requires_data: Some(DataRequirement {
                data_type: "experience".to_string(),
                creation_tool: "record_experience".to_string(),
                min_count: 1,
            }),
            expected_behavior: "Retrieves a specific experience by ID".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "id".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "experience_list".to_string(),
            function_name: "list_experiences".to_string(),
            category: "Experience".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists recent experiences".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "experiences".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "experience_stats".to_string(),
            function_name: "get_experience_stats".to_string(),
            category: "Experience".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns experience statistics".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "stats".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
    ]
}
