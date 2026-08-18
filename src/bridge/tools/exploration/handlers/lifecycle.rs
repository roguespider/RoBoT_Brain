//! Exploration lifecycle handlers: start, pause, resume, complete, abandon.

use std::sync::Arc;
use uuid::Uuid;

use crate::bridge::tools::ToolOutput;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::exploration::{Exploration, ExplorationFinding, ExplorationStatus};
use crate::experience::types::ExperienceContext;

use super::store::{ensure_exploration, with_store};
use crate::bridge::tools::exploration::{
    CompleteExplorationInput, GetExplorationStatusInput, StartExplorationInput,
};

pub fn execute_start_exploration(input: StartExplorationInput) -> ToolOutput {
    let id = Uuid::new_v4().to_string();
    let context = ExperienceContext::default();

    let mut exploration = Exploration::new(id.clone(), input.title, input.purpose, context);
    exploration.start();

    with_store(|store| {
        store.insert(id.clone(), exploration);
        ToolOutput::success(serde_json::json!({
            "exploration_id": id,
            "status": "active",
            "message": "Exploration started. Use get_exploration_status to monitor progress."
        }))
    })
}

pub fn execute_pause_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    with_store(|store| {
        ensure_exploration(store, &input.exploration_id);
        match store.get_mut(&input.exploration_id) {
            Some(exp) => {
                if exp.is_active() {
                    exp.pause();
                }
                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "status": "paused",
                    "message": "Exploration paused."
                }))
            }
            None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
        }
    })
}

pub fn execute_resume_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    with_store(|store| {
        ensure_exploration(store, &input.exploration_id);
        match store.get_mut(&input.exploration_id) {
            Some(exp) => {
                if exp.status != ExplorationStatus::Paused {
                    exp.pause();
                }
                exp.start();
                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "status": "active",
                    "message": "Exploration resumed."
                }))
            }
            None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
        }
    })
}

pub fn execute_complete_exploration(
    input: CompleteExplorationInput,
    coordinator: &Arc<ExperienceCoordinator>,
) -> ToolOutput {
    with_store(|store| {
        ensure_exploration(store, &input.exploration_id);
        match store.get_mut(&input.exploration_id) {
            Some(exp) => {
                for finding_input in input.findings {
                    let finding = ExplorationFinding::new(
                        Uuid::new_v4().to_string(),
                        finding_input.description,
                        finding_input.confidence,
                    );
                    exp.add_finding(finding);
                }

                exp.complete();
                let finding_count = exp.findings.len();

                // Wire into the experience coordinator for metric tracking
                // (Architecture §07: coordinator.process completes the learning
                // pipeline for exploration outcomes).
                let coord = coordinator.clone();
                let exp_id = exp.id.clone();
                tokio::spawn(async move {
                    coord.complete_exploration(&exp_id);
                });

                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "status": "completed",
                    "finding_count": finding_count,
                    "message": format!("Exploration completed with {} findings.", finding_count)
                }))
            }
            None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
        }
    })
}

pub fn execute_abandon_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    with_store(|store| {
        ensure_exploration(store, &input.exploration_id);
        match store.get_mut(&input.exploration_id) {
            Some(exp) => {
                exp.abandon();
                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "status": "abandoned",
                    "message": "Exploration abandoned."
                }))
            }
            None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
        }
    })
}
