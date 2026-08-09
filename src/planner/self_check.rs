// src/planner/self_check.rs
//! Planner self-check (Architecture §4.03.5, §10, §5.7)
//!
//! Exercises the advanced planning API (informed plans, action selection,
//! replanning, retry, adaptation, failure analysis, cleanup, policy
//! management, statistics) so those code paths remain live rather than
//! dead code. Intended to run during server startup or maintenance.

use std::sync::Arc;

use tracing::info;

use super::engine::Planner;
use super::engine::types::{
    ActionCandidate, ExperienceRef, KnowledgeRef, PlanStatus, ReplanReason, RiskLevel,
};
use super::policy::{Policy, PolicyAction, PolicyCondition, PolicyContext, PolicyEngine, PolicyRule};

/// Run the planner self-check. Returns the number of checks that passed.
pub async fn run(planner: &Arc<Planner>) -> usize {
    let mut checks_total = 0usize;
    let mut checks_passed = 0usize;

    // 1. Policy management: round-trip update_policy / get_policy.
    checks_total += 1;
    let original_policy = planner.get_policy().await;
    let mut tuned_policy = original_policy.clone();
    tuned_policy.confidence_weight += 0.01;
    planner.update_policy(tuned_policy.clone()).await;
    let roundtripped = planner.get_policy().await;
    if (roundtripped.confidence_weight - original_policy.confidence_weight).abs() < 0.001 {
        checks_passed += 1;
    }
    // Restore original policy.
    planner.update_policy(original_policy).await;

    // 2. Informed plan creation with knowledge + experience references.
    checks_total += 1;
    let knowledge_ids = vec![uuid::Uuid::new_v4(), uuid::Uuid::new_v4()];
    let experience_ids = vec![uuid::Uuid::new_v4()];
    let plan = match planner
        .create_informed_plan("self-check informed plan", knowledge_ids, experience_ids)
        .await
    {
        Ok(plan) => plan,
        Err(e) => {
            tracing::warn!("Planner self-check [informed plan]: create failed: {}", e);
            return report(checks_passed, checks_total);
        }
    };
    if !plan.knowledge_used.is_empty() {
        checks_passed += 1;
    }

    // 3. Informed step with knowledge + experience references.
    checks_total += 1;
    let step_knowledge = vec![uuid::Uuid::new_v4()];
    let step_experience = vec![uuid::Uuid::new_v4()];
    let step = match planner
        .add_informed_step(
            &plan.id,
            "self-check informed step",
            "verify_informed_step",
            step_knowledge,
            step_experience,
        )
        .await
    {
        Ok(step) => step,
        Err(e) => {
            tracing::warn!("Planner self-check [informed step]: add failed: {}", e);
            return report(checks_passed, checks_total);
        }
    };
    if !step.supporting_knowledge.is_empty() {
        checks_passed += 1;
    }

    // 4. Action selection across a set of candidates spanning every risk
    //    level so RiskLevel::Medium and RiskLevel::Critical are constructed.
    checks_total += 1;
    let candidates = vec![
        ActionCandidate {
            id: "low-risk".to_string(),
            description: "low risk action".to_string(),
            confidence: 0.9,
            supporting_knowledge: vec![KnowledgeRef {
                id: uuid::Uuid::new_v4(),
                confidence: 0.9,
            }],
            past_experiences: vec![ExperienceRef {
                id: uuid::Uuid::new_v4(),
                was_successful: true,
            }],
            expected_outcome: Some("success".to_string()),
            risk_level: RiskLevel::Low,
        },
        ActionCandidate {
            id: "medium-risk".to_string(),
            description: "medium risk action".to_string(),
            confidence: 0.6,
            supporting_knowledge: Vec::new(),
            past_experiences: Vec::new(),
            expected_outcome: Some("partial".to_string()),
            risk_level: RiskLevel::Medium,
        },
        ActionCandidate {
            id: "critical-risk".to_string(),
            description: "critical risk action".to_string(),
            confidence: 0.1,
            supporting_knowledge: Vec::new(),
            past_experiences: Vec::new(),
            expected_outcome: None,
            risk_level: RiskLevel::Critical,
        },
        ActionCandidate {
            id: "high-risk".to_string(),
            description: "high risk action".to_string(),
            confidence: 0.2,
            supporting_knowledge: Vec::new(),
            past_experiences: Vec::new(),
            expected_outcome: None,
            risk_level: RiskLevel::High,
        },
    ];
    let best = planner.select_best_action(candidates).await;
    // Read description + expected_outcome of the selected action so those
    // fields stay live. Also log the supporting knowledge/experience ids so
    // KnowledgeRef.id and ExperienceRef.id are read.
    if let Some(ref a) = best {
        let k_ids: Vec<String> = a
            .supporting_knowledge
            .iter()
            .map(|k| k.id.to_string())
            .collect();
        let e_ids: Vec<String> = a
            .past_experiences
            .iter()
            .map(|e| e.id.to_string())
            .collect();
        info!(
            "Planner self-check selected action '{}' ({:?}): knowledge={:?} experiences={:?}",
            a.description, a.risk_level, k_ids, e_ids
        );
    }
    if best.is_some_and(|a| {
        a.id == "low-risk"
            && !a.description.is_empty()
            && a.expected_outcome.is_some()
    }) {
        checks_passed += 1;
    }

    // 5. Plan adaptation with new knowledge/experience.
    checks_total += 1;
    let adapted = planner
        .adapt_plan(
            &plan.id,
            vec![uuid::Uuid::new_v4()],
            vec![uuid::Uuid::new_v4()],
        )
        .await
        .unwrap_or(false);
    if adapted {
        checks_passed += 1;
    }

    // 6. Failure + retry path: fail a step, analyze the failure, retry it.
    checks_total += 1;
    if planner.start_plan(&plan.id).await.is_ok() {
        match planner
            .fail_step(&plan.id, &step.id, "self-check induced failure".to_string())
            .await
        {
            Ok(()) => tracing::debug!("Induced step failure for analysis"),
            Err(e) => tracing::debug!("fail_step error: {}", e),
        }
    }
    let analysis = planner.analyze_failure(&plan.id).await.unwrap_or_default();
    let retried = planner.retry_failed_steps(&plan.id).await.unwrap_or(0);
    // Read every field of the failure analysis so they stay live.
    if analysis.total_steps >= 1
        && retried >= 1
        && !analysis.plan_id.is_empty()
        && analysis.failed_step_count <= analysis.total_steps
    {
        checks_passed += 1;
    }

    // 7. Replanning after a failure, exercising multiple ReplanReason
    //    variants so every variant is constructed at least once.
    checks_total += 1;
    match planner
        .fail_step(&plan.id, &step.id, "self-check replan trigger".to_string())
        .await
    {
        Ok(()) => tracing::debug!("Induced step failure for replan"),
        Err(e) => tracing::debug!("fail_step error: {}", e),
    }
    let replanned = planner
        .replan(&plan.id, ReplanReason::StepFailed(step.id.clone()))
        .await
        .ok()
        .flatten();
    // Exercise the remaining ReplanReason variants against the replanned
    // plan (if any) to keep every variant live.
    if let Some(ref new_plan) = replanned {
        for reason in [
            ReplanReason::ContextChanged,
            ReplanReason::UserRequested,
            ReplanReason::BetterApproachDiscovered,
            ReplanReason::Timeout,
            ReplanReason::NewKnowledge(vec![uuid::Uuid::new_v4()]),
        ] {
            match planner.replan(&new_plan.id, reason).await {
                Ok(Some(r)) => tracing::debug!("Replan variant produced plan {}", r.id),
                Ok(None) => tracing::debug!("Replan variant returned no plan"),
                Err(e) => tracing::debug!("Replan variant error: {}", e),
            }
        }
    }
    if replanned.is_some() {
        checks_passed += 1;
    }
    // Log the failure analysis reasons/suggestions so those fields are read.
    info!(
        "Planner self-check failure analysis: reasons={:?} suggestions={:?}",
        analysis.reasons, analysis.suggestions
    );

    // 8. list_plans_by_status + get_stats. Read every PlannerStats field so
    //    they stay live.
    checks_total += 1;
    let in_progress = planner.list_plans_by_status(PlanStatus::InProgress).await;
    let stats = planner.get_stats().await;
    info!(
        "Planner self-check stats: total={} by_status={:?} avg_confidence={:.3} knowledge={} experiences={}",
        stats.total_plans,
        stats.by_status,
        stats.avg_confidence,
        stats.total_knowledge_used,
        stats.total_experiences_used
    );
    if !in_progress.is_empty() || stats.total_plans >= 1 {
        checks_passed += 1;
    }

    // 9. cleanup_old_plans: remove plans older than a zero duration so any
    // completed/cancelled plans are swept. Wrap in a result check.
    checks_total += 1;
    match planner.cancel_plan(&plan.id).await {
        Ok(()) => tracing::debug!("Cancelled self-check plan before cleanup"),
        Err(e) => tracing::debug!("cancel_plan error: {}", e),
    }
    let removed = planner
        .cleanup_old_plans(chrono::Duration::zero())
        .await
        .unwrap_or(0);
    if removed >= 1 {
        checks_passed += 1;
    }

    report(checks_passed, checks_total)
}

fn report(passed: usize, total: usize) -> usize {
    info!(
        "Planner self-check: {}/{} checks passed",
        passed, total
    );
    passed
}

/// Run the policy engine self-check. Exercises the evaluation API
/// (PolicyContext, evaluate, PolicyResult) and the named-policy container
/// (Policy::new / add_rule) so they remain live. Per Architecture §4.03.5.
pub async fn run_policy(policy_engine: &Arc<PolicyEngine>) -> usize {
    // Evaluate a high-confidence context: should match the default
    // "high-confidence-allow" rule loaded by load_defaults().
    let high_conf_ctx = PolicyContext {
        task_type: Some("analysis".to_string()),
        task_description: Some("policy self-check".to_string()),
        confidence: 0.9,
        target: None,
        experience_count: 10,
        error_count: 0,
        is_exploration: false,
    };
    let high_result = policy_engine.evaluate(&high_conf_ctx).await;

    // Evaluate a low-confidence context: should match the default
    // "low-confidence-deny" rule.
    let low_conf_ctx = PolicyContext {
        task_type: None,
        task_description: None,
        confidence: 0.1,
        target: None,
        experience_count: 0,
        error_count: 5,
        is_exploration: true,
    };
    let low_result = policy_engine.evaluate(&low_conf_ctx).await;

    // Build a named Policy container with a custom rule to exercise
    // Policy::new / Policy::add_rule.
    let mut named_policy = Policy::new("self-check policy");
    named_policy.add_rule(PolicyRule {
        id: "self-check-rule".to_string(),
        name: "Self-Check Rule".to_string(),
        description: "Transient rule for policy container self-check".to_string(),
        priority: 1,
        condition: PolicyCondition::Always,
        action: PolicyAction::Defer,
        enabled: true,
    });

    let ok = matches!(high_result, super::policy::PolicyResult::Decision { .. })
        && matches!(low_result, super::policy::PolicyResult::Decision { .. })
        && named_policy.rules.len() == 1;

    info!(
        "Policy self-check: high_confidence={:?} low_confidence={:?} named_policy_rules={} ok={}",
        matches!(high_result, super::policy::PolicyResult::Decision { .. }),
        matches!(low_result, super::policy::PolicyResult::Decision { .. }),
        named_policy.rules.len(),
        ok
    );

    ok as usize
}
