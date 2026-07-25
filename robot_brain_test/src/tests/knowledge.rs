//! Knowledge tool tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_knowledge() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("add_knowledge", serde_json::json!({
            "statement": "Test knowledge from integration testing",
            "knowledge_type": "fact",
            "confidence": 0.9,
            "source": "test"
        })).await.expect("add_knowledge call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: add_knowledge ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: add_knowledge - {:?}", response.error);
        }
        assert!(passed, "add_knowledge should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_query_knowledge() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("query_knowledge", serde_json::json!({
            "query": "test",
            "limit": 5
        })).await.expect("query_knowledge call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: query_knowledge ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: query_knowledge - {:?}", response.error);
        }
        assert!(passed, "query_knowledge should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_record_knowledge_application() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("record_knowledge_application", serde_json::json!({
            "knowledge_id": "00000000-0000-0000-0000-000000000000",
            "success": true
        })).await.expect("record_knowledge_application call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: record_knowledge_application ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: record_knowledge_application - {:?}", response.error);
        }
        assert!(passed, "record_knowledge_application should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_knowledge_stats() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_knowledge_stats", serde_json::json!({})).await.expect("get_knowledge_stats call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: get_knowledge_stats ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_knowledge_stats - {:?}", response.error);
        }
        assert!(passed, "get_knowledge_stats should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_mature_knowledge() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_mature_knowledge", serde_json::json!({
            "limit": 10
        })).await.expect("get_mature_knowledge call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: get_mature_knowledge ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_mature_knowledge - {:?}", response.error);
        }
        assert!(passed, "get_mature_knowledge should succeed");
        
        client.stop().await;
    }
}
