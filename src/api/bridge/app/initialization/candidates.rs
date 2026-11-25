// src/bridge/app/initialization/candidates.rs
//! CandidateGenerator lifecycle probe at startup

/// Verify the CandidateGenerator lifecycle works at startup.
/// Returns `Ok(())` on success, `Err(msg)` on failure.
pub(crate) async fn verify_candidates() -> std::result::Result<(), String> {
    use crate::learning::candidates::{CandidateGenerator, CandidateScore, CandidateType};

    let generator = CandidateGenerator::new();
    generator
        .generate(
            "probe-candidate",
            "startup lifecycle probe",
            CandidateType::Behavior,
        )
        .await;
    generator
        .generate(
            "probe-candidate-2",
            "low-risk probe",
            CandidateType::Strategy,
        )
        .await;
    let top = generator.get_top(2).await;
    let low_risk = generator.get_low_risk().await;
    // Exercise RiskLevel::as_str so the helper stays live.
    let risk_labels: Vec<&'static str> = low_risk.iter().map(|c| c.risk_level.as_str()).collect();
    let by_type = generator.get_by_type(CandidateType::Behavior).await;
    let all = generator.list().await;
    if let Some(first) = top.first() {
        if let Err(e) = generator
            .update_score(&first.id, CandidateScore::new(0.9))
            .await
        {
            tracing::warn!("CandidateGenerator update_score failed: {}", e);
        }
        if let Err(e) = generator.select(&first.id).await {
            tracing::warn!("CandidateGenerator select failed: {}", e);
        }
        if generator.get(&first.id).await.is_some() {
            tracing::debug!("CandidateGenerator get ok");
        }
        let removed = generator.remove(&first.id).await;
        if let Some(c) = removed {
            generator.add(c).await;
        }
    }
    let history = generator.get_history().await;
    let stats = generator.stats().await;
    generator.clear().await;
    tracing::info!(
        "CandidateGenerator lifecycle verified: top={}, low_risk={}, by_type={}, all={}, history={}, stats_total={}, risk_labels={}",
        top.len(),
        low_risk.len(),
        by_type.len(),
        all.len(),
        history.len(),
        stats.total,
        risk_labels.len()
    );
    Ok(())
}
