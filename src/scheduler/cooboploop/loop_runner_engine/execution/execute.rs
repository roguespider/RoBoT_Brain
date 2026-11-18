//! Execution logic for the CoObOpLoop.

use crate::scheduler::cooboploop::loop_runner_engine::{LoopRunner, LoopStage};
use crate::scheduler::cooboploop::queue::AgentGoal;

/// Execute the selected objective through its lifecycle.
/// Updates capability levels based on execution outcomes per CoObOpLoop.md §6.
pub fn execute(runner: &mut LoopRunner) {
    runner.current_stage = LoopStage::Execute;

    if let Some(ref goal) = runner.selected_goal {
        // Determine execution outcome based on goal status and execution result.
        let outcome = runner.execution_result.as_deref().unwrap_or("pending");

        // Update capability levels based on outcome per CoObOpLoop.md §6.
        // Capability assessments SHALL inform future planning.
        match outcome {
            "completed" => {
                tracing::info!("Execution completed: {}", goal.title);
                // Increase capability level for required capabilities.
                let mut cap_guard = match runner.capability_registry.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                let mut updates = Vec::new();
                for cap_id_str in &goal.required_capabilities {
                    let cap_id =
                        crate::scheduler::cooboploop::capability::CapabilityId::from_string(
                            cap_id_str,
                        );
                    if let Some(assessment) = cap_guard.get(&cap_id) {
                        let new_level = (assessment.level + 0.05).min(1.0);
                        updates.push(
                            crate::scheduler::cooboploop::capability::CapabilityAssessment {
                                id: cap_id.clone(),
                                name: cap_id_str.clone(),
                                level: new_level,
                                last_assessed: Some(chrono::Utc::now()),
                                success_rate: assessment.success_rate,
                            },
                        );
                        tracing::debug!(
                            "Capability updated: {} level {} -> {}",
                            cap_id_str,
                            assessment.level,
                            new_level
                        );
                    }
                }
                for updated in updates {
                    cap_guard.update(updated);
                }
            }
            "failed" => {
                tracing::warn!("Execution failed: {}", goal.title);
                // Record failure in self-improvement pipeline.
                if let Ok(mut pipeline) = runner.self_improvement_pipeline.lock() {
                    pipeline.record_failure(goal.title.clone());
                }
                // Decrease capability level for required capabilities.
                let mut cap_guard = match runner.capability_registry.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                let mut updates = Vec::new();
                for cap_id_str in &goal.required_capabilities {
                    let cap_id =
                        crate::scheduler::cooboploop::capability::CapabilityId::from_string(
                            cap_id_str,
                        );
                    if let Some(assessment) = cap_guard.get(&cap_id) {
                        let new_level = (assessment.level - 0.03).max(0.0);
                        updates.push(
                            crate::scheduler::cooboploop::capability::CapabilityAssessment {
                                id: cap_id.clone(),
                                name: cap_id_str.clone(),
                                level: new_level,
                                last_assessed: Some(chrono::Utc::now()),
                                success_rate: assessment.success_rate,
                            },
                        );
                        tracing::debug!(
                            "Capability reduced: {} level {} -> {}",
                            cap_id_str,
                            assessment.level,
                            new_level
                        );
                    }
                }
                for updated in updates {
                    cap_guard.update(updated);
                }
            }
            _ => {
                tracing::debug!("Execution outcome: {} for goal: {}", outcome, goal.title);
            }
        }

        // Trigger capability development if capability gaps detected.
        if let Some(ref cap_gap) = runner.selected_goal {
            for cap_id_str in &cap_gap.required_capabilities {
                let cap_id =
                    crate::scheduler::cooboploop::capability::CapabilityId::from_string(cap_id_str);
                let gap_guard = match runner.capability_registry.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                if let Some(assessment) = gap_guard.get(&cap_id)
                    && assessment.level < 0.5
                {
                    // Capability gap detected — generate development objective.
                    let dev_goal = AgentGoal {
                        id: format!("capability_dev_{}", uuid::Uuid::new_v4()),
                        title: format!(
                            "Develop capability: {} (current level: {:.2})",
                            cap_id_str, assessment.level
                        ),
                        description: format!(
                            "Capability {} at {:.2} requires development before execution",
                            cap_id_str, assessment.level
                        ),
                        status: crate::scheduler::cooboploop::queue::GoalStatus::Discovered,
                        priority: 0.8,
                        source: crate::scheduler::cooboploop::sources::ObjectiveSource::ImprovementTarget,
                        expected_value: 0.9,
                        risk: 0.2,
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
                    if runner.objective_queue.enqueue(&dev_goal).is_ok() {
                        tracing::info!("Capability development objective enqueued: {}", cap_id_str);
                    }
                    break; // Only generate one development objective per cycle.
                }
            }
        }
    } else {
        tracing::debug!("No selected goal to execute");
    }
}
