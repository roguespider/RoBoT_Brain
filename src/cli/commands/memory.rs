// src/cli/commands/memory.rs
//! Memory management commands

use anyhow::Result;
use crate::database::queries;
use crate::cli::output;

pub fn run(args: &[String]) -> Result<()> {
    if args.is_empty() {
        print_memory_usage();
        return Ok(());
    }
    
    match args[0].as_str() {
        "list" => list_memories(args),
        "search" => search_memories(args),
        "add" => add_memory(args),
        "stats" => memory_stats(),
        _ => {
            eprintln!("Unknown memory command: {}", args[0]);
            print_memory_usage();
            std::process::exit(1);
        }
    }
}

fn print_memory_usage() {
    println!("Memory commands:");
    println!("  memory list [limit]    List recent memories");
    println!("  memory search <query>  Search memories");
    println!("  memory add <content>  Add a new memory");
    println!("  memory stats          Show memory statistics");
}

fn list_memories(args: &[String]) -> Result<()> {
    let limit = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);
    
    let db = crate::database::sqlite::SqliteDatabase::initialize()?;
    let conn = db.connection()?;
    
    let memories = queries::list_memories(&conn, None, limit)?;
    
    println!("Recent Memories ({} total)", memories.len());
    println!("{}", output::Separator::Line);
    
    for m in &memories {
        println!("[{:?}] {}", m.memory_type, &m.content[..m.content.len().min(80)]);
        println!("  Confidence: {:.2} | Importance: {:.2}", m.confidence, m.importance);
        println!();
    }
    
    Ok(())
}

fn search_memories(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        eprintln!("Error: search requires a query");
        print_memory_usage();
        std::process::exit(1);
    }
    
    let query = &args[1];
    let db = crate::database::sqlite::SqliteDatabase::initialize()?;
    let conn = db.connection()?;
    
    let results = queries::search_memory(&conn, query, 50)?;
    
    println!("Search results for '{}' ({} found)", query, results.len());
    println!("{}", output::Separator::Line);
    
    for m in &results {
        println!("[{:?}] {}", m.memory_type, &m.content[..m.content.len().min(80)]);
        println!();
    }
    
    Ok(())
}

fn add_memory(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        eprintln!("Error: add requires content");
        print_memory_usage();
        std::process::exit(1);
    }
    
    let content = &args[1];
    let db = crate::database::sqlite::SqliteDatabase::initialize()?;
    let conn = db.connection()?;
    
    let memory = crate::database::models::MemoryCard {
        id: uuid::Uuid::new_v4(),
        content: content.to_string(),
        memory_type: crate::database::models::MemoryType::Note,
        confidence: 0.5,
        importance: 0.5,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    queries::insert_memory(&conn, &memory)?;
    
    println!("✓ Memory added successfully");
    println!("  ID: {}", memory.id);
    
    Ok(())
}

fn memory_stats() -> Result<()> {
    let db = crate::database::sqlite::SqliteDatabase::initialize()?;
    let conn = db.connection()?;
    
    let memories = queries::list_memories(&conn, None, 1000)?;
    
    let mut by_type: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut total_confidence: f32 = 0.0;
    
    for m in &memories {
        *by_type.entry(format!("{:?}", m.memory_type)).or_insert(0) += 1;
        total_confidence += m.confidence;
    }
    
    let avg_confidence = if memories.is_empty() { 0.0 } else { total_confidence / memories.len() as f32 };
    
    println!("Memory Statistics");
    println!("{}", output::Separator::Line);
    println!("Total memories: {}", memories.len());
    println!("Average confidence: {:.2}", avg_confidence);
    println!();
    println!("By type:");
    for (mem_type, count) in &by_type {
        println!("  {}: {}", mem_type, count);
    }
    
    Ok(())
}
