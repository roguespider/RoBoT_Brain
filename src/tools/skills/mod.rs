// src/tools/skills/mod.rs
//! Skills tool implementations
//! Per Architecture §15: Skills represent reusable capabilities discovered through experience

use crate::bridge::mcp::McpContext;
use crate::skills::registry::{ExecutionContext, Skill, SkillCategory, SkillMetadata, SkillSource};
use crate::tools::ToolOutput;
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// =============================================================================
// INPUT TYPES
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RegisterSkillInput {
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Option<Vec<String>>,
    pub examples: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiscoverSkillInput {
    pub name: String,
    pub description: String,
    pub category: String,
    pub source_experience_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetSkillInput {
    pub skill_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListSkillsInput {
    pub category: Option<String>,
    pub enabled_only: Option<bool>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateSkillMasteryInput {
    pub skill_id: String,
    pub success: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetSkillRecommendationsInput {
    pub limit: Option<u32>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteSkillInput {
    pub skill_id: String,
    pub task: Option<String>,
    pub parameters: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub time_limit_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetSkillStatsInput {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApplySkillDecayInput {
    pub decay_rate: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnableDisableSkillInput {
    pub skill_id: String,
    pub enable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchSkillsInput {
    pub query: String,
    pub category: Option<String>,
    pub min_mastery: Option<f32>,
}

/// Skill tool definitions
pub mod definitions {
    use crate::bridge::mcp::McpTool;

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

    pub fn all() -> Vec<McpTool> {
        vec![
            McpTool {
                name: REGISTER_SKILL.to_string(),
                description: "Register a new skill in the skill registry. Skills represent reusable capabilities.".to_string(),
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
                description: "Create a skill discovered from an experience. Per Architecture §15: Skills emerge from experience.".to_string(),
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
                description: "Get details of a specific skill including mastery level, usage statistics, and prerequisites.".to_string(),
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
                description: "List all registered skills, optionally filtered by category or enabled status.".to_string(),
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
                description: "Update skill mastery based on execution outcome. Records success or failure for the skill.".to_string(),
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
                description: "Get skill recommendations based on readiness and usage patterns. Per Architecture §15.".to_string(),
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
                description: "Execute a skill with provided task and parameters. Per Architecture §15: Skill::execute(&context).".to_string(),
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
                description: "Get comprehensive statistics about the skill registry including mastery distribution.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            McpTool {
                name: APPLY_SKILL_DECAY.to_string(),
                description: "Apply mastery decay to unused skills. Per Architecture §15: Skills include decay.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "decay_rate": { "type": "number", "description": "Decay rate (default: 0.05)" }
                    }
                }),
            },
            McpTool {
                name: ENABLE_DISABLE_SKILL.to_string(),
                description: "Enable or disable a skill. Disabled skills cannot be executed.".to_string(),
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
                description: "Search skills by query, category, or minimum mastery level.".to_string(),
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
        ]
    }
}

/// Execute register_skill tool
pub async fn execute_register_skill(
    input: RegisterSkillInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let skill = Skill::new(SkillMetadata {
        name: input.name,
        description: input.description,
        category: parse_category(&input.category)?,
        version: input.version.unwrap_or_else(|| "1.0.0".to_string()),
        author: input.author,
        tags: input.tags.unwrap_or_default(),
        examples: input.examples.unwrap_or_default(),
    });

    let skill_id = context
        .skills
        .register(skill)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(ToolOutput::success(serde_json::json!({
        "status": "registered",
        "skill_id": skill_id,
        "message": "Skill registered successfully"
    })))
}

/// Execute discover_skill tool - create skill from experience
pub async fn execute_discover_skill(
    input: DiscoverSkillInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    use uuid::Uuid;

    let experience_id = input
        .source_experience_id
        .parse::<Uuid>()
        .map_err(|e| anyhow::anyhow!("Invalid experience ID: {}", e))?;

    let skill = Skill::discovered(
        input.name,
        input.description,
        parse_category(&input.category)?,
        experience_id,
    );

    let skill_id = context
        .skills
        .register(skill)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(ToolOutput::success(serde_json::json!({
        "status": "discovered",
        "skill_id": skill_id,
        "message": "Skill discovered from experience",
        "mastery": 0.3,
        "note": "Skill starts with low mastery until proven through practice"
    })))
}

/// Execute get_skill tool
pub async fn execute_get_skill(input: GetSkillInput, context: &McpContext) -> Result<ToolOutput> {
    let skill = context
        .skills
        .get(&input.skill_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", input.skill_id))?;

    Ok(ToolOutput::success(serde_json::json!({
        "skill": {
            "id": skill.id,
            "name": skill.metadata.name,
            "description": skill.metadata.description,
            "category": skill.metadata.category.as_str(),
            "version": skill.metadata.version,
            "enabled": skill.enabled,
            "mastery": skill.mastery,
            "usage_count": skill.usage_count,
            "success_count": skill.success_count,
            "failure_count": skill.failure_count,
            "success_rate": skill.success_rate(),
            "execution_score": skill.execution_score(),
            "last_used": skill.last_used.map(|dt| dt.to_rfc3339()),
            "prerequisites": skill.prerequisites,
            "source": match &skill.source {
                SkillSource::Manual => "manual",
                SkillSource::Discovered { .. } => "discovered",
                SkillSource::Learned { .. } => "learned",
            }
        }
    })))
}

/// Execute list_skills tool
pub async fn execute_list_skills(
    input: ListSkillsInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let skills = if let Some(category) = &input.category {
        context
            .skills
            .list_by_category(parse_category(category)?)
            .await
    } else if input.enabled_only.unwrap_or(false) {
        context.skills.list_enabled().await
    } else {
        context.skills.list().await
    };

    let skill_list: Vec<serde_json::Value> = skills
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.metadata.name,
                "description": s.metadata.description,
                "category": s.metadata.category.as_str(),
                "enabled": s.enabled,
                "mastery": s.mastery,
                "usage_count": s.usage_count,
                "success_rate": s.success_rate(),
            })
        })
        .collect();

    Ok(ToolOutput::success(serde_json::json!({
        "skills": skill_list,
        "count": skill_list.len()
    })))
}

/// Execute update_skill_mastery tool
pub async fn execute_update_skill_mastery(
    input: UpdateSkillMasteryInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let success = input.success.unwrap_or(true);
    context
        .skills
        .record_usage(&input.skill_id, success)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let skill = context
        .skills
        .get(&input.skill_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("Skill not found"))?;

    Ok(ToolOutput::success(serde_json::json!({
        "status": "updated",
        "skill_id": input.skill_id,
        "mastery": skill.mastery,
        "success_rate": skill.success_rate(),
        "usage_count": skill.usage_count
    })))
}

/// Execute get_skill_recommendations tool
pub async fn execute_get_skill_recommendations(
    input: GetSkillRecommendationsInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(5) as usize;
    let ready_skills = context.skills.get_ready_skills().await;
    let most_used = context.skills.get_most_used(limit).await;

    // Combine and deduplicate
    let mut recommendations = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for skill in ready_skills.iter().take(limit) {
        if seen.insert(&skill.id) {
            recommendations.push(serde_json::json!({
                "id": skill.id,
                "name": skill.metadata.name,
                "description": skill.metadata.description,
                "category": skill.metadata.category.as_str(),
                "mastery": skill.mastery,
                "reason": "ready_to_use"
            }));
        }
    }

    for skill in most_used.iter() {
        if seen.insert(&skill.id) && recommendations.len() < limit {
            recommendations.push(serde_json::json!({
                "id": skill.id,
                "name": skill.metadata.name,
                "description": skill.metadata.description,
                "category": skill.metadata.category.as_str(),
                "usage_count": skill.usage_count,
                "reason": "frequently_used"
            }));
        }
    }

    Ok(ToolOutput::success(serde_json::json!({
        "recommendations": recommendations,
        "count": recommendations.len()
    })))
}

/// Execute execute_skill tool
pub async fn execute_execute_skill(
    input: ExecuteSkillInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let skill = context
        .skills
        .get(&input.skill_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", input.skill_id))?;

    if !skill.enabled {
        anyhow::bail!("Skill is disabled");
    }

    if !skill.is_ready() {
        anyhow::bail!("Skill is not ready (mastery too low: {})", skill.mastery);
    }

    // Check prerequisites
    if !skill.prerequisites_met(&[]) {
        anyhow::bail!("Prerequisites not met for skill: {}", skill.metadata.name);
    }

    // Create execution context
    let exec_context = ExecutionContext {
        task: input
            .task
            .unwrap_or_else(|| skill.metadata.description.clone()),
        parameters: input.parameters.unwrap_or_default(),
        working_memory: std::collections::HashMap::new(),
        knowledge_context: Vec::new(),
        time_limit_secs: input.time_limit_secs,
    };

    // Execute skill based on category (simulated for now - real execution would be external)
    let result = match skill.metadata.category {
        SkillCategory::FileOperation => {
            serde_json::json!({
                "status": "executed",
                "category": "file_operation",
                "task": exec_context.task,
                "output": "Simulated file operation execution"
            })
        }
        SkillCategory::Search => {
            serde_json::json!({
                "status": "executed",
                "category": "search",
                "task": exec_context.task,
                "output": "Simulated search execution"
            })
        }
        SkillCategory::CodeAnalysis => {
            serde_json::json!({
                "status": "executed",
                "category": "code_analysis",
                "task": exec_context.task,
                "output": "Simulated code analysis execution"
            })
        }
        _ => {
            serde_json::json!({
                "status": "executed",
                "category": skill.metadata.category.as_str(),
                "task": exec_context.task,
                "output": "Skill execution simulated"
            })
        }
    };

    // Record successful execution
    context
        .skills
        .record_usage(&input.skill_id, true)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let updated_skill = context.skills.get(&input.skill_id).await.unwrap();

    Ok(ToolOutput::success(serde_json::json!({
        "result": result,
        "skill_id": input.skill_id,
        "mastery_after": updated_skill.mastery,
        "execution_score": updated_skill.execution_score()
    })))
}

/// Execute get_skill_stats tool
pub async fn execute_get_skill_stats(
    input: GetSkillStatsInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let stats = context.skills.get_discovery_stats().await;
    let mastered = context.skills.get_mastered_skills(0.8).await;
    let most_successful = context.skills.get_most_successful(3).await;

    Ok(ToolOutput::success(serde_json::json!({
        "stats": {
            "total_skills": stats.total_skills,
            "manual_skills": stats.manual_skills,
            "discovered_skills": stats.discovered_skills,
            "learned_skills": stats.learned_skills,
            "avg_mastery": stats.avg_mastery,
            "mastered_skills": stats.mastered_skills,
            "highly_successful": most_successful.len(),
            "ready_to_use": context.skills.get_ready_skills().await.len()
        },
        "mastered_skills": mastered.iter().map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.metadata.name,
                "mastery": s.mastery,
                "execution_score": s.execution_score()
            })
        }).collect::<Vec<_>>()
    })))
}

/// Execute apply_skill_decay tool
pub async fn execute_apply_skill_decay(
    input: ApplySkillDecayInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let decay_rate = input.decay_rate.unwrap_or(0.05);
    let decayed_count = context.skills.apply_decay_all(decay_rate).await;

    Ok(ToolOutput::success(serde_json::json!({
        "status": "decay_applied",
        "skills_decayed": decayed_count,
        "decay_rate": decay_rate,
        "message": format!("Decay applied to {} skills", decayed_count)
    })))
}

/// Execute enable_disable_skill tool
pub async fn execute_enable_disable_skill(
    input: EnableDisableSkillInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    if input.enable.unwrap_or(false) {
        context
            .skills
            .enable(&input.skill_id)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(ToolOutput::success(serde_json::json!({
            "status": "enabled",
            "skill_id": input.skill_id
        })))
    } else {
        context
            .skills
            .disable(&input.skill_id)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(ToolOutput::success(serde_json::json!({
            "status": "disabled",
            "skill_id": input.skill_id
        })))
    }
}

/// Execute search_skills tool
pub async fn execute_search_skills(
    input: SearchSkillsInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let all_skills = context.skills.list().await;

    let results: Vec<_> = all_skills
        .iter()
        .filter(|s| {
            let matches_query = s
                .metadata
                .name
                .to_lowercase()
                .contains(&input.query.to_lowercase())
                || s.metadata
                    .description
                    .to_lowercase()
                    .contains(&input.query.to_lowercase());

            let matches_category = input
                .category
                .as_ref()
                .map(|c| s.metadata.category.as_str() == c)
                .unwrap_or(true);

            let matches_mastery = input.min_mastery.map(|m| s.mastery >= m).unwrap_or(true);

            matches_query && matches_category && matches_mastery
        })
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.metadata.name,
                "description": s.metadata.description,
                "category": s.metadata.category.as_str(),
                "mastery": s.mastery,
                "success_rate": s.success_rate()
            })
        })
        .collect();

    Ok(ToolOutput::success(serde_json::json!({
        "results": results,
        "count": results.len()
    })))
}

/// Parse skill category from string
fn parse_category(category: &str) -> Result<SkillCategory> {
    match category.to_lowercase().as_str() {
        "file_operation" | "fileoperation" | "file" => Ok(SkillCategory::FileOperation),
        "code_analysis" | "codeanalysis" | "code" => Ok(SkillCategory::CodeAnalysis),
        "search" => Ok(SkillCategory::Search),
        "memory" => Ok(SkillCategory::Memory),
        "learning" => Ok(SkillCategory::Learning),
        "planning" => Ok(SkillCategory::Planning),
        "communication" => Ok(SkillCategory::Communication),
        "web" => Ok(SkillCategory::Web),
        "database" | "db" => Ok(SkillCategory::Database),
        "system" => Ok(SkillCategory::System),
        "custom" => Ok(SkillCategory::Custom),
        _ => anyhow::bail!("Unknown skill category: {}", category),
    }
}
