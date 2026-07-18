// src/app.rs

use std::sync::Arc;

use anyhow::Result;

use crate::database::sqlite::SqliteDatabase;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::mcp_bridge::McpBridge;


/// Root application container.
///
/// Owns long-running services required by RoBoT.
pub struct App {
    /// Persistent database layer.
    _database: Arc<SqliteDatabase>,

    /// Experience system coordinator.
    _coordinator: Arc<ExperienceCoordinator>,

    /// MCP communication server.
    bridge: McpBridge,
}


impl App {

    /// Build the application.
    pub async fn new() -> Result<Self> {

        let database =
            Arc::new(
                SqliteDatabase::initialize()?
            );


        let coordinator =
            Arc::new(
                ExperienceCoordinator::new(
                    database.clone()
                )
            );


        let bridge =
            McpBridge::new(
                coordinator.clone()
            )
            .await?;


        Ok(Self {
            _database: database,
            _coordinator: coordinator,
            bridge,
        })
    }


    /// Start the runtime.
    pub async fn run(self) -> Result<()> {

        self.bridge.run().await

    }
}
