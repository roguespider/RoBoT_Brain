// tool_traits.rs
// Defines traits for each tool category
// Each *_tools.rs module implements its corresponding trait for a local handler struct
// McpServerHandler aggregates all tool sub-handlers via these traits

use crate::bridge::rmcp::types::{McpContext, WorkflowEnforcer};
use crate::tools::ToolOutput;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Context needed by tool handlers
pub struct ToolContext {
    pub context: Arc<McpContext>,
    pub session_id: String,
    pub enforcer: Arc<Mutex<WorkflowEnforcer>>,
}

/// Trait for memory-related tools
pub trait MemoryToolsHandlerTrait: Send + Sync {
    fn execute_get_workflow(
        &self,
        context: &ToolContext,
        input: crate::tools::agent::GetWorkflowInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_store_memory(
        &self,
        context: &ToolContext,
        input: crate::tools::memory::StoreMemoryInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_search_memory(
        &self,
        context: &ToolContext,
        input: crate::tools::memory::SearchMemoryInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_memory(
        &self,
        context: &ToolContext,
        input: crate::tools::memory::GetMemoryInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_memories(
        &self,
        context: &ToolContext,
        input: crate::tools::memory::ListMemoriesInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_store_embedding(
        &self,
        context: &ToolContext,
        input: crate::tools::memory::StoreEmbeddingInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_embedding(
        &self,
        context: &ToolContext,
        input: crate::tools::memory::GetEmbeddingInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_search_similar(
        &self,
        context: &ToolContext,
        input: crate::tools::memory::SearchSimilarInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_embeddings(
        &self,
        context: &ToolContext,
        input: crate::tools::memory::ListEmbeddingsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_delete_embedding(
        &self,
        context: &ToolContext,
        input: crate::tools::memory::DeleteEmbeddingInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_embedding_stats(
        &self,
        context: &ToolContext,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}

/// Trait for experience-related tools
pub trait ExperienceToolsHandlerTrait: Send + Sync {
    fn execute_record_experience(
        &self,
        context: &ToolContext,
        input: crate::tools::experience::RecordExperienceInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_experience_stats(
        &self,
        context: &ToolContext,
        input: crate::tools::experience::GetExperienceStatsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_experiences(
        &self,
        context: &ToolContext,
        input: crate::tools::experience::ListExperiencesInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_experience(
        &self,
        context: &ToolContext,
        input: crate::tools::experience::GetExperienceInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_search_experiences(
        &self,
        context: &ToolContext,
        input: crate::tools::experience::SearchExperiencesInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_delete_experience(
        &self,
        context: &ToolContext,
        input: crate::tools::experience::DeleteExperienceInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}

/// Trait for reflection-related tools
pub trait ReflectionToolsHandlerTrait: Send + Sync {
    fn execute_reflect_on_action(
        &self,
        context: &ToolContext,
        input: crate::tools::reflection::ReflectOnActionInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_reflection_insights(
        &self,
        context: &ToolContext,
        input: crate::tools::reflection::GetReflectionInsightsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_reflections(
        &self,
        context: &ToolContext,
        input: crate::tools::reflection::ListReflectionsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_reflection(
        &self,
        context: &ToolContext,
        input: crate::tools::reflection::GetReflectionInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_generate_insights(
        &self,
        context: &ToolContext,
        input: crate::tools::reflection::GenerateInsightsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}

/// Trait for search-related tools
pub trait SearchToolsHandlerTrait: Send + Sync {
    fn execute_search_knowledge(
        &self,
        context: &ToolContext,
        input: crate::tools::search::SearchKnowledgeInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_search_experiences(
        &self,
        context: &ToolContext,
        input: crate::tools::search::SearchExperiencesInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_search_memories(
        &self,
        context: &ToolContext,
        input: crate::tools::search::SearchMemoriesInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_advanced_search(
        &self,
        context: &ToolContext,
        input: crate::tools::search::AdvancedSearchInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}

/// Trait for ingestor-related tools
pub trait IngestorToolsHandlerTrait: Send + Sync {
    fn execute_ingest_files(
        &self,
        context: &ToolContext,
        input: crate::tools::ingestor::IngestFilesInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_importable(
        &self,
        context: &ToolContext,
        input: crate::tools::ingestor::ListImportableInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_ingested_files(
        &self,
        context: &ToolContext,
        input: crate::tools::ingestor::ListIngestedFilesInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_delete_ingested_files(
        &self,
        context: &ToolContext,
        input: crate::tools::ingestor::DeleteIngestedFilesInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_transcribe_audio(
        &self,
        context: &ToolContext,
        input: crate::tools::ingestor::TranscribeAudioInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}

/// Trait for agent-related tools
pub trait AgentToolsHandlerTrait: Send + Sync {
    fn execute_call_mcp_tool(
        &self,
        context: &ToolContext,
        input: crate::tools::agent::CallMcpToolInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_connect_mcp_server(
        &self,
        context: &ToolContext,
        input: crate::tools::agent::ConnectMcpServerInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_tool(
        &self,
        context: &ToolContext,
        input: crate::tools::agent::GetToolInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_workflow(
        &self,
        context: &ToolContext,
        input: crate::tools::agent::GetWorkflowInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_tools(
        &self,
        context: &ToolContext,
        input: crate::tools::agent::ListToolsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}

/// Trait for hypothesis-related tools
pub trait HypothesisToolsHandlerTrait: Send + Sync {
    fn execute_create_hypothesis(
        &self,
        context: &ToolContext,
        input: crate::tools::hypothesis::CreateHypothesisInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_hypothesis(
        &self,
        context: &ToolContext,
        input: crate::tools::hypothesis::GetHypothesisInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_hypotheses(
        &self,
        context: &ToolContext,
        input: crate::tools::hypothesis::ListHypothesesInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_evaluate_hypothesis(
        &self,
        context: &ToolContext,
        input: crate::tools::hypothesis::EvaluateHypothesisInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_record_observation(
        &self,
        context: &ToolContext,
        input: crate::tools::hypothesis::RecordObservationInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_observations(
        &self,
        context: &ToolContext,
        input: crate::tools::hypothesis::ListObservationsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_add_evidence(
        &self,
        context: &ToolContext,
        input: crate::tools::hypothesis::AddEvidenceInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_evidence(
        &self,
        context: &ToolContext,
        input: crate::tools::hypothesis::GetEvidenceInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_evidence(
        &self,
        context: &ToolContext,
        input: crate::tools::hypothesis::ListEvidenceInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_extract_knowledge(
        &self,
        context: &ToolContext,
        input: crate::tools::hypothesis::ExtractKnowledgeInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_knowledge(
        &self,
        context: &ToolContext,
        input: crate::tools::hypothesis::GetKnowledgeInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}

/// Trait for knowledge-related tools
pub trait KnowledgeToolsHandlerTrait: Send + Sync {
    fn execute_create_knowledge(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::CreateKnowledgeInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_knowledge(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::GetKnowledgeInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_knowledge(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::ListKnowledgeInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_search_knowledge(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::SearchKnowledgeInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_update_knowledge(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::UpdateKnowledgeInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_delete_knowledge(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::DeleteKnowledgeInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_knowledge_stats(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::GetKnowledgeStatsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_create_knowledge_base(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::CreateKnowledgeBaseInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_knowledge_base(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::GetKnowledgeBaseInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_knowledge_bases(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::ListKnowledgeBasesInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_delete_knowledge_base(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::DeleteKnowledgeBaseInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_add_source(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::AddSourceInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_source(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::GetSourceInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_sources(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::ListSourcesInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_delete_source(
        &self,
        context: &ToolContext,
        input: crate::tools::knowledge::DeleteSourceInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}

/// Trait for planner-related tools
pub trait PlannerToolsHandlerTrait: Send + Sync {
    fn execute_create_plan(
        &self,
        context: &ToolContext,
        input: crate::tools::planner::CreatePlanInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_plan(
        &self,
        context: &ToolContext,
        input: crate::tools::planner::GetPlanInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_plans(
        &self,
        context: &ToolContext,
        input: crate::tools::planner::ListPlansInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_update_plan(
        &self,
        context: &ToolContext,
        input: crate::tools::planner::UpdatePlanInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_delete_plan(
        &self,
        context: &ToolContext,
        input: crate::tools::planner::DeletePlanInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_execute_plan(
        &self,
        context: &ToolContext,
        input: crate::tools::planner::ExecutePlanInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_plan_status(
        &self,
        context: &ToolContext,
        input: crate::tools::planner::GetPlanStatusInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}

/// Trait for workflow-related tools
pub trait WorkflowToolsHandlerTrait: Send + Sync {
    fn execute_create_workflow(
        &self,
        context: &ToolContext,
        input: crate::tools::workflow::CreateWorkflowInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_workflow(
        &self,
        context: &ToolContext,
        input: crate::tools::workflow::GetWorkflowInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_workflows(
        &self,
        context: &ToolContext,
        input: crate::tools::workflow::ListWorkflowsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_execute_workflow(
        &self,
        context: &ToolContext,
        input: crate::tools::workflow::ExecuteWorkflowInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_pause_workflow(
        &self,
        context: &ToolContext,
        input: crate::tools::workflow::PauseWorkflowInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_resume_workflow(
        &self,
        context: &ToolContext,
        input: crate::tools::workflow::ResumeWorkflowInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_cancel_workflow(
        &self,
        context: &ToolContext,
        input: crate::tools::workflow::CancelWorkflowInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_workflow_status(
        &self,
        context: &ToolContext,
        input: crate::tools::workflow::GetWorkflowStatusInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_workflow_executions(
        &self,
        context: &ToolContext,
        input: crate::tools::workflow::ListWorkflowExecutionsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_workflow_execution(
        &self,
        context: &ToolContext,
        input: crate::tools::workflow::GetWorkflowExecutionInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}

/// Trait for exploration-related tools
pub trait ExplorationToolsHandlerTrait: Send + Sync {
    fn execute_start_exploration(
        &self,
        context: &ToolContext,
        input: crate::tools::exploration::StartExplorationInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_exploration(
        &self,
        context: &ToolContext,
        input: crate::tools::exploration::GetExplorationInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_explorations(
        &self,
        context: &ToolContext,
        input: crate::tools::exploration::ListExplorationsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_pause_exploration(
        &self,
        context: &ToolContext,
        input: crate::tools::exploration::PauseExplorationInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_resume_exploration(
        &self,
        context: &ToolContext,
        input: crate::tools::exploration::ResumeExplorationInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_complete_exploration(
        &self,
        context: &ToolContext,
        input: crate::tools::exploration::CompleteExplorationInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_cancel_exploration(
        &self,
        context: &ToolContext,
        input: crate::tools::exploration::CancelExplorationInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_exploration_status(
        &self,
        context: &ToolContext,
        input: crate::tools::exploration::GetExplorationStatusInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_record_exploration_finding(
        &self,
        context: &ToolContext,
        input: crate::tools::exploration::RecordExplorationFindingInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_exploration_findings(
        &self,
        context: &ToolContext,
        input: crate::tools::exploration::GetExplorationFindingsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_exploration_findings(
        &self,
        context: &ToolContext,
        input: crate::tools::exploration::ListExplorationFindingsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}

/// Trait for skills-related tools
pub trait SkillsToolsHandlerTrait: Send + Sync {
    fn execute_install_skill(
        &self,
        context: &ToolContext,
        input: crate::tools::skills::InstallSkillInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_uninstall_skill(
        &self,
        context: &ToolContext,
        input: crate::tools::skills::UninstallSkillInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_list_skills(
        &self,
        context: &ToolContext,
        input: crate::tools::skills::ListSkillsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_skill(
        &self,
        context: &ToolContext,
        input: crate::tools::skills::GetSkillInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_update_skill(
        &self,
        context: &ToolContext,
        input: crate::tools::skills::UpdateSkillInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_enable_skill(
        &self,
        context: &ToolContext,
        input: crate::tools::skills::EnableSkillInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_disable_skill(
        &self,
        context: &ToolContext,
        input: crate::tools::skills::DisableSkillInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_skill_metrics(
        &self,
        context: &ToolContext,
        input: crate::tools::skills::GetSkillMetricsInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_get_skill_status(
        &self,
        context: &ToolContext,
        input: crate::tools::skills::GetSkillStatusInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_activate_skill(
        &self,
        context: &ToolContext,
        input: crate::tools::skills::ActivateSkillInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn execute_deactivate_skill(
        &self,
        context: &ToolContext,
        input: crate::tools::skills::DeactivateSkillInput,
    ) -> impl std::future::Future<Output = ToolOutput> + Send;

    fn list_tools(&self) -> Vec<rmcp::tool::Tool>;
}
