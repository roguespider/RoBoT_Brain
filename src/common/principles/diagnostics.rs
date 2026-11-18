//! Diagnostics persistence — records diagnostic events from the architecture
//! enforcement system (Chapter 24).

use anyhow::Result;
use std::sync::Arc;

use crate::database::queries::diagnostics::insert_diagnostic;
use crate::database::sqlite::SqliteDatabase;

/// Record a diagnostic event from the architecture enforcement gate.
///
/// Called whenever the architecture acceptance gate produces a decision
/// (accept or reject).  The severity is `INFO` for accepted contributions
/// and `WARN` for rejected ones.
pub async fn record_diagnostic(
    db: &Arc<SqliteDatabase>,
    severity: &str,
    category: &str,
    message: &str,
) -> Result<()> {
    let id = uuid::Uuid::new_v4().to_string();
    let conn = db.connection()?;
    insert_diagnostic(&conn, &id, severity, category, message, "", 0)?;
    tracing::debug!(
        diagnostic_id = id,
        severity,
        category,
        "Diagnostic recorded"
    );
    Ok(())
}
