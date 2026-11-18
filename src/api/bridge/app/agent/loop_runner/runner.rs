// src/api/bridge/app/agent/loop_runner/runner.rs
//! AgentLoop - the goal-driven cognitive loop (Architecture §5.7 Decision Flow, §5.8)

use anyhow::Result;

use crate::common::principles::acceptance::{
    ComplexityImpact, ContributionGate, ContributionRequest,
};
use crate::common::principles::alignment::ArchitectureRisk;
use crate::common::principles::autonomous::{ContributorType, DevelopmentTask};
use crate::common::principles::confidence_history::record_confidence_change;
use crate::common::principles::diagnostics::record_diagnostic;
use crate::common::principles::enforcer::ValidationParams;
use crate::common::principles::knowledge_graph::{insert_edge, insert_graph};
use crate::common::principles::trace::record_trace;
use crate::context::ContextEngine;
use crate::database::maintenance;
use crate::experience::types::{ExperienceContext, ExperienceOutcome, ExperienceType, OutcomeKind};
use crate::prompt::{AssemblyParams, PromptBuilder};

use super::super::context::AgentDeps;
use super::super::decision::{
    ActionSelector, SelectedAction, check_internal_sources, trigger_research_on_failure,
};
use super::super::safety_gate::SafetyDecision;
use super::super::types::{AgentGoal, GoalStatus};
use super::types::AgentLoopOutcome;

/// The goal-driven agent loop. Owns no business logic; composes `AgentDeps`.
pub struct AgentLoop {
    deps: AgentDeps,
}

impl AgentLoop {
    pub fn new(deps: AgentDeps) -> Self {
        Self { deps }
    }

    /// Run the cognitive loop for a single goal and record the outcome.
    pub async fn run(&self, mut goal: AgentGoal) -> Result<AgentLoopOutcome> {
        let start = std::time::Instant::now();
        goal.status = GoalStatus::InProgress;

        // Reset per-iteration safety counters (§16 sandbox budget + rollback
        // journal).
        self.deps.safety_gate.reset_iteration();

        // Assemble cognitive context via ContextEngine (Architecture Chapter 7).
        let correlation_id = format!("goal-{}", goal.id);
        let mut engine = match self.deps.context_engine.lock() {
            Ok(mut guard) => std::mem::take(&mut *guard),
            Err(_) => {
                tracing::warn!("Context engine lock poisoned; using fresh instance");
                ContextEngine::default()
            }
        };
        let context_assembly = engine
            .assemble_context(&correlation_id, &goal.description, &goal.description, 50)
            .await;
        tracing::info!(
            correlation_id = %correlation_id,
            context_size = context_assembly.stages.len(),
            "Context assembled via ContextEngine"
        );

        // 1. Plan (§2.8)
        let mut dev_task = DevelopmentTask::new(
            goal.id.clone(),
            goal.description.clone(),
            ContributorType::AutonomousAgent,
        );
        if dev_task.progress().is_err() {
            tracing::warn!("Failed to progress DevelopmentTask to Plan stage");
        }
        if dev_task.progress().is_err() {
            tracing::warn!("Failed to progress DevelopmentTask past Plan stage");
        }
        if dev_task.progress().is_err() {
            tracing::warn!("Failed to progress DevelopmentTask to Plan stage");
        }

        let plan = match self
            .deps
            .planner
            .create_plan(goal.description.clone())
            .await
        {
            Ok(plan) => plan,
            Err(e) => {
                let experience_id = self
                    .record_failure(&goal, format!("Planning failed: {}", e))
                    .await
                    .ok();
                goal.status = GoalStatus::Failed;
                goal.completed_at = Some(chrono::Utc::now());
                let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
                self.deps.metrics.record_loop_latency(latency_ms).await;
                return Ok(AgentLoopOutcome {
                    goal_id: goal.id,
                    status: goal.status,
                    action_description: None,
                    confidence_value: None,
                    abstain_reason: Some(format!("Planning failed: {}", e)),
                    experience_id,
                });
            }
        };

        // 2. Retrieve supporting evidence
        let memory = self.deps.memory_retrieval.retrieve(&goal.description).await;
        let hybrid_results = crate::retrieval::hybrid_retrieve_with_sources(
            &goal.description,
            vec![
                crate::retrieval::RetrievalSource::SemanticMemory,
                crate::retrieval::RetrievalSource::EpisodicMemory,
                crate::retrieval::RetrievalSource::ExperienceMemory,
            ],
            10,
        );
        tracing::debug!(
            hybrid_results = hybrid_results.len(),
            "Hybrid retrieval pipeline returned {} candidate(s)",
            hybrid_results.len()
        );

        let knowledge = self
            .deps
            .persistence
            .knowledge_store
            .search(goal.description.as_str());
        let experience_memory_items: Vec<_> = memory
            .iter()
            .filter(|r| r.item.memory_type == crate::memory::types::MemoryType::Experience)
            .map(|r| r.item.clone())
            .collect();

        let experiences: Vec<_> = experience_memory_items
            .iter()
            .map(|m| crate::experience::types::Experience {
                id: m.id,
                score: Some(crate::experience::types::ExperienceScore {
                    confidence: m.confidence,
                    importance: m.confidence,
                    novelty: 0.0,
                    reliability: m.confidence,
                }),
                ..crate::experience::types::Experience::new(
                    m.content.clone(),
                    m.content.clone(),
                    ExperienceType::MemoryLookup,
                    Vec::new(),
                )
            })
            .collect();

        // 1b. Build prompt
        let prompt_builder = PromptBuilder::new();
        let planning_params = AssemblyParams {
            identity: "RoBoT Agent".to_string(),
            rules: "Plan, execute, verify, learn, reflect.".to_string(),
            capabilities: "Planning, Memory, Knowledge, Experience, Safety".to_string(),
            context_items: vec![goal.description.clone()],
            knowledge_items: knowledge.iter().map(|k| k.id.to_string()).collect(),
            experience_items: experiences.iter().map(|e| e.id.to_string()).collect(),
            task_description: goal.description.clone(),
            output_format: "Plan with steps, dependencies, confidence".to_string(),
        };
        let constructed_prompt = prompt_builder.build(&planning_params);
        tracing::debug!(
            prompt_layers = ?constructed_prompt.layers,
            token_estimate = constructed_prompt.token_estimate,
            "PromptBuilder assembled prompt with {} layers (budget-aware)",
            constructed_prompt.layers.len()
        );

        // 2b. 9-tier cascade
        let cascade_result = check_internal_sources(&memory, &knowledge, &experiences);
        match cascade_result {
            crate::api::bridge::app::agent::decision::TierResult::MemoryPassed => {
                tracing::debug!("Internal sources passed cascade: memory");
            }
            crate::api::bridge::app::agent::decision::TierResult::KnowledgePassed => {
                tracing::debug!("Internal sources passed cascade: knowledge");
            }
            crate::api::bridge::app::agent::decision::TierResult::ExperiencePassed => {
                tracing::debug!("Internal sources passed cascade: experience");
            }
            crate::api::bridge::app::agent::decision::TierResult::SkillsPassed => {
                tracing::debug!("Internal sources passed cascade: skills");
            }
            crate::api::bridge::app::agent::decision::TierResult::ReflectionsPassed => {
                tracing::debug!("Internal sources passed cascade: reflections");
            }
            crate::api::bridge::app::agent::decision::TierResult::WorkflowsPassed => {
                tracing::debug!("Internal sources passed cascade: workflows/plans");
            }
            crate::api::bridge::app::agent::decision::TierResult::WorldModelPassed => {
                tracing::debug!("Internal sources passed cascade: world model");
                if let Ok(goal_uuid) = goal.id.parse::<uuid::Uuid>() {
                    let blockers = self.deps.world_model.blockers_of(goal_uuid).await;
                    if !blockers.is_empty() {
                        tracing::warn!(
                            blocker_count = blockers.len(),
                            "Goal has {} blocker(s)",
                            blockers.len()
                        );
                    }
                    let deps = self.deps.world_model.dependencies_of(goal_uuid).await;
                    if !deps.is_empty() {
                        tracing::debug!(
                            dep_count = deps.len(),
                            "Goal depends on {} entity(ies)",
                            deps.len()
                        );
                    }
                    let resources = self.deps.world_model.resources_consumed_by(goal_uuid).await;
                    for resource in &resources {
                        if let Some(path) = self
                            .deps
                            .world_model
                            .find_path(goal_uuid, resource.id)
                            .await
                        {
                            tracing::debug!(
                                path_length = path.len(),
                                "Path from goal to resource '{}' has {} hop(s)",
                                resource.name,
                                path.len()
                            );
                        }
                    }
                }
            }
            crate::api::bridge::app::agent::decision::TierResult::HypothesesPassed => {
                tracing::debug!("Internal sources passed cascade: hypotheses");
            }
            crate::api::bridge::app::agent::decision::TierResult::AllFailed => {
                tracing::warn!("All internal source tiers failed — triggering research cascade");
                if let Some(research_result) = trigger_research_on_failure(&goal.description).await
                {
                    tracing::info!(
                        research_confidence = research_result.confidence,
                        source_count = research_result.sources.len(),
                        "Research cascade returned {} source(s) at confidence {:.2}",
                        research_result.sources.len(),
                        research_result.confidence
                    );
                    let knowledge_store = self.deps.persistence.knowledge_store.clone();
                    let found = crate::knowledge::promote_research_findings(
                        &knowledge_store,
                        &research_result.findings,
                        &goal.description,
                    )
                    .await;
                    tracing::info!(
                        knowledge_items_promoted = found,
                        "Research findings promoted to knowledge store"
                    );
                } else {
                    tracing::warn!("Research cascade returned no results");
                }
            }
        }

        // 3. Select the best-supported action (§5.7)
        let selected = ActionSelector::select(
            &plan.steps,
            &memory,
            &knowledge,
            &experiences,
            goal.confidence_threshold,
        );
        let Some(selected) = selected else {
            let experience_id = self
                .record_abstention(&goal, "No actionable plan step remained".to_string())
                .await
                .ok();
            goal.status = GoalStatus::Abstained;
            goal.completed_at = Some(chrono::Utc::now());
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            self.deps.metrics.record_loop_latency(latency_ms).await;
            return Ok(AgentLoopOutcome {
                goal_id: goal.id,
                status: goal.status,
                action_description: None,
                confidence_value: None,
                abstain_reason: Some("No actionable plan step remained".to_string()),
                experience_id,
            });
        };

        // 3b. Emotional weighting
        let emotional_weight = {
            let personality = self.deps.personality.lock();
            match personality {
                Ok(p) => p.emotional_weight(),
                Err(_) => 0.0,
            }
        };
        let emotion_adjusted = (selected.confidence.value + emotional_weight).clamp(0.0, 1.0);
        let mut selected = selected;
        let drift = (emotion_adjusted - selected.confidence.value).abs();
        self.deps
            .metrics
            .record_confidence_drift(drift as f64)
            .await;
        selected.confidence.value = emotion_adjusted;

        // Record confidence change
        if let Err(e) = record_confidence_change(
            &self.deps.persistence.database,
            &selected.step.action,
            (selected.confidence.value - emotional_weight) as f64,
            emotion_adjusted as f64,
            &format!("emotion_weight={:.3}", emotional_weight),
        )
        .await
        {
            tracing::warn!("Failed to record confidence change: {e}");
        }

        // 3c. Architecture acceptance (Chapter 24)
        let contribution_gate = ContributionGate::new();
        let owner =
            crate::common::principles::alignment::subsystem_for_action(&selected.step.action)
                .unwrap_or_default();
        let risk_class =
            crate::common::principles::alignment::action_risk_class(&selected.step.action);
        let contribution = ContributionRequest {
            artifact: selected.step.action.clone(),
            owner: owner.to_string(),
            caller: format!("agent_loop:{}", goal.id),
            purpose: selected.rationale(),
            error_behavior: "A failing action is recorded as an abstention experience and surfaced in the loop outcome; it never leaves the process in a partial state.".to_string(),
            validation_path: "The architecture acceptance gate (Chapter 24) and the safety gate (Architecture §16) both evaluate the action before it executes.".to_string(),
            complexity: ComplexityImpact::default(),
            contributor_kind: Some(ContributorType::AutonomousAgent),
            params: ValidationParams {
                subsystem: owner.to_string(), action: selected.step.action.clone(),
                correlation_id: goal.id.clone(),
                provenance: Self::provenance_chain(&goal, &plan, &selected),
                confidence: selected.confidence.value,
                provider: String::new(),
                lifetime: match risk_class {
                    ArchitectureRisk::Read => "ephemeral",
                    ArchitectureRisk::Mutate => "persistent_operational",
                    ArchitectureRisk::Destructive => "",
                }.to_string(),
                authorized: risk_class != ArchitectureRisk::Destructive,
                traceable: true, observable: true, failure_visible: true,
                version: env!("CARGO_PKG_VERSION").to_string(),
                high_risk: risk_class == ArchitectureRisk::Destructive,
                approved: false, backward_compatible: true,
            },
        };
        let decision = contribution_gate.evaluate(&contribution);
        let trace = decision.trace().to_string();
        let trace_fn = selected.step.action.clone();
        let trace_callers = format!("agent_loop:{}", goal.id);
        let trace_deps = format!("planning:{}", selected.step.action);
        let trace_data_flow = "goal → plan → action → execute";
        if let Err(e) = record_trace(
            &self.deps.persistence.database,
            &trace_fn,
            &trace_callers,
            &trace_deps,
            trace_data_flow,
            &trace,
        )
        .await
        {
            tracing::warn!("Failed to record architecture trace: {e}");
        }
        let diag_severity = if decision.is_accepted() {
            "INFO"
        } else {
            "WARN"
        };
        if let Err(e) = record_diagnostic(
            &self.deps.persistence.database,
            diag_severity,
            "architecture_gate",
            &format!(
                "Contribution '{}' evaluation: {}",
                decision.trace(),
                decision.checks()
            ),
        )
        .await
        {
            tracing::warn!("Failed to record diagnostic: {e}");
        }
        if decision.is_accepted() {
            if let Err(e) = insert_graph(
                &self.deps.persistence.database,
                &selected.step.action,
                owner,
                "action",
            )
            .await
            {
                tracing::warn!("Failed to record knowledge graph: {e}");
            }
            if let Err(e) = insert_edge(
                &self.deps.persistence.database,
                &format!("edge:{}", selected.step.action),
                "agent_loop",
                &selected.step.action,
                "executes",
                0.8,
            )
            .await
            {
                tracing::warn!("Failed to record knowledge graph edge: {e}");
            }
            tracing::debug!(action = %selected.step.action, owner = %owner, checks = decision.checks(), trace = %trace, "Contribution accepted (Architecture §24.10)");
        } else {
            let reason = format!(
                "Architecture acceptance rejected '{}': {}",
                selected.step.action,
                decision.reasons().join("; ")
            );
            tracing::warn!(action = %selected.step.action, trace = %trace, reason = %reason, "Contribution rejected by the architecture acceptance gate");
            let experience_id = self.record_abstention(&goal, reason.clone()).await.ok();
            goal.status = GoalStatus::Abstained;
            goal.completed_at = Some(chrono::Utc::now());
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            self.deps.metrics.record_loop_latency(latency_ms).await;
            return Ok(AgentLoopOutcome {
                goal_id: goal.id,
                status: goal.status,
                action_description: Some(selected.step.action.clone()),
                confidence_value: Some(selected.confidence.value),
                abstain_reason: Some(reason),
                experience_id,
            });
        }

        // 4. Route decision through coordination layer
        let orchestrator = crate::common::coordination::Orchestrator::default();
        let decision = crate::common::data_contracts::decision::Decision::new(
            &selected.step.action,
            selected.rationale(),
            selected.confidence.value,
        );
        match crate::common::coordination::route_decision(&orchestrator, &decision) {
            Ok(_) => {
                tracing::debug!(routed_action = %selected.step.action, "Decision routed through coordination layer");
            }
            Err(e) => {
                tracing::warn!("Coordination layer rejected decision: {}", e);
                let experience_id = self
                    .record_abstention(&goal, format!("Coordination rejected: {}", e))
                    .await
                    .ok();
                goal.status = GoalStatus::Abstained;
                goal.completed_at = Some(chrono::Utc::now());
                let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
                self.deps.metrics.record_loop_latency(latency_ms).await;
                return Ok(AgentLoopOutcome {
                    goal_id: goal.id,
                    status: goal.status,
                    action_description: None,
                    confidence_value: None,
                    abstain_reason: Some(format!("Coordination rejected: {}", e)),
                    experience_id,
                });
            }
        }

        // 5. Safety gate (§16)
        match self.deps.safety_gate.evaluate_full(&mut selected) {
            SafetyDecision::Block { reason, report } => {
                self.observe_emotion(false, 0.3);
                tracing::info!(
                    "Safety gate blocked action '{}': {} (hallucination={}, low_confidence={}, sandbox={}, evidence={})",
                    selected.step.action,
                    reason,
                    report.hallucination_risk,
                    report.low_confidence,
                    report.sandbox_blocked,
                    report.evidence_count
                );
                let active = self.deps.safety_gate.active_mutations();
                if active > 0 {
                    for entry in self.deps.safety_gate.journal_entries() {
                        if !entry.rolled_back {
                            self.deps.safety_gate.rollback_target(&entry.target_id);
                        }
                    }
                }
                let experience_id = self.record_abstention(&goal, reason.clone()).await.ok();
                goal.status = GoalStatus::Abstained;
                goal.completed_at = Some(chrono::Utc::now());
                let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
                self.deps.metrics.record_loop_latency(latency_ms).await;
                Ok(AgentLoopOutcome {
                    goal_id: goal.id,
                    status: goal.status,
                    action_description: Some(selected.step.action.clone()),
                    confidence_value: Some(selected.confidence.value),
                    abstain_reason: Some(reason),
                    experience_id,
                })
            }
            SafetyDecision::Allow => {
                let rationale = selected.rationale();
                tracing::info!("Agent loop proceeding: {}", rationale);
                let outcome_summary = format!(
                    "Selected action '{}' (confidence {:.2}, emotion {:+.2}) for goal '{}'",
                    selected.step.action,
                    selected.confidence.value,
                    emotional_weight,
                    goal.description
                );
                let experience_id = self
                    .record_success(&goal, outcome_summary.clone())
                    .await
                    .ok();
                if let Some(ref eid) = experience_id {
                    self.deps
                        .safety_gate
                        .record_mutation(&selected.step.action, eid.clone());
                }
                let journal = self.deps.safety_gate.journal_entries();
                if !journal.is_empty() {
                    tracing::debug!(
                        "Safety gate journal: {} mutation(s) recorded this iteration",
                        journal.len()
                    );
                }
                self.observe_emotion(true, 0.5);
                goal.status = GoalStatus::Achieved;
                goal.completed_at = Some(chrono::Utc::now());
                let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
                self.deps.metrics.record_loop_latency(latency_ms).await;
                let confirmed = self
                    .deps
                    .metrics
                    .collector()
                    .get_counter(crate::experience::metrics::metric_names::HYPOTHESES_CONFIRMED)
                    .await;
                self.deps
                    .metrics
                    .record_promotion_throughput(confirmed as f64)
                    .await;
                let exec_id = uuid::Uuid::new_v4().to_string();
                let conn = self.deps.persistence.database.connection()?;
                crate::database::queries::insert_execution(
                    &conn,
                    &exec_id,
                    &plan.id,
                    &selected.step.action,
                    &outcome_summary,
                    true,
                )?;
                let planning_id = uuid::Uuid::new_v4().to_string();
                let conn = self.deps.persistence.database.connection()?;
                crate::database::queries::insert_planning(
                    &conn,
                    &planning_id,
                    &plan.id,
                    &goal.description,
                    "completed",
                    1.0,
                )?;
                if let Err(e) = maintenance::maintain(&self.deps.persistence.database).await {
                    tracing::warn!("Database maintenance failed: {e}");
                }
                Ok(AgentLoopOutcome {
                    goal_id: goal.id,
                    status: goal.status,
                    action_description: Some(selected.step.action.clone()),
                    confidence_value: Some(selected.confidence.value),
                    abstain_reason: None,
                    experience_id,
                })
            }
        }
    }

    /// Build the provenance chain of an action (Architecture §2.2)
    fn provenance_chain(
        goal: &AgentGoal,
        plan: &crate::planning::engine::types::Plan,
        selected: &SelectedAction,
    ) -> Vec<String> {
        let mut provenance = vec![format!("goal:{}", goal.id), format!("plan:{}", plan.id)];
        for item in &selected.supporting_memory {
            provenance.push(format!("memory:{}", item.item.id));
        }
        for item in &selected.supporting_knowledge {
            provenance.push(format!("knowledge:{}", item.id));
        }
        for experience in &selected.supporting_experiences {
            provenance.push(format!("experience:{}", experience.id));
        }
        provenance
    }

    /// Update the personality's emotional state from an outcome (§13)
    fn observe_emotion(&self, success: bool, effort: f32) {
        let personality = self.deps.personality.lock();
        if let Ok(mut p) = personality {
            p.observe_emotional_outcome(success, effort);
        }
    }

    /// Record a successful action outcome
    async fn record_success(&self, goal: &AgentGoal, summary: String) -> Result<String> {
        let mut experience = crate::experience::types::Experience::new(
            format!("Agent action: {}", goal.description),
            summary,
            ExperienceType::Planning,
            Vec::new(),
        );
        experience.context = ExperienceContext {
            source: Some("agent-loop".to_string()),
            ..ExperienceContext::default()
        };
        experience.outcome = ExperienceOutcome::success();
        let processed = self.deps.persistence.coordinator.process(experience);
        let conn = self.deps.persistence.database.connection()?;
        let memory = crate::database::models::MemoryCard::from_experience(&processed);
        crate::database::queries::insert_memory(&conn, &memory)?;
        Ok(processed.id.to_string())
    }

    /// Record an abstention (safety block / low confidence)
    async fn record_abstention(&self, goal: &AgentGoal, reason: String) -> Result<String> {
        let mut experience = crate::experience::types::Experience::new(
            format!("Agent abstained: {}", goal.description),
            reason.clone(),
            ExperienceType::Planning,
            Vec::new(),
        );
        experience.outcome = ExperienceOutcome {
            kind: OutcomeKind::Interrupted,
            message: Some("abstained".to_string()),
            error: Some(reason),
            duration_ms: None,
        };
        let processed = self.deps.persistence.coordinator.process(experience);
        let conn = self.deps.persistence.database.connection()?;
        let memory = crate::database::models::MemoryCard::from_experience(&processed);
        crate::database::queries::insert_memory(&conn, &memory)?;
        Ok(processed.id.to_string())
    }

    /// Record a failure (e.g. planning error)
    async fn record_failure(&self, goal: &AgentGoal, reason: String) -> Result<String> {
        let rolled_back = self.deps.safety_gate.rollback_all();
        if !rolled_back.is_empty() {
            let targets: Vec<String> = rolled_back
                .iter()
                .map(|e| {
                    format!(
                        "{}({}) at {} [id={}{}]",
                        e.action,
                        e.target_id,
                        e.timestamp.format("%H:%M:%S"),
                        e.id,
                        if e.rolled_back { ", reversed" } else { "" }
                    )
                })
                .collect();
            tracing::info!(
                "Safety gate rolled back {} mutation(s): {}",
                rolled_back.len(),
                targets.join("; ")
            );
        }
        let active = self.deps.safety_gate.active_mutations();
        if active > 0 {
            tracing::warn!(
                "Safety gate: {} mutation(s) still active after rollback",
                active
            );
        }
        let mut experience = crate::experience::types::Experience::new(
            format!("Agent goal failed: {}", goal.description),
            reason.clone(),
            ExperienceType::Error,
            Vec::new(),
        );
        experience.outcome = ExperienceOutcome::failure(reason);
        let processed = self.deps.persistence.coordinator.process(experience);
        let conn = self.deps.persistence.database.connection()?;
        let memory = crate::database::models::MemoryCard::from_experience(&processed);
        crate::database::queries::insert_memory(&conn, &memory)?;
        Ok(processed.id.to_string())
    }
}
