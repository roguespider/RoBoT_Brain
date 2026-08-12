//! ACP Tools Module
//!
//! Defines test requirements for ACP (Agent Communication Protocol) tools.
//! These tools are tested via the comprehensive test runner's FunctionRegistry
//! pipeline so the coverage cross-check counts them as tested.

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

/// Returns test requirements for ACP tools
pub fn acp_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "acp_list_agents".to_string(),
            function_name: "list_acp_agents".to_string(),
            category: "ACP".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists all registered ACP agents".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "acp_agent_count".to_string(),
            function_name: "acp_agent_count".to_string(),
            category: "ACP".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns the count of registered ACP agents".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "acp_router".to_string(),
            function_name: "acp_router".to_string(),
            category: "ACP".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns the ACP router info".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "acp_registry".to_string(),
            function_name: "acp_registry".to_string(),
            category: "ACP".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns the ACP agent registry".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "acp_register_agent".to_string(),
            function_name: "register_agent".to_string(),
            category: "ACP".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Registers a test ACP agent".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "acp_unregister_agent".to_string(),
            function_name: "unregister_agent".to_string(),
            category: "ACP".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Unregisters a test ACP agent".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "acp_create_message".to_string(),
            function_name: "create_acp_message".to_string(),
            category: "ACP".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Creates an ACP message without routing".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "acp_route_message".to_string(),
            function_name: "route_acp_message".to_string(),
            category: "ACP".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Routes an ACP message (expected to fail for unregistered receiver)"
                .to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: Some("false".to_string()),
            }],
            priority: 2,
        },
        TestRequirement {
            id: "acp_get_agent_capabilities".to_string(),
            function_name: "get_agent_capabilities".to_string(),
            category: "ACP".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Gets capabilities of an ACP agent (returns default agent for unknown id)"
                .to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::IsSuccess,
                field: "success".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
    ]
}
