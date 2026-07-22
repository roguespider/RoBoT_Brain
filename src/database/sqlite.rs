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

    /// Checkpoint the WAL file to clean up temporary files.
    /// This writes pending changes to the main database and truncates the WAL file.
    pub fn checkpoint(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        
        // TRUNCATE checkpoint - writes all WAL content to db and truncates WAL file
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
        
        tracing::debug!("WAL checkpoint completed, temporary files cleaned up");
        Ok(())
    }

    /// Get the current WAL file size (for debugging).
    pub fn wal_size(&self) -> Result<u64> {
        let wal_path = self.db_path.with_extension("db-wal");
        if wal_path.exists() {
            Ok(std::fs::metadata(&wal_path)?.len())
        } else {
            Ok(0)
        }
    }

    /// Get the current SHM file size (for debugging).
    pub fn shm_size(&self) -> Result<u64> {
        let shm_path = self.db_path.with_extension("db-shm");
        if shm_path.exists() {
            Ok(std::fs::metadata(&shm_path)?.len())
        } else {
            Ok(0)
        }
    }

    /// Cleanup WAL and SHM files by running a checkpoint.
    /// Call this periodically or after batch operations to clean up.
    pub fn cleanup_wal_files(&self) -> Result<()> {
        let wal_size_before = self.wal_size().unwrap_or(0);
        let shm_size_before = self.shm_size().unwrap_or(0);
        
        self.checkpoint()?;
        
        let wal_size_after = self.wal_size().unwrap_or(0);
        let shm_size_after = self.shm_size().unwrap_or(0);
        
        if wal_size_before > 0 || shm_size_before > 0 {
            tracing::info!(
                "WAL cleanup: WAL {} -> {} bytes, SHM {} -> {} bytes",
                wal_size_before, wal_size_after, shm_size_before, shm_size_after
            );
        }
        
        Ok(())
    }

    /// Database file path.
    pub fn path(&self) -> &Path {
        &self.db_path
    }
}
