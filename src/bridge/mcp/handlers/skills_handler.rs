// src/bridge/tools/handlers/skills_handler.rs
// Skills tools handler - handles skill registry operations

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::skills;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitResult, ToolHandler};

/// Handler for skills-related tools
#[derive(Clone)]
pub struct SkillsToolsHandler {
    context: Arc<McpContext>,
}

impl SkillsToolsHandler {
    /// Create a new skills tools handler
    pub fn new(
        context: Arc<McpContext>,
    ) -> HandlerInitResult<Self> {
        Ok(Self { context })
    }

    /// Register a new skill
    pub async fn execute_register_skill(
        &self,
        input: skills::RegisterSkillInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_register_skill(input, &self.context).await
    }

    /// Discover a skill from experience
    pub async fn execute_discover_skill(
        &self,
        input: skills::DiscoverSkillInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_discover_skill(input, &self.context).await
    }

    /// Get skill details
    pub async fn execute_get_skill(
        &self,
        input: skills::GetSkillInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_get_skill(input, &self.context).await
    }

    /// List all skills
    pub async fn execute_list_skills(
        &self,
        input: skills::ListSkillsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_list_skills(input, &self.context).await
    }

    /// Update skill mastery
    pub async fn execute_update_skill_mastery(
        &self,
        input: skills::UpdateSkillMasteryInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_update_skill_mastery(input, &self.context).await
    }

    /// Get skill recommendations
    pub async fn execute_get_skill_recommendations(
        &self,
        input: skills::GetSkillRecommendationsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_get_skill_recommendations(input, &self.context).await
    }

    /// Execute a skill
    pub async fn execute_execute_skill(
        &self,
        input: skills::ExecuteSkillInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_execute_skill(input, &self.context).await
    }

    /// Get skill statistics
    pub async fn execute_get_skill_stats(
        &self,
        input: skills::GetSkillStatsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_get_skill_stats(input, &self.context).await
    }

    /// Apply skill decay
    pub async fn execute_apply_skill_decay(
        &self,
        input: skills::ApplySkillDecayInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_apply_skill_decay(input, &self.context).await
    }

    /// Enable or disable a skill
    pub async fn execute_enable_disable_skill(
        &self,
        input: skills::EnableDisableSkillInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_enable_disable_skill(input, &self.context).await
    }

    /// Search skills
    pub async fn execute_search_skills(
        &self,
        input: skills::SearchSkillsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_search_skills(input, &self.context).await
    }

    /// Get skill execution metrics
    pub async fn execute_get_skill_metrics(
        &self,
        input: skills::GetSkillMetricsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_get_skill_metrics(input, &self.context).await
    }

    /// Get unreliable skills
    pub async fn execute_get_unreliable_skills(
        &self,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_get_unreliable_skills(&self.context).await
    }

    /// Clear skill metrics
    pub async fn execute_clear_skill_metrics(
        &self,
        input: skills::ClearSkillMetricsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_clear_skill_metrics(input, &self.context).await
    }

    /// Search skills by tag
    pub async fn execute_search_skills_by_tag(
        &self,
        input: skills::SearchSkillsByTagInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_search_skills_by_tag(input, &self.context).await
    }

    /// Unregister a skill
    pub async fn execute_unregister_skill(
        &self,
        input: skills::UnregisterSkillInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        skills::execute_unregister_skill(input, &self.context).await
    }
}

impl ToolHandler for SkillsToolsHandler {
    fn category(&self) -> &str {
        "skills"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "register_skill".to_string(),
            "discover_skill".to_string(),
            "get_skill".to_string(),
            "list_skills".to_string(),
            "update_skill_mastery".to_string(),
            "get_skill_recommendations".to_string(),
            "execute_skill".to_string(),
            "get_skill_stats".to_string(),
            "apply_skill_decay".to_string(),
            "enable_disable_skill".to_string(),
            "search_skills".to_string(),
            "get_skill_metrics".to_string(),
            "get_unreliable_skills".to_string(),
            "clear_skill_metrics".to_string(),
            "search_skills_by_tag".to_string(),
            "unregister_skill".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        true
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "register_skill",
                "Register a new skill in the skill registry",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Skill name" },
                        "description": { "type": "string", "description": "Skill description" },
                        "category": { "type": "string", "description": "Skill category" },
                        "parameters": { "type": "string", "description": "JSON schema for parameters" }
                    },
                    "required": ["name", "description", "category"]
                })),
            ).with_title("Register Skill"),
            rmcp::model::Tool::new(
                "discover_skill",
                "Discover a new skill from experience",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Skill name" },
                        "description": { "type": "string", "description": "Skill description" },
                        "category": { "type": "string", "description": "Skill category" }
                    },
                    "required": ["name", "description", "category"]
                })),
            ).with_title("Discover Skill"),
            rmcp::model::Tool::new(
                "get_skill",
                "Get skill details by ID",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "skill_id": { "type": "string", "description": "Skill ID" }
                    },
                    "required": ["skill_id"]
                })),
            ).with_title("Get Skill"),
            rmcp::model::Tool::new(
                "list_skills",
                "List all skills with optional filtering",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "category": { "type": "string", "description": "Filter by category" },
                        "enabled_only": { "type": "boolean", "description": "Only enabled skills" }
                    }
                })),
            ).with_title("List Skills"),
            rmcp::model::Tool::new(
                "update_skill_mastery",
                "Update mastery level for a skill",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "skill_id": { "type": "string", "description": "Skill ID" },
                        "mastery": { "type": "number", "description": "Mastery level (0.0-1.0)" },
                        "success": { "type": "boolean", "description": "Was skill successful" }
                    },
                    "required": ["skill_id", "mastery"]
                })),
            ).with_title("Update Skill Mastery"),
            rmcp::model::Tool::new(
                "get_skill_recommendations",
                "Get recommended skills based on context",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task": { "type": "string", "description": "Task description" }
                    }
                })),
            ).with_title("Get Skill Recommendations"),
            rmcp::model::Tool::new(
                "execute_skill",
                "Execute a registered skill",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "skill_id": { "type": "string", "description": "Skill ID to execute" },
                        "parameters": { "type": "string", "description": "JSON parameters" }
                    },
                    "required": ["skill_id"]
                })),
            ).with_title("Execute Skill"),
            rmcp::model::Tool::new(
                "get_skill_stats",
                "Get skill statistics",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Get Skill Stats"),
            rmcp::model::Tool::new(
                "apply_skill_decay",
                "Apply decay to skill mastery over time",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Apply Skill Decay"),
            rmcp::model::Tool::new(
                "enable_disable_skill",
                "Enable or disable a skill",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "skill_id": { "type": "string", "description": "Skill ID" },
                        "enabled": { "type": "boolean", "description": "Enable or disable" }
                    },
                    "required": ["skill_id", "enabled"]
                })),
            ).with_title("Enable/Disable Skill"),
            rmcp::model::Tool::new(
                "search_skills",
                "Search skills by name or description",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query" }
                    },
                    "required": ["query"]
                })),
            ).with_title("Search Skills"),
            rmcp::model::Tool::new(
                "get_skill_metrics",
                "Get execution metrics for a skill or all skills",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "skill_id": { "type": "string", "description": "Optional skill ID. If omitted, returns all metrics." }
                    }
                })),
            ).with_title("Get Skill Metrics"),
            rmcp::model::Tool::new(
                "get_unreliable_skills",
                "List skills marked as unreliable by execution metrics",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Get Unreliable Skills"),
            rmcp::model::Tool::new(
                "clear_skill_metrics",
                "Clear execution metrics for a skill or all skills",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "skill_id": { "type": "string", "description": "Optional skill ID. If omitted, clears all." }
                    }
                })),
            ).with_title("Clear Skill Metrics"),
            rmcp::model::Tool::new(
                "search_skills_by_tag",
                "Search skills by tag",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "tag": { "type": "string", "description": "Tag to search for" }
                    },
                    "required": ["tag"]
                })),
            ).with_title("Search Skills By Tag"),
            rmcp::model::Tool::new(
                "unregister_skill",
                "Unregister a skill from the registry",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "skill_id": { "type": "string", "description": "Skill ID to unregister" }
                    },
                    "required": ["skill_id"]
                })),
            ).with_title("Unregister Skill"),
        ]
    }

    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            match name {
                "register_skill" => {
                    let input: skills::RegisterSkillInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_register_skill(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "discover_skill" => {
                    let input: skills::DiscoverSkillInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_discover_skill(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_skill" => {
                    let input: skills::GetSkillInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_skill(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_skills" => {
                    let input: skills::ListSkillsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_list_skills(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "update_skill_mastery" => {
                    let input: skills::UpdateSkillMasteryInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_update_skill_mastery(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_skill_recommendations" => {
                    let input: skills::GetSkillRecommendationsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_get_skill_recommendations(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "execute_skill" => {
                    let input: skills::ExecuteSkillInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_execute_skill(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_skill_stats" => {
                    let input: skills::GetSkillStatsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_get_skill_stats(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "apply_skill_decay" => {
                    let input: skills::ApplySkillDecayInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_apply_skill_decay(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "enable_disable_skill" => {
                    let input: skills::EnableDisableSkillInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_enable_disable_skill(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "search_skills" => {
                    let input: skills::SearchSkillsInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_search_skills(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_skill_metrics" => {
                    let input: skills::GetSkillMetricsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_get_skill_metrics(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_unreliable_skills" => {
                    self.execute_get_unreliable_skills().await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "clear_skill_metrics" => {
                    let input: skills::ClearSkillMetricsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_clear_skill_metrics(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "search_skills_by_tag" => {
                    let input: skills::SearchSkillsByTagInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_search_skills_by_tag(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "unregister_skill" => {
                    let input: skills::UnregisterSkillInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_unregister_skill(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string()))
            }
        }
    }
}
