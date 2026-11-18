// src/api/bridge/tools/knowledge/definitions.rs
//! Knowledge tool definitions (constants and schemas)

/// Tool name constants
pub const ADD_KNOWLEDGE: &str = "add_knowledge";
pub const QUERY_KNOWLEDGE: &str = "query_knowledge";
pub const RECORD_KNOWLEDGE_APPLICATION: &str = "record_knowledge_application";
pub const GET_KNOWLEDGE_STATS: &str = "get_knowledge_stats";
pub const GET_MATURE_KNOWLEDGE: &str = "get_mature_knowledge";
pub const UPDATE_KNOWLEDGE: &str = "update_knowledge";
pub const DELETE_KNOWLEDGE: &str = "delete_knowledge";
pub const GET_RELATED_KNOWLEDGE: &str = "get_related_knowledge";
pub const VALIDATE_KNOWLEDGE_DEPENDENCIES: &str = "validate_knowledge_dependencies";
pub const BUMP_KNOWLEDGE_VERSION: &str = "bump_knowledge_version";
pub const SET_KNOWLEDGE_STATUS: &str = "set_knowledge_status";
pub const MANAGE_KNOWLEDGE_DEPENDENCY: &str = "manage_knowledge_dependency";
pub const ADD_KNOWLEDGE_RELATION: &str = "add_knowledge_relation";
pub const SEARCH_KNOWLEDGE_BY_TAG: &str = "search_knowledge_by_tag";

pub fn all() -> Vec<crate::api::mcp::McpTool> {
    macro_rules! desc {
        ($s:expr) => {
            format!("[WORKFLOW: get_workflow + search_memory first] {}", $s)
        };
    }
    vec![
        crate::api::mcp::McpTool {
            name: ADD_KNOWLEDGE.to_string(),
            description: desc!("Add new validated knowledge to the knowledge base"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "statement": {
                        "type": "string",
                        "description": "The knowledge statement to add"
                    },
                    "knowledge_type": {
                        "type": "string",
                        "description": "Type: fact, procedure, causality, pattern, insight, rule, concept",
                        "enum": ["fact", "procedure", "causality", "pattern", "insight", "rule", "concept"]
                    },
                    "confidence": {
                        "type": "number",
                        "description": "Initial confidence (0.0 - 1.0)"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Tags for categorization"
                    },
                    "source": {
                        "type": "string",
                        "description": "Source: user, tool, planner, reflection, hypothesis, exploration, external"
                    }
                },
                "required": ["statement"]
            }),
        },
        crate::api::mcp::McpTool {
            name: QUERY_KNOWLEDGE.to_string(),
            description: desc!("Query the knowledge base for relevant knowledge"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "knowledge_type": {
                        "type": "string",
                        "description": "Filter by type"
                    },
                    "min_confidence": {
                        "type": "number",
                        "description": "Minimum confidence threshold"
                    },
                    "mature_only": {
                        "type": "boolean",
                        "description": "Only return mature (high confidence) knowledge"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum results to return"
                    }
                },
                "required": ["query"]
            }),
        },
        crate::api::mcp::McpTool {
            name: RECORD_KNOWLEDGE_APPLICATION.to_string(),
            description: desc!("Record the result of applying knowledge"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_id": {
                        "type": "string",
                        "description": "ID of the knowledge that was applied"
                    },
                    "success": {
                        "type": "boolean",
                        "description": "Whether the application was successful"
                    }
                },
                "required": ["knowledge_id", "success"]
            }),
        },
        crate::api::mcp::McpTool {
            name: GET_KNOWLEDGE_STATS.to_string(),
            description: desc!("Get statistics about the knowledge base"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        crate::api::mcp::McpTool {
            name: GET_MATURE_KNOWLEDGE.to_string(),
            description: desc!("Get all mature (high-confidence) knowledge"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "number",
                        "description": "Maximum results to return"
                    }
                }
            }),
        },
        crate::api::mcp::McpTool {
            name: UPDATE_KNOWLEDGE.to_string(),
            description: desc!(
                "Update an existing knowledge item (statement, confidence, or tags)"
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_id": { "type": "string", "description": "ID of the knowledge item to update" },
                    "statement": { "type": "string", "description": "Updated statement" },
                    "confidence": { "type": "number", "description": "Updated confidence (0.0-1.0)" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "Replaces existing tags" }
                },
                "required": ["knowledge_id"]
            }),
        },
        crate::api::mcp::McpTool {
            name: DELETE_KNOWLEDGE.to_string(),
            description: desc!("Delete a knowledge item by ID"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_id": { "type": "string", "description": "ID of the knowledge item to delete" }
                },
                "required": ["knowledge_id"]
            }),
        },
        crate::api::mcp::McpTool {
            name: GET_RELATED_KNOWLEDGE.to_string(),
            description: desc!("Get knowledge items related to a given item via its relations"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_id": { "type": "string", "description": "ID of the knowledge item to find relations for" }
                },
                "required": ["knowledge_id"]
            }),
        },
        crate::api::mcp::McpTool {
            name: VALIDATE_KNOWLEDGE_DEPENDENCIES.to_string(),
            description: desc!(
                "Validate all knowledge dependencies and return items with unsatisfied/conflicting deps"
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        crate::api::mcp::McpTool {
            name: BUMP_KNOWLEDGE_VERSION.to_string(),
            description: desc!("Bump the version of a knowledge item (major, minor, or patch)"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_id": { "type": "string", "description": "ID of the knowledge item to bump" },
                    "bump_type": { "type": "string", "enum": ["major", "minor", "patch"], "description": "Version bump type (default: minor)" },
                    "initial_version": { "type": "string", "description": "Initial version if version tracking not yet initialized (e.g. 1.0.0)" }
                },
                "required": ["knowledge_id"]
            }),
        },
        crate::api::mcp::McpTool {
            name: SET_KNOWLEDGE_STATUS.to_string(),
            description: desc!("Set knowledge status: activate, suspend, or disprove"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_id": { "type": "string", "description": "ID of the knowledge item" },
                    "action": { "type": "string", "enum": ["activate", "suspend", "disprove"], "description": "Status action" }
                },
                "required": ["knowledge_id", "action"]
            }),
        },
        crate::api::mcp::McpTool {
            name: MANAGE_KNOWLEDGE_DEPENDENCY.to_string(),
            description: desc!(
                "Manage knowledge dependencies: add, remove, get dependencies, or get impact set"
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_id": { "type": "string", "description": "ID of the knowledge item" },
                    "action": { "type": "string", "enum": ["add", "remove", "get", "impact"], "description": "Dependency action" },
                    "depends_on_id": { "type": "string", "description": "ID of the dependency target (for add/remove)" },
                    "dependency_type": { "type": "string", "enum": ["required", "optional", "conflict", "replaces"], "description": "Dependency type (for add)" }
                },
                "required": ["knowledge_id", "action"]
            }),
        },
        crate::api::mcp::McpTool {
            name: ADD_KNOWLEDGE_RELATION.to_string(),
            description: desc!("Add a relation between two knowledge items"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_id": { "type": "string", "description": "ID of the source knowledge item" },
                    "related_id": { "type": "string", "description": "ID of the target knowledge item" },
                    "relation_type": { "type": "string", "enum": ["related", "supports", "contradicts", "specializes", "generalizes", "prerequisite"], "description": "Relation type (default: related)" },
                    "confidence": { "type": "number", "description": "Confidence in the relation (0.0-1.0, default: 0.5)" }
                },
                "required": ["knowledge_id", "related_id"]
            }),
        },
        crate::api::mcp::McpTool {
            name: SEARCH_KNOWLEDGE_BY_TAG.to_string(),
            description: desc!("Search knowledge by tag or get items needing review"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "tag": { "type": "string", "description": "Tag to search for (if omitted, returns items needing review)" }
                }
            }),
        },
    ]
}
