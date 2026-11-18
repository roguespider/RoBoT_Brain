//! Confidence history persistence — records confidence changes for the
//! confidence rules enforcement (Architecture §2.5, §19).

use anyhow::Result;
use std::sync::Arc;

use crate::database::queries::confidence_history::insert_confidence_history;
use crate::database::sqlite::SqliteDatabase;

/// Record a confidence change in the confidence_history table.
///
/// Called whenever the confidence system records a score update, so
/// the audit trail can be queried later.
pub async fn record_confidence_change(
    db: &Arc<SqliteDatabase>,
    item_id: &str,
    old_confidence: f64,
    new_confidence: f64,
    reason: &str,
) -> Result<()> {
    let id = uuid::Uuid::new_v4().to_string();
    let conn = db.connection()?;
    insert_confidence_history(&conn, &id, item_id, old_confidence, new_confidence, reason)?;
    tracing::debug!(item_id = item_id, "Confidence change recorded");
    Ok(())
}
