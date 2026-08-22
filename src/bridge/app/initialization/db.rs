// src/bridge/app/initialization/db.rs
//! Database initialization

use std::sync::Arc;

use anyhow::Result;

use crate::database::sqlite::SqliteDatabase;

/// Initialize the SQLite database connection.
pub(crate) fn init_database() -> Result<Arc<SqliteDatabase>> {
    let database = Arc::new(SqliteDatabase::initialize()?);
    tracing::info!("Database initialized");
    Ok(database)
}
