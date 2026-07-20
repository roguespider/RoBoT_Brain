// src/cli/commands/init.rs
//! Database initialization command

use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Initializing RoBoT database...");
    
    // Initialize database
    let db = crate::database::sqlite::SqliteDatabase::initialize()?;
    println!("✓ Database initialized successfully");
    println!("  Location: {:?}", db.path());
    
    Ok(())
}
