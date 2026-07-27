//! RoBoT Brain MCP Server Comprehensive Test Suite
//!
//! This test suite comprehensively tests ALL 57+ MCP tools available in the RoBoT Brain server.
//! It simulates real agent usage scenarios with success and failure cases.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::env;
use std::time::Duration;
use std::fs;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tokio::process::{Command as AsyncCommand, ChildStdout, Child};
use tokio::time::timeout;

mod test_environment;
mod tests;

use test_environment::TestEnvironment;

/// Test statistics
#[derive(Default)]
pub struct TestStats {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

impl TestStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(60));
        println!("TEST SUMMARY");
        println!("{}", "=".repeat(60));
        println!("  Passed:  {} ✅", self.passed);
        println!("  Failed:  {} ❌", self.failed);
        println!("  Skipped: {}", self.skipped);
        println!("{}", "=".repeat(60));
        
        if self.failed == 0 {
            println!("\n🎉 ALL TESTS PASSED! 🎉\n");
        } else {
            println!("\n⚠️  SOME TESTS FAILED\n");
        }
    }
}

/// Build the robot_brain server
async fn build_server() -> anyhow::Result<PathBuf> {
    println!("\n{}", "=".repeat(60));
    println!("BUILDING ROBOT_BRAIN SERVER");
    println!("{}", "=".repeat(60));
    
    let robot_brain_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let release_path = robot_brain_dir.join("target/release/robot_brain");
    
    if release_path.exists() {
        println!("✓ Server already built at: {}", release_path.display());
        return Ok(release_path);
    }
    
    println!("Building robot_brain...");
    
    let output = AsyncCommand::new("cargo")
        .current_dir(&robot_brain_dir)
        .args(["build", "--release"])
        .output()
        .await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to build robot_brain:\n{}", stderr);
    }
    
    println!("✓ Server built successfully: {}", release_path.display());
    Ok(release_path)
}

/// Setup test environment
fn setup_test_environment(server_path: &Path) -> anyhow::Result<TestEnvironment> {
    println!("\n{}", "=".repeat(60));
    println!("SETTING UP TEST ENVIRONMENT");
    println!("{}", "=".repeat(60));
    
    let test_dir = server_path.parent().unwrap()
        .join("robot_brain_test_env");
    
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    fs::create_dir_all(&test_dir)?;
    
    let test_server = test_dir.join("robot_brain");
    fs::copy(server_path, &test_server)?;
    
    // Create TestEnvironment first to get the files_folder path
    let test_env = TestEnvironment::new(test_dir, test_server);
    let files_folder = &test_env.files_folder;
    fs::create_dir_all(files_folder)?;
    
    let sample_files = [
        ("readme.txt", "This is a sample README file.\nIt contains important information."),
        ("notes.txt", "Meeting Notes - Project Planning\n\n1. Define requirements\n2. Design architecture"),
        ("todo.txt", "TODO List:\n- Write tests\n- Fix bugs\n- Deploy"),
        ("config.json", r#"{"name": "test", "version": "1.0.0"}"#),
        ("data.csv", "id,name,value\n1,alpha,100\n2,beta,200"),
    ];
    
    for (filename, content) in sample_files {
        fs::write(files_folder.join(filename), content)?;
    }
    
    println!("✓ Test directory: {}", test_env.root_dir.display());
    println!("✓ Server: {}", test_env.server_path.display());
    println!("✓ Files folder: {}", files_folder.display());
    
    Ok(test_env)
}

/// MCP Client wrapper for testing
pub struct TestMcpClient {
    /// The child process (kept alive to maintain the server)
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<ChildStdout>,
    send_id: u64,
}

impl TestMcpClient {
    pub async fn new(server_path: &Path) -> anyhow::Result<Self> {
        let mut child = AsyncCommand::new(server_path)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        
        let mut client = Self {
            child,
            stdin,
            stdout,
            send_id: 1,
        };
        
        client.send_request("initialize", serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "clientInfo": { "name": "robot_brain_test", "version": "1.0.0" }
        })).await?;
        
        client.read_response_line(5).await?;
        client.stdin.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\",\"params\":{}}\n").await?;
        
        client.send_request("tools/call", serde_json::json!({
            "name": "get_workflow",
            "arguments": {}
        })).await?;
        client.read_response_line(5).await?;
        
        println!("✓ MCP connection established");
        
        Ok(client)
    }
    
    async fn send_request(&mut self, method: &str, params: serde_json::Value) -> anyhow::Result<()> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.send_id,
            "method": method,
            "params": params
        });
        self.send_id += 1;
        let s = serde_json::to_string(&request)?;
        self.stdin.write_all(s.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        Ok(())
    }
    
    async fn read_response_line(&mut self, timeout_secs: u64) -> anyhow::Result<Option<String>> {
        let mut line = String::new();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);
        
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Ok(None);
            }
            
            match timeout(remaining, self.stdout.read_line(&mut line)).await {
                Ok(Ok(0)) => return Ok(None),
                Ok(Ok(_)) => {
                    let trimmed = line.trim();
                    if trimmed.starts_with('{') && trimmed.contains("\"jsonrpc\"") {
                        return Ok(Some(line.clone()));
                    }
                    line.clear();
                }
                Ok(Err(e)) => return Err(anyhow::anyhow!("Read error: {}", e)),
                Err(_) => return Ok(None),
            }
        }
    }
    
    pub async fn call_tool(&mut self, name: &str, arguments: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        self.send_request("tools/call", serde_json::json!({
            "name": name,
            "arguments": arguments
        })).await?;
        
        let response = self.read_response_line(10).await?
            .ok_or_else(|| anyhow::anyhow!("No response from server"))?;
        
        let json: serde_json::Value = serde_json::from_str(&response)?;
        
        if let Some(error) = json.get("error") {
            return Err(anyhow::anyhow!("Tool error: {:?}", error));
        }
        
        // Check if result contains success: false (tool execution error)
        if let Some(result) = json.get("result") {
            if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
                if let Some(text) = content.get("text") {
                    if let Ok(text_str) = text.as_str().ok_or_else(|| anyhow::anyhow!("Expected text")) {
                        if let Ok(content_json) = serde_json::from_str::<serde_json::Value>(text_str) {
                            if content_json.get("success").and_then(|s| s.as_bool()) == Some(false) {
                                let error_msg = content_json.get("error")
                                    .map(|e| e.to_string())
                                    .unwrap_or_else(|| "Unknown error".to_string());
                                return Err(anyhow::anyhow!("Tool returned error: {}", error_msg));
                            }
                        }
                    }
                }
            }
        }
        
        json.get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No result in response"))
    }
    
    /// Check if the child process is still running
    pub fn is_running(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }
    
    /// Get the child process PID (for debugging/logging)
    pub fn pid(&self) -> Option<u32> {
        self.child.id()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\n{}", "#".repeat(60));
    println!("#  RoBoT Brain MCP Server - Comprehensive Test Suite");
    println!("#  Testing all 57+ MCP tools with real agent scenarios");
    println!("{}", "#".repeat(60));
    
    let server_path = build_server().await?;
    let env = setup_test_environment(&server_path)?;
    let mut client = TestMcpClient::new(&env.server_path).await?;
    let mut stats = TestStats::new();
    
    println!("\n{}", "=".repeat(60));
    println!("RUNNING COMPREHENSIVE TOOL TESTS");
    println!("{}", "=".repeat(60));
    
    tests::run_memory_tests(&mut client, &mut stats, None).await?;
    tests::run_experience_tests(&mut client, &mut stats, None).await?;
    tests::run_knowledge_tests(&mut client, &mut stats, None).await?;
    tests::run_workflow_tests(&mut client, &mut stats, None).await?;
    tests::run_planner_tests(&mut client, &mut stats, None).await?;
    tests::run_hypothesis_tests(&mut client, &mut stats, None).await?;
    tests::run_reflection_tests(&mut client, &mut stats, None).await?;
    tests::run_search_tests(&mut client, &mut stats, None).await?;
    tests::run_ingestor_tests(&mut client, &mut stats, None, &env).await?;
    tests::run_agent_tests(&mut client, &mut stats, None).await?;
    tests::run_error_handling_tests(&mut client, &mut stats, None).await?;
    
    stats.print_summary();
    
    if stats.failed > 0 {
        std::process::exit(1);
    }
    
    Ok(())
}
