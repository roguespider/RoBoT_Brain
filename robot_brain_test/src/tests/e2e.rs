//! End-to-end workflow tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_e2e_store_and_search_memory() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let store_response = client.call_tool("store_memory", serde_json::json!({
            "content": "E2E test memory for search",
            "memory_type": "note",
            "confidence": 0.99
        })).await.expect("store_memory failed");
        
        assert!(store_response.error.is_none(), "Store should succeed");
        
        let search_response = client.call_tool("search_memory", serde_json::json!({
            "query": "E2E test",
            "limit": 5
        })).await.expect("search_memory failed");
        
        assert!(search_response.error.is_none(), "Search should succeed");
        
        println!("✓ PASS: E2E store and search memory");
        client.stop().await;
    }

    #[tokio::test]
    async fn test_e2e_add_and_query_knowledge() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let _add_response = client.call_tool("add_knowledge", serde_json::json!({
            "statement": "E2E test knowledge",
            "knowledge_type": "fact",
            "confidence": 0.95
        })).await.expect("add_knowledge failed");
        
        let query_response = client.call_tool("query_knowledge", serde_json::json!({
            "query": "E2E test",
            "limit": 5
        })).await.expect("query_knowledge failed");
        
        assert!(query_response.error.is_none() || query_response.result.is_some(), "Query should work");
        
        println!("✓ PASS: E2E add and query knowledge");
        client.stop().await;
    }

    #[tokio::test]
    async fn test_e2e_workflow_lifecycle() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let _create_response = client.call_tool("create_workflow", serde_json::json!({
            "name": "E2E Test Workflow",
            "description": "Created by E2E integration test"
        })).await.expect("create_workflow failed");
        
        let list_response = client.call_tool("list_workflows", serde_json::json!({})).await.expect("list_workflows failed");
        
        assert!(list_response.error.is_none() || list_response.result.is_some(), "List should work");
        
        println!("✓ PASS: E2E workflow lifecycle");
        client.stop().await;
    }

    #[tokio::test]
    async fn test_e2e_hypothesis_workflow() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let _obs_response = client.call_tool("record_observation", serde_json::json!({
            "content": "E2E test observation",
            "context": "integration test",
            "observation_type": "success"
        })).await.expect("record_observation failed");
        
        let _hyp_response = client.call_tool("create_hypothesis", serde_json::json!({
            "statement": "E2E test hypothesis",
            "domain": "testing"
        })).await.expect("create_hypothesis failed");
        
        let list_response = client.call_tool("list_hypotheses", serde_json::json!({})).await.expect("list_hypotheses failed");
        
        assert!(list_response.error.is_none() || list_response.result.is_some(), "List should work");
        
        println!("✓ PASS: E2E hypothesis workflow");
        client.stop().await;
    }
}
