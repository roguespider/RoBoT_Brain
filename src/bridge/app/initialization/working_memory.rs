// src/bridge/app/initialization/working_memory.rs
//! Learning WorkingMemory lifecycle probe at startup

/// Verify the Learning WorkingMemory lifecycle at startup.
/// Returns `Ok(())` on success, `Err(msg)` on failure.
pub(crate) async fn verify_working_memory() -> std::result::Result<(), String> {
    use crate::learning::working_memory::{
        MemoryItemType, WorkingMemory, WorkingMemoryItem,
        memory_state::{MemoryState, StateTransition, StateTransitionRecord},
        promotion::{PromotionEvaluation, PromotionPolicy},
    };
    let wm = WorkingMemory::with_policy(100, PromotionPolicy::lenient());
    wm.set_policy(PromotionPolicy::strict());
    let policy_ref = wm.policy();
    tracing::debug!(
        "WorkingMemory policy: min_access={}, min_conf={}",
        policy_ref.min_access_count,
        policy_ref.min_confidence
    );

    wm.store("probe:context", "ctx-val", MemoryItemType::Context, 0.9)
        .await
        .ok();
    wm.store("probe:task", "task-val", MemoryItemType::Task, 0.7)
        .await
        .ok();
    wm.store("probe:probe:result", "res-val", MemoryItemType::Result, 0.6)
        .await
        .ok();

    // Access repeatedly to drive the Active -> Repeated transition.
    for access in 0..4u32 {
        wm.get("probe:task").await;
        tracing::trace!("WorkingMemory probe access #{}", access);
    }
    wm.confirm("probe:task").await;
    wm.contradict("probe:context").await;
    wm.set_importance("probe:task", 0.95).await;
    wm.set_ttl("probe:task", Some(3600)).await;
    wm.reject("probe:context").await;

    let len = wm.len().await;
    let is_empty = wm.is_empty().await;
    let keys = wm.keys().await;
    let values = wm.values().await;
    let items = wm.items().await;
    let by_type = wm.get_by_type(MemoryItemType::Task).await;
    let by_state = wm.get_by_state(MemoryState::Confirmed).await;
    let promotable = wm.get_promotable().await;
    let recent = wm.get_recent(2).await;
    let important = wm.get_important(0.5).await;
    let by_pattern = wm.get_by_key_pattern("probe").await;
    let state = wm.get_state("probe:task").await;
    let history = wm.get_history("probe:task").await;
    let contains = wm.contains("probe:task").await;
    let peeked = wm.peek("probe:task").await;

    // Exercise the standalone PromotionEvaluation builders and the
    // confidence calculator directly.
    let eval_promote = PromotionEvaluation::promote(0.9, vec!["thresholds met".to_string()]);
    let eval_reject = PromotionEvaluation::reject(0.2, vec!["too few".to_string()]);
    let conf = policy_ref.calculate_confidence(5, 2);
    tracing::debug!(
        "PromotionEvaluation: promote={}, reject={}",
        eval_promote.should_promote,
        eval_reject.should_promote
    );

    // Exercise the item-level helpers on a constructed item.
    let mut probe_item = WorkingMemoryItem::new(
        "probe:item".to_string(),
        "v".to_string(),
        MemoryItemType::Context,
        0.8,
    );
    probe_item.record_access();
    let should_promote = probe_item.should_promote(policy_ref);
    let expired = probe_item.is_expired();
    tracing::debug!(
        "WorkingMemoryItem: should_promote={}, expired={}",
        should_promote,
        expired
    );

    // Exercise StateTransition/MemoryState methods directly.
    let st = StateTransition::Confirm;
    let valid_from = st.is_valid_from(MemoryState::Repeated);
    let target = st.target_state();
    let can_t = MemoryState::Active.can_transition(&StateTransition::Observe);
    let trans_to = MemoryState::Active.transition_to(&StateTransition::Observe);
    let record = StateTransitionRecord::new(
        MemoryState::Active,
        MemoryState::Repeated,
        StateTransition::Observe,
        Some("probe".to_string()),
    );
    tracing::debug!(
        "StateTransition: valid_from={}, target={:?}, can_transition={}, trans_to={:?}, record={:?}",
        valid_from,
        target,
        can_t,
        trans_to,
        record.transition
    );

    // process_all evaluates promotion policy across all items.
    let processed = wm.process_all().await;
    let stats = wm.stats().await;
    let promoted = wm.promote("probe:task").await;

    // Cleanup paths: clear_by_type, clear_by_state, remove, remove_many, clear_all.
    let cleared_type = wm.clear_by_type(MemoryItemType::Result).await;
    let cleared_state = wm.clear_by_state(MemoryState::Discarded).await;
    let removed = wm.remove("probe:task").await;
    let removed_many = wm.remove_many(&["probe:context", "missing"]).await;
    wm.clear_all().await;

    tracing::info!(
        "WorkingMemory lifecycle verified: len={}, empty={}, keys={}, values={}, items={}, by_type={}, by_state={}, promotable={}, recent={}, important={}, by_pattern={}, state={:?}, history={}, contains={}, peeked={}, processed={}, promoted={}, cleared_type={}, cleared_state={}, removed={}, removed_many={}, stats_total={}, conf={:.3}",
        len,
        is_empty,
        keys.len(),
        values.len(),
        items.len(),
        by_type.len(),
        by_state.len(),
        promotable.len(),
        recent.len(),
        important.len(),
        by_pattern.len(),
        state,
        history.map(|h| h.len()).unwrap_or(0),
        contains,
        peeked.is_some(),
        processed,
        promoted.is_some(),
        cleared_type,
        cleared_state,
        removed.is_some(),
        removed_many,
        stats.total_items,
        conf
    );
    Ok(())
}
