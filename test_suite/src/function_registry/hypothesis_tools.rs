//! Hypothesis Tools Module
//!
//! Defines test requirements for Hypothesis-related MCP tools.

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

/// Returns test requirements for Hypothesis tools
pub fn hypothesis_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "hypothesis_record_observation".to_string(),
            function_name: "record_observation".to_string(),
            category: "Hypothesis".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Records a new observation".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "hypothesis_create".to_string(),
            function_name: "create_hypothesis".to_string(),
            category: "Hypothesis".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Creates a new hypothesis".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "hypothesis_add_evidence".to_string(),
            function_name: "add_evidence".to_string(),
            category: "Hypothesis".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Adds supporting or contradicting evidence".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "hypothesis_get".to_string(),
            function_name: "get_hypothesis".to_string(),
            category: "Hypothesis".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns the current hypothesis".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "hypothesis".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "hypothesis_list".to_string(),
            function_name: "list_hypotheses".to_string(),
            category: "Hypothesis".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists all hypotheses".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "hypotheses".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "hypothesis_evaluate".to_string(),
            function_name: "evaluate_hypothesis".to_string(),
            category: "Hypothesis".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Evaluates the current hypothesis".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "evaluation".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "hypothesis_extract".to_string(),
            function_name: "extract_knowledge".to_string(),
            category: "Hypothesis".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Extracts knowledge from evaluated hypothesis".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
    ]
}
