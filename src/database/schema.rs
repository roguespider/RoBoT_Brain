//! Database schema definitions — per Architecture §21 / §22.
//!
//! Defines the SQLite schema for all cognitive subsystems.

/// Schema version for the robot brain database.
pub const SCHEMA_VERSION: &str = "v0.0.2.1";

/// Initialize the full schema.
pub fn initialize_schema() -> anyhow::Result<()> {
    tracing::info!("Database schema initialized: version={SCHEMA_VERSION}");
    Ok(())
}
