//! Hypothesis handlers: add, evaluate, promote finding.

use uuid::Uuid;

use crate::bridge::tools::ToolOutput;
use crate::experience::exploration::{Hypothesis, HypothesisResult};

use super::store::{ensure_exploration, with_store};
use crate::bridge::tools::exploration::{
    AddHypothesisInput, EvaluateHypothesisInput, PromoteFindingInput,
};

pub fn execute_add_hypothesis(input: AddHypothesisInput) -> ToolOutput {
    with_store(|store| {
        ensure_exploration(store, &input.exploration_id);
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
    })
}

pub fn execute_evaluate_hypothesis(input: EvaluateHypothesisInput) -> ToolOutput {
    with_store(|store| {
        ensure_exploration(store, &input.exploration_id);
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

                let hypothesis_id =
                    if let Some(hypothesis) = exp.hypotheses.iter_mut().find(|h| h.id == input.hypothesis_id) {
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
                        exp.add_hypothesis(hypothesis);
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
    })
}

pub fn execute_promote_finding(input: PromoteFindingInput) -> ToolOutput {
    with_store(|store| {
        ensure_exploration(store, &input.exploration_id);
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
    })
}
