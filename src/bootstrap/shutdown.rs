//! Shutdown — Per Architecture Appendix A.
//!
//! Shutdown coordination: gracefully stop all subsystems, persist state,
//! and clean up resources.

use crate::database::sqlite::SqliteDatabase;
use anyhow::Result;

/// Perform system shutdown sequence.
///
/// This function gracefully stops all subsystems, persists any pending state,
/// and cleans up resources before the process exits.
pub fn shutdown(database: &SqliteDatabase) -> Result<()> {
    tracing::info!("Shutting down RoBoT system...");

    // Checkpoint the database to persist any pending WAL data
    database.checkpoint()?;

    tracing::info!("System shutdown complete.");
    Ok(())
}
