//! Strategic learning — wiring the strategic learning engine into the cognitive cycle.
//!
//! Contains `wire_strategic_learning()` and related strategic objective
//! management methods, extracted from loop_runner.rs to reduce file size.

use super::LoopRunner;

impl LoopRunner {
    /// Wire strategic learning engine into the cognitive cycle
    /// (Section 18 / Architecture).
    pub fn wire_strategic_learning(&mut self) {
        // Initialize default strategic objectives if none exist
        let active_count = self.active_strategic_objectives().len();
        if active_count == 0 {
            let maint_obj = crate::learning::strategic::StrategicObjective::new(
                "default-maintenance",
                "System Maintenance",
                crate::learning::strategic::StrategicCategory::SystemReliability,
                "Monitor system health and reliability",
                1,
            );
            self.add_strategic_objective(maint_obj);

            let memory_obj = crate::learning::strategic::StrategicObjective::new(
                "default-memory",
                "Memory Optimization",
                crate::learning::strategic::StrategicCategory::MemoryRetrieval,
                "Optimize memory retrieval and retention",
                2,
            );
            self.add_strategic_objective(memory_obj);

            let tool_obj = crate::learning::strategic::StrategicObjective::new(
                "default-tools",
                "Tool Capability Expansion",
                crate::learning::strategic::StrategicCategory::ToolCapability,
                "Expand tool capabilities and integrations",
                3,
            );
            self.add_strategic_objective(tool_obj);

            let inference_obj = crate::learning::strategic::StrategicObjective::new(
                "default-inference",
                "Inference Efficiency",
                crate::learning::strategic::StrategicCategory::InferenceEfficiency,
                "Improve inference performance and reduce latency",
                4,
            );
            self.add_strategic_objective(inference_obj);

            let hardware_obj = crate::learning::strategic::StrategicObjective::new(
                "default-hardware",
                "Hardware Optimization",
                crate::learning::strategic::StrategicCategory::HardwareUtilization,
                "Optimize hardware utilization and resource allocation",
                5,
            );
            self.add_strategic_objective(hardware_obj);

            let failure_obj = crate::learning::strategic::StrategicObjective::new(
                "default-failure",
                "Failure Reduction",
                crate::learning::strategic::StrategicCategory::FailureReduction,
                "Reduce repeated failures and improve reliability",
                6,
            );
            self.add_strategic_objective(failure_obj);

            let research_obj = crate::learning::strategic::StrategicObjective::new(
                "default-research",
                "Research Initiative",
                crate::learning::strategic::StrategicCategory::ResearchCapability,
                "Investigate capability gaps and improvement opportunities",
                7,
            );
            self.add_strategic_objective(research_obj);

            let planning_obj = crate::learning::strategic::StrategicObjective::new(
                "default-planning",
                "Planning Reliability",
                crate::learning::strategic::StrategicCategory::PlanningReliability,
                "Improve planning reliability and reduce failures",
                8,
            );
            self.add_strategic_objective(planning_obj);

            // Initialize strategic evaluations (Section 18.4)
            self.strategic_learning_engine
                .evaluate("default-maintenance", 0.8);
            self.strategic_learning_engine
                .add_evaluation_success("default-maintenance", "System baseline established");
            self.strategic_learning_engine
                .evaluate("default-memory", 0.6);
            self.strategic_learning_engine
                .add_evaluation_recommendation(
                    "default-memory",
                    "Review memory consolidation patterns",
                );
            self.strategic_learning_engine
                .evaluate("default-tools", 0.4);
            self.strategic_learning_engine.add_evaluation_failure(
                "default-tools",
                "Some tools require additional configuration",
            );
            let feedback = crate::learning::strategic::StrategicFeedback::new(
                "system",
                "default-maintenance",
                "positive",
                "System initialized successfully with strategic objectives",
                0.5,
            );
            self.add_strategic_feedback(feedback);
        }
        let active_objectives = self.strategic_learning_engine.active_objectives();
        let pending_updates = self.strategic_learning_engine.pending_updates();
        // Wire strategic objectives into the strategic registry
        if let Ok(mut reg_guard) = self.strategic_registry.lock() {
            for obj in active_objectives {
                if !reg_guard.list().iter().any(|r| r.id == obj.id) {
                    let category_str = obj.category.label().to_string();
                    let category = match category_str.as_str() {
                        "SystemReliability" => crate::scheduler::cooboploop::strategic::StrategicObjective::MaintainSystemReliability,
                        "MemoryRetrieval" => crate::scheduler::cooboploop::strategic::StrategicObjective::ImproveMemoryRetrieval,
                        "InferenceEfficiency" => crate::scheduler::cooboploop::strategic::StrategicObjective::IncreaseInferenceEfficiency,
                        "ToolCapability" => crate::scheduler::cooboploop::strategic::StrategicObjective::ExpandToolCapability,
                        "HardwareUtilization" => crate::scheduler::cooboploop::strategic::StrategicObjective::ImproveHardwareUtilization,
                        "FailureReduction" => crate::scheduler::cooboploop::strategic::StrategicObjective::ReduceRepeatedFailures,
                        "ResearchCapability" => crate::scheduler::cooboploop::strategic::StrategicObjective::DevelopResearchCapability,
                        "PlanningReliability" => crate::scheduler::cooboploop::strategic::StrategicObjective::ImprovePlanningReliability,
                        _ => crate::scheduler::cooboploop::strategic::StrategicObjective::MaintainSystemReliability,
                    };
                    let record_id = obj.id.clone();
                    let record =
                        crate::scheduler::cooboploop::strategic::StrategicObjectiveRecord {
                            id: record_id.clone(),
                            name: obj.name.clone(),
                            category,
                            status: obj.status.clone(),
                            priority: obj.priority as f32,
                            created_at: chrono::DateTime::from_timestamp(obj.created_at, 0)
                                .unwrap_or_default(),
                            updated_at: chrono::DateTime::from_timestamp(obj.updated_at, 0)
                                .unwrap_or_default(),
                        };
                    if let Err(e) = reg_guard.add(record) {
                        tracing::warn!("Failed to add strategic objective: {e}");
                    }
                }
            }
        }
        // Collect pending updates before processing feedback (borrow checker).
        let pending_ids: Vec<String> = pending_updates.iter().map(|u| u.id.clone()).collect();
        // Process strategic feedback: update objectives and refine strategies (Section 18.5).
        let feedback_results = self.strategic_learning_engine.process_feedback();
        for result in &feedback_results {
            tracing::debug!("Strategic feedback: {result}");
        }
        // Wire feedback into objective priority updates (Section 18.5).
        self.strategic_learning_engine
            .update_objectives_from_feedback();
        // Wire strategy refinement based on feedback patterns (Section 18.4-18.5).
        let refined_strategies = self.strategic_learning_engine.refine_strategies();
        for strategy in &refined_strategies {
            tracing::debug!("Strategic refinement: {strategy}");
        }
        // Generate new improvement objectives from feedback (Section 18.5).
        let new_objectives = self
            .strategic_learning_engine
            .generate_objectives_from_feedback();
        for obj in &new_objectives {
            self.strategic_learning_engine.add_objective(obj.clone());
            tracing::debug!("Generated improvement objective: {}", obj.id);
        }
        // Apply pending strategic updates
        for id in pending_ids {
            self.strategic_learning_engine.apply_update(&id);
        }
    }

    /// Add a strategic objective to the learning engine.
    pub(crate) fn add_strategic_objective(
        &mut self,
        objective: crate::learning::strategic::StrategicObjective,
    ) {
        self.strategic_learning_engine.add_objective(objective);
    }

    /// Evaluate strategic progress for an objective.
    pub fn evaluate_strategic_progress(&mut self, objective_id: &str, progress: f32) {
        self.strategic_learning_engine
            .evaluate(objective_id, progress);
    }

    /// Add strategic feedback.
    pub(crate) fn add_strategic_feedback(
        &mut self,
        feedback: crate::learning::strategic::StrategicFeedback,
    ) {
        self.strategic_learning_engine.add_feedback(feedback);
    }

    /// Get active strategic objectives.
    pub fn active_strategic_objectives(&self) -> Vec<String> {
        self.strategic_learning_engine
            .active_objectives()
            .into_iter()
            .map(|o| o.id.clone())
            .collect()
    }

    /// Get pending strategic updates.
    pub fn pending_strategic_updates(&self) -> Vec<String> {
        self.strategic_learning_engine
            .pending_updates()
            .into_iter()
            .map(|u| u.id.clone())
            .collect()
    }

    /// Complete a strategic objective (Section 18.2).
    pub fn complete_strategic_objective(&mut self, objective_id: &str) {
        self.strategic_learning_engine
            .update_objective(objective_id, |obj| {
                obj.complete();
            });
        tracing::debug!("Completed strategic objective: {objective_id}");
    }

    /// Suspend a strategic objective (Section 18.2).
    pub fn suspend_strategic_objective(&mut self, objective_id: &str) {
        self.strategic_learning_engine
            .update_objective(objective_id, |obj| {
                obj.suspend();
            });
        tracing::debug!("Suspended strategic objective: {objective_id}");
    }

    /// Add a related capability to a strategic objective (Section 18.2).
    pub fn add_objective_capability(&mut self, objective_id: &str, capability: &str) {
        self.strategic_learning_engine
            .update_objective(objective_id, |obj| {
                obj.add_capability(capability);
            });
        tracing::debug!("Added capability '{capability}' to objective: {objective_id}");
    }

    /// Create a new strategic update (Section 18.3).
    pub fn create_strategic_update(
        &mut self,
        id: &str,
        objective_id: &str,
        action: &str,
        target: &str,
        details: &str,
    ) {
        let update = crate::learning::strategic::StrategicUpdate::new(
            id,
            objective_id,
            action,
            target,
            details,
        );
        self.strategic_learning_engine.add_update(update);
        tracing::debug!("Created strategic update: {id}");
    }

    /// Fail a strategic update (Section 18.3).
    pub fn fail_strategic_update(&mut self, update_id: &str) {
        self.strategic_learning_engine
            .update_strategic_update(update_id, |update| {
                update.fail();
            });
        tracing::debug!("Failed strategic update: {update_id}");
    }

    /// Add success indicator to strategic evaluation (Section 18.4).
    pub fn add_evaluation_success(&mut self, objective_id: &str, indicator: &str) {
        self.strategic_learning_engine.evaluate(objective_id, 1.0);
        tracing::debug!("Added success indicator '{indicator}' for objective: {objective_id}");
    }

    /// Add failure indicator to strategic evaluation (Section 18.4).
    pub fn add_evaluation_failure(&mut self, objective_id: &str, indicator: &str) {
        self.strategic_learning_engine.evaluate(objective_id, 0.0);
        tracing::debug!("Added failure indicator '{indicator}' for objective: {objective_id}");
    }

    /// Add recommendation to strategic evaluation (Section 18.4).
    pub fn add_evaluation_recommendation(&mut self, objective_id: &str, recommendation: &str) {
        self.strategic_learning_engine.evaluate(objective_id, 0.5);
        tracing::debug!("Added recommendation '{recommendation}' for objective: {objective_id}");
    }
}
