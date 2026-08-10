// src/bridge/tools/personality/mod.rs
//! Personality MCP tools (Architecture §13: Personality)
//!
//! Exposes the personality system — traits, presets, communication style,
//! decision-making, and emotional state — via MCP so the self_check probe
//! is no longer the only thing keeping these code paths live (TASK-V2-09).

use crate::bridge::mcp::McpContext;
use crate::bridge::tools::ToolOutput;
use crate::personality::{
    CommunicationStyle, Decision, DecisionApproach, DecisionContext, PersonalityTraits,
};
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// =============================================================================
// INPUT TYPES
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct GetPersonalityInput {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetPersonalityTraitsInput {
    pub curiosity: Option<f32>,
    pub caution: Option<f32>,
    pub creativity: Option<f32>,
    pub patience: Option<f32>,
    pub thoroughness: Option<f32>,
    pub verbosity: Option<f32>,
    pub risk_tolerance: Option<f32>,
    pub humor_level: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApplyPersonalityPresetInput {
    pub preset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListPersonalityPresetsInput {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetPersonalityDecisionInput {
    pub confidence: f32,
    pub potential_gain: f32,
    pub potential_loss: f32,
    pub uncertainty: f32,
    pub time_available: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FormatResponseInput {
    pub content: String,
    pub style: Option<String>,
}

// =============================================================================
// EXECUTORS
// =============================================================================

/// get_personality: returns current traits, preset, communication style,
/// emotional state, humor, interaction mode, and preferences.
pub async fn execute_get_personality(
    _input: GetPersonalityInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let personality = context.personality.lock();
    match personality {
        Ok(p) => {
            let traits = p.get_traits().clone();
            let preset = p.get_current_preset().to_string();
            let style = p.get_communication_style();
            let emotional_weight = p.emotional_weight();
            let humor = p.humor_level();
            let mode = p.interaction_mode();
            let prefs = p.preferences();
            Ok(ToolOutput::success(serde_json::json!({
                "preset": preset,
                "communication_style": format!("{:?}", style),
                "traits": traits,
                "emotional_weight": emotional_weight,
                "humor_level": humor,
                "interaction_mode": format!("{:?}", mode),
                "preferences": {
                    "brevity": prefs.brevity,
                    "familiarity": prefs.familiarity,
                    "reversibility": prefs.reversibility,
                },
            })))
        }
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned in get_personality, recovering");
            let p = poisoned.into_inner();
            let traits = p.get_traits().clone();
            let preset = p.get_current_preset().to_string();
            Ok(ToolOutput::success(serde_json::json!({
                "preset": preset,
                "traits": traits,
                "recovered_from_poison": true,
            })))
        }
    }
}

/// set_personality_traits: updates individual trait values.
pub async fn execute_set_personality_traits(
    input: SetPersonalityTraitsInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let personality = context.personality.lock();
    match personality {
        Ok(mut p) => {
            let traits = p.traits_mut();
            if let Some(v) = input.curiosity {
                traits.curiosity = v;
            }
            if let Some(v) = input.caution {
                traits.caution = v;
            }
            if let Some(v) = input.creativity {
                traits.creativity = v;
            }
            if let Some(v) = input.patience {
                traits.patience = v;
            }
            if let Some(v) = input.thoroughness {
                traits.thoroughness = v;
            }
            if let Some(v) = input.verbosity {
                traits.verbosity = v;
            }
            if let Some(v) = input.risk_tolerance {
                traits.risk_tolerance = v;
            }
            if let Some(v) = input.humor_level {
                p.set_humor_level(v);
            }
            let traits = p.get_traits().clone();
            Ok(ToolOutput::success(serde_json::json!({
                "updated": true,
                "traits": traits,
            })))
        }
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned in set_traits, recovering");
            poisoned.into_inner().set_traits(PersonalityTraits::default());
            Ok(ToolOutput::success(serde_json::json!({
                "updated": true,
                "recovered_from_poison": true,
            })))
        }
    }
}

/// apply_personality_preset: applies a named preset (balanced, analytical, etc.)
pub async fn execute_apply_personality_preset(
    input: ApplyPersonalityPresetInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let personality = context.personality.lock();
    let applied = match personality {
        Ok(mut p) => p.apply_preset(&input.preset),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned in apply_preset, recovering");
            poisoned.into_inner().apply_preset(&input.preset)
        }
    };
    Ok(ToolOutput::success(serde_json::json!({
        "applied": applied,
        "preset": input.preset,
    })))
}

/// list_personality_presets: lists all available presets.
pub async fn execute_list_personality_presets(
    _input: ListPersonalityPresetsInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let (presets, current) = match context.personality.lock() {
        Ok(p) => (p.list_presets(), p.get_current_preset().to_string()),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned in list_presets, recovering");
            let p = poisoned.into_inner();
            (p.list_presets(), p.get_current_preset().to_string())
        }
    };
    Ok(ToolOutput::success(serde_json::json!({
        "presets": presets,
        "current": current,
    })))
}

/// get_personality_decision: exercises Personality::decide with a DecisionContext,
/// returning the Decision (should_act, reason, approach, confidence).
pub async fn execute_get_personality_decision(
    input: GetPersonalityDecisionInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let ctx = DecisionContext {
        confidence: input.confidence,
        potential_gain: input.potential_gain,
        potential_loss: input.potential_loss,
        uncertainty: input.uncertainty,
        time_available: input.time_available.unwrap_or(30),
    };
    let personality = context.personality.lock();
    let decision: Decision = match personality {
        Ok(p) => p.decide(&ctx),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned in decide, recovering");
            poisoned.into_inner().decide(&ctx)
        }
    };
    let approach: DecisionApproach = decision.approach;
    Ok(ToolOutput::success(serde_json::json!({
        "should_act": decision.should_act,
        "reason": decision.reason,
        "approach": format!("{:?}", approach),
        "confidence": decision.confidence,
    })))
}

/// format_response: exercises CommunicationStyle::format_response so the
/// communication style formatting code stays live at runtime.
pub async fn execute_format_response(
    input: FormatResponseInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let formatted = match context.personality.lock() {
        Ok(p) => {
            let style = match input.style.as_deref() {
                Some("concise") => CommunicationStyle::Concise,
                Some("detailed") => CommunicationStyle::Detailed,
                Some("balanced") => CommunicationStyle::Balanced,
                Some(_) => p.get_communication_style(),
                None => p.get_communication_style(),
            };
            // Use Personality::format_response which applies the current
            // communication style; for explicit styles, format directly.
            if input.style.is_some() {
                style.format_response(&input.content)
            } else {
                p.format_response(&input.content)
            }
        }
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned in format_response, recovering");
            let style = poisoned.into_inner().get_communication_style();
            style.format_response(&input.content)
        }
    };
    Ok(ToolOutput::success(serde_json::json!({
        "formatted": formatted,
    })))
}
