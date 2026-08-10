//! Hallucination detection for the safety gate (Architecture §16 —
//! hallucination handling).
//!
//! A hallucination occurs when the agent commits to an action with high
//! confidence but insufficient supporting evidence. The check evaluates
//! evidence **diversity**: an action backed by multiple independent evidence
//! channels (memory + knowledge + experience) is far less likely to be a
//! hallucination than one backed by a single channel or none.
//!
//! The check does NOT block actions on its own — it flags a risk in the
//! `UncertaintyReport` so the safety gate can combine the hallucination flag
//! with the confidence threshold and sandbox boundary before making a final
//! decision.

use crate::agent::decision::{ActionConfidence, ConfidenceComponents, SelectedAction};

/// Result of a hallucination check.
#[derive(Debug, Clone)]
pub struct HallucinationCheck {
    /// Whether a hallucination risk was detected.
    pub risk_detected: bool,
    /// Human-readable explanation of the risk (empty if no risk).
    pub reason: String,
    /// Number of distinct evidence channels that contributed (0–3).
    pub evidence_channels: usize,
    /// Total number of evidence items across all channels.
    pub evidence_count: usize,
}

impl HallucinationCheck {
    /// Evaluate whether the selected action shows hallucination risk.
    ///
    /// Heuristics:
    /// 1. **No evidence**: if the action has zero supporting items across all
    ///    channels, any confidence above the minimum is suspicious.
    /// 2. **Single-channel overconfidence**: if only one channel has evidence
    ///    and confidence > 0.8, flag as risk — high confidence from one
    ///    source is not robust.
    /// 3. **Confidence-evidence gap**: if confidence > 0.7 but total evidence
    ///    items < 2, the agent may be confabulating.
    pub fn evaluate(selected: &SelectedAction) -> Self {
        let memory_count = selected.supporting_memory.len();
        let knowledge_count = selected.supporting_knowledge.len();
        let experience_count = selected.supporting_experiences.len();
        let evidence_count = memory_count + knowledge_count + experience_count;

        let mut channels = 0;
        if memory_count > 0 {
            channels += 1;
        }
        if knowledge_count > 0 {
            channels += 1;
        }
        if experience_count > 0 {
            channels += 1;
        }

        let components = &selected.confidence.components;
        let conf = selected.confidence.value;

        // Check 1: No evidence at all.
        if evidence_count == 0 {
            return Self {
                risk_detected: true,
                reason: format!(
                    "No supporting evidence across any channel (confidence {:.2} is unsupported)",
                    conf
                ),
                evidence_channels: 0,
                evidence_count: 0,
            };
        }

        // Check 2: Single-channel overconfidence.
        if channels == 1 && conf > 0.8 {
            let channel_name = Self::channel_name(components);
            return Self {
                risk_detected: true,
                reason: format!(
                    "High confidence ({:.2}) rests on a single evidence channel ({}) — not robust",
                    conf, channel_name
                ),
                evidence_channels: 1,
                evidence_count,
            };
        }

        // Check 3: Confidence-evidence gap.
        if conf > 0.7 && evidence_count < 2 {
            return Self {
                risk_detected: true,
                reason: format!(
                    "Confidence ({:.2}) exceeds evidence breadth ({} item) — possible confabulation",
                    conf, evidence_count
                ),
                evidence_channels: channels,
                evidence_count,
            };
        }

        Self {
            risk_detected: false,
            reason: String::new(),
            evidence_channels: channels,
            evidence_count,
        }
    }

    /// Name the channel that contributed evidence (for reporting).
    fn channel_name(components: &ConfidenceComponents) -> &'static str {
        if components.memory_support > 0.0 {
            "memory"
        } else if components.knowledge_support > 0.0 {
            "knowledge"
        } else if components.experience_support > 0.0 {
            "experience"
        } else {
            "unknown"
        }
    }
}

/// Adjust confidence downward when hallucination risk is detected.
///
/// This implements the "hallucination handling" requirement: when risk is
/// detected, the confidence is penalized so the safety gate's threshold
/// check is more likely to block the action.
pub fn apply_hallucination_penalty(
    confidence: &mut ActionConfidence,
    check: &HallucinationCheck,
) {
    if check.risk_detected {
        // Penalize by 0.15 — enough to push borderline actions below
        // threshold without making all hallucination-flagged actions
        // impossible.
        confidence.value = (confidence.value - 0.15).clamp(0.0, 1.0);
    }
}
