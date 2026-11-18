// /src/database/mod.rs
//! Database layer for RoBoT Brain — Per Architecture Chapter 21 (Storage Architecture).

use std::fs;
use std::path::Path;

pub mod maintenance;
pub mod migrations;
pub mod models;
pub mod queries;
pub mod sqlite;

/// Storage layers for the hybrid database model.
///
/// Per Architecture §21: the system uses a hybrid model — relational +
/// vector + graph + object storage, each mapped to a distinct layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageLayer {
    /// Working memory layer — active data in the current session.
    Working,
    /// Session-scoped storage — persists across operations in one session.
    Session,
    /// Experience records — captured outcomes from completed actions.
    Experience,
    /// Semantic memory — vector-indexed facts and concepts.
    Semantic,
    /// Skill descriptors and performance data.
    Skill,
    /// Knowledge graph nodes and edges.
    Graph,
    /// Archived records — long-term retention of completed items.
    Archive,
    /// Operational metadata — system configuration, logs, diagnostics.
    Operational,
}

impl std::fmt::Display for StorageLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageLayer::Working => write!(f, "working"),
            StorageLayer::Session => write!(f, "session"),
            StorageLayer::Experience => write!(f, "experience"),
            StorageLayer::Semantic => write!(f, "semantic"),
            StorageLayer::Skill => write!(f, "skill"),
            StorageLayer::Graph => write!(f, "graph"),
            StorageLayer::Archive => write!(f, "archive"),
            StorageLayer::Operational => write!(f, "operational"),
        }
    }
}

/// Database error types for backup/restore operations.
#[derive(Debug)]
pub enum DatabaseError {
    /// Source file does not exist.
    SourceNotFound(String),
    /// Backup destination could not be created.
    DestinationError(String),
    /// File copy failed.
    IoError(std::io::Error),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::SourceNotFound(path) => write!(f, "Source not found: {path}"),
            DatabaseError::DestinationError(path) => write!(f, "Cannot write to: {path}"),
            DatabaseError::IoError(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl From<std::io::Error> for DatabaseError {
    fn from(e: std::io::Error) -> Self {
        DatabaseError::IoError(e)
    }
}

/// Backup the database to the specified path.
///
/// Per Architecture §21: backup copies the primary database file
/// to preserve data integrity for recovery.
pub fn backup_database(path: &str) -> Result<(), DatabaseError> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .ok_or_else(|| DatabaseError::DestinationError("Cannot determine exe directory".into()))?;

    let source = exe_dir.join("robot_brain.db");
    if !source.exists() {
        return Err(DatabaseError::SourceNotFound(
            source.to_string_lossy().into(),
        ));
    }

    let dest = Path::new(path);
    fs::copy(&source, dest).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            DatabaseError::DestinationError(dest.to_string_lossy().into())
        } else {
            DatabaseError::IoError(e)
        }
    })?;

    tracing::info!("Database backed up to {path}");
    Ok(())
}

/// Restore the database from a backup file.
///
/// Per Architecture §21: restore replaces the current database with
/// a previously saved backup.
pub fn restore_database(source: &str, target: &str) -> Result<(), DatabaseError> {
    let src_path = Path::new(source);
    if !src_path.exists() {
        return Err(DatabaseError::SourceNotFound(
            src_path.to_string_lossy().into(),
        ));
    }

    let target_path = Path::new(target);
    let target_dir = target_path
        .parent()
        .ok_or_else(|| DatabaseError::DestinationError("Invalid target path".into()))?;

    if !target_dir.exists() {
        fs::create_dir_all(target_dir).map_err(|io_err| {
            tracing::debug!("Failed to create directory: {io_err}");
            DatabaseError::DestinationError(target_dir.to_string_lossy().into())
        })?;
    }

    fs::copy(src_path, target_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            DatabaseError::DestinationError(target_path.to_string_lossy().into())
        } else {
            DatabaseError::IoError(e)
        }
    })?;

    tracing::info!("Database restored from {source} to {target}");
    Ok(())
}

