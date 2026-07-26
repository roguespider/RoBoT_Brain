// src/tools/exploration_tools/handlers.rs
//! Tool execution handlers

use std::collections::HashMap;
use std::sync::RwLock;

use uuid::Uuid;
use lazy_static::lazy_static;

use crate::experience::exploration::{
    Exploration, ExplorationAttempt, ExplorationFinding, ExplorationStatus, Hypothesis,
    HypothesisResult,
};
use crate::experience::types::ExperienceContext;
use crate::tools::ToolOutput;

use super::{
    StartExplorationInput, GetExplorationStatusInput, CompleteExplorationInput,
    RecordAttemptInput, AddHypothesisInput, EvaluateHypothesisInput, PromoteFindingInput,
};

lazy_static::lazy_static! {
    static ref EXPLORATION_STORE: RwLock<HashMap<String, Exploration>> = RwLock::new(HashMap::new());
}

pub fn execute_start_exploration(input: StartExplorationInput) -> ToolOutput {
    let id = Uuid::new_v4().to_string();
    let context = ExperienceContext::default();
    
    let mut exploration = Exploration::new(id.clone(), input.title, input.purpose, context);
    exploration.start();
    
    let mut store = EXPLORATION_STORE.write().unwrap();
    store.insert(id.clone(), exploration);
    
    ToolOutput::success(serde_json::json!({
        "exploration_id": id,
        "status": "active",
        "message": "Exploration started. Use get_exploration_status to monitor progress."
    }))
}

pub fn execute_pause_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            if exp.is_active() {
                exp.pause();
                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "status": "paused",
                    "message": "Exploration paused."
                }))
            } else {
                ToolOutput::error(format!("Exploration is not active (current status: {:?})", exp.status))
            }
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

pub fn execute_resume_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            if exp.status == ExplorationStatus::Paused {
                exp.start();
                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "status": "active",
                    "message": "Exploration resumed."
                }))
            } else {
                ToolOutput::error(format!("Exploration is not paused (current status: {:?})", exp.status))
            }
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

pub fn execute_promote_finding(input: PromoteFindingInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            if let Some(f) = exp.findings.iter_mut().find(|f| f.id == input.finding_id) {
                f.promote();
                ToolOutput::success(serde_json::json!({
                    "finding_id": f.id,
                    "promoted": true,
                    "message": format!("Finding '{}' has been promoted to reusable knowledge.", f.description)
                }))
            } else {
                ToolOutput::error(format!("Finding not found: {}", input.finding_id))
            }
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

pub fn execute_get_exploration_status(input: GetExplorationStatusInput) -> ToolOutput {
    let store = EXPLORATION_STORE.read().unwrap();
    
    match store.get(&input.exploration_id) {
        Some(exp) => {
            let hypotheses_json: Vec<_> = exp.hypotheses.iter().map(|h| serde_json::json!({
                "id": h.id,
                "statement": h.statement,
                "confidence": h.confidence,
                "result": h.result.as_ref().map(|r| format!("{:?}", r))
            })).collect();
            
            let attempts_json: Vec<_> = exp.attempts.iter().map(|a| serde_json::json!({
                "id": a.id,
                "action": a.action,
                "expected_result": a.expected_result,
                "actual_result": a.actual_result,
                "success": a.success
            })).collect();
            
            let findings_json: Vec<_> = exp.findings.iter().map(|f| serde_json::json!({
                "id": f.id,
                "description": f.description,
                "confidence": f.confidence,
                "promoted": f.promoted
            })).collect();
            
            ToolOutput::success(serde_json::json!({
                "exploration_id": exp.id,
                "title": exp.title,
                "purpose": exp.purpose,
                "status": format!("{:?}", exp.status),
                "started_at": exp.started_at.to_rfc3339(),
                "completed_at": exp.completed_at.map(|t| t.to_rfc3339()),
                "is_active": exp.is_active(),
                "is_complete": exp.is_complete(),
                "hypotheses": hypotheses_json,
                "attempts": attempts_json,
                "findings": findings_json
            }))
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

pub fn execute_complete_exploration(input: CompleteExplorationInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
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
            
            ToolOutput::success(serde_json::json!({
                "exploration_id": exp.id,
                "status": "completed",
                "finding_count": finding_count,
                "message": format!("Exploration completed with {} findings.", finding_count)
            }))
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

pub fn execute_abandon_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
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
}

pub fn execute_record_attempt(input: RecordAttemptInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            let mut attempt = ExplorationAttempt::new(Uuid::new_v4().to_string(), input.action);
            
            if let Some(expected) = input.expected_result {
                attempt = attempt.with_expected_result(expected);
            }
            if let Some(actual) = input.actual_result {
                attempt = attempt.with_actual_result(actual);
            }
            
            exp.add_attempt(attempt);
            
            ToolOutput::success(serde_json::json!({
                "exploration_id": exp.id,
                "attempt_count": exp.attempts.len(),
                "message": "Attempt recorded."
            }))
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

pub fn execute_add_hypothesis(input: AddHypothesisInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            let hypothesis = Hypothesis::new(
                Uuid::new_v4().to_string(),
                input.statement,
                input.initial_confidence.unwrap_or(0.5),
            );
            
            exp.add_hypothesis(hypothesis);
            
            ToolOutput::success(serde_json::json!({
                "exploration_id": exp.id,
                "hypothesis_count": exp.hypotheses.len(),
                "message": "Hypothesis added to exploration."
            }))
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}

pub fn execute_evaluate_hypothesis(input: EvaluateHypothesisInput) -> ToolOutput {
    let mut store = EXPLORATION_STORE.write().unwrap();
    
    match store.get_mut(&input.exploration_id) {
        Some(exp) => {
            let result = match input.result.to_lowercase().as_str() {
                "supported" => HypothesisResult::Supported,
                "partially_supported" => HypothesisResult::PartiallySupported,
                "rejected" => HypothesisResult::Rejected,
                _ => HypothesisResult::Unknown,
            };
            
            let new_confidence = match result {
                HypothesisResult::Supported => 0.9,
                HypothesisResult::PartiallySupported => 0.6,
                HypothesisResult::Rejected => 0.1,
                HypothesisResult::Unknown => 0.5,
            };
            
            if let Some(hypothesis) = exp.hypotheses.iter_mut().find(|h| h.id == input.hypothesis_id) {
                hypothesis.set_result(result.clone());
                hypothesis.update_confidence(new_confidence);
                
                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "hypothesis_id": hypothesis.id,
                    "result": format!("{:?}", result),
                    "confidence": hypothesis.confidence,
                    "message": "Hypothesis evaluated."
                }))
            } else {
                ToolOutput::error(format!("Hypothesis not found: {}", input.hypothesis_id))
            }
        }
        None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
    }
}
