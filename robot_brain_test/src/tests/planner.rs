//! Planner tool tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_plan() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("create_plan", serde_json::json!({
            "goal": "Test plan for integration testing"
        })).await.expect("create_plan call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: create_plan ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: create_plan - {:?}", response.error);
        }
        assert!(passed, "create_plan should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_list_plans() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("list_plans", serde_json::json!({})).await.expect("list_plans call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: list_plans ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: list_plans - {:?}", response.error);
        }
        assert!(passed, "list_plans should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_plan() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_plan", serde_json::json!({
            "plan_id": "non-existent-plan"
        })).await.expect("get_plan call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: get_plan ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_plan - {:?}", response.error);
        }
        assert!(passed, "get_plan should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_add_plan_step() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("add_plan_step", serde_json::json!({
            "plan_id": "non-existent-plan",
            "description": "Test step",
            "action": "test_action"
        })).await.expect("add_plan_step call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: add_plan_step ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: add_plan_step - {:?}", response.error);
        }
        assert!(passed, "add_plan_step should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_start_plan() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("start_plan", serde_json::json!({
            "plan_id": "non-existent-plan"
        })).await.expect("start_plan call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: start_plan ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: start_plan - {:?}", response.error);
        }
        assert!(passed, "start_plan should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_complete_step() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("complete_step", serde_json::json!({
            "plan_id": "non-existent",
            "step_id": "non-existent",
            "result": "test result"
        })).await.expect("complete_step call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: complete_step ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: complete_step - {:?}", response.error);
        }
        assert!(passed, "complete_step should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_fail_step() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("fail_step", serde_json::json!({
            "plan_id": "non-existent",
            "step_id": "non-existent",
            "error": "Test error"
        })).await.expect("fail_step call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: fail_step ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: fail_step - {:?}", response.error);
        }
        assert!(passed, "fail_step should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_cancel_plan() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("cancel_plan", serde_json::json!({
            "plan_id": "non-existent-plan"
        })).await.expect("cancel_plan call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: cancel_plan ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: cancel_plan - {:?}", response.error);
        }
        assert!(passed, "cancel_plan should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_add_step_dependency() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("add_step_dependency", serde_json::json!({
            "plan_id": "non-existent",
            "step_id": "step-1",
            "depends_on": "step-0"
        })).await.expect("add_step_dependency call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: add_step_dependency ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: add_step_dependency - {:?}", response.error);
        }
        assert!(passed, "add_step_dependency should return response");
        
        client.stop().await;
    }
}
