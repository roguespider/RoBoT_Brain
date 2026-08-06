// reflection_tools.rs - Reflection and pattern analysis tools

use crate::bridge::rmcp::generated::tool_traits::{
    ReflectionToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for reflection tools - implements ReflectionToolsHandlerTrait
pub struct ReflectionToolsHandler;

impl ReflectionToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReflectionToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ReflectionToolsHandlerTrait for ReflectionToolsHandler {
    async fn execute_reflect_on_action(
        &self,
        context: &ToolContext,
        input: tools::reflection::ReflectOnActionInput,
    ) -> ToolOutput {
        match tools::reflection::execute_reflect_on_action(input, &context.context.reflection)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_reflection_insights(
        &self,
        context: &ToolContext,
        input: tools::reflection::GetReflectionInsightsInput,
    ) -> ToolOutput {
        match tools::reflection::execute_get_reflection_insights(input, &context.context.reflection)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_reflections(
        &self,
        context: &ToolContext,
        input: tools::reflection::ListReflectionsInput,
    ) -> ToolOutput {
        match tools::reflection::execute_list_reflections(input, &context.context.reflection)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_reflection(
        &self,
        context: &ToolContext,
        input: tools::reflection::GetReflectionInput,
    ) -> ToolOutput {
        match tools::reflection::execute_get_reflection(input, &context.context.reflection).await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_generate_insights(
        &self,
        context: &ToolContext,
        input: tools::reflection::GenerateInsightsInput,
    ) -> ToolOutput {
        match tools::reflection::execute_generate_insights(input, &context.context.reflection)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::reflection::reflect_on_action_tool(),
            tools::reflection::get_reflection_insights_tool(),
            tools::reflection::list_reflections_tool(),
            tools::reflection::get_reflection_tool(),
            tools::reflection::generate_insights_tool(),
        ]
    }
}
