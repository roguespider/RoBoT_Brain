// src/bridge/app/initialization/db.rs
//! Database initialization

use std::sync::Arc;

use anyhow::Result;

use crate::data_contracts::contract_validator::{ContractValidator, reference_contract_validator};
use crate::database::sqlite::SqliteDatabase;

/// Initialize the SQLite database connection.
pub(crate) fn init_database() -> Result<Arc<SqliteDatabase>> {
    let database = Arc::new(SqliteDatabase::initialize()?);
    // Per Architecture Chapter 5: validate contracts at DB init
    let results = ContractValidator::validate_contract(
        crate::data_contracts::version::CONTRACT_VERSION,
        "db-init",
        "database",
        1.0,
    );
    if !ContractValidator::all_valid(&results) {
        tracing::warn!("Contract validation warnings during DB init");
    }
    reference_contract_validator();
    tracing::info!("Database initialized");
    Ok(database)
}
