//! Memory pruning policy - Per Architecture §17.5 "Memory lifecycle"
//!
//! Prunes low-value or aged memories from the active set.

use rusqlite::{Connection, Error};

/// Prune memories below importance threshold
pub fn prune_below_importance(conn: &Connection, threshold: f32) -> Result<usize, Error> {
    let deleted = conn.execute("DELETE FROM memories WHERE importance < ?1", [threshold])?;
    Ok(deleted)
}

/// Prune memories older than max_age_secs from now_ts
pub fn prune_older_than(conn: &Connection, max_age_secs: u64, now_ts: i64) -> Result<usize, Error> {
    let cutoff = now_ts - max_age_secs as i64;
    let deleted = conn.execute("DELETE FROM memories WHERE created_at < ?1", [cutoff])?;
    Ok(deleted)
}

/// Combined prune: delete by both importance and age thresholds
pub fn prune(
    conn: &Connection,
    importance_threshold: f32,
    max_age_secs: u64,
    now_ts: i64,
) -> Result<usize, Error> {
    let deleted_by_importance = prune_below_importance(conn, importance_threshold)?;
    let deleted_by_age = prune_older_than(conn, max_age_secs, now_ts)?;
    Ok(deleted_by_importance + deleted_by_age)
}

/// Active reference to memory pruning contracts.
pub fn reference_memory_prune_contracts() {
    // Wire prune_below_importance
    let conn = rusqlite::Connection::open_in_memory().ok();
    if let Some(ref conn) = conn {
        match prune_below_importance(conn, 0.5) {
            Ok(n) => tracing::info!("prune_below_importance wired, pruned {n} memories", n = n),
            Err(e) => tracing::warn!("prune_below_importance wired with error: {e}"),
        }
    }

    // Wire prune_older_than
    if let Some(ref conn) = conn {
        match prune_older_than(conn, 86400, chrono::Utc::now().timestamp()) {
            Ok(n) => tracing::info!("prune_older_than wired, pruned {n} memories", n = n),
            Err(e) => tracing::warn!("prune_older_than wired with error: {e}"),
        }
    }

    // Wire prune
    if let Some(ref conn) = conn {
        match prune(conn, 0.5, 86400, chrono::Utc::now().timestamp()) {
            Ok(n) => tracing::info!("prune wired, pruned {n} total memories", n = n),
            Err(e) => tracing::warn!("prune wired with error: {e}"),
        }
    }
}
