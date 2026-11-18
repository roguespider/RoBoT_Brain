// src/main.rs — RoBoT Brain MCP Server binary entry point
//
// This is the process entry point that starts the MCP server with stdio transport.
// The server implements MCP Protocol 2025-03-26 for AI agent communication.

use robot_brain::api::bridge::app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the application (database, core infrastructure, engines)
    let app = App::new().await?;

    // Run the MCP server (blocks until process is terminated)
    app.run().await?;

    Ok(())
}
