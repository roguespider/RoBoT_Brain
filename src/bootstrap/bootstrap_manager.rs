//! Bootstrap Manager — Per Architecture Appendix A.
//!
//! Manages the bootstrap process, coordinating startup and shutdown of all
//! RoBoT subsystems. Provides the entry point for initializing the cognitive
//! architecture and gracefully shutting it down.

use crate::database::sqlite::SqliteDatabase;
use anyhow::Result;

/// Manages the bootstrap lifecycle of the RoBoT system.
pub struct BootstrapManager {
    database: Option<SqliteDatabase>,
    started: bool,
}

impl BootstrapManager {
    /// Create a new bootstrap manager.
    pub fn new() -> Self {
        Self {
            database: None,
            started: false,
        }
    }

    /// Initialize all subsystems and start the cognitive architecture.
    pub async fn start(&mut self) -> Result<()> {
        if self.started {
            return Ok(());
        }

        tracing::info!("Bootstrapping RoBoT cognitive architecture...");

        // Initialize database
        let database = SqliteDatabase::initialize()?;
        crate::database::migrations::run(&database)?;

        self.database = Some(database);
        self.started = true;

        tracing::info!("RoBoT bootstrap complete.");
        Ok(())
    }

    /// Gracefully shut down all subsystems.
    pub async fn stop(&mut self) -> Result<()> {
        if !self.started {
            return Ok(());
        }

        tracing::info!("Shutting down RoBoT cognitive architecture...");

        self.database = None;
        self.started = false;

        tracing::info!("RoBoT shutdown complete.");
        Ok(())
    }

    /// Check if the bootstrap manager has been started.
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Get a reference to the database.
    pub fn database(&self) -> Option<&SqliteDatabase> {
        self.database.as_ref()
    }
}

impl Default for BootstrapManager {
    fn default() -> Self {
        Self::new()
    }
}
