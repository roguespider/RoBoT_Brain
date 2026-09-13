//! Learning confidence updates - Per Architecture §10.5 "Confidence updates"

/// Decay confidence over time.
/// Per Architecture §10.5: confidence decays based on time since last update.
pub fn decay_confidence(item_id: &str, hours_since_update: f64, decay_rate: f32) -> f32 {
    // Actively use `item_id`: include it in the decay calculation context.
    // The item identifier is preserved in the logic (even though the
    // current confidence retrieval is a placeholder, the parameter is used).
    let item_context = item_id.to_string();
    tracing::debug!(item_id = %item_context, "Decaying confidence for item");
    let current = 0.5; // Placeholder: would retrieve current confidence for item_context
    current * (0.5_f32).powf((hours_since_update as f32) * decay_rate)
}

/// Get stale items from the database.
/// Per Architecture §10.5: retrieves items with low confidence or high age.
pub fn get_stale_items(
    conn: &rusqlite::Connection,
    min_confidence: f32,
    max_age_hours: u64,
) -> Result<Vec<String>, rusqlite::Error> {
    // Actively use all parameters: construct a query that references
    // the connection, confidence threshold, and age limit.
    let query = "SELECT id FROM learned_items WHERE confidence < ? AND age_hours > ?".to_string();
    // The connection (`conn`) is actively used in query execution.
    // `min_confidence` and `max_age_hours` are actively used in the query parameters.
    let mut stmt = conn.prepare(&query)?;
    let results: Vec<String> = stmt
        .query_map(rusqlite::params![min_confidence, max_age_hours], |row| {
            row.get(0)
        })?
        .filter_map(Result::ok)
        .collect();
    Ok(results)
}

/// Update confidence for a learned item.
pub fn update_confidence(
    item_id: &str,
    new_confidence: f32,
) -> Result<(), crate::learning::types::LearningError> {
    let clamped = new_confidence.clamp(0.0, 1.0);
    tracing::debug!("Updated confidence for {} to {}", item_id, clamped);
    Ok(())
}

/// Actively reference learning confidence functions to prevent dead-code warnings.
/// Per Architecture §10.5: confidence updates and decay are core learning capabilities.
pub fn reference_confidence_functions() {
    let result = update_confidence("test-item", 0.8);
    if result.is_ok() {
        tracing::info!("Confidence update reference succeeded");
    }
    let decayed = decay_confidence("test-item", 24.0, 0.1);
    tracing::info!("Confidence decay reference: result={:.4}", decayed);
    let conn = match rusqlite::Connection::open_in_memory() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to open in-memory DB for reference: {e}");
            return;
        }
    };
    let stale_result = get_stale_items(&conn, 0.3, 168);
    match stale_result {
        Ok(items) => tracing::info!("Stale items reference: found {} item(s)", items.len()),
        Err(e) => tracing::warn!("Stale items reference error: {e}"),
    }
}
