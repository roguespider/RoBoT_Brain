//! Exploration Tools Module
//!
//! Defines test requirements for Exploration-related MCP tools.

use crate::function_registry::types::{CheckType, DataRequirement, TestRequirement, ValidationCheck};

/// Returns test requirements for Exploration tools
pub fn exploration_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "exploration_start".to_string(),
            function_name: "start_exploration".to_string(),
            category: "Exploration".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Starts a new exploration".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "exploration_id".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "exploration_status".to_string(),
            function_name: "get_exploration_status".to_string(),
            category: "Exploration".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns exploration status".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "status".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "exploration_record_attempt".to_string(),
            function_name: "record_attempt".to_string(),
            category: "Exploration".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Records an exploration attempt".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "exploration_add_hypothesis".to_string(),
            function_name: "add_hypothesis".to_string(),
            category: "Exploration".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Adds a hypothesis to exploration".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "hypothesis_count".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "exploration_complete".to_string(),
            function_name: "complete_exploration".to_string(),
            category: "Exploration".to_string(),
            requires_workflow: true,
            requires_data: Some(DataRequirement {
                data_type: "exploration".to_string(),
                creation_tool: "start_exploration".to_string(),
                min_count: 1,
            }),
            expected_behavior: "Completes an exploration with findings".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "status".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "finding_count".to_string(),
                    expected_value: None,
                },
            ],
            priority: 2,
        },
        TestRequirement {
            id: "exploration_abandon".to_string(),
            function_name: "abandon_exploration".to_string(),
            category: "Exploration".to_string(),
            requires_workflow: true,
            requires_data: Some(DataRequirement {
                data_type: "exploration".to_string(),
                creation_tool: "start_exploration".to_string(),
                min_count: 1,
            }),
            expected_behavior: "Abandons an exploration without completing it".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "exploration_evaluate_hypothesis".to_string(),
            function_name: "evaluate_hypothesis".to_string(),
            category: "Exploration".to_string(),
            requires_workflow: true,
            requires_data: Some(DataRequirement {
                data_type: "exploration".to_string(),
                creation_tool: "start_exploration".to_string(),
                min_count: 1,
            }),
            expected_behavior: "Sets the result for a hypothesis based on evidence".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "result".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "exploration_promote_finding".to_string(),
            function_name: "promote_finding".to_string(),
            category: "Exploration".to_string(),
            requires_workflow: true,
            requires_data: Some(DataRequirement {
                data_type: "exploration".to_string(),
                creation_tool: "start_exploration".to_string(),
                min_count: 1,
            }),
            expected_behavior: "Promotes a finding from an exploration to reusable knowledge"
                .to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "exploration_pause".to_string(),
            function_name: "pause_exploration".to_string(),
            category: "Exploration".to_string(),
            requires_workflow: true,
            requires_data: Some(DataRequirement {
                data_type: "exploration".to_string(),
                creation_tool: "start_exploration".to_string(),
                min_count: 1,
            }),
            expected_behavior: "Pauses an active exploration".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "exploration_resume".to_string(),
            function_name: "resume_exploration".to_string(),
            category: "Exploration".to_string(),
            requires_workflow: true,
            requires_data: Some(DataRequirement {
                data_type: "exploration".to_string(),
                creation_tool: "start_exploration".to_string(),
                min_count: 1,
            }),
            expected_behavior: "Resumes a paused exploration".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
    ]
}
