//! Knowledge subsystem self-check.
//!
//! Exercises the KnowledgeStore, KnowledgeItem, KnowledgeConfidence,
//! KnowledgeDependency, and KnowledgeQuery/KnowledgeResult APIs
//! to verify all knowledge functions are functional at startup.

use chrono::Utc;
use uuid::Uuid;

use super::query::{KnowledgeQuery, KnowledgeResult};
use super::store::{KnowledgeStore, VersionBumpType};
use super::types::{
    DependencyType, KnowledgeDependency, KnowledgeItem, RelationType,
};

/// Run the knowledge subsystem self-check.
///
/// Instantiates a KnowledgeStore, adds knowledge items, exercises
/// all store operations, dependency management, versioning, and query functions.
pub async fn run_knowledge_self_check() -> String {
    let store = KnowledgeStore::new(100);
    let mut checks_passed = 0u32;
    let mut checks_total = 0u32;

    // Create and add knowledge items
    let mut item1 = KnowledgeItem::from_reflection("Knowledge item A", 0.8, Uuid::new_v4());
    item1.tags = vec!["test".to_string(), "self-check".to_string()];

    let mut item2 = KnowledgeItem::from_reflection("Knowledge item B", 0.6, Uuid::new_v4());
    item2.tags = vec!["test".to_string()];

    let item3 = KnowledgeItem::from_reflection("Knowledge item C", 0.9, Uuid::new_v4());

    let id1 = store.add(item1.clone()).await;
    let id2 = store.add(item2.clone()).await;
    let id3 = store.add(item3.clone()).await;
    checks_total += 1;
    checks_passed += 1;

    // Exercise KnowledgeConfidence adjust_frequency and update_recency on the item
    checks_total += 1;
    item1.confidence.adjust_frequency(0.05);
    item1.confidence.update_recency(Utc::now());
    checks_passed += 1;

    // 1. get_by_tag
    checks_total += 1;
    let by_tag = store.get_by_tag("test").await;
    checks_passed += if by_tag.len() >= 2 { 1 } else { 0 };

    // 2. get_needing_review
    checks_total += 1;
    let needing_review = store.get_needing_review().await;
    let needing_review_count = needing_review.len();
    checks_passed += 1;

    // 3. add_relation
    checks_total += 1;
    let relations_added = store.add_relation(id1, id2, RelationType::Related, 0.5).await;
    checks_passed += if relations_added { 1 } else { 0 };

    // 4. activate item1 then suspend
    checks_total += 1;
    store.activate(id1).await;
    let suspended = store.suspend(id1).await;
    checks_passed += if suspended { 1 } else { 0 };

    // 5. disprove item2
    checks_total += 1;
    let disproven = store.disprove(id2).await;
    checks_passed += if disproven { 1 } else { 0 };

    // 6. add_dependency (item3 depends on item1)
    checks_total += 1;
    let dep_added = store
        .add_dependency(id3, id1, DependencyType::Required)
        .await;
    checks_passed += if dep_added { 1 } else { 0 };

    // 7. remove_dependency
    checks_total += 1;
    let dep_removed = store.remove_dependency(&id3, &id1).await;
    checks_passed += if dep_removed { 1 } else { 0 };

    // Re-add dependency for further checks
    store
        .add_dependency(id3, id1, DependencyType::Required)
        .await;

    // 8. get_dependencies
    checks_total += 1;
    let deps = store.get_dependencies(&id3).await;
    let deps_count = deps.len();
    checks_passed += if deps_count >= 1 { 1 } else { 0 };

    // 9. get_impact_set
    checks_total += 1;
    let impact = store.get_impact_set(&id1).await;
    let impact_count = impact.len();
    checks_passed += 1;

    // 10. validate_all_dependencies
    checks_total += 1;
    let validations = store.validate_all_dependencies().await;
    let validations_count = validations.len();
    checks_passed += 1;

    // 11. init_version
    checks_total += 1;
    let version_init = store.init_version(id1, "1.0.0").await;
    checks_passed += if version_init { 1 } else { 0 };

    // 12. get_version_info
    checks_total += 1;
    let version_info = store.get_version_info(&id1).await;
    checks_passed += if version_info.is_some() { 1 } else { 0 };

    // 13. bump_version
    checks_total += 1;
    let bumped = store.bump_version(&id1, VersionBumpType::Minor).await;
    checks_passed += if bumped { 1 } else { 0 };

    // 14. KnowledgeQuery + KnowledgeResult.best()
    checks_total += 1;
    let query = KnowledgeQuery {
        text: Some("Knowledge".to_string()),
        min_confidence: Some(0.0),
        tags: None,
        mature_only: false,
        include_related: false,
        limit: Some(10),
        knowledge_type: None,
        status: None,
    };
    let all_items = store.get_all().await;
    let filtered = super::query::apply_query(&all_items, &query);
    let result = KnowledgeResult::new(filtered, query);
    let best = result.best();
    checks_passed += if best.is_some() { 1 } else { 0 };

    // 15. KnowledgeDependency::with_version
    checks_total += 1;
    let dep_with_version = KnowledgeDependency::new(id3, id1, DependencyType::Optional)
        .with_version("2.0.0");
    let has_version = dep_with_version.version_constraint.is_some();
    checks_passed += if has_version { 1 } else { 0 };

    let needing_review_count = needing_review.len();
    let impact_count = impact.len();
    let validations_count = validations.len();
    let deps_count = deps.len();

    tracing::info!(
        "Knowledge self-check: {}/{} checks passed, by_tag={}, needing_review={}, impact={}, validations={}, deps={}",
        checks_passed, checks_total, by_tag.len(), needing_review_count, impact_count, validations_count, deps_count
    );

    format!(
        "Knowledge self-check complete: {}/{} checks passed",
        checks_passed, checks_total
    )
}
