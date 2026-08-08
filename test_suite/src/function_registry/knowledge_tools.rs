//! Knowledge Tools Module
//!
//! Defines test requirements for Knowledge-related MCP tools.

use crate::function_registry::types::{CheckType, DataRequirement, TestRequirement, ValidationCheck};

/// Returns test requirements for Knowledge tools
pub fn knowledge_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "knowledge_add".to_string(),
            function_name: "add_knowledge".to_string(),
            category: "Knowledge".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Adds new knowledge".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "knowledge_id".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "knowledge_query".to_string(),
            function_name: "query_knowledge".to_string(),
            category: "Knowledge".to_string(),
            requires_workflow: true,
            requires_data: Some(DataRequirement {
                data_type: "knowledge".to_string(),
                creation_tool: "add_knowledge".to_string(),
                min_count: 1,
            }),
            expected_behavior: "Queries knowledge base".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "items".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "knowledge_mature".to_string(),
            function_name: "get_mature_knowledge".to_string(),
            category: "Knowledge".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Gets knowledge that has been applied multiple times".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "items".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "knowledge_stats".to_string(),
            function_name: "get_knowledge_stats".to_string(),
            category: "Knowledge".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns knowledge statistics".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "total".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "knowledge_record_application".to_string(),
            function_name: "record_knowledge_application".to_string(),
            category: "Knowledge".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Records knowledge application outcome (fails with fake UUID)"
                .to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: Some("false".to_string()),
            }],
            priority: 2,
        },
    ]
}
