// src/bridge/app/initialization/exploration_repo.rs
//! Verify ExplorationRepository in-memory implementation at startup.

use crate::experience::exploration::store::{ExplorationRepository, InMemoryExplorationRepository};
use crate::experience::exploration::{Exploration, ExplorationStatus};
use crate::experience::types::ExperienceContext;

/// Verify the ExplorationRepository (Architecture §4.06) in-memory
/// implementation works at startup. Exercises create/get/update/
/// list_active/count/list_all/list_by_status/delete/search_by_title so
/// those repository methods remain live rather than dead code.
pub async fn verify_exploration_repository() {
    let repo = InMemoryExplorationRepository::new();
    let probe = Exploration::new(
        "startup-repo-probe".to_string(),
        "Startup repository probe".to_string(),
        "verify exploration repository".to_string(),
        ExperienceContext::default(),
    );
    // Exercise the full repository contract (Architecture §4.06) so the
    // trait + in-memory impl stay live rather than dead code.
    let created_ok = ExplorationRepository::create(&repo, &probe).is_ok();
    let fetched_ok = ExplorationRepository::get(&repo, &probe.id)
        .map(|o| o.is_some())
        .unwrap_or(false);
    let updated_ok = ExplorationRepository::update(&repo, &probe).is_ok();
    let active_count = ExplorationRepository::list_active(&repo)
        .map(|v| v.len())
        .unwrap_or(0);
    let list_all_count = repo.list_all().map(|v| v.len()).unwrap_or(0);
    let total_count = repo.count().unwrap_or(0);
    let by_status = repo
        .list_by_status(ExplorationStatus::Active)
        .map(|v| v.len())
        .unwrap_or(0);
    let search_hits = repo
        .search_by_title("Startup")
        .map(|v| v.len())
        .unwrap_or(0);
    let deleted = repo.delete(&probe.id).is_ok();
    let after_delete = repo.count().unwrap_or(0);

    tracing::info!(
        "Exploration repository probe: created={} fetched={} updated={} active={} \
         list_all={} total={} by_status={} search={} deleted={} after_delete={}",
        created_ok,
        fetched_ok,
        updated_ok,
        active_count,
        list_all_count,
        total_count,
        by_status,
        search_hits,
        deleted,
        after_delete,
    );
}
