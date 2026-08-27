// src/main.rs

mod agent;
mod bridge;
mod cli;
mod database;
mod experience;
mod knowledge;
mod learning;
mod memory;
mod personality;
mod planner;

mod skills;
mod workflows;
mod world_model;

use bridge::app::App;
use bridge::logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // On Windows, attach to parent console if running without one
    // This fixes issues with GUI applications (like Zed Editor) that spawn
    // subprocesses without a console, causing stdio to fail
    #[cfg(target_os = "windows")]
    {
        bridge::windows_console::attach_console();
    }

    init_logging();

    // Check if CLI mode is requested
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "server" => {
                App::new().await?.run().await?;
            }
            "diagnose" => {
                // Explicit subsystem diagnostics (P2-001C). Runs inside the
                // existing tokio runtime, then exits.
                let app = App::new().await?;
                let result =
                    bridge::app::initialization::diagnostics::run_startup_diagnostics(&app).await;
                if result.failed > 0 {
                    eprintln!("Diagnostics completed with {} failure(s)", result.failed);
                    std::process::exit(1);
                }
            }
            _ => {
                // Run CLI commands
                cli::run()?;
            }
        }
    } else {
        // Default: run as MCP server
        App::new().await?.run().await?;
    }

    Ok(())
}
