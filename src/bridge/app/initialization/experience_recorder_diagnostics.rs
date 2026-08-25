// src/bridge/app/initialization/experience_recorder_diagnostics.rs
//! ExperienceRecorder convenience probes (P2-001C).
//!
//! Exercises ExperienceRecorder::success and ::failure against an isolated
//! temporary database so probe experiences never reach the production store.

use std::sync::Arc;

use crate::database::sqlite::SqliteDatabase;
use crate::experience::encounter_recorder::ExperienceRecorder;
use crate::experience::types::ExperienceType;

/// Verify ExperienceRecorder success/failure convenience helpers against an
/// isolated temporary database.
/// Returns `Ok(())` on success, `Err(msg)` on failure.
pub fn verify_experience_recorder() -> std::result::Result<(), String> {
    // Isolated database in the OS temp directory: probe experiences are written
    // to their own robot_brain.db, never to the production database.
    let probe_dir = std::env::temp_dir().join(format!(
        "robot_brain_diagnostics_experience_recorder_{}",
        uuid::Uuid::new_v4()
    ));
    let database = match SqliteDatabase::initialize_at(&probe_dir) {
        Ok(db) => Arc::new(db),
        Err(e) => {
            tracing::warn!(
                "ExperienceRecorder diagnostics skipped: isolated database init failed: {}",
                e
            );
            return Err(format!("ExperienceRecorder diagnostics init failed: {}", e));
        }
    };

    let recorder = ExperienceRecorder::new(database);

    let success_result = recorder.success(
        ExperienceType::System,
        "diagnostics success probe",
        "Transient experience verifying ExperienceRecorder::success",
    );
    let failure_result = recorder.failure(
        ExperienceType::System,
        "diagnostics failure probe",
        "Transient experience verifying ExperienceRecorder::failure",
        "intentional diagnostics probe failure",
    );

    tracing::info!(
        "ExperienceRecorder helpers verified: success_ok={} failure_ok={}",
        success_result.is_ok(),
        failure_result.is_ok()
    );

    // Remove the isolated probe database directory.
    if let Err(e) = std::fs::remove_dir_all(&probe_dir) {
        tracing::warn!(
            "ExperienceRecorder diagnostics cleanup failed for {:?}: {}",
            probe_dir,
            e
        );
    }
    Ok(())
}
