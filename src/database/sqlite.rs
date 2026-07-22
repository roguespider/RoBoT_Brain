// src/database/sqlite.rs

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Main SQLite database service.
///
/// Owns the database location and provides fresh SQLite connections
/// to the rest of the application.
///
/// Each operation opens its own connection, avoiding the need to
/// share a Connection across threads.
#[derive(Debug, Clone)]
pub struct SqliteDatabase {
    db_path: PathBuf,
}

impl SqliteDatabase {
    /// Open (or create) the application's database beside the executable.
    pub fn initialize() -> Result<Self> {
        let exe_path = std::env::current_exe()
            .context("Failed to get executable path")?;
        let exe_dir = exe_path.parent()
            .context("Executable has no parent directory")?;
        Self::initialize_at(exe_dir)
    }

    /// Open (or create) a database at a specific location.
    pub fn initialize_at<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let db_path = data_dir.as_ref().join("robot_brain.db");

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Unable to create database directory")?;
        }

        let database = Self { db_path };

        database.run_migrations()?;

        // Configure the database for optimal concurrency
        database.configure_connection()?;

        Ok(database)
    }

    /// Execute database schema migrations.
    fn run_migrations(&self) -> Result<()> {
        crate::database::migrations::run(self)
    }

    /// Configure SQLite for better concurrency with WAL mode.
    fn configure_connection(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        
        // Enable WAL mode for better concurrency (allows concurrent reads during writes)
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA busy_timeout=30000;
             PRAGMA cache_size=-64000;
             PRAGMA temp_store=MEMORY;
             PRAGMA mmap_size=268435456;"
        )?;
        
        tracing::info!("SQLite configured with WAL mode for improved concurrency");
        Ok(())
    }

    /// Open a fresh SQLite connection with optimized settings.
    pub fn connection(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        
        // Ensure WAL mode is enabled on each connection
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=30000;"
        )?;
        
        Ok(conn)
    }

    /// Database file path.
    pub fn path(&self) -> &Path {
        &self.db_path
    }
}
