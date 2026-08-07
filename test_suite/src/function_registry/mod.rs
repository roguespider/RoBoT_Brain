//! Function Registry Module
//!
//! Defines all MCP tools and their test requirements.
//! This registry is the source of truth for what functions need to be tested.

pub mod agent_tools;
pub mod background_workers_tools;
pub mod experience_tools;
pub mod exploration_tools;
pub mod hypothesis_tools;
pub mod ingestor_tools;
pub mod knowledge_tools;
pub mod memory_tools;
pub mod planner_tools;
pub mod reflection_tools;
pub mod search_tools;
pub mod skills_tools;
pub mod types;
pub mod vector_index_tools;
pub mod workflow_tools;

pub use types::{
    CheckType, DataRequirement, TestRequirement, ValidationCheck,
};

/// All registered functions that need testing
pub struct FunctionRegistry;

impl FunctionRegistry {
    /// Get all functions that need to be tested
    pub fn get_all_functions() -> Vec<TestRequirement> {
        let mut functions = Vec::new();

        functions.extend(agent_tools::agent_tools());
        functions.extend(memory_tools::memory_tools());
        functions.extend(vector_index_tools::vector_index_tools());
        functions.extend(experience_tools::experience_tools());
        functions.extend(background_workers_tools::background_workers_tools());
        functions.extend(reflection_tools::reflection_tools());
        functions.extend(search_tools::search_tools());
        functions.extend(ingestor_tools::ingestor_tools());
        functions.extend(hypothesis_tools::hypothesis_tools());
        functions.extend(exploration_tools::exploration_tools());
        functions.extend(knowledge_tools::knowledge_tools());
        functions.extend(planner_tools::planner_tools());
        functions.extend(workflow_tools::workflow_tools());
        functions.extend(skills_tools::skills_tools());

        functions
    }
}
