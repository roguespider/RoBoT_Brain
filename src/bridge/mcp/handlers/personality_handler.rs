// src/bridge/mcp/handlers/personality_handler.rs
// Personality tools handler - exposes personality system via MCP
// (Architecture §13: Personality)

use std::sync::Arc;

use crate::bridge::mcp::McpContext;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitResult, ToolHandler};
use crate::bridge::tools::personality::{
    ApplyPersonalityPresetInput, FormatResponseInput, GetPersonalityDecisionInput,
    SetPersonalityTraitsInput, execute_apply_personality_preset, execute_format_response,
    execute_get_personality, execute_get_personality_decision, execute_list_personality_presets,
    execute_set_personality_traits,
};

/// Handler for personality-related tools
#[derive(Clone)]
pub struct PersonalityToolsHandler {
    context: Arc<McpContext>,
}

impl PersonalityToolsHandler {
    /// Create a new personality tools handler
    pub fn new(context: Arc<McpContext>) -> HandlerInitResult<Self> {
        // Validate that personality exists (it always does — it's not optional)
        Ok(Self { context })
    }

    pub async fn execute_get_personality(
        &self,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_get_personality(&self.context).await
    }

    pub async fn execute_set_personality_traits(
        &self,
        input: SetPersonalityTraitsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_set_personality_traits(input, &self.context).await
    }

    pub async fn execute_apply_personality_preset(
        &self,
        input: ApplyPersonalityPresetInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_apply_personality_preset(input, &self.context).await
    }

    pub async fn execute_list_personality_presets(
        &self,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_list_personality_presets(&self.context).await
    }

    pub async fn execute_get_personality_decision(
        &self,
        input: GetPersonalityDecisionInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_get_personality_decision(input, &self.context).await
    }

    pub async fn execute_format_response(
        &self,
        input: FormatResponseInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_format_response(input, &self.context).await
    }
}

impl ToolHandler for PersonalityToolsHandler {
    fn category(&self) -> &str {
        "personality"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "get_personality".to_string(),
            "set_personality_traits".to_string(),
            "apply_personality_preset".to_string(),
            "list_personality_presets".to_string(),
            "get_personality_decision".to_string(),
            "format_response".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        true
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "get_personality",
                "Get current personality: traits, preset, communication style, emotional state, humor, interaction mode, and preferences",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Get Personality"),
            rmcp::model::Tool::new(
                "set_personality_traits",
                "Update individual personality traits (curiosity, caution, creativity, patience, thoroughness, verbosity, risk_tolerance, humor_level)",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "curiosity": { "type": "number", "description": "Curiosity (0.0-1.0)" },
                        "caution": { "type": "number", "description": "Caution (0.0-1.0)" },
                        "creativity": { "type": "number", "description": "Creativity (0.0-1.0)" },
                        "patience": { "type": "number", "description": "Patience (0.0-1.0)" },
                        "thoroughness": { "type": "number", "description": "Thoroughness (0.0-1.0)" },
                        "verbosity": { "type": "number", "description": "Verbosity (0.0-1.0)" },
                        "risk_tolerance": { "type": "number", "description": "Risk tolerance (0.0-1.0)" },
                        "humor_level": { "type": "number", "description": "Humor level (0.0-1.0, 0=serious, 1=playful)" }
                    }
                })),
            ).with_title("Set Personality Traits"),
            rmcp::model::Tool::new(
                "apply_personality_preset",
                "Apply a named personality preset (e.g. balanced, analytical, creative, cautious, bold)",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "preset": { "type": "string", "description": "Preset name" }
                    },
                    "required": ["preset"]
                })),
            ).with_title("Apply Personality Preset"),
            rmcp::model::Tool::new(
                "list_personality_presets",
                "List all available personality presets and the currently active one",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("List Personality Presets"),
            rmcp::model::Tool::new(
                "get_personality_decision",
                "Given a decision context (confidence, gain, loss, uncertainty, time), get the personality-driven decision: should_act, reason, approach, confidence",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "confidence": { "type": "number", "description": "Current confidence (0.0-1.0)" },
                        "potential_gain": { "type": "number", "description": "Potential gain" },
                        "potential_loss": { "type": "number", "description": "Potential loss" },
                        "uncertainty": { "type": "number", "description": "Uncertainty (0.0-1.0)" },
                        "time_available": { "type": "integer", "description": "Time available in seconds (default 30)" }
                    },
                    "required": ["confidence", "potential_gain", "potential_loss", "uncertainty"]
                })),
            ).with_title("Get Personality Decision"),
            rmcp::model::Tool::new(
                "format_response",
                "Format text content according to a communication style (concise, balanced, detailed)",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "Content to format" },
                        "style": { "type": "string", "description": "Style: concise, balanced, or detailed (default: current personality style)" }
                    },
                    "required": ["content"]
                })),
            ).with_title("Format Response"),
        ]
    }

    async fn execute_tool(&self, name: &str, args: serde_json::Value) -> Result<crate::bridge::tools::ToolOutput, HandlerError> {
            match name {
                "get_personality" => {
                    self.execute_get_personality().await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "set_personality_traits" => {
                    let input: SetPersonalityTraitsInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_set_personality_traits(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "apply_personality_preset" => {
                    let input: ApplyPersonalityPresetInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_apply_personality_preset(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_personality_presets" => {
                    self.execute_list_personality_presets().await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_personality_decision" => {
                    let input: GetPersonalityDecisionInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_personality_decision(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "format_response" => {
                    let input: FormatResponseInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_format_response(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string())),
            }
    }
}
