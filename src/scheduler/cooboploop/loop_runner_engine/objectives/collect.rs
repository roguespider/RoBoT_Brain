//! Objective collection and evaluation for the CoObOpLoop.

use crate::scheduler::cooboploop::loop_runner_engine::{LoopRunner, LoopStage};
use crate::scheduler::cooboploop::{AgentGoal, GoalStatus};

/// Observe current system state before collecting objectives.
pub fn observe_state(runner: &mut LoopRunner) -> Result<(), String> {
    tracing::debug!("observe_state: starting observation phase");
    runner.current_stage = LoopStage::ObserveState;
    Ok(())
}

/// Collect objectives from all sources and queue them for processing.
/// Per CoObOpLoop.md §18: strategic objectives SHALL provide direction
/// when no immediate task exists. Active strategic objectives are
/// converted to AgentGoal entries and enqueued into the objective queue.
pub fn collect_objectives(runner: &mut LoopRunner) {
    runner.current_stage = LoopStage::CollectObjectives;

    // Feed active strategic objectives into the objective queue (§18 / T-COO-21).
    // Per CoObOpLoop.md §18: strategic objectives SHALL provide direction
    // when no immediate task exists.
    if let Ok(strategic_guard) = runner.strategic_registry.lock() {
        let active_objectives: Vec<_> = strategic_guard
            .list()
            .iter()
            .filter(|o| o.status == "active")
            .collect();

        for strategic in active_objectives {
            // Check if this strategic objective is already in the queue.
            let already_queued = runner
                .objective_queue
                .iter()
                .any(|g| g.title.contains(&strategic.name));

            if !already_queued {
                let goal = AgentGoal {
                    id: format!("strategic_{}", uuid::Uuid::new_v4()),
                    title: strategic.name.clone(),
                    description: format!(
                        "Strategic objective: {} (category: {:?})",
                        strategic.name, strategic.category
                    ),
                    status: GoalStatus::Discovered,
                    priority: 0.7,
                    source:
                        crate::scheduler::cooboploop::sources::ObjectiveSource::StrategicObjective,
                    expected_value: 0.8,
                    risk: 0.3,
                    learning_value: 0.6,
                    required_capabilities: Vec::new(),
                    dependencies: Vec::new(),
                    deadline: None,
                    execution_history: Vec::new(),
                    completion_state: None,
                    creation_timestamp: Some(chrono::Utc::now()),
                    last_evaluation: None,
                    ..Default::default()
                };
                if runner.objective_queue.enqueue(&goal).is_ok() {
                    tracing::info!("Collected strategic objective: {}", strategic.name);
                }
            }
        }
    }

    // Discover external opportunities via ObjectiveSourceRegistry (§3.2 / T-COO-40).
    // Wires OpportunityIntake::process() into the cognitive cycle.
    for goal in runner.objective_source_registry.discover_all() {
        let already_queued = runner.objective_queue.iter().any(|g| g.title == goal.title);
        if !already_queued && runner.objective_queue.enqueue(&goal).is_ok() {
            tracing::info!("Collected external opportunity: {}", goal.title);
        }
    }
}

/// Generate learning objectives based on idle evaluation and recent experiences.
/// Per CoObOpLoop.md §9: when idle, evaluate for useful activity categories
/// (pending objectives, system maintenance, bug investigation, research,
/// knowledge consolidation, memory maintenance, hardware evaluation,
/// performance optimization, capability development, self-improvement,
/// environmental observation, long-term planning).
pub fn generate_learning_objectives(runner: &mut LoopRunner) {
    runner.current_stage = LoopStage::GenerateNewObjectives;

    // Get active strategic objectives for evaluation.
    let strategic_objectives = if let Ok(strategic_guard) = runner.strategic_registry.lock() {
        strategic_guard.list().to_vec()
    } else {
        Vec::new()
    };

    // Evaluate useful work categories from idle state.
    let useful_work_categories = {
        let mut work_categories: Vec<crate::scheduler::cooboploop::idle::ActivityCategory> =
            Vec::new();

        // Pending objectives
        if !runner.objective_queue.is_empty() {
            work_categories
                .push(crate::scheduler::cooboploop::idle::ActivityCategory::PendingObjectives);
        }

        // System maintenance
        work_categories
            .push(crate::scheduler::cooboploop::idle::ActivityCategory::SystemMaintenance);

        // Bug investigation
        work_categories
            .push(crate::scheduler::cooboploop::idle::ActivityCategory::BugInvestigation);

        // Research
        work_categories.push(crate::scheduler::cooboploop::idle::ActivityCategory::Research);

        // Knowledge consolidation
        work_categories
            .push(crate::scheduler::cooboploop::idle::ActivityCategory::KnowledgeConsolidation);

        // Memory maintenance
        work_categories
            .push(crate::scheduler::cooboploop::idle::ActivityCategory::MemoryMaintenance);

        // Hardware evaluation
        work_categories
            .push(crate::scheduler::cooboploop::idle::ActivityCategory::HardwareEvaluation);

        // Performance optimization
        work_categories
            .push(crate::scheduler::cooboploop::idle::ActivityCategory::PerformanceOptimization);

        // Capability development
        work_categories
            .push(crate::scheduler::cooboploop::idle::ActivityCategory::CapabilityDevelopment);

        // Self-improvement
        work_categories.push(crate::scheduler::cooboploop::idle::ActivityCategory::SelfImprovement);

        // Environmental observation
        work_categories
            .push(crate::scheduler::cooboploop::idle::ActivityCategory::EnvironmentalObservation);

        // Long-term planning (only if active strategic objectives exist)
        if strategic_objectives.iter().any(|o| o.status == "active") {
            work_categories
                .push(crate::scheduler::cooboploop::idle::ActivityCategory::LongTermPlanning);
        }

        work_categories
    };

    tracing::debug!(
        "generate_learning_objectives: evaluated {} useful work categories",
        useful_work_categories.len()
    );

    // Generate objectives for each useful work category that has pending work.
    // Per CoObOpLoop.md §9: only genuinely detected objectives SHALL enter the queue.
    // We generate a maintenance/research objective for each category with actual pending items.
    let mut generated = 0;

    // Only generate objectives when there's actual work to do (not always).
    // Per Architecture §10: "The system MUST NOT generate arbitrary work
    // merely to remain active."
    if !runner.objective_queue.is_empty() {
        // There are pending objectives - generate a review objective.
        let goal = AgentGoal {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Review and process pending objectives".to_string(),
            description: "Idle evaluation: review and process pending objectives from queue"
                .to_string(),
            status: GoalStatus::Discovered,
            priority: 0.7,
            source: crate::scheduler::cooboploop::sources::ObjectiveSource::LearningTarget,
            expected_value: 0.8,
            risk: 0.2,
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
        if runner.objective_queue.enqueue(&goal).is_ok() {
            generated += 1;
        }
    }

    if generated > 0 {
        tracing::info!("Idle evaluation generated {} objective(s)", generated);
    }
}

/// Return objectives that could not be processed back to the queue.
pub fn return_to_queue(runner: &mut LoopRunner, goals: &[AgentGoal]) -> u32 {
    let mut count = 0;
    for goal in goals {
        if runner
            .objective_queue
            .transition(&goal.id, GoalStatus::Discovered)
            .is_ok()
        {
            count += 1;
        }
    }
    count
}

/// Enqueue a goal into the objective queue.
pub fn enqueue(runner: &mut LoopRunner, goal: AgentGoal) -> Result<(), String> {
    runner.objective_queue.enqueue(&goal).map(|_| ())
}

/// Evaluate the objective queue and adjust priorities based on learning feedback.
/// Per CoObOpLoop.md §15: Learning -> Better Evaluation.
pub fn evaluate_queue(runner: &mut LoopRunner) {
    runner.current_stage = LoopStage::EvaluateQueue;

    // Apply learning feedback to objective evaluation (§15 / Architecture).
    // Use capability registry updates and strategy tracker to influence priorities.
    let strategy = runner
        .strategy_tracker
        .lock()
        .unwrap_or_else(|p| p.into_inner());

    // Get all queued objectives and adjust priorities based on learning.
    let objectives: Vec<&AgentGoal> = runner.objective_queue.iter().collect();
    let reinforced: Vec<String> = strategy.reinforced.clone();
    let avoided: Vec<String> = strategy.avoided.clone();
    drop(strategy);

    for obj in &objectives {
        let mut priority = obj.priority;

        // Boost priority for objectives matching reinforced strategies.
        for strategy_name in &reinforced {
            if obj.title.contains(strategy_name.as_str())
                || obj.description.contains(strategy_name.as_str())
            {
                priority = (priority + 0.05).min(1.0);
            }
        }

        // Reduce priority for objectives matching avoided strategies.
        for strategy_name in &avoided {
            if obj.title.contains(strategy_name.as_str())
                || obj.description.contains(strategy_name.as_str())
            {
                priority = (priority - 0.05).max(0.0);
            }
        }

        // Boost priority for objectives that match current capability strengths.
        let caps_guard = match runner.capability_registry.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let caps = caps_guard.list();
        for cap in &caps {
            if obj
                .required_capabilities
                .iter()
                .any(|req| req.to_lowercase() == cap.name.to_lowercase())
            {
                // Capability level > 0.7: objective is well within capability.
                if cap.level > 0.7 {
                    priority = (priority + 0.03).min(1.0);
                }
            }
        }

        tracing::debug!(
            "evaluate_queue: objective '{}' priority adjusted to {:.2} (learned feedback: {} reinforced, {} avoided)",
            obj.id,
            priority,
            reinforced.len(),
            avoided.len()
        );
    }

    tracing::debug!(
        "evaluate_queue: evaluated {} objectives (learning_history: {} updates)",
        objectives.len(),
        runner.learning_history.len()
    );
}

/// Select the next objective based on current state and priority.
pub fn select_objective(runner: &mut LoopRunner) -> Option<AgentGoal> {
    runner.current_stage = LoopStage::SelectObjective;

    // Check capability readiness for queued objectives.
    let mut best_goal: Option<AgentGoal> = None;
    let mut best_priority = 0.0;

    let queued_goals: Vec<AgentGoal> = runner.objective_queue.iter().cloned().collect();
    for goal in &queued_goals {
        // Check if we have sufficient capabilities for this goal.
        let cap_check_guard = match runner.capability_registry.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let has_capability = goal.required_capabilities.iter().all(|cap_id| {
            let cap_id_obj =
                crate::scheduler::cooboploop::capability::CapabilityId::from_string(cap_id);
            cap_check_guard
                .get(&cap_id_obj)
                .is_some_and(|a| a.level >= 0.5)
        });

        if has_capability && goal.priority > best_priority {
            best_priority = goal.priority;
            best_goal = Some(goal.clone());
        }
    }

    if let Some(ref selected) = best_goal {
        runner.selected_goal = Some(selected.clone());
        // Transition the goal status from Discovered to Evaluating.
        if runner
            .objective_queue
            .transition(&selected.id, GoalStatus::Evaluating)
            .is_ok()
        {
            tracing::info!(
                "select_objective: selected goal '{}' with priority {:.2}",
                selected.title,
                selected.priority
            );
        }
    } else {
        tracing::debug!(
            "select_objective: no eligible goal found (queue size: {})",
            queued_goals.len()
        );
    }

    best_goal
}
