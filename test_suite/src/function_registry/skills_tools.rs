//! Skills Tools Module
//!
//! Defines test requirements for Skills-related MCP tools.

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

/// Returns test requirements for Skills tools
pub fn skills_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "skills_register".to_string(),
            function_name: "register_skill".to_string(),
            category: "Skills".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Registers a new skill".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "id".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "skills_discover".to_string(),
            function_name: "discover_skill".to_string(),
            category: "Skills".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Creates a skill from experience".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "id".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "skills_get".to_string(),
            function_name: "get_skill".to_string(),
            category: "Skills".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Gets skill details (fails with fake UUID)".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: Some("false".to_string()),
            }],
            priority: 2,
        },
        TestRequirement {
            id: "skills_list".to_string(),
            function_name: "list_skills".to_string(),
            category: "Skills".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists all skills".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "skills".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "skills_update_mastery".to_string(),
            function_name: "update_skill_mastery".to_string(),
            category: "Skills".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Updates skill mastery (fails with fake UUID)".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: Some("false".to_string()),
            }],
            priority: 2,
        },
        TestRequirement {
            id: "skills_recommendations".to_string(),
            function_name: "get_skill_recommendations".to_string(),
            category: "Skills".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Gets skill recommendations".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "recommendations".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "skills_execute".to_string(),
            function_name: "execute_skill".to_string(),
            category: "Skills".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Executes a skill (fails with fake UUID)".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: Some("false".to_string()),
            }],
            priority: 2,
        },
        TestRequirement {
            id: "skills_stats".to_string(),
            function_name: "get_skill_stats".to_string(),
            category: "Skills".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Gets skill statistics".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "stats".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "skills_decay".to_string(),
            function_name: "apply_skill_decay".to_string(),
            category: "Skills".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Applies skill decay".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 3,
        },
        TestRequirement {
            id: "skills_enable_disable".to_string(),
            function_name: "enable_disable_skill".to_string(),
            category: "Skills".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Enables or disables a skill".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 3,
        },
        TestRequirement {
            id: "skills_search".to_string(),
            function_name: "search_skills".to_string(),
            category: "Skills".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Searches skills".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "results".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
    ]
}
