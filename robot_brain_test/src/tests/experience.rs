//! Experience tool tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_experience() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("record_experience", serde_json::json!({
            "title": "MCP Integration Test",
            "description": "Recording experience from integration test",
            "experience_type": "tool_execution",
            "outcome": "success"
        })).await.expect("record_experience call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: record_experience ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: record_experience - {:?}", response.error);
        }
        assert!(passed, "record_experience should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_experience_stats() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_experience_stats", serde_json::json!({
            "period": "all"
        })).await.expect("get_experience_stats call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: get_experience_stats ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_experience_stats - {:?}", response.error);
        }
        assert!(passed, "get_experience_stats should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_list_experiences() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("list_experiences", serde_json::json!({
            "limit": 10
        })).await.expect("list_experiences call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: list_experiences ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: list_experiences - {:?}", response.error);
        }
        assert!(passed, "list_experiences should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_experience() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_experience", serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000"
        })).await.expect("get_experience call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: get_experience ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_experience - {:?}", response.error);
        }
        assert!(passed, "get_experience should return response");
        
        client.stop().await;
    }
}
