// src/tools/experience/mod.rs
//! Experience recording tool.
//!
//! Records experiences through the ExperienceCoordinator and persists them
//! via the database.

use serde::{Deserialize, Serialize};

/// Input for recording an experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordExperienceInput {
    /// Title of the experience.
    pub title: String,
    /// Description of the experience.
    pub description: String,
    /// Type/category of the experience.
    pub experience_type: String,
    /// Overall outcome.
    pub outcome: crate::experience::types::outcome::OutcomeKind,
    /// Optional context JSON.
    pub context: Option<String>,
    /// Optional confidence (0.0-1.0).
    pub confidence: Option<f32>,
    /// Optional importance (0.0-1.0).
    pub importance: Option<f32>,
    /// Optional tags.
    pub tags: Option<Vec<String>>,
}

/// Execute experience recording.
///
/// Creates an experience record and persists it to the database.
pub async fn execute_record_experience(
    input: RecordExperienceInput,
    coordinator: &std::sync::Arc<crate::experience::coordinator::ExperienceCoordinator>,
    db: &std::sync::Arc<crate::database::sqlite::SqliteDatabase>,
) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
    // Create experience record
    let experience = crate::experience::types::experience::Experience::new(
        input.title.clone(),
        input.description.clone(),
        crate::experience::types::experience::ExperienceType::Custom(input.experience_type.clone()),
        Vec::new(),
    );

    // Wire coordinator into production per Architecture §9.4 (Experience Lifecycle)
    let processed = coordinator.process(experience.clone());
    let scored = processed.score.clone().unwrap_or_default();
    tracing::debug!(
        experience_id = %experience.id,
        score_confidence = %scored.confidence,
        score_importance = %scored.importance,
        "Experience scored via coordinator"
    );

    // Persist to database
    let card = crate::database::models::MemoryCard::from_experience(&experience);
    let conn = db.connection()?;
    crate::database::queries::insert_memory(&conn, &card)?;

    Ok(crate::bridge::tools::ToolOutput::success(
        serde_json::json!({
            "status": "recorded",
            "experience_id": experience.id.to_string(),
            "title": input.title,
        }),
    ))
}
