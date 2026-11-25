//! Cognitive cycle — the single-cycle execution logic.
//!
//! Extracted from loop_runner.rs to reduce file size. Contains `run_cycle()`
//! which orchestrates the full CoObOpLoop cognitive pipeline:
//! observe -> collect objectives -> evaluate -> select -> plan -> execute ->
//! verify -> record experience -> update knowledge -> reflect -> evaluate state
//! -> generate new objectives.

use super::super::Inspector;
use super::super::idle::{IdlePhase, IdleState};
use super::super::learning_feedback::apply_learning_feedback;
use super::super::queue::{AgentGoal, GoalStatus};
use super::super::sources::ObjectiveSource;
use super::LoopRunner;
use super::enums::{CyclePhase, LoopStage};
use crate::memory::restructure_hierarchy;
use std::sync::Arc;

impl LoopRunner {
    /// Run a single cognitive cycle (Architecture Chapter 23 / T-COO-50).
    ///
    /// Executes the full cognitive pipeline in sequence, wiring all subsystems:
    /// learning feedback, self-improvement, skill refinement, hardware adaptation,
    /// software inspection, research dispatch, strategic objectives, and
    /// autonomous conversation.
    pub fn run_cycle(&mut self) -> Result<(), String> {
        // Reload from persistent queue before cycle to ensure continuity
        if let Err(e) = self.objective_queue.reload() {
            tracing::warn!("Failed to reload objective queue from DB: {e}");
        }

        // Wire idle state evaluation into cycle start (§9-10).
        // Before executing the full cycle, evaluate whether useful work exists.
        // If no useful work exists and the system is not already active,
        // enter deliberate inactivity instead of running all stages.
        {
            let mut idle_state = IdleState::new(self.reevaluation_interval_secs);
            idle_state.phase = match self.phase {
                CyclePhase::Wait => IdlePhase::Waiting,
                _ => IdlePhase::Active,
            };
            idle_state.seconds_since_activity = self.heartbeat_secs;
            idle_state.objectives_processed = self.task_completion_count;
            idle_state.queue_empty = self.objective_queue.goals.is_empty();

            let strategic: Vec<crate::scheduler::cooboploop::strategic::StrategicObjectiveRecord> =
                match self.strategic_registry.lock() {
                    Ok(reg) => reg.list().to_vec(),
                    Err(_) => Vec::new(),
                };

            let (useful_work, inactivity) = idle_state.evaluate_useful_work(&strategic);

            if useful_work.is_empty() {
                if let Some(inactivity_event) = inactivity {
                    tracing::debug!("Deliberate inactivity: {}", inactivity_event.reason);
                    self.event_tracer
                        .log(&format!("Idle: {}", inactivity_event.reason));
                }
                self.enter_wait();
                return Ok(());
            }

            tracing::debug!(
                "Idle evaluation: {} useful work categories found",
                useful_work.len()
            );
        }

        self.current_stage = LoopStage::ObserveState;
        self.current_stage = LoopStage::CollectObjectives;
        self.current_stage = LoopStage::EvaluateQueue;
        self.current_stage = LoopStage::SelectObjective;
        self.current_stage = LoopStage::Plan;
        self.current_stage = LoopStage::Execute;
        self.current_stage = LoopStage::Verify;
        self.current_stage = LoopStage::RecordExperience;
        self.current_stage = LoopStage::UpdateKnowledge;
        self.current_stage = LoopStage::Reflect;
        self.current_stage = LoopStage::EvaluateCurrentState;
        self.current_stage = LoopStage::GenerateNewObjectives;
        // Execute full cognitive cycle (architecture compliance)
        if let Err(e) = super::objectives::observe_state(self) {
            tracing::warn!("observe_state failed: {e}");
        }
        super::objectives::collect_objectives(self);
        super::objectives::evaluate_queue(self);
        let selected_goal_result = super::objectives::select_objective(self);
        tracing::debug!("Selected objective: {:?}", selected_goal_result);
        super::planning::plan(self);
        super::execution::execute(self);
        super::verification::verify(self);
        super::experience::record_experience(self);
        super::experience::update_knowledge(self);
        super::reflection::reflect(self);
        // Wire learning pipeline into cycle (Section 15 / v0.0.2 Chapter 26):
        // Process experience into LearningUpdate, feed into capability_registry.
        if let Some(ref exp) = self.latest_experience {
            let learning_update =
                crate::scheduler::cooboploop::learning_pipeline::LearningPipeline::process(exp);
            // Apply learning feedback: knowledge store, capability registry, strategy tracker (§15).
            let mut cap_guard = match self.capability_registry.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            let mut strategy_guard = match self.strategy_tracker.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            apply_learning_feedback(
                &self.knowledge_store,
                &mut cap_guard,
                &mut strategy_guard,
                &learning_update,
                exp,
            );
            // Wire risk_adjustments into evaluation system (Section 15: Risk Adjustment).
            for risk_adj in &learning_update.risk_adjustments {
                if risk_adj.adjustment == "increase_trust" {
                    self.post_task_evaluation
                        .increase_trust(&risk_adj.domain, risk_adj.confidence_impact);
                } else if risk_adj.adjustment == "decrease_trust" {
                    self.post_task_evaluation
                        .decrease_trust(&risk_adj.domain, risk_adj.confidence_impact);
                }
            }
            // Wire post-task evaluation with actual cycle data from experience
            for lesson in &exp.lessons_learned {
                self.post_task_evaluation.add_knowledge_gap(lesson.clone());
            }
            for failure in &exp.failures {
                self.post_task_evaluation.add_new_bug(failure.clone());
            }
            if let Some(ref constraint) = exp.discovered_constraints.first() {
                self.post_task_evaluation
                    .set_capability_limitation(constraint.to_string());
            }
            if let Some(ref score) = exp.score {
                self.post_task_evaluation
                    .set_efficiency_score(score.importance);
            }
            if let Some(ref refinement) = learning_update.strategy_refinements.first() {
                self.post_task_evaluation
                    .set_future_planning_adjustment(refinement.to_string());
            }
            if let Some(ref strategy) = exp.successful_strategies.first() {
                self.post_task_evaluation
                    .set_improvement_opportunity(strategy.to_string());
            }
            // Append to learning_history for cycle tracking
            self.learning_history.push(learning_update.clone());
        }
        super::state_evaluation::evaluate_current_state(self);
        // Wire post-task evaluation results into new objectives (Section 8 / T-COO-31).
        let evaluation = self.post_task_evaluation();
        let new_objectives = evaluation.generate_objectives();
        for obj in &new_objectives {
            if let Err(e) = self.objective_queue.enqueue(obj) {
                tracing::warn!("Failed to enqueue post-task evaluation objective: {e}");
            }
        }
        if !new_objectives.is_empty() {
            tracing::debug!(
                "Post-task evaluation generated {} new objectives",
                new_objectives.len()
            );
        }
        // Generate learning objectives from idle evaluation (Section 9 / §10).
        // Only generates objectives for genuinely detected useful work, not
        // arbitrary empty goals — per CoObOpLoop.md §10: "MUST NOT generate
        // arbitrary work merely to remain active."
        super::objectives::generate_learning_objectives(self);
        // Wire self-improvement pipeline into cycle (Section 14 / Section 26.22):
        if let Ok(mut pipeline_guard) = self.self_improvement_pipeline.lock() {
            tracing::debug!("Pipeline stage: {:?}", pipeline_guard.current_stage());
            let failure_count = pipeline_guard.recent_failures_count();
            tracing::debug!("Recent failures: {failure_count}");
            if failure_count > 0 {
                match pipeline_guard.run() {
                    Ok(proposal) => {
                        tracing::info!("Self-improvement proposal generated: {}", proposal.title);
                        let objective = AgentGoal {
                            id: format!("improvement_{}", uuid::Uuid::new_v4()),
                            title: proposal.title.clone(),
                            description: proposal.description.clone(),
                            status: GoalStatus::Discovered,
                            priority: 0.8,
                            source: ObjectiveSource::ImprovementTarget,
                            expected_value: 0.8,
                            risk: 0.4,
                            learning_value: 0.9,
                            required_capabilities: Vec::new(),
                            dependencies: Vec::new(),
                            deadline: None,
                            execution_history: Vec::new(),
                            completion_state: None,
                            creation_timestamp: Some(chrono::Utc::now()),
                            last_evaluation: None,
                            ..Default::default()
                        };
                        if let Err(e) = self.objective_queue.enqueue(&objective) {
                            tracing::warn!("Failed to enqueue self-improvement objective: {e}");
                        } else {
                            tracing::info!(
                                "Self-improvement objective enqueued: {}",
                                proposal.title
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Self-improvement proposal generation failed: {e}");
                    }
                }
            }
        }
        // Autonomous skill refinement (Architecture Chapter 32):
        if let Some(ref executor) = self.skill_executor {
            let metrics = executor.get_execution_metrics_for_all();
            let mut refinement_count = 0u64;
            for (skill_id, exec_metrics) in &metrics {
                if exec_metrics.total_executions == 0 {
                    continue;
                }
                let base_mastery = exec_metrics.success_rate();
                let consecutive_successes = exec_metrics.consecutive_successes as f32;
                let consecutive_failures = exec_metrics.consecutive_failures as f32;
                let momentum_bonus = if consecutive_successes >= 3.0 {
                    (consecutive_successes.min(10.0) / 100.0).min(0.05)
                } else {
                    0.0
                };
                let recency_penalty = if consecutive_failures >= 2.0 {
                    (consecutive_failures.min(5.0) / 50.0).min(0.10)
                } else {
                    0.0
                };
                let mut final_mastery = base_mastery + momentum_bonus - recency_penalty;
                final_mastery = final_mastery.clamp(0.0, 1.0);
                if let Some(last_exec) = exec_metrics.last_execution {
                    let elapsed_secs = chrono::Utc::now()
                        .signed_duration_since(last_exec)
                        .num_seconds();
                    if elapsed_secs > 7 * 24 * 3600 {
                        let idle_days = elapsed_secs as f32 / (24.0 * 3600.0);
                        let decay = (idle_days - 7.0).max(0.0) * 0.01;
                        final_mastery = (final_mastery - decay).max(0.0);
                    }
                }
                executor.record_skill_mastery_sync(skill_id, final_mastery);
                refinement_count += 1;
            }
            if refinement_count > 0 {
                tracing::debug!(
                    "Autonomous skill refinement: updated {} skills",
                    refinement_count
                );
            }
        }
        if let Ok(research_guard) = self.research_manager.lock() {
            let research_list = research_guard.list();
            tracing::debug!("Research objectives: {}", research_list.len());
        }
        // Run autonomous maintenance tasks (Architecture Chapter 23):
        if let Ok(mut scheduler) = self.maintenance_scheduler.lock() {
            if let Ok(db) = crate::database::sqlite::SqliteDatabase::initialize() {
                match scheduler.run_maintenance(&db) {
                    Ok(results) => {
                        let successful: u64 = results.iter().filter(|r| r.success).count() as u64;
                        tracing::debug!(
                            "Autonomous maintenance: {} tasks ran, {} successful",
                            results.len(),
                            successful
                        );
                    }
                    Err(e) => {
                        tracing::warn!("Autonomous maintenance failed: {}", e);
                    }
                }
            } else {
                tracing::warn!("Autonomous maintenance: failed to initialize database");
            }
        }
        // Wire memory hierarchy maintenance into cycle (Architecture §14):
        let promoted = restructure_hierarchy(&mut self.memory_hierarchy);
        tracing::debug!(
            "Memory hierarchy restructured: {} records promoted",
            promoted
        );

        // Hardware-adaptive capability assessment (Section 12 / Section 22):
        if let Ok(mut discovery) = self.hardware_discovery.lock() {
            if let Some(profile) = discovery.detect() {
                tracing::debug!(
                    "Hardware profile: {} cores, {}MB RAM",
                    profile.cpu_cores,
                    profile.memory_available_mb
                );
                // Persist hardware profile to DB via HardwareRegistry.
                if let Some(ref reg) = self.hardware_registry {
                    let guard = match reg.lock() {
                        Ok(g) => g,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    if let Err(e) = guard.update(&profile) {
                        tracing::warn!("Failed to persist hardware profile: {e}");
                    }
                }
                let mut hw_guard = match self.capability_registry.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                hw_guard.update_from_hardware_profile(&profile);
                // Generate hardware_adaptation objectives when inadequate.
                if profile.cpu_cores < 4 || profile.memory_available_mb < 2048 {
                    let objective = AgentGoal {
                        id: format!("hardware_adapt_{}", uuid::Uuid::new_v4()),
                        title: "Hardware adaptation required".to_string(),
                        description: format!(
                            "Current hardware ({0} cores, {1}MB RAM) is below minimum requirements. Consider upgrading or optimizing.",
                            profile.cpu_cores, profile.memory_available_mb
                        ),
                        status: GoalStatus::Discovered,
                        priority: 0.9,
                        source: ObjectiveSource::SystemTrigger,
                        expected_value: 0.8,
                        risk: 0.3,
                        learning_value: 0.5,
                        required_capabilities: Vec::new(),
                        dependencies: Vec::new(),
                        deadline: None,
                        execution_history: Vec::new(),
                        completion_state: None,
                        creation_timestamp: Some(chrono::Utc::now()),
                        last_evaluation: None,
                        ..Default::default()
                    };
                    if let Err(e) = self.objective_queue.enqueue(&objective) {
                        tracing::warn!("Failed to enqueue hardware_adaptation objective: {e}");
                    } else {
                        tracing::info!("Hardware_adaptation objective enqueued");
                    }
                }
            }
        } else {
            tracing::debug!("Hardware detection returned empty profile");
        }
        // Wire software inspection into cycle (Section 13):
        if let Ok(mut inspector) = self.inspector.lock() {
            let issues = inspector.scan();
            if !issues.is_empty() {
                tracing::debug!("Inspection found {} issues", issues.len());
                let objectives = Inspector::issues_to_objectives(&issues);
                for obj in &objectives {
                    if let Err(e) = self.objective_queue.enqueue(obj) {
                        tracing::warn!("Failed to enqueue inspection objective: {e}");
                    }
                }
                tracing::debug!("Enqueued {} inspection objectives", objectives.len());
            } else {
                tracing::debug!("Inspection passed: no issues detected");
            }
        }
        // Wire research dispatch into cycle (Section 11 / Section 20):
        if let Ok(mut research_guard) = self.research_manager.lock() {
            let repo_clone: Option<Arc<dyn crate::memory::repository::MemoryRepository>> =
                research_guard.memory_repository_ref().cloned();
            let memory_repo_ref: Option<&dyn crate::memory::repository::MemoryRepository> =
                repo_clone.as_ref().map(|arc| arc.as_ref());
            match research_guard.dispatch_next(memory_repo_ref) {
                Ok(id) => tracing::debug!("Research dispatched: {id}"),
                Err(e) => tracing::debug!("Research dispatch: {e}"),
            }
        }
        {
            let list_guard = match self.capability_registry.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            tracing::debug!("Capabilities: {}", list_guard.list().len());
        }
        if let Ok(strategic_guard) = self.strategic_registry.lock() {
            let strategic_active = strategic_guard.list().iter().any(|o| o.status == "active");
            tracing::debug!("Strategic active: {strategic_active}");
        }
        // Wire strategic learning engine into cycle (Section 18)
        self.wire_strategic_learning();
        // Wire strategic objectives into objective generation (Section 18 / T-COO-21).
        if let Ok(strategic_guard) = self.strategic_registry.lock() {
            for objective in strategic_guard.list() {
                if objective.status == "active" {
                    let strategic_goal = AgentGoal {
                        id: format!("strategic_{}", objective.id),
                        title: objective.name.clone(),
                        description: format!("Strategic direction: {}", objective.name),
                        status: GoalStatus::Discovered,
                        priority: 0.9,
                        source: ObjectiveSource::StrategicObjective,
                        expected_value: 0.8,
                        risk: 0.3,
                        learning_value: 0.7,
                        required_capabilities: Vec::new(),
                        dependencies: Vec::new(),
                        deadline: None,
                        execution_history: Vec::new(),
                        completion_state: None,
                        creation_timestamp: Some(chrono::Utc::now()),
                        last_evaluation: None,
                        ..Default::default()
                    };
                    if let Err(e) = self.objective_queue.enqueue(&strategic_goal) {
                        tracing::warn!("Failed to enqueue strategic objective: {e}");
                    } else {
                        tracing::debug!("Strategic objective enqueued: {}", objective.name);
                    }
                }
            }
        }
        if let Ok(reflection_guard) = self.reflection_engine.lock() {
            drop(reflection_guard);
        }
        // Check human handler state during the cycle (Architecture Chapter 6)
        let audit = self.human_handler.audit_summary();
        tracing::debug!("Human handler audit: {audit}");
        self.event_tracer.log("Cycle completed");
        self.event_tracer.log_learning_cycle("Cycle reflection");
        // Enter wait when autonomous disabled (Sections 9-10)
        if self.should_continue() && !self.autonomous_operation_enabled {
            self.enter_wait();
        }
        // Wire autonomous conversation generation (Section 6.30):
        if self.idle_state.queue_empty
            && self.idle_state.phase != super::super::idle::IdlePhase::Active
        {
            let mut conv = match self.autonomous_conversation.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            let topic = "autonomous_reflection";
            conv.generate_autonomous_turn(topic.to_string());
            tracing::debug!(
                "Autonomous conversation turn generated, total turns={}",
                conv.state.turns.len()
            );
        }
        self.cycle_count = self.cycle_count.saturating_add(1);
        self.task_completion_count = self.task_completion_count.saturating_add(1);
        Ok(())
    }
}
