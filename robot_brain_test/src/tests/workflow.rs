//! Workflow tool tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_workflow() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("create_workflow", serde_json::json!({
            "name": "Test Workflow",
            "description": "Workflow created by integration test"
        })).await.expect("create_workflow call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: create_workflow ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: create_workflow - {:?}", response.error);
        }
        assert!(passed, "create_workflow should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_list_workflows() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("list_workflows", serde_json::json!({})).await.expect("list_workflows call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: list_workflows ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: list_workflows - {:?}", response.error);
        }
        assert!(passed, "list_workflows should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_workflow_status() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_workflow_status", serde_json::json!({
            "workflow_id": "non-existent"
        })).await.expect("get_workflow_status call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: get_workflow_status ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_workflow_status - {:?}", response.error);
        }
        assert!(passed, "get_workflow_status should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_add_workflow_step() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("add_workflow_step", serde_json::json!({
            "workflow_id": "non-existent",
            "name": "Test Step",
            "action": "test_action"
        })).await.expect("add_workflow_step call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: add_workflow_step ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: add_workflow_step - {:?}", response.error);
        }
        assert!(passed, "add_workflow_step should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_start_workflow() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("start_workflow", serde_json::json!({
            "workflow_id": "non-existent"
        })).await.expect("start_workflow call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: start_workflow ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: start_workflow - {:?}", response.error);
        }
        assert!(passed, "start_workflow should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_pause_workflow() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("pause_workflow", serde_json::json!({
            "workflow_id": "non-existent"
        })).await.expect("pause_workflow call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: pause_workflow ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: pause_workflow - {:?}", response.error);
        }
        assert!(passed, "pause_workflow should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_resume_workflow() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("resume_workflow", serde_json::json!({
            "workflow_id": "non-existent"
        })).await.expect("resume_workflow call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: resume_workflow ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: resume_workflow - {:?}", response.error);
        }
        assert!(passed, "resume_workflow should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_cancel_workflow() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("cancel_workflow", serde_json::json!({
            "workflow_id": "non-existent"
        })).await.expect("cancel_workflow call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: cancel_workflow ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: cancel_workflow - {:?}", response.error);
        }
        assert!(passed, "cancel_workflow should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_delete_workflow() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("delete_workflow", serde_json::json!({
            "workflow_id": "non-existent"
        })).await.expect("delete_workflow call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: delete_workflow ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: delete_workflow - {:?}", response.error);
        }
        assert!(passed, "delete_workflow should return response");
        
        client.stop().await;
    }
}
