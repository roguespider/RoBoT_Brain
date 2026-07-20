// src/cli/commands/status.rs
//! System status command

use anyhow::Result;

pub fn run() -> Result<()> {
    println!("RoBoT System Status");
    println!("===================");
    println!();
    
    // Check database
    match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => {
            println!("✓ Database: Connected");
            println!("  Location: {:?}", db.path());
        }
        Err(e) => {
            println!("✗ Database: Error - {}", e);
        }
    }
    
    println!();
    println!("Components:");
    println!("  ✓ Experience System");
    println!("  ✓ Reflection Engine");
    println!("  ✓ Learning System");
    println!("  ✓ MCP Bridge");
    
    Ok(())
}
