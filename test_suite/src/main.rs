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
//!
//! OUTPUT: All test results are automatically saved to `test_suite_output.txt`

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdout, Command as AsyncCommand};
use tokio::sync::Mutex;
use tokio::time::timeout;

// Shared output module - provides teeprintln macro for all modules
pub mod output;

mod code_analyzer;
mod comprehensive_test;
mod function_registry;
mod paths;
mod test_environment;
mod test_results;
mod tests;

use comprehensive_test::run_comprehensive_tests;
use test_environment::TestEnvironment;

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
        teeprintln!(
            "
{}",
            "=".repeat(60)
        );
        teeprintln!("TEST SUMMARY");
        teeprintln!("{}", "=".repeat(60));
        teeprintln!("  Passed:  {} [OK]", self.passed);
        teeprintln!("  Failed:  {} [FAIL]", self.failed);
        teeprintln!("  Skipped: {}", self.skipped);
        teeprintln!("{}", "=".repeat(60));

        if self.failed == 0 {
            teeprintln!(
                "
[DONE] ALL TESTS PASSED! [DONE]
"
            );
        } else {
            teeprintln!(
                "
[WARN] SOME TESTS FAILED
"
            );
        }
    }
}

/// Resolve the server binary path at runtime (see `paths::server_binary`).
fn find_server_binary(robot_brain_dir: &Path) -> Option<PathBuf> {
    let native = if cfg!(windows) {
        "robot_brain.exe"
    } else {
        "robot_brain"
    };
    let primary = robot_brain_dir.join("target/release").join(native);
    if primary.exists() {
        return Some(primary);
    }
    // Cross-platform fallbacks for unusual build outputs.
    let fallbacks = [
        robot_brain_dir.join("target/release/robot_brain"),
        robot_brain_dir.join("target/release/robot_brain.exe"),
    ];
    fallbacks.into_iter().find(|path| path.exists())
}

/// Build the robot_brain server
async fn build_server() -> anyhow::Result<PathBuf> {
    teeprintln!(
        "
{}",
        "=".repeat(60)
    );
    teeprintln!("BUILDING ROBOT_BRAIN SERVER");
    teeprintln!("{}", "=".repeat(60));

    let robot_brain_dir = paths::project_root();

    // ALWAYS rebuild robot_brain, even if the binary already exists. Cargo's
    // incremental compilation makes the no-op case (no source changes) fast,
    // but skipping the build when the binary exists means the gate tests
    // against a STALE binary and never sees source changes — letting
    // violations (unwrap, cfg-test, warnings) slip through undetected. The
    // gate's value depends on testing the code as it currently is, not as it
    // was at some past build.
    teeprintln!("Cleaning robot_brain build artifacts (cargo clean)...");
    let clean_output = AsyncCommand::new("cargo")
        .current_dir(&robot_brain_dir)
        .args(["clean"])
        .output()
        .await?;

    if !clean_output.status.success() {
        let stderr = String::from_utf8_lossy(&clean_output.stderr);
        anyhow::bail!(
            "Failed to clean robot_brain:
{}",
            stderr
        );
    }

    teeprintln!("Rebuilding robot_brain (cargo build --release)...");

    let output = AsyncCommand::new("cargo")
        .current_dir(&robot_brain_dir)
        .args(["build", "--release"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Failed to build robot_brain:
{}",
            stderr
        );
    }

    teeprintln!("[OK] robot_brain rebuilt");

    // Find the binary after build
    find_server_binary(&robot_brain_dir).ok_or_else(|| {
        anyhow::anyhow!("Built successfully but server binary not found in target/release/")
    })
}

/// Create a minimal valid WAV file for testing transcription
fn create_minimal_wav() -> Vec<u8> {
    let sample_rate: u32 = 8000;
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let duration_secs: u32 = 1;
    let num_samples = sample_rate * duration_secs;
    let data_size = num_samples * u32::from(bits_per_sample / 8);

    let mut wav = Vec::new();

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size).to_le_bytes()); // File size - 8
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // Audio format (PCM)
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(
        &(sample_rate * u32::from(num_channels) * u32::from(bits_per_sample / 8)).to_le_bytes(),
    ); // Byte rate
    wav.extend_from_slice(&(num_channels * bits_per_sample / 8).to_le_bytes()); // Block align
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    // Audio data (silence - all zeros)
    wav.resize(wav.len() + data_size as usize, 0);

    wav
}

/// Setup test environment
fn setup_test_environment(server_path: &Path) -> anyhow::Result<TestEnvironment> {
    teeprintln!(
        "
{}",
        "=".repeat(60)
    );
    teeprintln!("SETTING UP TEST ENVIRONMENT");
    teeprintln!("{}", "=".repeat(60));

    let test_dir = server_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Server path has no parent directory"))?
        .join("test_suite_env");

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
        (
            "readme.txt",
            "This is a sample README file.
It contains important information.",
        ),
        (
            "notes.txt",
            "Meeting Notes - Project Planning

1. Define requirements
2. Design architecture",
        ),
        (
            "todo.txt",
            "TODO List:
- Write tests
- Fix bugs
- Deploy",
        ),
        ("config.json", r#"{"name": "test", "version": "1.0.0"}"#),
        (
            "data.csv",
            "id,name,value
1,alpha,100
2,beta,200",
        ),
    ];

    for (filename, content) in sample_files {
        fs::write(files_folder.join(filename), content)?;
    }

    // Create subdirectories for testing recursive ingestion
    let subdirs = ["code_samples", "config_files", "documents", "archives"];

    for subdir in subdirs {
        fs::create_dir_all(files_folder.join(subdir))?;
    }

    // ============================================
    // Create test files for EACH supported file type
    // ============================================

    // Text files (standard)
    fs::write(
        files_folder.join("sample.md"),
        "# Sample Markdown

This is a **test** file.

## Features
- Item 1
- Item 2
",
    )?;
    fs::write(
        files_folder.join("sample.rst"),
        "Sample RST
==========

This is a reStructuredText file.

Section
-------

Content here.
",
    )?;
    fs::write(
        files_folder.join("sample.log"),
        "[2024-01-01 10:00:00] INFO: Application started
[2024-01-01 10:00:01] DEBUG: Loading config
[2024-01-01 10:00:02] WARN: Optional module not found
",
    )?;
    fs::write(
        files_folder.join("sample.xml"),
        "<?xml version=\"1.0\"?>
<root>
  <item id=\"1\">First item</item>
  <item id=\"2\">Second item</item>
</root>
",
    )?;
    fs::write(
        files_folder.join("sample.html"),
        "<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
  <h1>Hello World</h1>
  <p>This is a test page.</p>
</body>
</html>
",
    )?;

    // Code files
    fs::write(
        files_folder.join("code_samples/sample.rs"),
        "/// A sample Rust function
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
",
    )?;
    fs::write(
        files_folder.join("code_samples/sample.py"),
        "def greet(name):
    \"\"\"Greet someone by name.\"\"\"
    return f\"Hello, {name}!\"

if __name__ == \"__main__\":
    print(greet(\"World\"))
",
    )?;
    fs::write(
        files_folder.join("code_samples/sample.js"),
        "/**
 * Sample JavaScript function
 */
function greet(name) {
    return `Hello, ${name}!`;
}

console.log(greet(\"World\"));
",
    )?;
    fs::write(
        files_folder.join("code_samples/sample.ts"),
        "/**
 * Sample TypeScript function
 */
function greet(name: string): string {
    return `Hello, ${name}!`;
}

console.log(greet(\"World\"));
",
    )?;

    // Config files
    fs::write(
        files_folder.join("config_files/app.yaml"),
        "app:
  name: test-app
  version: 1.0.0
  debug: true

database:
  host: localhost
  port: 5432
",
    )?;
    fs::write(
        files_folder.join("config_files/settings.ini"),
        "[DEFAULT]
app_name = TestApp
version = 1.0

[database]
host = localhost
port = 5432
",
    )?;
    fs::write(
        files_folder.join("config_files/config.toml"),
        "[package]
name = \"test-crate\"
version = \"0.1.0\"

[dependencies]
serde = \"1.0\"
tokio = { version = \"1.0\", features = [\"full\"] }
",
    )?;

    // Scripts
    fs::write(
        files_folder.join("code_samples/script.sh"),
        "#!/bin/bash
# Sample shell script
echo \"Hello from shell script!\"
for i in 1 2 3; do
    echo \"Count: $i\"
done
",
    )?;
    fs::write(
        files_folder.join("code_samples/query.sql"),
        "-- Sample SQL query
SELECT users.name, COUNT(orders.id) as order_count
FROM users
LEFT JOIN orders ON users.id = orders.user_id
GROUP BY users.id
ORDER BY order_count DESC;
",
    )?;

    // Data formats
    fs::write(
        files_folder.join("config_files/data.json"),
        serde_json::json!({
            "users": [
                {"id": 1, "name": "Alice", "email": "alice@example.com"},
                {"id": 2, "name": "Bob", "email": "bob@example.com"}
            ],
            "total": 2
        })
        .to_string(),
    )?;
    fs::write(
        files_folder.join("config_files/data.jsonl"),
        "{\"id\":1,\"action\":\"start\",\"timestamp\":\"2024-01-01T10:00:00Z\"}
{\"id\":2,\"action\":\"stop\",\"timestamp\":\"2024-01-01T10:05:00Z\"}
{\"id\":3,\"action\":\"restart\",\"timestamp\":\"2024-01-01T10:10:00Z\"}
",
    )?;
    fs::write(
        files_folder.join("config_files/data.csv"),
        "id,name,score,active
1,Alice,95,true
2,Bob,87,false
3,Charlie,92,true
",
    )?;

    // Subtitles
    fs::write(
        files_folder.join("sample.srt"),
        "1
00:00:00,000 --> 00:00:02,500
Hello, this is the first subtitle.

2
00:00:03,000 --> 00:00:06,000
This is the second subtitle line.

3
00:00:07,000 --> 00:00:10,000
And this is the third subtitle.
",
    )?;

    // Image metadata test (we can't create actual images, but we create SVG)
    fs::write(
        files_folder.join("sample.svg"),
        "<?xml version=\"1.0\"?>
<svg width=\"200\" height=\"100\" viewBox=\"0 0 200 100\" xmlns=\"http://www.w3.org/2000/svg\">
  <rect x=\"10\" y=\"10\" width=\"180\" height=\"80\" fill=\"blue\" />
  <circle cx=\"100\" cy=\"50\" r=\"30\" fill=\"red\" />
  <text x=\"100\" y=\"55\" text-anchor=\"middle\" fill=\"white\">Test SVG</text>
</svg>
",
    )?;

    // Audio file for transcription test (minimal WAV header with silence)
    // Create a minimal valid 16-bit mono 8kHz WAV file with 1 second of silence
    let wav_data = create_minimal_wav();
    fs::write(files_folder.join("sample.wav"), wav_data)?;

    // Create a minimal ZIP archive for testing
    let zip_path = files_folder.join("archives/test.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip_writer = zip::ZipWriter::new(zip_file);
    zip_writer.start_file("inside.txt", zip::write::SimpleFileOptions::default())?;
    zip_writer.write_all(
        b"This file is inside the ZIP archive.
It should be extracted and ingested.
",
    )?;
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

    teeprintln!("[OK] Created {} test subdirectories", subdirs.len());
    teeprintln!("[OK] Created test files for all supported file types:");
    teeprintln!("  - Text files: txt, md, rst, log, xml, html");
    teeprintln!("  - Code files: rs, py, js, ts");
    teeprintln!("  - Config files: yaml, ini, toml, json, jsonl, csv");
    teeprintln!("  - Scripts: sh, sql");
    teeprintln!("  - Subtitles: srt");
    teeprintln!("  - Images: svg (metadata only)");
    teeprintln!("  - Archives: zip, tar.gz");
    teeprintln!("[OK] Test directory: {}", test_env.root_dir.display());
    teeprintln!("[OK] Server: {}", test_env.server_path.display());
    teeprintln!("[OK] Files folder: {}", files_folder.display());

    Ok(test_env)
}

/// A ring buffer of recent server log lines captured from stderr.
///
/// The server (robot_brain) writes all `tracing` output to stderr. Capturing
/// it lets us attach server-side context to failing test results, turning a
/// bare "Tool returned error: X" into "X | server: WARN ...".
pub type ServerLogBuffer = Arc<Mutex<std::collections::VecDeque<String>>>;

/// Maximum number of log lines retained in the ring buffer.
const SERVER_LOG_BUFFER_CAPACITY: usize = 500;

/// Build a shared server-log ring buffer.
fn new_server_log_buffer() -> ServerLogBuffer {
    Arc::new(Mutex::new(std::collections::VecDeque::with_capacity(
        SERVER_LOG_BUFFER_CAPACITY,
    )))
}

/// MCP Client wrapper for testing
pub struct TestMcpClient {
    /// The child process (kept alive to maintain the server)
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<ChildStdout>,
    send_id: u64,
    /// Recent server log lines captured from stderr (shared with a background reader task).
    server_logs: ServerLogBuffer,
}

impl TestMcpClient {
    pub async fn new(server_path: &Path) -> anyhow::Result<Self> {
        let mut child = AsyncCommand::new(server_path)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| {
            anyhow::anyhow!("Failed to take stdin - process may not have been spawned correctly")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            anyhow::anyhow!("Failed to take stdout - process may not have been spawned correctly")
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            anyhow::anyhow!("Failed to take stderr - process may not have been spawned correctly")
        })?;

        let server_logs = new_server_log_buffer();
        // Spawn a background task that continuously reads stderr lines into the
        // ring buffer so we always have recent server-side context available.
        spawn_stderr_reader(stderr, server_logs.clone());

        let mut client = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            send_id: 1,
            server_logs,
        };

        client
            .send_request(
                "initialize",
                serde_json::json!({
                    "protocolVersion": "2025-03-26",
                    "capabilities": { "tools": {} },
                    "clientInfo": { "name": "test_suite", "version": "1.0.0" }
                }),
            )
            .await?;

        client.read_response_line(5).await?;
        client
            .stdin
            .write_all(
                b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\",\"params\":{}}
",
            )
            .await?;

        client
            .send_request(
                "tools/call",
                serde_json::json!({
                    "name": "get_workflow",
                    "arguments": {}
                }),
            )
            .await?;
        client.read_response_line(5).await?;

        teeprintln!("[OK] MCP connection established");

        Ok(client)
    }

    async fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<()> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.send_id,
            "method": method,
            "params": params
        });
        self.send_id += 1;
        let s = serde_json::to_string(&request)?;
        self.stdin.write_all(s.as_bytes()).await?;
        self.stdin
            .write_all(
                b"
",
            )
            .await?;
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

    pub async fn call_tool(
        &mut self,
        name: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        self.send_request(
            "tools/call",
            serde_json::json!({
                "name": name,
                "arguments": arguments
            }),
        )
        .await?;

        let response = self
            .read_response_line(10)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No response from server"))?;

        let json: serde_json::Value = serde_json::from_str(&response)?;

        if let Some(error) = json.get("error") {
            return Err(anyhow::anyhow!("Tool error: {:?}", error));
        }

        // Check for isError field in the result (MCP error response format).
        // IMPORTANT: an isError result is still a valid tool RESPONSE - the
        // runner's validation layer decides whether an error was expected.
        // We surface it as a parsed JSON object with the isError flag intact
        // instead of collapsing it into Err, which would bypass validation.
        if let Some(result) = json.get("result") {
            let is_error = result
                .get("isError")
                .and_then(|e| e.as_bool())
                .unwrap_or(false);

            // Extract the first content text payload (if any) so we can parse
            // the structured tool output embedded in the content block.
            let content_json = result
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|item| item.get("text"))
                .and_then(|t| t.as_str())
                .and_then(|text_str| serde_json::from_str::<serde_json::Value>(text_str).ok());

            if let Some(mut parsed) = content_json {
                // Preserve the MCP-level error flag on the parsed payload so
                // validation::is_success can treat it as success=false.
                if is_error {
                    parsed["isError"] = serde_json::json!(true);
                    if parsed.get("success").is_none() {
                        parsed["success"] = serde_json::json!(false);
                    }
                }
                return Ok(parsed);
            }

            if is_error {
                // isError with no parseable content: synthesize a failure
                // payload so expected-error validations still match.
                return Ok(serde_json::json!({
                    "success": false,
                    "isError": true,
                    "error": "Tool returned an error response with no content"
                }));
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

    /// Get protocol info (returns server info if available)
    pub fn get_protocol_info(&self) -> Option<String> {
        Some("MCP Protocol: 2025-03-26".to_string())
    }

    /// List all available tools (MCP protocol: tools/list)
    pub async fn list_tools(&mut self) -> anyhow::Result<Vec<serde_json::Value>> {
        self.send_request("tools/list", serde_json::json!({}))
            .await?;

        let response = self
            .read_response_line(10)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No response from server"))?;

        let json: serde_json::Value = serde_json::from_str(&response)?;

        if let Some(error) = json.get("error") {
            return Err(anyhow::anyhow!("Tool error: {:?}", error));
        }

        let result = json
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No result in response"))?;

        let tools = result
            .get("tools")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(tools)
    }

    /// Retrieve the most recent `count` server log lines (from stderr).
    ///
    /// Returns lines oldest-first. Used to attach server-side context to
    /// failing test results so diagnosis doesn't require a separate log hunt.
    pub async fn recent_server_logs(&self, count: usize) -> Vec<String> {
        let buf = self.server_logs.lock().await;
        let len = buf.len();
        let start = len.saturating_sub(count);
        buf.iter().skip(start).cloned().collect()
    }

    /// Retrieve all server log lines containing `needle` (case-insensitive),
    /// plus one line of surrounding context when available.
    pub async fn server_logs_matching(&self, needle: &str) -> Vec<String> {
        let needle_lower = needle.to_lowercase();
        let buf = self.server_logs.lock().await;
        let lines: Vec<&String> = buf.iter().collect();
        let mut matches = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            if line.to_lowercase().contains(&needle_lower) {
                let mut entry = String::new();
                if idx > 0 {
                    entry.push_str(lines[idx - 1].as_str());
                    entry.push('\n');
                }
                entry.push_str(line.as_str());
                if idx + 1 < lines.len() {
                    entry.push('\n');
                    entry.push_str(lines[idx + 1].as_str());
                }
                matches.push(entry);
            }
        }
        matches
    }
}

/// Background task that continuously reads server stderr into a ring buffer.
///
/// This keeps the most recent `SERVER_LOG_BUFFER_CAPACITY` log lines available
/// for attaching to failing test results. Older lines are evicted automatically
/// by the `VecDeque` capacity bound.
fn spawn_stderr_reader(mut stderr: tokio::process::ChildStderr, buffer: ServerLogBuffer) {
    tokio::spawn(async move {
        let mut reader = BufReader::new(&mut stderr);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF: server closed stderr
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let mut buf = buffer.lock().await;
                    if buf.len() >= SERVER_LOG_BUFFER_CAPACITY {
                        buf.pop_front();
                    }
                    buf.push_back(trimmed.to_string());
                }
                Err(e) => {
                    // Read error — surface it once then stop the reader.
                    let mut buf = buffer.lock().await;
                    if buf.len() >= SERVER_LOG_BUFFER_CAPACITY {
                        buf.pop_front();
                    }
                    buf.push_back(format!("[test_suite stderr reader error: {}]", e));
                    break;
                }
            }
        }
    });
}

/// `test_suite --list`: quick smoke check. Connects to the live server,
/// lists every advertised tool, and prints the count. This is the fastest
/// way to confirm the server is alive and discoverable.
async fn run_list(server_path: &Path) -> anyhow::Result<()> {
    let mut client = TestMcpClient::new(server_path).await?;
    let tools = client.list_tools().await?;
    println!("=== robot_brain tool list ({}) ===", tools.len());
    for t in &tools {
        let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let desc = t
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("");
        let required = t
            .get("inputSchema")
            .and_then(|s| s.get("required"))
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        println!("  {:40} {}", name, desc);
        if !required.is_empty() {
            println!("    required: {required}");
        }
    }
    println!("=== {} tools ===", tools.len());
    Ok(())
}

/// `test_suite --probe TOOL`: introspect a single tool's live inputSchema.
///
/// Connects to the running server, fetches the tool's JSON schema, and prints
/// its required and optional fields with types. This is the live equivalent of
/// the Python `RobotBrainClient.list_tools()` introspection — it lets you
/// discover the exact parameters a tool expects without guessing.
async fn run_probe(server_path: &Path, tool_name: &str) -> anyhow::Result<()> {
    let mut client = TestMcpClient::new(server_path).await?;
    let tools = client.list_tools().await?;
    let tool = tools
        .iter()
        .find(|t| t.get("name").and_then(|v| v.as_str()) == Some(tool_name))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Tool '{}' not found. Server advertises {} tools. Use `test_suite --list` to see them.",
                tool_name,
                tools.len()
            )
        })?;

    println!("=== probe: {} ===", tool_name);
    if let Some(desc) = tool.get("description").and_then(|v| v.as_str()) {
        println!("description: {desc}");
    }

    let schema = tool
        .get("inputSchema")
        .cloned()
        .unwrap_or(serde_json::json!({}));
    let required: Vec<String> = schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let properties = schema
        .get("properties")
        .and_then(|p| p.as_object())
        .cloned()
        .unwrap_or_default();

    println!("\nparameters ({}):", properties.len());
    let mut entries: Vec<(&String, &serde_json::Value)> = properties.iter().collect();
    entries.sort_by(|a, b| {
        let ar = required.contains(a.0);
        let br = required.contains(b.0);
        br.cmp(&ar).then(a.0.cmp(b.0))
    });
    for (name, spec) in entries {
        let is_required = required.contains(name);
        let typ = spec
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let marker = if is_required { "*" } else { " " };
        let mut line = format!("  {marker} {name}: {typ}");
        if let Some(items) = spec
            .get("items")
            .and_then(|i| i.get("type"))
            .and_then(|v| v.as_str())
        {
            line.push_str(&format!("<{items}>"));
        }
        if let Some(desc) = spec.get("description").and_then(|v| v.as_str()) {
            let short = desc.lines().next().unwrap_or(desc);
            if !short.is_empty() {
                line.push_str(&format!(" — {short}"));
            }
        }
        println!("{line}");
    }
    if !required.is_empty() {
        println!("\n(* = required)");
    }
    Ok(())
}

/// `test_suite --gate`: enforce the AGENTS.md quality wall on the most
/// recent JSON report. Reads `test_suite_report.json` and exits 0 only if
/// all tests passed AND compiler_warnings == 0 AND code_issues == 0 AND
/// untested_tools == 0. Replaces the old Python `gate_quality.py`.
fn run_gate() -> anyhow::Result<()> {
    let report_path = paths::test_suite_dir().join("test_suite_report.json");
    if !report_path.exists() {
        eprintln!(
            "QUALITY WALL RED: test_suite report not found at {}",
            report_path.display()
        );
        eprintln!("Run `test_suite` (full suite) first to generate the report.");
        std::process::exit(1);
    }

    let json = std::fs::read_to_string(&report_path)?;
    let report: serde_json::Value = serde_json::from_str(&json)?;

    let summary = report
        .get("summary")
        .ok_or_else(|| anyhow::anyhow!("report missing 'summary' key"))?;

    let passed = summary.get("passed").and_then(|v| v.as_u64()).unwrap_or(0);
    let failed = summary.get("failed").and_then(|v| v.as_u64()).unwrap_or(0)
        + summary.get("errors").and_then(|v| v.as_u64()).unwrap_or(0);
    let warnings = summary
        .get("compiler_warnings")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let code_issues = summary
        .get("code_issues")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let untested = summary
        .get("untested_tools")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let red = "\x1b[31m";
    let green = "\x1b[32m";
    let rst = "\x1b[0m";

    let checks = [
        (
            "tests",
            failed == 0,
            format!("passed={}, failed/err={}", passed, failed),
        ),
        (
            "compiler_warnings",
            warnings == 0,
            format!("actual={}", warnings),
        ),
        (
            "code_issues",
            code_issues == 0,
            format!("actual={}", code_issues),
        ),
        (
            "untested_tools",
            untested == 0,
            format!("actual={}", untested),
        ),
    ];

    let mut bad = Vec::new();
    for (name, ok, detail) in &checks {
        let (color, verdict) = if *ok {
            (green, "OK")
        } else {
            (red, "VIOLATION")
        };
        println!("  {color}{name:<20} {detail:<30} {verdict}{rst}");
        if !ok {
            bad.push(*name);
        }
    }

    if bad.is_empty() {
        println!("{green}QUALITY WALL OK (0 warnings, 0 code-issues, 0 untested tools){rst}");
        Ok(())
    } else {
        eprintln!("\n{red}QUALITY WALL RED: {}{rst}", bad.join("; "));
        eprintln!(
            "Fix per AGENTS.md: wire the dead-code pub API into a real caller. \
             Do NOT use #[allow] or `_` to silence."
        );
        std::process::exit(1);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // CLI: `test_suite`            → full suite (default)
    //      `test_suite --probe TOOL` → introspect one tool's live inputSchema
    //      `test_suite --list`       → list all server tools (quick smoke check)
    //      `test_suite --gate`       → enforce quality wall on last JSON report
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--gate") {
        return run_gate();
    }

    let server_path = build_server().await?;

    if let Some(idx) = args.iter().position(|a| a == "--probe") {
        let tool = args.get(idx + 1).ok_or_else(|| {
            anyhow::anyhow!(
                "--probe requires a tool name, e.g. `test_suite --probe register_agent`"
            )
        })?;
        return run_probe(&server_path, tool).await;
    }
    if args.iter().any(|a| a == "--list") {
        return run_list(&server_path).await;
    }

    // Initialize file output - all output will be written to both stdout and file
    let output_file = paths::test_suite_dir().join("test_suite_output.txt");
    output::init(&output_file)
        .map_err(|e| anyhow::anyhow!("Failed to create output file: {}", e))?;

    teeprintln!(
        "
{}",
        "#".repeat(120)
    );
    teeprintln!("#  RoBoT Brain MCP Server - Comprehensive End-to-End Test Suite");
    teeprintln!("#  Testing every function 100% end-to-end");
    teeprintln!("#  Output saved to: {}", output_file.display());
    teeprintln!("{}", "#".repeat(120));

    let server_path = build_server().await?;
    let env = setup_test_environment(&server_path)?;
    let mut client = TestMcpClient::new(&env.server_path).await?;
    let mut stats = TestStats::new();

    // Run comprehensive test suite with code analysis
    let report = run_comprehensive_tests(&mut client, &mut stats, &env).await?;

    // Also run the traditional test suite
    teeprintln!(
        "\n
{}",
        "=".repeat(120)
    );
    teeprintln!("RUNNING ALL TESTS");
    teeprintln!("{}", "=".repeat(120));

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
    tests::run_mcp_workflow_tests(&mut client, &mut stats, None).await?;

    // Run new comprehensive tests
    tests::run_rmcp_tests(&mut client, &mut stats, None).await?;
    tests::run_acp_tests(&mut client, &mut stats, None).await?;
    tests::run_agent_simulation_tests(&mut client, &mut stats, None).await?;

    // T1-10: verify the SQLite JobQueue survives a process restart.
    tests::queue_durability::run_queue_durability_tests(&mut stats).await?;

    // T1-10B-10: migrated finding new+promote unit test (MCP-based).
    tests::exploration_finding::run_exploration_finding_tests(&mut client, &mut stats).await?;

    // T1-10B-11: migrated observations record+list unit test (MCP-based).
    tests::observations::run_observations_tests(&mut client, &mut stats).await?;

    // T1-10B-09: migrated attempt builder+success/failure unit test (MCP-based).
    tests::exploration_attempt::run_exploration_attempt_tests(&mut client, &mut stats).await?;

    // T1-10B-08: migrated hypothesis lifecycle+clamp unit test (MCP-based).
    tests::exploration_hypothesis::run_exploration_hypothesis_tests(&mut client, &mut stats)
        .await?;

    // T1-10B-04: migrated knowledge store add+get+mature unit test (MCP-based).
    tests::knowledge_store::run_knowledge_store_tests(&mut client, &mut stats).await?;

    // T1-10B-05: migrated knowledge query text/confidence/ranking unit test (MCP-based).
    tests::knowledge_query::run_knowledge_query_tests(&mut client, &mut stats).await?;

    // T1-10B-06: migrated memory retrieval working+unified unit test (MCP-based).
    tests::memory_retrieval::run_memory_retrieval_tests(&mut client, &mut stats).await?;

    // T1-10B-07: migrated audio transcriber is_audio_file + extensions test (MCP-based).
    tests::audio_transcriber::run_audio_transcriber_tests(&mut client, &mut stats).await?;

    // T1-10B-20: migrated embeddings get+delete by memory_id test (MCP-based).
    tests::embeddings::run_embeddings_tests(&mut client, &mut stats).await?;

    // T1-10B-17: migrated semantic chunker markdown+code parsing test (MCP-based).
    tests::semantic_chunker::run_semantic_chunker_tests(&mut client, &mut stats).await?;

    // T1-10B-01: migrated personality defaults/preset/traits/decision (MCP-based).
    tests::personality::run_personality_tests(&mut client, &mut stats).await?;

    // Run CLI-based tool tests (tests robot_brain CLI subcommands)
    teeprintln!(
        "
{}",
        "=".repeat(120)
    );
    teeprintln!("RUNNING CLI TOOL TESTS");
    teeprintln!("{}", "=".repeat(120));
    let cli_results = tests::cli_tools::run_cli_tool_tests().await;
    for cli_result in &cli_results {
        if cli_result.success {
            stats.passed += 1;
        } else {
            stats.failed += 1;
        }
    }

    // Print unified summary table
    teeprintln!("\n{}", "=".repeat(120));
    teeprintln!("{}", "=".repeat(120));

    // Print comprehensive test results table
    report.print_report();

    // Write machine-readable JSON report alongside the text output.
    // Enables run-to-run diffing and CI tooling.
    let json_path = paths::test_suite_dir().join("test_suite_report.json");
    match report.write_json(&json_path) {
        Ok(()) => {
            teeprintln!("\n[OK] JSON report saved to: {}", json_path.display());
        }
        Err(e) => {
            teeprintln!("\n[WARN] Failed to write JSON report: {}", e);
        }
    }

    // Print overall summary
    teeprintln!("\n{}", "=".repeat(120));
    teeprintln!("OVERALL SUMMARY");
    teeprintln!("{}", "=".repeat(120));

    let total_tests = stats.passed + stats.failed;
    let pass_rate = if total_tests > 0 {
        (stats.passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    teeprintln!("\n  ┌{:─<116}┐", "");
    teeprintln!("  │ {:^112} │", "TEST RESULTS");
    teeprintln!("  ├{:─<116}┤", "");
    teeprintln!(
        "  │ {:<40} {:>70} │",
        "Total Tests:",
        format!("{}", total_tests)
    );
    teeprintln!(
        "  │ {:<40} {:>63} [OK] │",
        "Passed:",
        format!("{}", stats.passed)
    );
    teeprintln!(
        "  │ {:<40} {:>62} [FAIL] │",
        "Failed:",
        format!("{}", stats.failed)
    );
    teeprintln!("  │ {:<40} {:>65.1}% │", "Pass Rate:", pass_rate);
    teeprintln!("  │ {:<40} {:>65} │", "Skipped:", stats.skipped);
    teeprintln!("  └{:─<116}┘", "");

    teeprintln!("\n  ┌{:─<116}┐", "");
    teeprintln!("  │ {:^112} │", "CODE QUALITY");
    teeprintln!("  ├{:─<116}┤", "");
    teeprintln!(
        "  │ {:<40} {:>70} │",
        "Code Issues (stubs, #[allow]):",
        report.code_issues.len()
    );
    teeprintln!(
        "  │ {:<40} {:>70} │",
        "Compiler Errors:",
        report.lint_errors
    );
    teeprintln!(
        "  │ {:<40} {:>70} │",
        "Compiler Warnings:",
        report.lint_warnings
    );
    teeprintln!("  └{:─<116}┘", "");

    // Exit with error if there are issues
    if report.has_issues() || stats.failed > 0 || report.lint_errors > 0 {
        teeprintln!("\n{}", "═".repeat(120));
        teeprintln!("  {:^116}", "[WARN] TEST SUITE COMPLETED WITH ISSUES");
        teeprintln!("{}", "═".repeat(120));
        output::flush();
        std::process::exit(1);
    }

    teeprintln!("\n{}", "═".repeat(120));
    teeprintln!("  {:^116}", "[DONE] ALL TESTS PASSED - SYSTEM READY!");
    teeprintln!("{}", "═".repeat(120));
    teeprintln!("\n[OK] Text output saved to: {}", output_file.display());
    teeprintln!("[OK] JSON report saved to: {}", json_path.display());
    output::flush();
    Ok(())
}
