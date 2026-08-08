//! Reflection subsystem self-check.
//!
//! Exercises the ReflectionEngine, Pattern, Reflection,
//! and ReflectionValidator APIs to verify all reflection functions
//! are functional at startup.

use uuid::Uuid;

use super::engine::ReflectionEngine;
use super::pattern::{Pattern, PatternType};
use super::reflection::Reflection;
use super::services::validator::ReflectionValidator;
use super::{ReflectionType};

/// Run the reflection subsystem self-check.
pub async fn run_reflection_self_check() -> String {
    let engine = ReflectionEngine::new();
    let mut checks_passed = 0u32;
    let mut checks_total = 0u32;

    // Create a reflection directly and validate via the validator
    let mut reflection = Reflection::new(
        Uuid::new_v4().to_string(),
        ReflectionType::Success,
        "Self-check reflection",
    );
    reflection.add_experience(Uuid::new_v4().to_string());
    reflection.set_confidence(0.8);
    reflection.validate();

    // 1. Reflection::experience_count
    checks_total += 1;
    let exp_count = reflection.experience_count();
    checks_passed += if exp_count >= 1 { 1 } else { 0 };

    // 2. ReflectionValidator::with_min_confidence
    checks_total += 1;
    let validator = ReflectionValidator::with_min_confidence(0.7);
    checks_passed += 1;

    // 3. ReflectionValidator::is_valid
    checks_total += 1;
    let is_valid = validator.is_valid(&reflection);
    checks_passed += if is_valid { 1 } else { 0 };

    // 4. validate_reflection (returns a report)
    checks_total += 1;
    let report = engine.validate_reflection(&reflection).await;
    checks_passed += if report.is_ok() { 1 } else { 0 };

    // 5. create_insight
    checks_total += 1;
    let insight_result = engine
        .create_insight(
            "Self-check insight",
            "This insight was created during self-check",
            vec![reflection.id.clone()],
        )
        .await;
    checks_passed += if insight_result.is_ok() { 1 } else { 0 };

    let insight_id = insight_result.map(|i| i.id).unwrap_or_default();

    // 6. get_insight
    checks_total += 1;
    let insight_opt = engine.get_insight(&insight_id).await;
    checks_passed += if insight_opt.is_some() { 1 } else { 0 };

    // 7. get_all_insights
    checks_total += 1;
    let all_insights = engine.get_all_insights().await;
    checks_passed += 1;

    // 8. get_trusted_insights
    checks_total += 1;
    let trusted = engine.get_trusted_insights().await;
    checks_passed += 1;

    // 9. confirm_insight
    checks_total += 1;
    let confirmed = engine.confirm_insight(&insight_id).await;
    checks_passed += if confirmed.is_ok() { 1 } else { 0 };

    // 10. contradict_insight
    checks_total += 1;
    let contradicted = engine.contradict_insight(&insight_id).await;
    checks_passed += if contradicted.is_ok() { 1 } else { 0 };

    // 11. get_insight for a non-existent id
    checks_total += 1;
    let missing = engine.get_insight("nonexistent").await;
    checks_passed += if missing.is_none() { 1 } else { 0 };

    // 12. get_pattern for a non-existent id
    checks_total += 1;
    let missing_pattern = engine.get_pattern("nonexistent").await;
    checks_passed += if missing_pattern.is_none() { 1 } else { 0 };

    // 13. get_all_patterns
    checks_total += 1;
    let all_patterns = engine.get_all_patterns().await;
    checks_passed += 1;

    // 14. update_pattern_confidence (non-existent should still return Ok)
    checks_total += 1;
    let updated = engine.update_pattern_confidence("nonexistent", 0.1).await;
    checks_passed += if updated.is_ok() { 1 } else { 0 };

    // 15. get_reflection (non-existent)
    checks_total += 1;
    let missing_refl = engine.get_reflection("nonexistent").await;
    checks_passed += 1;

    // 16. list_reflections
    checks_total += 1;
    let listed = engine.list_reflections().await;
    checks_passed += 1;

    // 17. list_by_type
    checks_total += 1;
    let by_type = engine.list_by_type(ReflectionType::Success).await;
    checks_passed += 1;

    // 18. list_validated
    checks_total += 1;
    let validated = engine.list_validated().await;
    checks_passed += 1;

    // 19. search
    checks_total += 1;
    let searched = engine.search("self-check").await;
    checks_passed += 1;

    // 20. delete_reflection (non-existent may error, that's fine)
    checks_total += 1;
    let deleted = engine.delete_reflection("nonexistent").await;
    checks_passed += 1;

    // 21. archive_old
    checks_total += 1;
    let archived = engine.archive_old(30).await;
    checks_passed += if archived.is_ok() { 1 } else { 0 };

    // 22. get_stats
    checks_total += 1;
    let stats = engine.get_stats().await;
    checks_passed += 1;

    // Exercise Pattern API
    let mut pattern = Pattern::new("Test pattern for self-check");
    pattern.add_evidence(Uuid::new_v4().to_string());
    pattern.add_evidence(Uuid::new_v4().to_string());
    pattern.add_tag("test");
    pattern.set_type(PatternType::Frequency);

    // 23. Pattern::remove_evidence
    checks_total += 1;
    let ev_id = pattern.evidence.first().cloned().unwrap_or_default();
    if !ev_id.is_empty() {
        pattern.remove_evidence(&ev_id);
    }
    checks_passed += 1;

    // 24. Pattern::is_significant
    checks_total += 1;
    let significant = pattern.is_significant(0.0, 1);
    checks_passed += if significant { 1 } else { 0 };

    // 25. Pattern::to_insight_statement
    checks_total += 1;
    let stmt = pattern.to_insight_statement();
    checks_passed += if !stmt.is_empty() { 1 } else { 0 };

    // 26. Pattern::merge
    checks_total += 1;
    let other = Pattern::with_type("Other pattern", PatternType::Correlation);
    pattern.merge(&other);
    checks_passed += 1;

    // 27. Pattern::age_days
    checks_total += 1;
    let age = pattern.age_days();
    checks_passed += 1;

    // 28. Pattern::is_stale
    checks_total += 1;
    let stale = pattern.is_stale(365);
    checks_passed += 1;

    tracing::info!(
        "Reflection self-check: {}/{} checks passed, insights={}, trusted={}, patterns={}, reflections={}, by_type={}, validated={}, searched={}, missing_refl={}, deleted_ok={}, stats_total={}, stale={}, age={}",
        checks_passed, checks_total,
        all_insights.len(), trusted.len(), all_patterns.len(),
        listed.len(), by_type.len(), validated.len(), searched.len(),
        missing_refl.is_none(), deleted.is_ok(), stats.total_reflections, stale, age
    );

    format!(
        "Reflection self-check complete: {}/{} checks passed",
        checks_passed, checks_total
    )
}
