// src/cli/commands/experience.rs
//! Experience statistics command

use anyhow::Result;

pub fn run() -> Result<()> {
    let db = crate::database::sqlite::SqliteDatabase::initialize()?;
    let conn = db.connection()?;
    
    let memories = crate::database::queries::search_memory(&conn, "Experience:", 1000)?;
    
    let mut success = 0;
    let mut failure = 0;
    let mut total_confidence: f32 = 0.0;
    
    for m in &memories {
        total_confidence += m.confidence;
        if m.content.contains("Success") || m.content.contains("success") {
            success += 1;
        } else {
            failure += 1;
        }
    }
    
    let total = memories.len();
    let avg_confidence = if total == 0 { 0.0 } else { total_confidence / total as f32 };
    
    println!("Experience Statistics");
    println!("{}", crate::cli::output::Separator::Line);
    println!("Total experiences: {}", total);
    println!("Success rate: {:.1}%", if total > 0 { (success as f32 / total as f32) * 100.0 } else { 0.0 });
    println!("Average confidence: {:.2}", avg_confidence);
    println!();
    println!("Breakdown:");
    println!("  ✓ Success: {}", success);
    println!("  ✗ Failure: {}", failure);
    
    Ok(())
}
