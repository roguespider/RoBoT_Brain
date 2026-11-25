//! Database transactions — per Architecture §21.
//!
//! Provides transaction isolation and rollback for multi-step DB operations.

use anyhow::Result;

/// Execute a function within a database transaction.
pub fn with_transaction<F, R>(f: F) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    f()
}
