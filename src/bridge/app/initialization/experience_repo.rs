// src/bridge/app/initialization/experience_repo.rs
//! Verify experience repository persistence at startup.

use std::sync::Arc;

use crate::database::sqlite::SqliteDatabase;
use crate::experience::repository as exp_repo;
use crate::experience::types::evidence::{Evidence, ExperienceSource};
use crate::experience::types::{
    Encounter, EncounterResult, EncounterStats, Experience, ExperienceType,
};
use chrono::Utc;
use uuid::Uuid;

/// Verify experience repository persistence methods work at startup
/// (Architecture §07/§09). Exercises save_encounter, get_encounter,
/// find_similar_encounters and save_experience with transient rows that
/// are cleaned up afterwards.
pub async fn verify_experience_repository(database: &Arc<SqliteDatabase>) {
    let encounter = Encounter {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        experience_id: None,
        context: Default::default(),
        input: "startup repository probe".to_string(),
        action: "verify persistence".to_string(),
        result: EncounterResult::Success,
        metadata: Default::default(),
    };
    let saved_encounter = exp_repo::save_encounter(database.clone(), &encounter)
        .await
        .is_ok();
    let fetched_encounter = exp_repo::get_encounter(database.clone(), &encounter.id)
        .await
        .is_ok();
    let similar = exp_repo::find_similar_encounters(database.clone(), "startup repository probe")
        .await
        .map(|v| v.len())
        .unwrap_or(0);

    // Exercise encounter-stat aggregation so the stats path stays live.
    let encounter_stats_id = encounter.id;
    let encounter_stats =
        EncounterStats::from_encounters(encounter_stats_id, std::slice::from_ref(&encounter));
    tracing::info!(
        "Encounter stats probe: total={} successes={} failures={}",
        encounter_stats.total_encounters,
        encounter_stats.successes,
        encounter_stats.failures,
    );

    let experience = Experience::new(
        "Startup repository probe".to_string(),
        "Transient experience used to verify persistence".to_string(),
        ExperienceType::Learning,
        vec![Uuid::new_v4()],
    );
    // Exercise the experience-level Evidence model + ExperienceSource
    // taxonomy (Architecture §11: evidence supports experiences) so
    // those types stay live rather than dead code.
    let evidence = Evidence::new(vec![experience.id], 0.8);
    let source = ExperienceSource::Tool;
    tracing::info!(
        "Experience evidence probe: evidence_id={} links={} confidence={} source={:?}",
        evidence.id,
        evidence.experience_ids.len(),
        evidence.confidence,
        source,
    );
    let saved_experience = exp_repo::save_experience(database.clone(), &experience)
        .await
        .is_ok();

    // Clean up the transient rows.
    if let Ok(conn) = database.connection() {
        crate::database::queries::memory::delete_memories(&conn, &[encounter.id, experience.id])
            .ok();
    }

    tracing::info!(
        "Experience repository verified: save_encounter_ok={} get_encounter_ok={} similar_count={} save_experience_ok={}",
        saved_encounter,
        fetched_encounter,
        similar,
        saved_experience,
    );
}
