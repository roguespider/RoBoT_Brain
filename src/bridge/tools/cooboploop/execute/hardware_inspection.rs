use serde_json;
use std::sync::{Arc, Mutex};

use crate::bridge::tools::ToolOutput;
use crate::bridge::tools::cooboploop::inputs::*;
use crate::cooboploop::queue::ObjectiveQueue;

/// Execute cooboploop_get_hardware_profile (T8.10)
pub async fn execute_cooboploop_get_hardware_profile(
    hardware_discovery: &Arc<Mutex<crate::cooboploop::hardware::HardwareDiscovery>>,
    hardware_registry: &Arc<Mutex<crate::cooboploop::hardware::HardwareRegistry>>,
) -> ToolOutput {
    let registry = match hardware_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let stored_profile = registry.latest().ok().flatten();
    let mut discovery = match hardware_discovery.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let profile = if let Some(profile) = stored_profile {
        discovery.set_profile(profile.clone());
        discovery.update_snapshots();
        profile
    } else if let Some(profile) = discovery.detect() {
        if let Err(error) = registry.update(&profile) {
            tracing::warn!("Failed to update hardware registry: {}", error);
        }
        discovery.update_snapshots();
        profile
    } else {
        crate::cooboploop::hardware::HardwareProfile::default()
    };
    ToolOutput::success(serde_json::json!({
        "message": "Hardware profile retrieved",
        "status": "ok",
        "profile": profile,
    }))
}

/// Execute cooboploop_detect_hardware_changes (T8.11)
pub async fn execute_cooboploop_detect_hardware_changes(
    hardware_discovery: &Arc<Mutex<crate::cooboploop::hardware::HardwareDiscovery>>,
    hardware_registry: &Arc<Mutex<crate::cooboploop::hardware::HardwareRegistry>>,
) -> ToolOutput {
    let registry = match hardware_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let previous = registry.latest().ok().flatten();
    let mut discovery = match hardware_discovery.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(previous_profile) = previous {
        discovery.set_profile(previous_profile);
        discovery.update_snapshots();
    }
    let current = match discovery.detect() {
        Some(profile) => profile,
        None => {
            return ToolOutput::success(serde_json::json!({
                "message": "No hardware profile available",
                "status": "ok",
                "changes": Vec::<serde_json::Value>::new(),
                "change_count": 0,
                "current_profile": serde_json::Value::Null,
            }));
        }
    };
    let changes = discovery.detect_hardware_changes();
    if let Err(error) = registry.update(&current) {
        tracing::warn!("Failed to update hardware registry: {}", error);
    }
    ToolOutput::success(serde_json::json!({
        "message": "Hardware changes detected",
        "status": "ok",
        "changes": changes,
        "change_count": changes.len(),
        "current_profile": current,
    }))
}

/// Execute cooboploop_run_inspection (T8.17)
pub async fn execute_cooboploop_run_inspection(
    input: CooboploopRunInspectionInput,
    objective_queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    use crate::cooboploop::inspection::{InspectionTarget, Inspector};

    let target_filter = match input.target.as_deref() {
        Some(value) => match value.parse::<InspectionTarget>() {
            Ok(target) => Some(target),
            Err(error) => return ToolOutput::error(error),
        },
        None => None,
    };
    let mut inspector = Inspector::new();
    let issues: Vec<_> = inspector
        .scan()
        .into_iter()
        .filter(|issue| {
            target_filter
                .as_ref()
                .is_none_or(|target| issue.target == *target)
        })
        .collect();
    let objectives = Inspector::issues_to_objectives(&issues);
    let objective_ids: Vec<String> = objectives
        .iter()
        .map(|objective| objective.id.clone())
        .collect();
    let mut queue = match objective_queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    for objective in &objectives {
        if let Err(error) = queue.enqueue(objective) {
            return ToolOutput::error(format!(
                "Failed to enqueue inspection objective {}: {error}",
                objective.id
            ));
        }
    }
    ToolOutput::success(serde_json::json!({
        "message": "Inspection completed",
        "status": "ok",
        "target_filter": target_filter.map(|target| target.to_string()),
        "issues_found": issues.len(),
        "objectives_generated": objectives.len(),
        "issues": issues,
        "objective_ids": objective_ids,
    }))
}
