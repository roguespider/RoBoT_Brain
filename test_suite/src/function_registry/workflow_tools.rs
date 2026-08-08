//! Workflow Tools Module
//!
//! Defines test requirements for Workflow-related MCP tools.

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

/// Returns test requirements for Workflow tools
pub fn workflow_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "workflow_create".to_string(),
            function_name: "create_workflow".to_string(),
            category: "Workflow".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Creates a new workflow".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "id".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "workflow_add_step".to_string(),
            function_name: "add_workflow_step".to_string(),
            category: "Workflow".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Adds a step to the workflow".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "workflow_status".to_string(),
            function_name: "get_workflow_status".to_string(),
            category: "Workflow".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns workflow status".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "status".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "workflow_start".to_string(),
            function_name: "start_workflow".to_string(),
            category: "Workflow".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Starts workflow execution".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "workflow_pause".to_string(),
            function_name: "pause_workflow".to_string(),
            category: "Workflow".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Pauses workflow execution".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "workflow_resume".to_string(),
            function_name: "resume_workflow".to_string(),
            category: "Workflow".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Resumes workflow execution".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "workflow_cancel".to_string(),
            function_name: "cancel_workflow".to_string(),
            category: "Workflow".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Cancels workflow execution".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "workflow_delete".to_string(),
            function_name: "delete_workflow".to_string(),
            category: "Workflow".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Deletes a workflow".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "workflow_list".to_string(),
            function_name: "list_workflows".to_string(),
            category: "Workflow".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists all workflows".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "workflows".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
    ]
}
