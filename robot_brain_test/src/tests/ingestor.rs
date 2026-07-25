//! Ingestor tool tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_importable() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("list_importable", serde_json::json!({
            "limit": 10
        })).await.expect("list_importable call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: list_importable ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: list_importable - {:?}", response.error);
        }
        assert!(passed, "list_importable should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_ingest_files() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("ingest_files", serde_json::json!({
            "limit": 0
        })).await.expect("ingest_files call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: ingest_files ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: ingest_files - {:?}", response.error);
        }
        assert!(passed, "ingest_files should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_list_ingested_files() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("list_ingested_files", serde_json::json!({
            "limit": 10
        })).await.expect("list_ingested_files call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: list_ingested_files ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: list_ingested_files - {:?}", response.error);
        }
        assert!(passed, "list_ingested_files should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_transcribe_audio() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("transcribe_audio", serde_json::json!({
            "path": "/nonexistent/audio.wav"
        })).await.expect("transcribe_audio call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: transcribe_audio ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: transcribe_audio - {:?}", response.error);
        }
        assert!(passed, "transcribe_audio should return response");
        
        client.stop().await;
    }
}
