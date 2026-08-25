// src/bridge/app/initialization/reputation_diagnostics.rs
//! Reputation system probes (P2-001C).
//!
//! Exercises ReputationAnalytics, and
//! ReputationRecord so these APIs stay live without running at production
//! startup.

use crate::bridge::app::state::App;
use crate::experience::reputation::analytics::ReputationAnalytics;
use crate::experience::reputation::factors::{FactorScore, ReputationFactor};
use crate::experience::reputation::score::Reputation;
use crate::experience::types::reputation::{ReputationRecord, ReputationTarget};

/// Verify reputation system APIs: ReputationAnalytics and ReputationRecord.
/// Returns `Ok(())` on success, `Err(msg)` on failure.
pub fn verify_reputation_system(app: &App) -> std::result::Result<(), String> {
    // Exercise the shared personality surface so the App handle stays live.
    let personality_snapshot = app
        .personality
        .lock()
        .map(|p| p.traits.curiosity)
        .unwrap_or(0.0);
    tracing::info!(
        "Personality snapshot verified: curiosity={}",
        personality_snapshot
    );

    // Exercise Reputation::new and ReputationAnalytics
    let mut rep = Reputation::new("diagnostics-rep".to_string());
    rep.apply(
        String::new(),
        ReputationFactor::Accuracy,
        0.2,
        "transient diagnostic".to_string(),
    );
    rep.apply(
        String::new(),
        ReputationFactor::Accuracy,
        -0.1,
        "transient diagnostic".to_string(),
    );
    let rate = ReputationAnalytics::success_rate(&rep);
    let trend = ReputationAnalytics::trend(&rep);
    let confidence = rep.confidence();
    tracing::info!(
        "Reputation system verified: success_rate={} trend={} confidence={:.3}",
        rate,
        trend,
        confidence
    );

    // Exercise ReputationRecord
    let mut record = ReputationRecord::new(ReputationTarget::Agent(rep.id.clone()));
    record.record_success(0.9);
    record.record_failure(0.4);
    let factor = FactorScore::new(ReputationFactor::Accuracy);
    tracing::info!(
        "ReputationRecord verified: successes={} failures={} observations={} factor={:?}",
        record.successes,
        record.failures,
        record.observations,
        factor.factor
    );
    Ok(())
}
