//! RoBoT Brain MCP Server Comprehensive Test Suite
//!
//! This test suite comprehensively tests ALL 57+ MCP tools available in the RoBoT Brain server.
//! It simulates real agent usage scenarios with success and failure cases.
//!
//! NEW: This suite now includes:
//! - Code analysis to detect stub patterns, #[allow(*)], unimplemented!(), todo!()
//! - True end-to-end testing without stubs or mocking
//! - Table-based reporting showing pass/fail for every function
//! - Detection of partial implementations and incomplete sub-functions

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::env;
use std::time::Duration;
use std::fs;
use std::io::Write;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tokio::process::{Command as AsyncCommand, ChildStdout, Child};
use tokio::time::timeout;

mod test_environment;
mod tests;
mod code_analyzer;
mod function_registry;
mod test_results;
mod comprehensive_test;

use test_environment::TestEnvironment;
use comprehensive_test::run_comprehensive_tests;

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
        println!("
{}", "=".repeat(60));
        println!("TEST SUMMARY");
        println!("{}", "=".repeat(60));
        println!("  Passed:  {} ✅", self.passed);
        println!("  Failed:  {} ❌", self.failed);
        println!("  Skipped: {}", self.skipped);
        println!("{}", "=".repeat(60));
        
        if self.failed == 0 {
            println!("
🎉 ALL TESTS PASSED! 🎉
");
        } else {
            println!("
⚠️  SOME TESTS FAILED
");
        }
    }
}

/// Build the robot_brain server
async fn build_server() -> anyhow::Result<PathBuf> {
    println!("
{}", "=".repeat(60));
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
        anyhow::bail!("Failed to build robot_brain:
{}", stderr);
    }
    
    println!("✓ Server built successfully: {}", release_path.display());
    Ok(release_path)
}

/// Setup test environment
fn setup_test_environment(server_path: &Path) -> anyhow::Result<TestEnvironment> {
    println!("
{}", "=".repeat(60));
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
        ("readme.txt", "This is a sample README file.
It contains important information."),
        ("notes.txt", "Meeting Notes - Project Planning

1. Define requirements
2. Design architecture"),
        ("todo.txt", "TODO List:
- Write tests
- Fix bugs
- Deploy"),
        ("config.json", r#"{"name": "test", "version": "1.0.0"}"#),
        ("data.csv", "id,name,value
1,alpha,100
2,beta,200"),
    ];
    
    for (filename, content) in sample_files {
        fs::write(files_folder.join(filename), content)?;
    }
    
    // Create subdirectories for testing recursive ingestion
    let subdirs = [
        "code_samples",
        "config_files",
        "documents",
        "archives",
    ];
    
    for subdir in subdirs {
        fs::create_dir_all(files_folder.join(subdir))?;
    }
    
    // ============================================
    // Create test files for EACH supported file type
    // ============================================
    
    // Text files (standard)
    fs::write(files_folder.join("sample.md"), "# Sample Markdown

This is a **test** file.

## Features
- Item 1
- Item 2
")?;
    fs::write(files_folder.join("sample.rst"), "Sample RST
==========

This is a reStructuredText file.

Section
-------

Content here.
")?;
    fs::write(files_folder.join("sample.log"), "[2024-01-01 10:00:00] INFO: Application started
[2024-01-01 10:00:01] DEBUG: Loading config
[2024-01-01 10:00:02] WARN: Optional module not found
")?;
    fs::write(files_folder.join("sample.xml"), "<?xml version=\"1.0\"?>
<root>
  <item id=\"1\">First item</item>
  <item id=\"2\">Second item</item>
</root>
")?;
    fs::write(files_folder.join("sample.html"), "<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
  <h1>Hello World</h1>
  <p>This is a test page.</p>
</body>
</html>
")?;
    
    // Code files
    fs::write(files_folder.join("code_samples/sample.rs"), "/// A sample Rust function
pub fn greet(name: &str) -> String {
    format!(\"Hello, {}!\", name)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_greet() {
        assert_eq!(greet(\"World\"), \"Hello, World!\");
    }
}
")?;
    fs::write(files_folder.join("code_samples/sample.py"), "def greet(name):
    \"\"\"Greet someone by name.\"\"\"
    return f\"Hello, {name}!\"

if __name__ == \"__main__\":
    print(greet(\"World\"))
")?;
    fs::write(files_folder.join("code_samples/sample.js"), "/**
 * Sample JavaScript function
 */
function greet(name) {
    return `Hello, ${name}!`;
}

console.log(greet(\"World\"));
")?;
    fs::write(files_folder.join("code_samples/sample.ts"), "/**
 * Sample TypeScript function
 */
function greet(name: string): string {
    return `Hello, ${name}!`;
}

console.log(greet(\"World\"));
")?;
    
    // Config files
    fs::write(files_folder.join("config_files/app.yaml"), "app:
  name: test-app
  version: 1.0.0
  debug: true

database:
  host: localhost
  port: 5432
")?;
    fs::write(files_folder.join("config_files/settings.ini"), "[DEFAULT]
app_name = TestApp
version = 1.0

[database]
host = localhost
port = 5432
")?;
    fs::write(files_folder.join("config_files/config.toml"), "[package]
name = \"test-crate\"
version = \"0.1.0\"

[dependencies]
serde = \"1.0\"
tokio = { version = \"1.0\", features = [\"full\"] }
")?;
    
    // Scripts
    fs::write(files_folder.join("code_samples/script.sh"), "#!/bin/bash
# Sample shell script
echo \"Hello from shell script!\"
for i in 1 2 3; do
    echo \"Count: $i\"
done
")?;
    fs::write(files_folder.join("code_samples/query.sql"), "-- Sample SQL query
SELECT users.name, COUNT(orders.id) as order_count
FROM users
LEFT JOIN orders ON users.id = orders.user_id
GROUP BY users.id
ORDER BY order_count DESC;
")?;
    
    // Data formats
    fs::write(files_folder.join("config_files/data.json"), serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob", "email": "bob@example.com"}
        ],
        "total": 2
    }).to_string())?;
    fs::write(files_folder.join("config_files/data.jsonl"), "{\"id\":1,\"action\":\"start\",\"timestamp\":\"2024-01-01T10:00:00Z\"}
{\"id\":2,\"action\":\"stop\",\"timestamp\":\"2024-01-01T10:05:00Z\"}
{\"id\":3,\"action\":\"restart\",\"timestamp\":\"2024-01-01T10:10:00Z\"}
")?;
    fs::write(files_folder.join("config_files/data.csv"), "id,name,score,active
1,Alice,95,true
2,Bob,87,false
3,Charlie,92,true
")?;
    
    // Subtitles
    fs::write(files_folder.join("sample.srt"), "1
00:00:00,000 --> 00:00:02,500
Hello, this is the first subtitle.

2
00:00:03,000 --> 00:00:06,000
This is the second subtitle line.

3
00:00:07,000 --> 00:00:10,000
And this is the third subtitle.
")?;
    
    // Image metadata test (we can't create actual images, but we create SVG)
    fs::write(files_folder.join("sample.svg"), "<?xml version=\"1.0\"?>
<svg width=\"200\" height=\"100\" viewBox=\"0 0 200 100\" xmlns=\"http://www.w3.org/2000/svg\">
  <rect x=\"10\" y=\"10\" width=\"180\" height=\"80\" fill=\"blue\" />
  <circle cx=\"100\" cy=\"50\" r=\"30\" fill=\"red\" />
  <text x=\"100\" y=\"55\" text-anchor=\"middle\" fill=\"white\">Test SVG</text>
</svg>
")?;
    
    // Create a minimal ZIP archive for testing
    let zip_path = files_folder.join("archives/test.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip_writer = zip::ZipWriter::new(zip_file);
    zip_writer.start_file("inside.txt", zip::write::SimpleFileOptions::default())?;
    zip_writer.write_all(b"This file is inside the ZIP archive.
It should be extracted and ingested.
")?;
    zip_writer.finish()?;
    
    // Create a simple tar.gz archive for testing
    let tar_gz_path = files_folder.join("archives/test.tar.gz");
    let tar_gz_file = fs::File::create(&tar_gz_path)?;
    let enc = flate2::write::GzEncoder::new(tar_gz_file, flate2::Compression::default());
    let mut tar_builder = tar::Builder::new(enc);
    let tar_content = "This file is inside the TAR.GZ archive.";
    let mut header = tar::Header::new_gnu();
    header.set_path("inside_tar.txt")?;
    header.set_size(tar_content.len() as u64);
    header.set_cksum();
    tar_builder.append(&header, tar_content.as_bytes())?;
    tar_builder.finish()?;
    
    println!("✓ Created {} test subdirectories", subdirs.len());
    println!("✓ Created test files for all supported file types:");
    println!("  - Text files: txt, md, rst, log, xml, html");
    println!("  - Code files: rs, py, js, ts");
    println!("  - Config files: yaml, ini, toml, json, jsonl, csv");
    println!("  - Scripts: sh, sql");
    println!("  - Subtitles: srt");
    println!("  - Images: svg (metadata only)");
    println!("  - Archives: zip, tar.gz");
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
        client.stdin.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\",\"params\":{}}
").await?;
        
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
        self.stdin.write_all(b"
").await?;
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
    println!("
{}", "#".repeat(100));
    println!("#  RoBoT Brain MCP Server - Comprehensive End-to-End Test Suite");
    println!("#  Testing every function 100% end-to-end without stubs or #[allow(*)]");
    println!("{}", "#".repeat(100));
    
    let server_path = build_server().await?;
    let env = setup_test_environment(&server_path)?;
    let mut client = TestMcpClient::new(&env.server_path).await?;
    let mut stats = TestStats::new();
    
    // Run comprehensive test suite with code analysis
    let report = run_comprehensive_tests(&mut client, &mut stats, &env).await?;
    
    // Also run the traditional test suite for comparison
    println!("\n
{}", "=".repeat(100));
    println!("RUNNING TRADITIONAL TEST SUITE (for comparison)");
    println!("{}", "=".repeat(100));
    
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
    
    // Run MCP Workflow Integration Tests
    println!("\n
{}", "=".repeat(100));
    println!("RUNNING MCP WORKFLOW INTEGRATION TESTS");
    println!("{}", "=".repeat(100));
    
    tests::run_mcp_workflow_tests(&mut client, &mut stats, None).await?;
    
    stats.print_summary();
    
    // Print combined issues summary
    println!("\n{}", "=".repeat(80));
    println!("OVERALL QUALITY SUMMARY");
    println!("{}", "=".repeat(80));
    println!("  Code Quality Issues:    {:>6} (#[allow, stubs, etc.)", report.code_issues.len());
    println!("  Lint Errors:           {:>6}", report.lint_errors);
    println!("  Lint Warnings:         {:>6}", report.lint_warnings);
    println!("  Test Failures:         {:>6}", stats.failed);
    println!("{}", "=".repeat(80));
    
    // Exit with error if there are issues
    if report.has_issues() || stats.failed > 0 || report.lint_errors > 0 {
        println!("\n⚠️  TEST SUITE COMPLETED WITH ISSUES");
        std::process::exit(1);
    }
    
    println!("\n🎉 ALL TESTS PASSED - SYSTEM READY!");
    Ok(())
}
