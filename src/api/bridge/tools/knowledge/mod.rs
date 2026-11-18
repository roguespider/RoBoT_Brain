// src/api/bridge/tools/knowledge/mod.rs
//! Knowledge system MCP tools - directory module

pub mod definitions;
pub mod handler;
pub mod types;

// Re-export input types
pub use types::{
    AddKnowledgeInput, AddKnowledgeRelationInput, BumpKnowledgeVersionInput, DeleteKnowledgeInput,
    GetKnowledgeStatsInput, GetMatureKnowledgeInput, GetRelatedKnowledgeInput,
    ManageKnowledgeDependencyInput, QueryKnowledgeInput, RecordKnowledgeApplicationInput,
    SearchKnowledgeByTagInput, SetKnowledgeStatusInput, UpdateKnowledgeInput,
    ValidateKnowledgeDependenciesInput,
};

// Re-export tool definitions
pub use definitions::{
    ADD_KNOWLEDGE, ADD_KNOWLEDGE_RELATION, BUMP_KNOWLEDGE_VERSION, DELETE_KNOWLEDGE,
    GET_KNOWLEDGE_STATS, GET_MATURE_KNOWLEDGE, GET_RELATED_KNOWLEDGE, MANAGE_KNOWLEDGE_DEPENDENCY,
    QUERY_KNOWLEDGE, RECORD_KNOWLEDGE_APPLICATION, SEARCH_KNOWLEDGE_BY_TAG, SET_KNOWLEDGE_STATUS,
    UPDATE_KNOWLEDGE, VALIDATE_KNOWLEDGE_DEPENDENCIES, all,
};

// Re-export handler functions
pub use handler::{
    execute_add_knowledge, execute_add_knowledge_relation, execute_bump_knowledge_version,
    execute_delete_knowledge, execute_get_knowledge_stats, execute_get_mature_knowledge,
    execute_get_related_knowledge, execute_manage_knowledge_dependency, execute_query_knowledge,
    execute_record_knowledge_application, execute_search_knowledge_by_tag,
    execute_set_knowledge_status, execute_update_knowledge,
    execute_validate_knowledge_dependencies,
};
