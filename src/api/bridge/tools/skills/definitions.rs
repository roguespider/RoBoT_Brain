//! Skill tool definitions.

use crate::api::mcp::McpTool;

pub const REGISTER_SKILL: &str = "register_skill";
pub const DISCOVER_SKILL: &str = "discover_skill";
pub const GET_SKILL: &str = "get_skill";
pub const LIST_SKILLS: &str = "list_skills";
pub const UPDATE_SKILL_MASTERY: &str = "update_skill_mastery";
pub const GET_SKILL_RECOMMENDATIONS: &str = "get_skill_recommendations";
pub const EXECUTE_SKILL: &str = "execute_skill";
pub const GET_SKILL_STATS: &str = "get_skill_stats";
pub const APPLY_SKILL_DECAY: &str = "apply_skill_decay";
pub const ENABLE_DISABLE_SKILL: &str = "enable_disable_skill";
pub const SEARCH_SKILLS: &str = "search_skills";
pub const GET_SKILL_METRICS: &str = "get_skill_metrics";
pub const GET_UNRELIABLE_SKILLS: &str = "get_unreliable_skills";
pub const CLEAR_SKILL_METRICS: &str = "clear_skill_metrics";
pub const SEARCH_SKILLS_BY_TAG: &str = "search_skills_by_tag";
pub const UNREGISTER_SKILL: &str = "unregister_skill";

pub fn all() -> Vec<McpTool> {
    macro_rules! desc {
        ($s:expr) => {
            format!("[WORKFLOW: get_workflow + search_memory first] {}", $s)
        };
    }
    vec![
        McpTool {
            name: REGISTER_SKILL.to_string(),
            description: desc!(
                "Register a new skill in the skill registry. Skills represent reusable capabilities."
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Unique name for the skill" },
                    "description": { "type": "string", "description": "Description of what the skill does" },
                    "category": {
                        "type": "string",
                        "description": "Category: file_operation, code_analysis, search, memory, learning, planning, communication, web, database, system, custom",
                        "enum": ["file_operation", "code_analysis", "search", "memory", "learning", "planning", "communication", "web", "database", "system", "custom"]
                    },
                    "version": { "type": "string", "description": "Skill version (default: 1.0.0)" },
                    "author": { "type": "string", "description": "Author of the skill" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "Tags for categorization" },
                    "examples": { "type": "array", "items": { "type": "string" }, "description": "Usage examples" }
                },
                "required": ["name", "description", "category"]
            }),
        },
        McpTool {
            name: DISCOVER_SKILL.to_string(),
            description: desc!(
                "Create a skill discovered from an experience. Per Architecture §15: Skills emerge from experience."
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Name of the discovered skill" },
                    "description": { "type": "string", "description": "Description of the skill" },
                    "category": { "type": "string", "description": "Skill category" },
                    "source_experience_id": { "type": "string", "description": "ID of the experience that led to this discovery" }
                },
                "required": ["name", "description", "category", "source_experience_id"]
            }),
        },
        McpTool {
            name: GET_SKILL.to_string(),
            description: desc!(
                "Get details of a specific skill including mastery level, usage statistics, and prerequisites."
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string", "description": "ID of the skill to retrieve" }
                },
                "required": ["skill_id"]
            }),
        },
        McpTool {
            name: LIST_SKILLS.to_string(),
            description: desc!(
                "List all registered skills, optionally filtered by category or enabled status."
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "category": { "type": "string", "description": "Filter by category" },
                    "enabled_only": { "type": "boolean", "description": "Only list enabled skills" },
                    "limit": { "type": "integer", "description": "Maximum number to return" }
                }
            }),
        },
        McpTool {
            name: UPDATE_SKILL_MASTERY.to_string(),
            description: desc!(
                "Update skill mastery based on execution outcome. Records success or failure for the skill."
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string", "description": "ID of the skill" },
                    "success": { "type": "boolean", "description": "Whether the skill execution was successful (default: true)" }
                },
                "required": ["skill_id"]
            }),
        },
        McpTool {
            name: GET_SKILL_RECOMMENDATIONS.to_string(),
            description: desc!(
                "Get skill recommendations based on readiness and usage patterns. Per Architecture §15."
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "description": "Maximum recommendations to return (default: 5)" },
                    "category": { "type": "string", "description": "Filter by category" }
                }
            }),
        },
        McpTool {
            name: EXECUTE_SKILL.to_string(),
            description: desc!(
                "Execute a skill with provided task and parameters. Per Architecture §15: Skill::execute(&context)."
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string", "description": "ID of the skill to execute" },
                    "task": { "type": "string", "description": "Task description for the skill" },
                    "parameters": { "type": "object", "description": "Key-value parameters for execution" },
                    "time_limit_secs": { "type": "integer", "description": "Maximum execution time in seconds" }
                },
                "required": ["skill_id"]
            }),
        },
        McpTool {
            name: GET_SKILL_STATS.to_string(),
            description: desc!(
                "Get comprehensive statistics about the skill registry including mastery distribution."
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        McpTool {
            name: APPLY_SKILL_DECAY.to_string(),
            description: desc!(
                "Apply mastery decay to unused skills. Per Architecture §15: Skills include decay."
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "decay_rate": { "type": "number", "description": "Decay rate (default: 0.05)" }
                }
            }),
        },
        McpTool {
            name: ENABLE_DISABLE_SKILL.to_string(),
            description: desc!("Enable or disable a skill. Disabled skills cannot be executed."),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string", "description": "ID of the skill" },
                    "enable": { "type": "boolean", "description": "true to enable, false to disable" }
                },
                "required": ["skill_id"]
            }),
        },
        McpTool {
            name: SEARCH_SKILLS.to_string(),
            description: desc!("Search skills by query, category, or minimum mastery level."),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "category": { "type": "string", "description": "Filter by category" },
                    "min_mastery": { "type": "number", "description": "Minimum mastery level" }
                },
                "required": ["query"]
            }),
        },
        McpTool {
            name: GET_SKILL_METRICS.to_string(),
            description: desc!(
                "Get execution metrics for a specific skill or all skills. Returns success rate, avg duration, stability, and reliability."
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string", "description": "Optional skill ID to get metrics for. If omitted, returns all skill metrics." }
                }
            }),
        },
        McpTool {
            name: GET_UNRELIABLE_SKILLS.to_string(),
            description: desc!(
                "List skills that have been marked as unreliable based on execution metrics."
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        McpTool {
            name: CLEAR_SKILL_METRICS.to_string(),
            description: desc!("Clear execution metrics for a specific skill or all skills."),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string", "description": "Optional skill ID. If omitted, clears all metrics." }
                }
            }),
        },
        McpTool {
            name: SEARCH_SKILLS_BY_TAG.to_string(),
            description: desc!("Search skills by tag. Returns all skills matching the given tag."),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "tag": { "type": "string", "description": "Tag to search for" }
                },
                "required": ["tag"]
            }),
        },
        McpTool {
            name: UNREGISTER_SKILL.to_string(),
            description: desc!("Unregister a skill from the registry by its ID."),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string", "description": "ID of the skill to unregister" }
                },
                "required": ["skill_id"]
            }),
        },
    ]
}
