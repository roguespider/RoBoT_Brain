//! Architecture trace persistence — records traces for the architecture
//! acceptance gate (Chapter 24, §24.8 "Trace Before Changing").
//!
//! Every rejected contribution gets an `architecture_trace` row so the
//! decision can be audited later.  Accepted contributions also get a trace
//! row so the architecture alignment can be replayed.

use anyhow::Result;
use std::sync::Arc;

use crate::database::queries::architecture_trace::insert_architecture_trace;
use crate::database::sqlite::SqliteDatabase;

/// Record an architecture trace entry.
///
/// Called from the contribution acceptance gate whenever an action is
/// evaluated against the architecture rules.
pub async fn record_trace(
    db: &Arc<SqliteDatabase>,
    function_name: &str,
    callers: &str,
    dependencies: &str,
    data_flow: &str,
    trace_id: &str,
) -> Result<()> {
    let trace_id_hex = uuid::Uuid::new_v4().to_string();
    let conn = db.connection()?;
    insert_architecture_trace(
        &conn,
        &trace_id_hex,
        function_name,
        callers,
        dependencies,
        data_flow,
        trace_id,
    )?;
    tracing::debug!(trace_id = trace_id_hex, "Architecture trace recorded");
    Ok(())
}
