// src/bridge/tools/handlers/skills_handler.rs
// Skills tools handler - handles skill registry operations

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::skills;
use crate::bridge::mcp::handlers::{HandlerInitResult, ToolHandler};
use crate::workflows::enforcement::WorkflowEnforcer;

/// Handler for skills-related tools
#[derive(Clone)]
pub struct SkillsToolsHandler {
    context: Arc<McpContext>,
    enforcer: Arc<WorkflowEnforcer>,
}

impl SkillsToolsHandler {
    /// Create a new skills tools handler
    pub fn new(
        context: Arc<McpContext>,
        enforcer: Arc<WorkflowEnforcer>,
    ) -> HandlerInitResult<Self> {
        Ok(Self { context, enforcer })
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
        ]
    }

    fn is_healthy(&self) -> bool {
        true
    }
}
