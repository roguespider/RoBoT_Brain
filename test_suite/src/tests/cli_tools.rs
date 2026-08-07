//! CLI-Based Tool Tests
//!
//! This module tests the NEW tool system by invoking CLI commands directly.
//! The new system uses CLI commands that call the database directly,
//! bypassing the MCP protocol which isn't fully implemented yet.
//!
//! This is the NEW tool system that replaced MCP tools/call.

use std::process::Command;

/// Test result for a CLI command
#[derive(Debug, Clone)]
pub struct CliTestResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub test_name: String,
}

impl CliTestResult {
    pub fn new(test_name: &str, success: bool, output: String, error: Option<String>) -> Self {
        Self {
            success,
            output,
            error,
            test_name: test_name.to_string(),
        }
    }
}

/// Run a CLI command and return the result
pub fn run_cli_command(args: &[&str]) -> CliTestResult {
    let server_path = find_robot_brain();
    
    let output = Command::new(&server_path)
        .args(args)
        .output();
    
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            
            if out.status.success() {
                CliTestResult::new(
                    "cli_command",
                    true,
                    stdout,
                    if stderr.is_empty() { None } else { Some(stderr) },
                )
            } else {
                CliTestResult::new(
                    "cli_command",
                    false,
                    stdout,
                    Some(stderr),
                )
            }
        }
        Err(e) => CliTestResult::new(
            "cli_command",
            false,
            String::new(),
            Some(e.to_string()),
        ),
    }
}

/// Find the robot_brain binary path
fn find_robot_brain() -> String {
    let test_suite_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let possible_paths = [
        format!("{}/../target/release/robot_brain", test_suite_dir),
        format!("{}/../../target/release/robot_brain", test_suite_dir),
        "/workspace/project/RoBoT_Brain/target/release/robot_brain".to_string(),
    ];
    
    for path in &possible_paths {
        if std::path::Path::new(path).exists() {
            return path.clone();
        }
    }
    
    "robot_brain".to_string()
}

/// Test a CLI command and return the result
fn test_command(test_name: &str, args: &[&str]) -> CliTestResult {
    let mut result = run_cli_command(args);
    result.test_name = test_name.to_string();
    result
}

/// Run all CLI-based tool tests
/// 
/// This is the NEW tool system that replaces MCP protocol testing.
/// Each test category corresponds to a CLI command or set of commands.
pub async fn run_cli_tool_tests() -> Vec<CliTestResult> {
    let mut results = Vec::new();
    
    println!("\n{}", "=".repeat(80));
    println!("NEW TOOL SYSTEM - CLI-BASED TESTS");
    println!("Testing via CLI commands (the NEW system that replaced MCP tools/call)");
    println!("{}", "=".repeat(80));
    
    // ========================================
    // MEMORY TOOL TESTS
    // ========================================
    println!("\n📋 MEMORY TOOL TESTS:");
    println!("{}", "-".repeat(40));
    
    // Test memory list
    let result = test_command("memory.list", &["memory", "list", "10"]);
    println!("  memory list: {}", if result.success { "✓ PASS" } else { "✗ FAIL" });
    results.push(result);
    
    // Test memory stats
    let result = test_command("memory.stats", &["memory", "stats"]);
    println!("  memory stats: {}", if result.success { "✓ PASS" } else { "✗ FAIL" });
    results.push(result);
    
    // Test memory search
    let result = test_command("memory.search", &["memory", "search", "test"]);
    println!("  memory search: {}", if result.success { "✓ PASS" } else { "✗ FAIL" });
    results.push(result);
    
    // Test memory add
    let result = test_command("memory.add", &["memory", "add", "Test memory from test_suite"]);
    println!("  memory add: {}", if result.success { "✓ PASS" } else { "✗ FAIL" });
    results.push(result);
    
    // ========================================
    // SYSTEM TOOL TESTS
    // ========================================
    println!("\n📋 SYSTEM TOOL TESTS:");
    println!("{}", "-".repeat(40));
    
    // Test status
    let result = test_command("system.status", &["status"]);
    println!("  system status: {}", if result.success { "✓ PASS" } else { "✗ FAIL" });
    results.push(result);
    
    // Test experience
    let result = test_command("system.experience", &["experience"]);
    println!("  system experience: {}", if result.success { "✓ PASS" } else { "✗ FAIL" });
    results.push(result);
    
    // Test config
    let result = test_command("system.config", &["config"]);
    println!("  system config: {}", if result.success { "✓ PASS" } else { "✗ FAIL" });
    results.push(result);
    
    // ========================================
    // DATABASE TOOL TESTS
    // ========================================
    println!("\n📋 DATABASE TOOL TESTS:");
    println!("{}", "-".repeat(40));
    
    // Test init
    let result = test_command("db.init", &["init"]);
    println!("  db init: {}", if result.success { "✓ PASS" } else { "✗ FAIL" });
    results.push(result);
    
    // Test migrate
    let result = test_command("db.migrate", &["migrate"]);
    println!("  db migrate: {}", if result.success { "✓ PASS" } else { "✗ FAIL" });
    results.push(result);
    
    // ========================================
    // Print Summary
    // ========================================
    let passed = results.iter().filter(|r| r.success).count();
    let total = results.len();
    
    println!("\n{}", "=".repeat(80));
    println!("CLI TOOL TEST SUMMARY");
    println!("{}", "=".repeat(80));
    
    for result in &results {
        let status = if result.success { "✓" } else { "✗" };
        let test_type = result.test_name.split('.').next().unwrap_or("unknown");
        println!("  {:12} {:20} {}", status, result.test_name, 
            if result.success { "PASS" } else { "FAIL" });
    }
    
    println!("\n  Total: {}/{} passed ({:.0}%)", passed, total, 
        if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 });
    
    results
}
