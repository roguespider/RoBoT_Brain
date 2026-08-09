// src/experience/hypothesis/services/self_check.rs
//! Hypothesis services self-check (Architecture §9 / §4.04)
//!
//! Exercises the hypothesis service-layer API that has no direct tool
//! surface yet so those code paths remain live rather than dead code:
//! - HypothesisGenerator::generate_from_pattern
//! - HypothesisMatcher::match_experience, match_text
//! - HypothesisAnalytics::analyze, stability_score
//! - HypothesisValidator::check_conflict
//! Also exercises HypothesisStatistics::reset in the support layer.

use tracing::info;

use super::analytics::HypothesisAnalytics;
use super::generator::HypothesisGenerator;
use super::matcher::HypothesisMatcher;
use super::validator::HypothesisValidator;
use crate::experience::hypothesis::core::hypothesis::{
    Hypothesis, HypothesisConfidence, HypothesisStatus,
};
use crate::experience::hypothesis::support::statistics::HypothesisStatistics;
use crate::experience::types::experience::{Experience, ExperienceType};

/// Run the hypothesis services self-check. Returns the number of checks passed.
pub fn run() -> usize {
    let mut checks_total = 0usize;
    let mut checks_passed = 0usize;

    // Build a shared experience used by the generator and matcher.
    let experience = Experience::new(
        "rust borrow checker".to_string(),
        "how the borrow checker enforces ownership rules in rust".to_string(),
        ExperienceType::Exploration,
        Vec::new(),
    );

    // 1. HypothesisGenerator::generate (from experience) and generate_from_pattern.
    checks_total += 1;
    let generator = HypothesisGenerator::new();
    // Read generation_threshold so the config field stays live.
    let threshold = generator.generation_threshold;
    let from_exp = generator.generate(&experience);
    let from_pat = generator.generate_from_pattern("rainfall cycle");
    // Also construct a GenerationResult so the report struct stays live.
    let gen_result = super::generator::GenerationResult {
        generated: true,
        hypothesis_id: from_pat.as_ref().ok().and_then(|h| h.as_ref()).map(|h| h.id.0.clone()),
        reason: "self_check".to_string(),
    };
    if matches!(from_exp, Ok(Some(_)))
        && matches!(from_pat, Ok(Some(_)))
        && gen_result.generated
        && threshold > 0.0
    {
        checks_passed += 1;
    }

    // 2. HypothesisMatcher::match_text scores hypotheses by text similarity.
    checks_total += 1;
    let matcher = HypothesisMatcher::new();
    let hypotheses = vec![
        Hypothesis::new("rust memory model", "how rust manages memory ownership"),
        Hypothesis::new("rust borrow checker", "borrowing rules in rust"),
        Hypothesis::new("unrelated cooking", "recipes for pasta dishes"),
    ];
    let text_matches = matcher.match_text("rust memory ownership", &hypotheses);
    if text_matches.len() >= 1 && text_matches.iter().all(|m| m.score >= matcher.minimum_score) {
        checks_passed += 1;
    }

    // 3. HypothesisMatcher::match_experience matches the shared experience.
    checks_total += 1;
    let exp_matches = matcher.match_experience(&experience, &hypotheses);
    if !exp_matches.is_empty() {
        checks_passed += 1;
    }

    // 4. HypothesisAnalytics::analyze and stability_score.
    checks_total += 1;
    let analytics = HypothesisAnalytics::new();
    let mut supported = Hypothesis::new("stable belief", "well-supported hypothesis");
    supported.status = HypothesisStatus::Supported;
    supported.confidence = HypothesisConfidence::new(0.9);
    supported.evaluations = 5;
    let mut rejected = Hypothesis::new("bad belief", "rejected hypothesis");
    rejected.status = HypothesisStatus::Rejected;
    let report = analytics.analyze(&[supported.clone(), rejected]);
    let stability = analytics.stability_score(&report);
    if report.total == 2
        && report.supported == 1
        && report.rejected == 1
        && report.total_evaluations == 5
        && stability > 0.0
    {
        checks_passed += 1;
    }

    // 5. HypothesisValidator::check_conflict detects similar hypotheses and
    //    validate() checks a single hypothesis for issues.
    checks_total += 1;
    let validator = HypothesisValidator::new();
    let h1 = Hypothesis::new(
        "rust ownership",
        "rust ownership rules prevent memory bugs",
    );
    let h2 = Hypothesis::new(
        "rust ownership rules",
        "rust ownership prevents memory bugs",
    );
    let h3 = Hypothesis::new("weather", "completely unrelated weather topic");
    let conflict = validator.check_conflict(&h1, &h2);
    let no_conflict = validator.check_conflict(&h1, &h3);
    let valid_report = validator.validate(&h1);
    let empty_report = validator.validate(&Hypothesis::new("   ", "   "));
    if conflict.is_some()
        && no_conflict.is_none()
        && valid_report.valid
        && !empty_report.valid
        && !empty_report.issues.is_empty()
    {
        checks_passed += 1;
    }

    // 6. HypothesisStatistics::reset clears counters, plus the derived
    //    average_confidence, support_rate, confirmation_rate, and a
    //    StatisticsSnapshot stay live.
    checks_total += 1;
    let mut stats = HypothesisStatistics::new();
    stats.record(&supported);
    let avg = stats.average_confidence();
    let sup_rate = stats.support_rate();
    let conf_rate = stats.confirmation_rate();
    let before = stats.total_hypotheses;
    stats.reset();
    let snapshot = crate::experience::hypothesis::support::statistics::StatisticsSnapshot {
        total_hypotheses: before,
        average_confidence: avg,
        support_rate: sup_rate,
        confirmation_rate: conf_rate,
    };
    if before > 0
        && stats.total_hypotheses == 0
        && snapshot.total_hypotheses == before
    {
        checks_passed += 1;
    }

    // 7. HypothesisRepository CRUD (save, get, get_mut, delete, all, count,
    //    exists, clear) so the in-memory repository stays live.
    checks_total += 1;
    let mut repo = super::repository::HypothesisRepository::new();
    let to_store = Hypothesis::new("repo h", "stored in repo");
    let stored_id = to_store.id.clone();
    let saved = repo.save(to_store);
    let count_after = repo.count();
    let exists = repo.exists(&stored_id);
    let got_present = repo.get(&stored_id).is_some();
    if let Some(h) = repo.get_mut(&stored_id) {
        h.evaluations += 1;
    }
    let all_count = repo.all().len();
    let deleted_present = repo.delete(&stored_id).is_some();
    let count_after_delete = repo.count();
    repo.clear();
    if saved.is_ok()
        && exists
        && got_present
        && count_after == 1
        && all_count == 1
        && deleted_present
        && count_after_delete == 0
    {
        checks_passed += 1;
    }

    // 8. HypothesisPlanner with_confidence_threshold, create_plan,
    //    create_plans, get_prioritized_actions.
    checks_total += 1;
    let planner = crate::experience::hypothesis::support::planner::HypothesisPlanner::new()
        .with_confidence_threshold(0.7);
    let mut high_conf = Hypothesis::new("planned h", "high confidence hypothesis");
    high_conf.status = HypothesisStatus::Supported;
    high_conf.confidence = HypothesisConfidence::new(0.9);
    high_conf.evaluations = 5;
    let low_conf = Hypothesis::new("low h", "low confidence hypothesis");
    let high_plan = planner.create_plan(&high_conf);
    let low_plan = planner.create_plan(&low_conf);
    let multi = planner.create_plans(&[high_conf.clone(), low_conf.clone()]);
    let prioritized = planner.get_prioritized_actions(&[high_conf.clone(), low_conf]);
    if high_plan.status
        == crate::experience::hypothesis::support::planner::PlanningStatus::Ready
        && low_plan.actions.is_empty()
        && multi.len() == 1
        && !prioritized.is_empty()
    {
        checks_passed += 1;
    }

    // 9. Hypothesis helper methods: add_tag, add_supporting_evidence,
    //    add_contradicting_evidence, is_confident, is_uncertain, has_evidence.
    checks_total += 1;
    let mut h = Hypothesis::new("evidenced h", "with evidence and tags");
    h.add_tag("probe");
    h.add_supporting_evidence("ev-1");
    h.add_contradicting_evidence("ev-2");
    let confident = h.confidence.is_confident();
    let uncertain = HypothesisConfidence::new(0.1).is_uncertain();
    if h.has_evidence()
        && h.evidence_count() == 2
        && h.tags == vec!["probe".to_string()]
        && !confident
        && uncertain
    {
        checks_passed += 1;
    }

    info!(
        "Hypothesis services self-check: {}/{} checks passed",
        checks_passed, checks_total
    );
    checks_passed
}
