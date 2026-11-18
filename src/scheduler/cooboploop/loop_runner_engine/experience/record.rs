//! Experience recording for the CoObOpLoop.

use crate::experience::types::ExperienceType;
use crate::scheduler::cooboploop::loop_runner_engine::{LoopRunner, LoopStage};

/// Record experience after each cycle completion.
pub fn record_experience(runner: &mut LoopRunner) {
    runner.current_stage = LoopStage::RecordExperience;

    // Build an Experience record from the cycle state.
    if let Some(ref goal) = runner.selected_goal {
        let execution = runner.execution_result.as_deref().unwrap_or("unknown");
        let is_success = execution == "completed";

        let experience = crate::experience::types::Experience::new(
            format!("CoObOpLoop cycle: {}", goal.title),
            format!("Executed goal: {} (outcome: {})", goal.title, execution),
            ExperienceType::ToolExecution,
            Vec::new(),
        );

        // Set fields matching the actual Experience struct
        let mut exp = experience;
        exp.objective = goal.title.clone();
        exp.plan = format!("Plan for: {}", goal.title);
        exp.actions = vec![format!("Execute goal: {}", goal.title)];
        exp.tools_used = goal.required_capabilities.clone();
        exp.results = if is_success {
            vec![format!("Goal completed: {}", goal.title)]
        } else {
            vec![format!("Goal outcome: {}", execution)]
        };
        exp.failures = if !is_success {
            runner.post_task_evaluation.unexpected_problems.clone()
        } else {
            Vec::new()
        };
        exp.final_outcome = execution.to_string();
        exp.confidence = if is_success { 0.9 } else { 0.3 };
        exp.lessons_learned = if is_success {
            vec![format!("Successfully completed: {}", goal.title)]
        } else if execution == "failed" {
            runner.post_task_evaluation.unexpected_problems.clone()
        } else {
            Vec::new()
        };
        exp.tags = vec![goal.title.clone(), execution.to_string()];

        runner.latest_experience = Some(exp.clone());

        // Persist via experience coordinator if available.
        if let Some(coordinator) = &runner.experience_coordinator {
            coordinator.process(exp);
            tracing::info!(
                "record_experience: persisted experience for goal '{}'",
                goal.title
            );
        }
    } else {
        tracing::debug!("record_experience: no selected goal to record experience for");
    }
}

/// Update knowledge state based on cycle outcomes.
pub fn update_knowledge(runner: &mut LoopRunner) {
    runner.current_stage = LoopStage::UpdateKnowledge;

    // If we have a completed experience, note lessons for knowledge base.
    if let Some(exp) = &runner.latest_experience
        && exp.outcome.kind == crate::experience::types::OutcomeKind::Success
    {
        for lesson in &exp.lessons_learned {
            if lesson.len() > 10 {
                tracing::info!("update_knowledge: lesson worth storing: {}", lesson);
            }
        }
    }

    // Update capability registry based on learning history.
    let updates_count = runner.learning_history.len();
    let mut rec_guard = match runner.capability_registry.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    for update in &runner.learning_history {
        // Apply capability updates from learning.
        for cap_update in &update.capability_updates {
            let cap_id = crate::scheduler::cooboploop::capability::CapabilityId::from_string(
                &cap_update.capability_id,
            );
            if let Some(existing) = rec_guard.get(&cap_id) {
                let new_level = (existing.level + cap_update.level_delta).min(1.0);
                let assessment = crate::scheduler::cooboploop::capability::CapabilityAssessment {
                    id: cap_id,
                    name: cap_update.capability_id.clone(),
                    level: new_level,
                    last_assessed: Some(chrono::Utc::now()),
                    success_rate: existing.success_rate,
                };
                rec_guard.update(assessment);
            }
        }
    }

    tracing::debug!(
        "update_knowledge: {} learning updates applied",
        updates_count
    );
}
