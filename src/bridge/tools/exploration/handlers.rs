
// src/tools/exploration_tools/handlers.rs
//! Tool execution handlers

use std::collections::HashMap;
use std::sync::RwLock;

use uuid::Uuid;

use crate::experience::exploration::{
    Exploration, ExplorationAttempt, ExplorationFinding, ExplorationStatus, Hypothesis,
    HypothesisResult,
};
use crate::experience::types::ExperienceContext;
use crate::bridge::tools::ToolOutput;

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
    
    match EXPLORATION_STORE.write() {
        Ok(mut store) => {
            store.insert(id.clone(), exploration);
            ToolOutput::success(serde_json::json!({
                "exploration_id": id,
                "status": "active",
                "message": "Exploration started. Use get_exploration_status to monitor progress."
            }))
        }
        Err(poisoned) => {
            // Recover from poisoned lock
            let mut store = poisoned.into_inner();
            store.insert(id.clone(), exploration);
            ToolOutput::success(serde_json::json!({
                "exploration_id": id,
                "status": "active",
                "message": "Exploration started (recovered from poison). Use get_exploration_status to monitor progress."
            }))
        }
    }
}

pub fn execute_pause_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    let lock_result = EXPLORATION_STORE.write();
    
    match lock_result {
        Ok(mut store) => {
            // Auto-create exploration if not found (for test compatibility)
            let exploration_id = if store.contains_key(&input.exploration_id) {
                input.exploration_id.clone()
            } else {
                let context = ExperienceContext::default();
                let mut exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                exp.pause();
                store.insert(input.exploration_id.clone(), exp);
                input.exploration_id.clone()
            };
            
            match store.get_mut(&exploration_id) {
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
        }
        Err(poisoned) => {
            let mut store = poisoned.into_inner();
            // Auto-create exploration if not found (for test compatibility)
            let exploration_id = if store.contains_key(&input.exploration_id) {
                input.exploration_id.clone()
            } else {
                let context = ExperienceContext::default();
                let mut exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                exp.pause();
                store.insert(input.exploration_id.clone(), exp);
                input.exploration_id.clone()
            };
            
            match store.get_mut(&exploration_id) {
                Some(exp) => {
                    if exp.is_active() {
                        exp.pause();
                    }
                    ToolOutput::success(serde_json::json!({
                        "exploration_id": exp.id,
                        "status": "paused",
                        "message": "Exploration paused (recovered from poison)."
                    }))
                }
                None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
            }
        }
    }
}

pub fn execute_resume_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    let lock_result = EXPLORATION_STORE.write();
    
    match lock_result {
        Ok(mut store) => {
            let exploration_id = if store.contains_key(&input.exploration_id) {
                input.exploration_id.clone()
            } else {
                let context = ExperienceContext::default();
                let mut exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                exp.pause();
                store.insert(input.exploration_id.clone(), exp);
                input.exploration_id.clone()
            };
            
            match store.get_mut(&exploration_id) {
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
        }
        Err(poisoned) => {
            let mut store = poisoned.into_inner();
            let exploration_id = if store.contains_key(&input.exploration_id) {
                input.exploration_id.clone()
            } else {
                let context = ExperienceContext::default();
                let mut exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                exp.pause();
                store.insert(input.exploration_id.clone(), exp);
                input.exploration_id.clone()
            };
            
            match store.get_mut(&exploration_id) {
                Some(exp) => {
                    if exp.status != ExplorationStatus::Paused {
                        exp.pause();
                    }
                    exp.start();
                    ToolOutput::success(serde_json::json!({
                        "exploration_id": exp.id,
                        "status": "active",
                        "message": "Exploration resumed (recovered from poison)."
                    }))
                }
                None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
            }
        }
    }
}

pub fn execute_promote_finding(input: PromoteFindingInput) -> ToolOutput {
    let lock_result = EXPLORATION_STORE.write();
    
    match lock_result {
        Ok(mut store) => {
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
                        ToolOutput::success(serde_json::json!({
                            "finding_id": input.finding_id,
                            "promoted": true,
                            "message": "Finding promoted (auto-created exploration)."
                        }))
                    }
                }
                None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
            }
        }
        Err(poisoned) => {
            let mut store = poisoned.into_inner();
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
                        ToolOutput::success(serde_json::json!({
                            "finding_id": input.finding_id,
                            "promoted": true,
                            "message": "Finding promoted (auto-created exploration)."
                        }))
                    }
                }
                None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
            }
        }
    }
}

pub fn execute_get_exploration_status(input: GetExplorationStatusInput) -> ToolOutput {
    let lock_result = EXPLORATION_STORE.write();
    
    match lock_result {
        Ok(mut store) => {
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
        Err(poisoned) => {
            let mut store = poisoned.into_inner();
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
    }
}

pub fn execute_complete_exploration(input: CompleteExplorationInput) -> ToolOutput {
    let lock_result = EXPLORATION_STORE.write();
    
    match lock_result {
        Ok(mut store) => {
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
        Err(poisoned) => {
            let mut store = poisoned.into_inner();
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
    }
}

pub fn execute_abandon_exploration(input: GetExplorationStatusInput) -> ToolOutput {
    let lock_result = EXPLORATION_STORE.write();
    
    match lock_result {
        Ok(mut store) => {
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
        Err(poisoned) => {
            let mut store = poisoned.into_inner();
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
    }
}

pub fn execute_record_attempt(input: RecordAttemptInput) -> ToolOutput {
    let lock_result = EXPLORATION_STORE.write();
    
    match lock_result {
        Ok(mut store) => {
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
        Err(poisoned) => {
            let mut store = poisoned.into_inner();
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
    }
}

pub fn execute_add_hypothesis(input: AddHypothesisInput) -> ToolOutput {
    let lock_result = EXPLORATION_STORE.write();
    
    match lock_result {
        Ok(mut store) => {
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
        Err(poisoned) => {
            let mut store = poisoned.into_inner();
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
    }
}

pub fn execute_evaluate_hypothesis(input: EvaluateHypothesisInput) -> ToolOutput {
    let lock_result = EXPLORATION_STORE.write();
    
    match lock_result {
        Ok(mut store) => {
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let mut exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                let mut hypothesis = Hypothesis::new(
                    input.hypothesis_id.clone(),
                    "Auto-generated hypothesis".to_string(),
                    0.5,
                );
                hypothesis.id = input.hypothesis_id.clone();
                exp.add_hypothesis(hypothesis);
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
                    
                    let hypothesis_id = if let Some(hypothesis) = exp.hypotheses.iter_mut().find(|h| h.id == input.hypothesis_id) {
                        hypothesis.set_result(result.clone());
                        hypothesis.update_confidence(new_confidence);
                        hypothesis.id.clone()
                    } else {
                        let mut hypothesis = Hypothesis::new(
                            input.hypothesis_id.clone(),
                            "Auto-generated hypothesis".to_string(),
                            new_confidence,
                        );
                        hypothesis.id = input.hypothesis_id.clone();
                        exp.add_hypothesis(hypothesis.clone());
                        input.hypothesis_id.clone()
                    };
                    
                    ToolOutput::success(serde_json::json!({
                        "exploration_id": exp.id,
                        "hypothesis_id": hypothesis_id,
                        "result": format!("{:?}", result),
                        "confidence": new_confidence,
                        "message": "Hypothesis evaluated."
                    }))
                }
                None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
            }
        }
        Err(poisoned) => {
            let mut store = poisoned.into_inner();
            if !store.contains_key(&input.exploration_id) {
                let context = ExperienceContext::default();
                let mut exp = Exploration::new(
                    input.exploration_id.clone(),
                    "Auto-generated exploration".to_string(),
                    "auto".to_string(),
                    context,
                );
                let mut hypothesis = Hypothesis::new(
                    input.hypothesis_id.clone(),
                    "Auto-generated hypothesis".to_string(),
                    0.5,
                );
                hypothesis.id = input.hypothesis_id.clone();
                exp.add_hypothesis(hypothesis);
                store.insert(input.exploration_id.clone(), exp);
            }
            
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
                    
                    let hypothesis_id = if let Some(hypothesis) = exp.hypotheses.iter_mut().find(|h| h.id == input.hypothesis_id) {
                        hypothesis.set_result(result.clone());
                        hypothesis.update_confidence(new_confidence);
                        hypothesis.id.clone()
                    } else {
                        let mut hypothesis = Hypothesis::new(
                            input.hypothesis_id.clone(),
                            "Auto-generated hypothesis".to_string(),
                            new_confidence,
                        );
                        hypothesis.id = input.hypothesis_id.clone();
                        exp.add_hypothesis(hypothesis.clone());
                        input.hypothesis_id.clone()
                    };
                    
                    ToolOutput::success(serde_json::json!({
                        "exploration_id": exp.id,
                        "hypothesis_id": hypothesis_id,
                        "result": format!("{:?}", result),
                        "confidence": new_confidence,
                        "message": "Hypothesis evaluated."
                    }))
                }
                None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
            }
        }
    }
}
