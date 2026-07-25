//! Reflection tool tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_insights() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_insights", serde_json::json!({
            "limit": 10
        })).await.expect("get_insights call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: get_insights ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_insights - {:?}", response.error);
        }
        assert!(passed, "get_insights should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_create_reflection() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("create_reflection", serde_json::json!({
            "title": "Integration Test Reflection",
            "description": "Reflection from integration testing",
            "reflection_type": "general"
        })).await.expect("create_reflection call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: create_reflection ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: create_reflection - {:?}", response.error);
        }
        assert!(passed, "create_reflection should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_analyze_patterns() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("analyze_patterns", serde_json::json!({
            "experience_ids": []
        })).await.expect("analyze_patterns call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: analyze_patterns ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: analyze_patterns - {:?}", response.error);
        }
        assert!(passed, "analyze_patterns should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_patterns() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_patterns", serde_json::json!({})).await.expect("get_patterns call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: get_patterns ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_patterns - {:?}", response.error);
        }
        assert!(passed, "get_patterns should succeed");
        
        client.stop().await;
    }
}
