//! Agent Tools Module
//!
//! Defines test requirements for Agent-related MCP tools.

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

/// Returns test requirements for Agent tools
pub fn agent_tools() -> Vec<TestRequirement> {
    vec![
        TestRequirement {
            id: "agent_get_workflow_default".to_string(),
            function_name: "get_workflow".to_string(),
            category: "Agent".to_string(),
            requires_workflow: false,
            requires_data: None,
            expected_behavior: "Returns workflow rules when called with 'default' purpose"
                .to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "workflow".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
                },
            ],
            priority: 1,
        },
        TestRequirement {
            id: "agent_get_workflow_general".to_string(),
            function_name: "get_workflow".to_string(),
            category: "Agent".to_string(),
            requires_workflow: false,
            requires_data: None,
            expected_behavior: "Returns workflow rules when called with 'general' purpose"
                .to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "workflow".to_string(),
                expected_value: None,
            }],
            priority: 1,
        },
        TestRequirement {
            id: "agent_list_tools".to_string(),
            function_name: "list_tools".to_string(),
            category: "Agent".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists all available tools".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "tools".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "agent_list_tools_memory".to_string(),
            function_name: "list_tools".to_string(),
            category: "Agent".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Lists memory tools when filtered by 'memory' category"
                .to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "tools".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
        TestRequirement {
            id: "agent_get_tool".to_string(),
            function_name: "get_tool".to_string(),
            category: "Agent".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Returns tool definition for 'store_memory'".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "found".to_string(),
                    expected_value: Some("true".to_string()),
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "tool".to_string(),
                    expected_value: None,
                },
            ],
            priority: 2,
        },
        TestRequirement {
            id: "agent_connect_mcp".to_string(),
            function_name: "connect_mcp_server".to_string(),
            category: "Agent".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Connects to an external MCP server".to_string(),
            validation: vec![
                ValidationCheck {
                    check_type: CheckType::IsSuccess,
                    field: "success".to_string(),
                    expected_value: None,
                },
                ValidationCheck {
                    check_type: CheckType::HasField,
                    field: "connected".to_string(),
                    expected_value: None,
                },
            ],
            priority: 2,
        },
        TestRequirement {
            id: "agent_call_tool".to_string(),
            function_name: "call_tool".to_string(),
            category: "Agent".to_string(),
            requires_workflow: true,
            requires_data: None,
            expected_behavior: "Calls a tool on a connected MCP server".to_string(),
            validation: vec![ValidationCheck {
                check_type: CheckType::HasField,
                field: "content".to_string(),
                expected_value: None,
            }],
            priority: 2,
        },
    ]
}
