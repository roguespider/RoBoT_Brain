//! Ingestor Tools Module
//!
//! Defines test requirements for Ingestor-related MCP tools.

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

/// Returns test requirements for Ingestor tools
pub fn ingestor_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "ingestor_list_importable_recursive".to_string(),
            function_name: "list_importable".to_string(),
            category: "Ingestor".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists files recursively including subdirectories".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "files".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "ingestor_ingest_json".to_string(),
            function_name: "ingest_files".to_string(),
            category: "Ingestor".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Ingests a JSON file with smart extraction".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "ingestor_ingest_code".to_string(),
            function_name: "ingest_files".to_string(),
            category: "Ingestor".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Ingests a code file (Rust)".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "ingestor_list_ingested".to_string(),
            function_name: "list_ingested_files".to_string(),
            category: "Ingestor".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists all ingested files".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "files".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "ingestor_delete_blocked".to_string(),
            function_name: "delete_ingested_files".to_string(),
            category: "Ingestor".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Delete operation should be blocked without admin".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: Some("false".to_string()),
            }],
            priority: 3,
        },
        TestRequirement {
            id: "ingestor_transcribe_audio".to_string(),
            function_name: "transcribe_audio".to_string(),
            category: "Ingestor".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Transcribes an audio file using Whisper AI".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "text".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
                },
            ],
            priority: 2,
        },
    ]
}
