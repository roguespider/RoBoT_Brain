// src/cli/commands/config.rs
//! Configuration display command

use anyhow::Result;

pub fn run() -> Result<()> {
    println!("RoBoT Configuration");
    println!("===================");
    println!();
    
    println!("Package:");
    println!("  Name: {}", env!("CARGO_PKG_NAME"));
    println!("  Version: {}", env!("CARGO_PKG_VERSION"));
    println!("  Description: {}", env!("CARGO_PKG_DESCRIPTION"));
    println!();
    
    println!("Database:");
    if let Ok(db) = crate::database::sqlite::SqliteDatabase::initialize() {
        println!("  ✓ Database initialized");
        println!("  Path: {:?}", db.path());
    } else {
        println!("  ✗ Database not initialized");
    }
    println!();
    
    println!("Features:");
    println!("  ✓ Experience System");
    println!("  ✓ Reflection Engine");
    println!("  ✓ Learning System");
    println!("  ✓ MCP Bridge");
    println!("  ✓ CLI Interface");
    
    Ok(())
}
