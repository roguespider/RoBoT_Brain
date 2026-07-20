// src/app.rs

use std::sync::Arc;

use anyhow::Result;

use crate::database::sqlite::SqliteDatabase;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::scorer::ExperienceScorer;
use crate::bridge::rmcp::run_stdio_server;


/// Root application container.
///
/// Owns long-running services required by RoBoT.
pub struct App {
    /// Persistent database layer.
    _database: Arc<SqliteDatabase>,

    /// Experience system coordinator.
    _coordinator: Arc<ExperienceCoordinator>,
}


impl App {

    /// Build the application.
    pub async fn new() -> Result<Self> {

        let database =
            Arc::new(
                SqliteDatabase::initialize()?
            );


        let scorer = ExperienceScorer::new();
        let coordinator =
            Arc::new(
                ExperienceCoordinator::new(scorer)
            );

        Ok(Self {
            _database: database,
            _coordinator: coordinator,
        })
    }


    /// Start the runtime.
    pub async fn run(self) -> Result<()> {
        // Run the MCP server with stdio transport
        run_stdio_server(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).await
    }
}
