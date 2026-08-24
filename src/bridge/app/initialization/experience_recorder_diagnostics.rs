// src/bridge/app/initialization/experience_recorder_diagnostics.rs
//! ExperienceRecorder convenience helper probes (P2-001C).
//!
//! Exercises ExperienceRecorder::success and ::failure so these production
//! convenience methods stay live without running at production startup.

use crate::bridge::app::state::App;
use crate::experience::types::ExperienceType;

/// Verify ExperienceRecorder success/failure convenience helpers.
pub fn verify_experience_recorder(app: &App) {
    let recorder = &app.experience_recorder;

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
}
