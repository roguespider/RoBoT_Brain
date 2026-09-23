use serde_json;
use std::sync::{Arc, Mutex};

use crate::bridge::tools::ToolOutput;
use crate::bridge::tools::cooboploop::inputs::*;

/// Execute cooboploop_get_idle_state (S7.8)
pub async fn execute_cooboploop_get_idle_state(
    loop_runner: &Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
    strategic_registry: &Arc<Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
) -> ToolOutput {
    use crate::cooboploop::idle::{IdlePhase, IdleState};
    use crate::cooboploop::loop_runner::CyclePhase;

    let (phase, seconds_since_activity, reevaluation_interval_secs, objectives_processed) = {
        let runner = match loop_runner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let phase = match runner.phase {
            CyclePhase::Observe => IdlePhase::Active,
            CyclePhase::FindNextObjective => IdlePhase::Reprioritizing,
            CyclePhase::Wait => IdlePhase::Waiting,
        };
        (
            phase,
            runner.heartbeat_secs,
            runner.reevaluation_interval_secs,
            runner.cycle_count(),
        )
    };
    let strategic_candidates = {
        let registry = match strategic_registry.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        registry
            .list()
            .iter()
            .filter(|objective| objective.status == "active")
            .cloned()
            .collect::<Vec<_>>()
    };
    let mut state = IdleState::new(reevaluation_interval_secs);
    state.phase = phase;
    state.seconds_since_activity = seconds_since_activity;
    state.objectives_processed = objectives_processed;
    state.queue_empty = strategic_candidates.is_empty();
    let useful_work = state.evaluate_useful_work(&strategic_candidates);
    ToolOutput::success(serde_json::json!({
        "message": "Idle state retrieved",
        "status": "ok",
        "phase": format!("{:?}", state.phase),
        "seconds_since_activity": state.seconds_since_activity,
        "objectives_processed": state.objectives_processed,
        "reevaluation_interval_secs": state.reevaluation_interval_secs,
        "useful_work": useful_work,
        "strategic_candidates": strategic_candidates,
    }))
}

/// Execute cooboploop_configure_idle_reevaluation_interval (S7.9)
pub async fn execute_cooboploop_configure_idle_reevaluation_interval(
    input: CooboploopConfigureIdleReevaluationIntervalInput,
    loop_runner: &Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    if input.seconds <= 0 {
        return ToolOutput::error("Idle reevaluation interval must be greater than zero");
    }
    let interval = match u64::try_from(input.seconds) {
        Ok(seconds) => seconds,
        Err(error) => {
            return ToolOutput::error(format!("Invalid idle reevaluation interval: {error}"));
        }
    };
    let mut runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    runner.reevaluation_interval_secs = interval;
    ToolOutput::success(serde_json::json!({
        "message": "Idle reevaluation interval configured",
        "status": "ok",
        "seconds": runner.reevaluation_interval_secs,
    }))
}

/// Execute cooboploop_create_research_objective (T8.4)
pub async fn execute_cooboploop_create_research_objective(
    input: CooboploopCreateResearchObjectiveInput,
    research_manager: &Arc<Mutex<crate::cooboploop::research::ResearchManager>>,
    queue: &Arc<Mutex<crate::cooboploop::queue::ObjectiveQueue>>,
) -> ToolOutput {
    use crate::cooboploop::research::{PersistenceTarget, ResearchTrigger};

    let persistence_target = match input.persistence_target.as_deref() {
        None | Some("knowledge_base") => PersistenceTarget::KnowledgeBase,
        Some("experience_log") => PersistenceTarget::ExperienceLog,
        Some("both") => PersistenceTarget::Both,
        Some(value) => {
            return ToolOutput::error(format!(
                "Invalid persistence_target '{value}'; expected knowledge_base, experience_log, or both"
            ));
        }
    };
    let priority = input.priority.unwrap_or(0.0).max(0.0);
    let topic = input.topic;
    let expected_knowledge = format!("Research findings about {topic}");
    let mut manager = match research_manager.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let id = manager.create_objective(
        topic.clone(),
        ResearchTrigger::UnavailableInfo,
        persistence_target,
        expected_knowledge,
    );
    // Wire: enqueue research objective into the objective queue
    let goal = crate::cooboploop::queue::AgentGoal {
        id: format!("research_{id}"),
        title: format!("Research: {topic}"),
        description: format!("Research objective #{id}: {topic}"),
        status: crate::cooboploop::queue::GoalStatus::Discovered,
        priority,
        source: crate::cooboploop::sources::ObjectiveSource::SystemTrigger,
        expected_value: 0.7,
        risk: 0.3,
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
    if queue
        .lock()
        .map(|mut q| q.enqueue(&goal).is_ok())
        .unwrap_or(false)
    {
        tracing::debug!("Research objective enqueued successfully");
    }
    match manager.list().last() {
        Some(objective) => ToolOutput::success(serde_json::json!({
            "message": "Research objective created",
            "status": "ok",
            "id": id,
            "priority": priority,
            "objective": objective,
        })),
        None => ToolOutput::error("Research objective was not retained by the manager".to_string()),
    }
}
