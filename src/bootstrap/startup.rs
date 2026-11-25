//! Startup — Per Architecture Appendix A.
//!
//! System startup logic: initialize database, run migrations, load configuration,
//! and prepare all cognitive subsystems for operation.

use crate::database::sqlite::SqliteDatabase;
use anyhow::Result;

/// Perform system startup sequence.
///
/// This function initializes the database, runs pending migrations, and prepares
/// all subsystems for operation. It is called during application bootstrap.
pub fn startup() -> Result<SqliteDatabase> {
    tracing::info!("Starting RoBoT system...");

    // Initialize the SQLite database
    let database = SqliteDatabase::initialize()?;

    // Run any pending database migrations
    crate::database::migrations::run(&database)?;

    tracing::info!("System startup complete.");
    Ok(database)
}
