//! Background Workers Tools Module
//!
//! Defines test requirements for Background Workers related MCP tools (per Architecture §22).

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

/// Returns test requirements for Background Workers tools
pub fn background_workers_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "worker_get_stats".to_string(),
            function_name: "get_worker_stats".to_string(),
            category: "BackgroundWorkers".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Gets background worker statistics for all observers".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "stats".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "worker_count".to_string(),
                    expected_value: None,
                },
            ],
            priority: 1,
        },
        TestRequirement {
            id: "worker_get_stats_filtered".to_string(),
            function_name: "get_worker_stats".to_string(),
            category: "BackgroundWorkers".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Gets worker stats filtered by observer name".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "stats".to_string(),
                    expected_value: None,
                },
            ],
            priority: 2,
        },
        TestRequirement {
            id: "worker_get_count".to_string(),
            function_name: "get_worker_count".to_string(),
            category: "BackgroundWorkers".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Gets the number of active background workers".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "worker_count".to_string(),
                    expected_value: None,
                },
            ],
            priority: 1,
        },
    ]
}
