//! Function Registry Module
//!
//! Defines all MCP tools and their test requirements.
//! This registry is the source of truth for what functions need to be tested.
//!
//! Tool Order follows the MCP Pipeline (ORDER MATTERS - tools are listed in the order they're meant to be used):
//! 1. Agent (ENTRY POINT - get_workflow, list_tools MUST be called first)
//! 2. Memory (foundation - search_memory, store_memory, etc.)
//! 3. Experience (tracks all operations)
//! 4. Reflection (analyzes experience - get_patterns, get_insights)
//! 5. Search (uses memory, experience - global_search)
//! 6. Knowledge (stores learned info)
//! 7. Planner (planning operations)
//! 8. Exploration & Hypothesis (hypothesis generation & evaluation)
//! 9. Skills (uses planner, exploration)
//! 10. Workflow (workflow management)
//! 11. Ingestor (file ingestion)
//! 12. BackgroundWorkers (async workers)

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
    /// 
    /// Order follows the MCP Pipeline from the General MCP Workflow:
    /// - Agent (ENTRY POINT) MUST be called first - get_workflow, list_tools
    /// - Memory (check memory before action)
    /// - Reflection (review patterns for repetitive decisions)
    pub fn get_all_functions() -> Vec<TestRequirement> {
        let mut functions = Vec::new();

        // Phase 1: ENTRY POINT - Agent tools (get_workflow, list_tools MUST be called first)
        functions.extend(agent_tools::agent_tools());          // get_workflow, list_tools, connect_mcp_server, call_tool, get_tool
        
        // Phase 2: Memory Foundation (per workflow: check memory before action)
        functions.extend(memory_tools::memory_tools());          // search_memory, store_memory, etc.
        functions.extend(vector_index_tools::vector_index_tools()); // store_embedding, search_similar, etc.
        
        // Phase 3: Experience Tracking (records all operations)
        functions.extend(experience_tools::experience_tools());   // record_experience, get_experience_stats, etc.
        
        // Phase 4: Reflection & Analysis (uses experience)
        functions.extend(reflection_tools::reflection_tools());  // get_insights, create_reflection, analyze_patterns, etc.
        
        // Phase 5: Search (uses memory, experience)
        functions.extend(search_tools::search_tools());          // global_search, get_recommendations, get_reputation
        
        // Phase 6: Knowledge Base (stores learned info)
        functions.extend(knowledge_tools::knowledge_tools());    // add_knowledge, query_knowledge, etc.
        
        // Phase 7: Planning (planning operations)
        functions.extend(planner_tools::planner_tools());        // create_plan, add_plan_step, etc.
        
        // Phase 8: Exploration & Learning
        functions.extend(exploration_tools::exploration_tools()); // record_observation, create_hypothesis, etc.
        functions.extend(hypothesis_tools::hypothesis_tools()); // evaluate_exploration_hypothesis, etc.
        
        // Phase 9: Skills (uses planner, exploration)
        functions.extend(skills_tools::skills_tools());          // register_skill, discover_skill, execute_skill, etc.
        
        // Phase 10: Workflow Management
        functions.extend(workflow_tools::workflow_tools());      // create_workflow, add_workflow_step, start_workflow, etc.
        
        // Phase 11: File Operations
        functions.extend(ingestor_tools::ingestor_tools());      // ingest_files, list_importable, etc.
        
        // Phase 12: Background Workers
        functions.extend(background_workers_tools::background_workers_tools()); // get_worker_stats, get_worker_count

        functions
    }
}
