// skills_tools.rs - Skill management tools

use crate::bridge::rmcp::generated::tool_traits::{
    SkillsToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for skills tools - implements SkillsToolsHandlerTrait
pub struct SkillsToolsHandler;

impl SkillsToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SkillsToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillsToolsHandlerTrait for SkillsToolsHandler {
    async fn execute_install_skill(
        &self,
        context: &ToolContext,
        input: tools::skills::InstallSkillInput,
    ) -> ToolOutput {
        match tools::skills::execute_install_skill(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_uninstall_skill(
        &self,
        context: &ToolContext,
        input: tools::skills::UninstallSkillInput,
    ) -> ToolOutput {
        match tools::skills::execute_uninstall_skill(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_skills(
        &self,
        context: &ToolContext,
        input: tools::skills::ListSkillsInput,
    ) -> ToolOutput {
        match tools::skills::execute_list_skills(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_skill(
        &self,
        context: &ToolContext,
        input: tools::skills::GetSkillInput,
    ) -> ToolOutput {
        match tools::skills::execute_get_skill(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_update_skill(
        &self,
        context: &ToolContext,
        input: tools::skills::UpdateSkillInput,
    ) -> ToolOutput {
        match tools::skills::execute_update_skill(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_enable_skill(
        &self,
        context: &ToolContext,
        input: tools::skills::EnableSkillInput,
    ) -> ToolOutput {
        match tools::skills::execute_enable_skill(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_disable_skill(
        &self,
        context: &ToolContext,
        input: tools::skills::DisableSkillInput,
    ) -> ToolOutput {
        match tools::skills::execute_disable_skill(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_skill_metrics(
        &self,
        context: &ToolContext,
        input: tools::skills::GetSkillMetricsInput,
    ) -> ToolOutput {
        match tools::skills::execute_get_skill_metrics(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_skill_status(
        &self,
        context: &ToolContext,
        input: tools::skills::GetSkillStatusInput,
    ) -> ToolOutput {
        match tools::skills::execute_get_skill_status(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_activate_skill(
        &self,
        context: &ToolContext,
        input: tools::skills::ActivateSkillInput,
    ) -> ToolOutput {
        match tools::skills::execute_activate_skill(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_deactivate_skill(
        &self,
        context: &ToolContext,
        input: tools::skills::DeactivateSkillInput,
    ) -> ToolOutput {
        match tools::skills::execute_deactivate_skill(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::skills::install_skill_tool(),
            tools::skills::uninstall_skill_tool(),
            tools::skills::list_skills_tool(),
            tools::skills::get_skill_tool(),
            tools::skills::update_skill_tool(),
            tools::skills::enable_skill_tool(),
            tools::skills::disable_skill_tool(),
            tools::skills::get_skill_metrics_tool(),
            tools::skills::get_skill_status_tool(),
            tools::skills::activate_skill_tool(),
            tools::skills::deactivate_skill_tool(),
        ]
    }
}
