//! Planner Tools Module
//!
//! Defines test requirements for Planner-related MCP tools.

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

/// Returns test requirements for Planner tools
pub fn planner_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "planner_create".to_string(),
            function_name: "create_plan".to_string(),
            category: "Planner".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Creates a new plan".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "id".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "planner_add_step".to_string(),
            function_name: "add_plan_step".to_string(),
            category: "Planner".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Adds a step to the current plan".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "planner_add_dependency".to_string(),
            function_name: "add_step_dependency".to_string(),
            category: "Planner".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Adds a dependency between steps".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "planner_get".to_string(),
            function_name: "get_plan".to_string(),
            category: "Planner".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns the current plan".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "plan".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "planner_start".to_string(),
            function_name: "start_plan".to_string(),
            category: "Planner".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Starts executing the plan".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "planner_complete_step".to_string(),
            function_name: "complete_step".to_string(),
            category: "Planner".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Marks a step as completed".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "planner_fail_step".to_string(),
            function_name: "fail_step".to_string(),
            category: "Planner".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Marks a step as failed".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "planner_cancel".to_string(),
            function_name: "cancel_plan".to_string(),
            category: "Planner".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Cancels the current plan".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "planner_list".to_string(),
            function_name: "list_plans".to_string(),
            category: "Planner".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists all plans".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "plans".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
    ]
}
