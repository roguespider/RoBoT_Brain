use serde_json;
use std::sync::{Arc, Mutex};

use crate::bridge::tools::ToolOutput;
use crate::bridge::tools::cooboploop::inputs::*;

/// Execute cooboploop_get_modification_boundary (T9.18)
pub async fn execute_cooboploop_get_modification_boundary(
    pipeline: &Arc<Mutex<crate::cooboploop::self_improvement::SelfImprovementPipeline>>,
) -> ToolOutput {
    let pipeline = match pipeline.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    ToolOutput::success(serde_json::json!({
        "message": "Modification boundary retrieved",
        "status": "ok",
        "boundary": pipeline.get_boundary(),
    }))
}

/// Execute cooboploop_set_modification_boundary (T9.19)
pub async fn execute_cooboploop_set_modification_boundary(
    input: CooboploopSetModificationBoundaryInput,
    pipeline: &Arc<Mutex<crate::cooboploop::self_improvement::SelfImprovementPipeline>>,
) -> ToolOutput {
    use crate::cooboploop::self_improvement::ModificationBoundary;
    let boundary = match input.boundary.as_str() {
        "Apply" => ModificationBoundary::Apply,
        "Propose" => ModificationBoundary::Propose,
        invalid_boundary => {
            return ToolOutput::error(format!(
                "Invalid modification boundary '{invalid_boundary}'; expected Propose or Apply"
            ));
        }
    };
    let mut pipeline = match pipeline.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    pipeline.set_boundary(boundary.clone());
    ToolOutput::success(serde_json::json!({
        "message": "Modification boundary set",
        "status": "ok",
        "boundary": boundary,
    }))
}

/// Execute cooboploop_run_opportunity_intake (T12.16)
pub async fn execute_cooboploop_run_opportunity_intake(
    input: CooboploopRunOpportunityIntakeInput,
    capability_registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
    pending_opportunities: &Arc<Mutex<Vec<crate::cooboploop::opportunity::IntakeResult>>>,
    queue: &Arc<Mutex<crate::cooboploop::queue::ObjectiveQueue>>,
) -> ToolOutput {
    use crate::cooboploop::opportunity::{OpportunityAdapter, OpportunityIntake};
    let source = input.source_url.trim();
    if source.is_empty() {
        return ToolOutput::error("source_url must not be empty");
    }
    let lowercase_source = source.to_lowercase();
    let resolved_source_type = input
        .source_type
        .as_deref()
        .map(str::trim)
        .filter(|source_type| !source_type.is_empty())
        .map(str::to_lowercase)
        .or_else(|| {
            if lowercase_source.contains("fiverr") {
                Some("fiverr".to_string())
            } else if lowercase_source.contains("upwork") {
                Some("upwork".to_string())
            } else if lowercase_source.contains("github") {
                Some("github_issues".to_string())
            } else {
                None
            }
        });
    let Some(source_type) = resolved_source_type else {
        return ToolOutput::error(
            "source_type is required when it cannot be inferred from source_url",
        );
    };
    let adapter: Box<dyn OpportunityAdapter> = match source_type.as_str() {
        "fiverr" => Box::new(crate::cooboploop::opportunity::FiverrAdapter::new(
            input.source_url.clone(),
        )),
        "upwork" => Box::new(crate::cooboploop::opportunity::UpworkAdapter::new(
            input.source_url.clone(),
        )),
        "github" | "github_issues" => Box::new(
            crate::cooboploop::opportunity::GitHubIssuesAdapter::new(input.source_url.clone()),
        ),
        unsupported => {
            return ToolOutput::error(format!(
                "Unsupported opportunity source_type '{unsupported}'"
            ));
        }
    };
    let adapter_name = adapter.name().to_string();
    let opportunities = adapter.fetch();
    if opportunities.is_empty() {
        return ToolOutput::error("The opportunity source produced no records");
    }
    let registry = match capability_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let intake = OpportunityIntake::new();
    let mut results = Vec::new();
    for opportunity in opportunities {
        results.push(intake.process(&opportunity, &registry));
    }
    drop(registry);
    let deferred_results: Vec<crate::cooboploop::opportunity::IntakeResult> = results
        .iter()
        .filter(|result| result.decision == crate::cooboploop::opportunity::IntakeDecision::Defer)
        .cloned()
        .collect();
    if !deferred_results.is_empty() {
        let mut pending = match pending_opportunities.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        pending.extend(deferred_results);
    }
    // Wire: enqueue accepted opportunities into the objective queue
    let accepted_count = {
        let accepted: Vec<_> = results
            .iter()
            .filter(|r| r.decision == crate::cooboploop::opportunity::IntakeDecision::Accept)
            .cloned()
            .collect();
        let mut enqueued = 0;
        for result in accepted {
            if let Some(goal) =
                crate::cooboploop::opportunity::OpportunityIntake::intake_result_to_goal(&result)
            {
                let mut q = match queue.lock() {
                    Ok(g) => g,
                    Err(p) => p.into_inner(),
                };
                if q.enqueue(&goal).is_ok() {
                    enqueued += 1;
                }
            }
        }
        enqueued
    };
    ToolOutput::success(serde_json::json!({
        "message": "Opportunity intake completed",
        "status": "ok",
        "source_url": input.source_url,
        "source_type": source_type,
        "adapter": adapter_name,
        "results": results,
        "default_policy_never_auto_accept": intake.default_policy_never_auto_accept,
        "new_goals_enqueued": accepted_count,
    }))
}

/// Execute cooboploop_get_pending_external_opportunities (T12.17)
pub async fn execute_cooboploop_get_pending_external_opportunities(
    pending_opportunities: &Arc<Mutex<Vec<crate::cooboploop::opportunity::IntakeResult>>>,
) -> ToolOutput {
    let pending = match pending_opportunities.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    ToolOutput::success(serde_json::json!({
        "message": "Pending external opportunities retrieved",
        "status": "ok",
        "pending": pending.as_slice(),
        "count": pending.len(),
        "note": "All deferred external opportunities require human review",
    }))
}

/// Execute cooboploop_set_autonomous_mode (T11.5)
pub async fn execute_cooboploop_set_autonomous_mode(
    input: CooboploopSetAutonomousModeInput,
    loop_runner: &Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let mut runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    runner.set_autonomous(input.enabled);
    // Wire process_with_autonomy_check: process a sample action with autonomy check
    let mut handler = crate::cooboploop::human::HumanActionHandler::new();
    handler.set_autonomous(input.enabled);
    let autonomy_check_result =
        handler.process_with_autonomy_check(crate::cooboploop::human::HumanAction::PauseAutonomous);
    tracing::debug!("Autonomy check result: {autonomy_check_result}");
    ToolOutput::success(serde_json::json!({
        "message": "Autonomous mode set",
        "status": "ok",
        "enabled": runner.is_autonomous(),
    }))
}

/// Execute cooboploop_get_autonomous_mode (T11.6)
pub async fn execute_cooboploop_get_autonomous_mode(
    loop_runner: &Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    ToolOutput::success(serde_json::json!({
        "message": "Autonomous mode retrieved",
        "status": "ok",
        "enabled": runner.is_autonomous(),
    }))
}

/// Execute cooboploop_list_strategic_objectives (T13)
pub async fn execute_cooboploop_list_strategic_objectives(
    strategic_registry: &Arc<Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
) -> ToolOutput {
    let registry = match strategic_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let list = registry.list();
    ToolOutput::success(serde_json::json!({
        "message": "Strategic objectives listed",
        "status": "ok",
        "count": list.len(),
        "objectives": list,
    }))
}

/// Execute cooboploop_add_strategic_objective (T13)
pub async fn execute_cooboploop_add_strategic_objective(
    input: CooboploopAddStrategicObjectiveInput,
    strategic_registry: &Arc<Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
) -> ToolOutput {
    use crate::cooboploop::strategic::{StrategicObjective, StrategicObjectiveRecord};

    let name = input.name.trim().to_string();
    if name.is_empty() {
        return ToolOutput::error("Strategic objective name must not be empty");
    }
    let category = match input.category.as_deref() {
        Some(category) => match category.parse::<StrategicObjective>() {
            Ok(category) => category,
            Err(error) => return ToolOutput::error(error),
        },
        None => StrategicObjective::MaintainSystemReliability,
    };
    let category_name = format!("{:?}", category);
    let id = format!("strategic_{}", uuid::Uuid::new_v4());
    let objective = StrategicObjectiveRecord {
        id: id.clone(),
        name: name.clone(),
        category,
        status: "active".to_string(),
        priority: 0.5,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let mut registry = match strategic_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    match registry.add(objective) {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "message": "Strategic objective added",
            "status": "ok",
            "id": id,
            "name": name,
            "category": category_name,
        })),
        Err(error) => ToolOutput::error(error),
    }
}

/// Execute cooboploop_remove_strategic_objective (T13)
pub async fn execute_cooboploop_remove_strategic_objective(
    input: CooboploopRemoveStrategicObjectiveInput,
    strategic_registry: &Arc<Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
) -> ToolOutput {
    let id = input.id.trim().to_string();
    if id.is_empty() {
        return ToolOutput::error("Strategic objective id must not be empty");
    }
    let mut registry = match strategic_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    match registry.remove(&id) {
        Ok(removed) => ToolOutput::success(serde_json::json!({
            "message": if removed { "Strategic objective removed" } else { "Not found" },
            "status": if removed { "ok" } else { "not_found" },
            "id": id,
        })),
        Err(error) => ToolOutput::error(error),
    }
}

/// Execute cooboploop_get_objective_hierarchy (T13)
pub async fn execute_cooboploop_get_objective_hierarchy(
    strategic_registry: &Arc<Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
    mission: &Arc<Mutex<String>>,
    queue: &Arc<Mutex<crate::cooboploop::queue::ObjectiveQueue>>,
    loop_runner: &Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    use crate::cooboploop::strategic::{ObjectiveHierarchy, wire_dynamic_hierarchy};

    let mission_name = match mission.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    let registry = match strategic_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let hierarchy = ObjectiveHierarchy::build(&mission_name, registry.list());
    // Wire active goals to hierarchy (§19 / T-COO-49)
    let goals = {
        let q = match queue.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        q.iter().cloned().collect::<Vec<_>>()
    };
    // Wire planner execution steps with real execution-result binding (§19 / T-COO-49)
    let current_plan = {
        let runner = match loop_runner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        runner.current_plan().cloned()
    };
    let mut hierarchy = hierarchy;
    wire_dynamic_hierarchy(&mut hierarchy, &goals, current_plan.as_ref());
    ToolOutput::success(serde_json::json!({
        "message": "Objective hierarchy retrieved",
        "status": "ok",
        "count": hierarchy.nodes().len(),
        "nodes": hierarchy.nodes(),
    }))
}

/// Execute cooboploop_set_mission (T13)
pub async fn execute_cooboploop_set_mission(
    input: CooboploopSetMissionInput,
    mission: &Arc<Mutex<String>>,
) -> ToolOutput {
    let mission_name = input.mission.trim().to_string();
    if mission_name.is_empty() {
        return ToolOutput::error("Mission must not be empty");
    }
    let mut retained_mission = match mission.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *retained_mission = mission_name.clone();
    ToolOutput::success(serde_json::json!({
        "message": "Mission set",
        "status": "ok",
        "mission": mission_name,
    }))
}

/// Execute cooboploop_get_autonomy_levels (S22 / T15.15).
pub async fn execute_cooboploop_get_autonomy_levels(
    input: CooboploopGetAutonomyLevelsInput,
    registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
) -> ToolOutput {
    let request = match serde_json::to_value(input) {
        Ok(request) if request.is_object() => request,
        Ok(request) => {
            return ToolOutput::error(format!(
                "autonomy-level request must serialize as an object, received {request}"
            ));
        }
        Err(error) => {
            return ToolOutput::error(format!("serialize autonomy-level request: {error}"));
        }
    };
    let registry = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let autonomy: std::collections::BTreeMap<String, String> = registry
        .autonomy_levels()
        .into_iter()
        .map(|(capability_id, level)| (capability_id.key(), level.as_str().to_string()))
        .collect();
    ToolOutput::success(serde_json::json!({
        "message": "Autonomy levels retrieved",
        "status": "ok",
        "capability_count": autonomy.len(),
        "autonomy": autonomy,
        "request": request,
    }))
}

/// Execute cooboploop_promote_autonomy (S22 / T15.16).
pub async fn execute_cooboploop_promote_autonomy(
    input: CooboploopPromoteAutonomyInput,
    registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
) -> ToolOutput {
    use crate::cooboploop::capability::CapabilityId;

    let requested_id = input.capability_id.trim();
    if requested_id.is_empty() {
        return ToolOutput::error("Capability ID must not be empty");
    }

    let capability_id = CapabilityId::from_string(requested_id);
    let canonical_id = capability_id.key();
    let mut registry = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let (previous_level, new_level) = registry.promote_autonomy(capability_id);

    ToolOutput::success(serde_json::json!({
        "message": "Autonomy promoted",
        "status": "ok",
        "capability_id": canonical_id,
        "previous_level": previous_level.as_str(),
        "new_level": new_level.as_str(),
    }))
}
