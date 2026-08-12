//! Vector Index Tools Module
//!
//! Defines test requirements for Vector Index (Embedding operations) related MCP tools.

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

/// Returns test requirements for Vector Index tools
pub fn vector_index_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "vector_store_embedding".to_string(),
            function_name: "store_embedding".to_string(),
            category: "VectorIndex".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Stores a vector embedding for semantic memory search".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "id".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "dimension".to_string(),
                    expected_value: None,
                },
            ],
            priority: 1,
        },
        TestRequirement {
            id: "vector_get_embedding".to_string(),
            function_name: "get_embedding".to_string(),
            category: "VectorIndex".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Gets an embedding by memory ID (returns found=false for non-existent)"
                .to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
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
            id: "vector_search_similar".to_string(),
            function_name: "search_similar".to_string(),
            category: "VectorIndex".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior:
                "Searches for similar memories using vector cosine similarity".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "results".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "count".to_string(),
                    expected_value: None,
                },
            ],
            priority: 1,
        },
        TestRequirement {
            id: "vector_list_embeddings".to_string(),
            function_name: "list_embeddings".to_string(),
            category: "VectorIndex".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists all memory embeddings".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "embeddings".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "count".to_string(),
                    expected_value: None,
                },
            ],
            priority: 2,
        },
        TestRequirement {
            id: "vector_delete_embedding".to_string(),
            function_name: "delete_embedding".to_string(),
            category: "VectorIndex".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Deletes an embedding by memory ID".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "deleted".to_string(),
                    expected_value: None,
                },
            ],
            priority: 2,
        },
        TestRequirement {
            id: "vector_get_embedding_stats".to_string(),
            function_name: "get_embedding_stats".to_string(),
            category: "VectorIndex".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Gets vector index statistics".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "total_embeddings".to_string(),
                    expected_value: None,
                },
            ],
            priority: 2,
        },
    ]
}
