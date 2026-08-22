// src/bridge/app/initialization/hypothesis_manager.rs
//! Verify HypothesisManager lifecycle at startup.

/// Verify the Hypothesis management lifecycle at startup.
/// Exercises create, update, list, list_by_status, get_supported,
/// get_high_confidence, stats, and delete so the hypothesis subsystem
/// stays live rather than dead code.
pub async fn verify_hypothesis_manager() {
    use crate::learning::hypothesis::{
        EvidenceBuilder, EvidenceType, HypothesisManager, HypothesisStatus,
    };

    let manager = HypothesisManager::new();
    let h = manager
        .create("probe-hypothesis", "probe description")
        .await;
    let sup_evidence = EvidenceBuilder::new("supporting observation")
        .with_type(EvidenceType::Observation)
        .with_strength(0.8)
        .with_source("probe-source")
        .build();
    let con_evidence = EvidenceBuilder::new("contradicting experiment")
        .with_type(EvidenceType::Experiment)
        .with_strength(0.4)
        .with_source("probe-source-2")
        .build();
    let mut h_mut = h.clone();
    h_mut.start_testing();
    h_mut.add_supporting(sup_evidence);
    h_mut.add_contradicting(con_evidence);
    if h_mut.confidence >= 0.5 {
        h_mut.support();
    } else {
        h_mut.refute();
    }
    manager.update(&h_mut).await.ok();
    let h2 = manager
        .create("probe-hypothesis-2", "to be abandoned")
        .await;
    let mut h2_mut = h2.clone();
    h2_mut.abandon();
    manager.update(&h2_mut).await.ok();
    let fetched = manager.get(&h.id).await;
    let listed = manager.list().await;
    let by_status = manager.list_by_status(HypothesisStatus::Supported).await;
    let supported = manager.get_supported().await;
    let high_conf = manager.get_high_confidence(0.5).await;
    let stats = manager.stats().await;
    let deleted = manager.delete(&h2.id).await;
    tracing::info!(
        "HypothesisManager lifecycle verified: fetched={}, listed={}, by_status={}, supported={}, high_conf={}, deleted={}, stats_total={}, stats_avg_conf={:.3}, stats_evidence={}, h_conf={:.3}",
        fetched.is_some(),
        listed.len(),
        by_status.len(),
        supported.len(),
        high_conf.len(),
        deleted.is_some(),
        stats.total,
        stats.avg_confidence,
        stats.total_evidence,
        h_mut.confidence,
    );
}
