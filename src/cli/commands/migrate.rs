// src/cli/commands/migrate.rs
//! Database migration command

use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Running database migrations...");
    println!();
    
    let db = crate::database::sqlite::SqliteDatabase::initialize()?;
    
    // Run migrations
    crate::database::migrations::run(&db)?;
    
    println!("✓ Migrations completed successfully");
    
    Ok(())
}
