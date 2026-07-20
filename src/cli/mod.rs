// src/cli/mod.rs
//! Command-line interface module

pub mod commands;
pub mod output;

use anyhow::Result;

/// Run the CLI with the given arguments
pub fn run() -> Result<()> {
    cli()
}

/// Main CLI entry point
fn cli() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "server" => commands::server::run(),
        "init" => commands::init::run(),
        "status" => commands::status::run(),
        "memory" => commands::memory::run(&args[2..]),
        "experience" => commands::experience::run(),
        "config" => commands::config::run(),
        "migrate" => commands::migrate::run(),
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }
}

/// Print CLI usage information
fn print_usage() {
    println!("RoBoT MCP - Command Line Interface");
    println!();
    println!("Usage: robot <command> [options]");
    println!();
    println!("Commands:");
    println!("  server       Start the MCP server");
    println!("  init         Initialize the database");
    println!("  status       Check system status");
    println!("  memory       Memory management commands");
    println!("  experience   Show experience statistics");
    println!("  config       Show configuration");
    println!("  migrate      Run database migrations");
    println!("  help         Show this help message");
    println!();
    println!("Memory subcommands:");
    println!("  memory list [limit]     List memories");
    println!("  memory search <query>   Search memories");
    println!("  memory add <content>     Add a new memory");
}
