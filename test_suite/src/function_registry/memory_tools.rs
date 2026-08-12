//! Memory Tools Module
//!
//! Defines test requirements for Memory-related MCP tools.

use crate::function_registry::types::{CheckType, DataRequirement, TestRequirement, ValidationCheck};

/// Returns test requirements for Memory tools
pub fn memory_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "memory_store_basic".to_string(),
            function_name: "store_memory".to_string(),
            category: "Memory".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Stores a basic memory item".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: Some("true".to_string()),
                },
            ],
            priority: 1,
        },
        TestRequirement {
            id: "memory_store_with_metadata".to_string(),
            function_name: "store_memory".to_string(),
            category: "Memory".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Stores memory with confidence and importance scores".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: Some("true".to_string()),
                },
            ],
            priority: 1,
        },
        TestRequirement {
            id: "memory_search".to_string(),
            function_name: "search_memory".to_string(),
            category: "Memory".to_string(),
            requires_workflow: true,
            requires_data: Some(DataRequirement {
                data_type: "memory".to_string(),
                creation_tool: "store_memory".to_string(),
                min_count: 1,
            }),
            expected_behavior: "Finds memories matching query".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "results".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "memory_get".to_string(),
            function_name: "get_memory".to_string(),
            category: "Memory".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Retrieves a specific memory by ID (returns found=false for non-existent)"
                .to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: Some("true".to_string()),
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "found".to_string(),
                    expected_value: None,
                },
            ],
            priority: 1,
        },
        TestRequirement {
            id: "memory_get_invalid".to_string(),
            function_name: "get_memory".to_string(),
            category: "Memory".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Handles invalid UUID format gracefully (expected error)".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: Some("false".to_string()),
            }],
            priority: 2,
        },
        TestRequirement {
            id: "memory_list".to_string(),
            function_name: "list_memories".to_string(),
            category: "Memory".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists all recent memories".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "memories".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "memory_list_filtered".to_string(),
            function_name: "list_memories".to_string(),
            category: "Memory".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists memories filtered by type".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "memories".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
    ]
}
