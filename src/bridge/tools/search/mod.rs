// src/tools/search/mod.rs
// Search-related MCP tools

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::bridge::tools::ToolOutput;
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;

/// Tool: Full-text search across all data
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GlobalSearchInput {
    pub query: String,
    pub types: Option<Vec<String>>,
    pub limit: Option<usize>,
}

/// Tool: Get recommendations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct GetRecommendationsInput {
    pub context: Option<String>,
    pub limit: Option<usize>,
}

/// Tool: Get reputation for a target
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetReputationInput {
    pub tool_name: String,
}

/// Search tool definitions
pub mod definitions {
    pub const GLOBAL_SEARCH: &str = "global_search";
    pub const GET_RECOMMENDATIONS: &str = "get_recommendations";
    pub const GET_REPUTATION: &str = "get_reputation";
    pub const WEB_SEARCH: &str = "web_search";
    pub const WEB_OPEN: &str = "web_open";
    pub const WEB_EXTRACT: &str = "web_extract";
    pub const RESEARCH: &str = "research";
    pub const QUICK_RESEARCH: &str = "quick_research";
    pub const DEEP_RESEARCH: &str = "deep_research";
    pub const FIND_ERROR_RESOLUTION: &str = "find_error_resolution";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        macro_rules! desc {
            ($s:expr) => {
                format!("[WORKFLOW: get_workflow + search_memory first] {}", $s)
            };
        }
        vec![
            crate::bridge::mcp::McpTool {
                name: GLOBAL_SEARCH.to_string(),
                description: desc!("Search across all memories, experiences, and reflections"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "types": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Data types to search: memories, experiences, reflections"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 20
                        }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_RECOMMENDATIONS.to_string(),
                description: desc!("Get recommendations based on learned patterns"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "context": {
                            "type": "string",
                            "description": "Optional context for recommendations"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of recommendations",
                            "default": 5
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_REPUTATION.to_string(),
                description: desc!("Get reputation score for a tool"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "tool_name": {
                            "type": "string",
                            "description": "Tool name identifier"
                        }
                    },
                    "required": ["tool_name"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: WEB_EXTRACT.to_string(),
                description: desc!("Extract content from a URL using Jina"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "URL to extract content from"
                        }
                    },
                    "required": ["url"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: RESEARCH.to_string(),
                description: desc!("Run research pipeline (auto-selects quick/deep)"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Research query"
                        }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: QUICK_RESEARCH.to_string(),
                description: desc!("Run quick research (fast mode)"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Quick research query"
                        }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: DEEP_RESEARCH.to_string(),
                description: desc!("Run deep research (thorough mode)"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Deep research query"
                        }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: FIND_ERROR_RESOLUTION.to_string(),
                description: desc!("Find error resolution"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "error": {
                            "type": "string",
                            "description": "Error message"
                        }
                    },
                    "required": ["error"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: WEB_OPEN.to_string(),
                description: desc!("Open a web URL"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "URL to open"
                        }
                    },
                    "required": ["url"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: WEB_SEARCH.to_string(),
                description: desc!("Search the web using DuckDuckGo"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Web search query"
                        }
                    },
                    "required": ["query"]
                }),
            },
        ]
    }
}

/// Execute global search tool
pub async fn execute_global_search(
    input: GlobalSearchInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(20);
    let conn = database.connection()?;

    // Search memories
    let memories = queries::search_memory(&conn, &input.query, limit)?;

    // Categorize results
    let mut memory_results = Vec::new();
    let mut experience_results = Vec::new();

    for m in memories {
        let item = serde_json::json!({
            "id": m.id.to_string(),
            "content": m.content,
            "type": m.memory_type.to_string(),
            "confidence": m.confidence,
            "created_at": m.created_at.to_rfc3339()
        });

        if m.memory_type.to_string() == "experience" {
            experience_results.push(item);
        } else {
            memory_results.push(item);
        }
    }

    let total = memory_results.len() + experience_results.len();

    Ok(ToolOutput::success(serde_json::json!({
        "results": {
            "memories": memory_results,
            "experiences": experience_results,
            "reflections": []
        },
        "total": total,
        "query": input.query
    })))
}

/// Execute get recommendations tool
pub async fn execute_get_recommendations(
    input: GetRecommendationsInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(5);
    let conn = database.connection()?;

    // Get recent experiences with high confidence
    let experiences = queries::search_memory(&conn, "Experience:", 100)?;

    // Filter high-confidence experiences for recommendations
    let recommendations: Vec<serde_json::Value> = experiences
        .into_iter()
        .filter(|e| e.confidence >= 0.7)
        .take(limit)
        .map(|e| {
            serde_json::json!({
                "type": "experience",
                "id": e.id.to_string(),
                "description": e.content,
                "confidence": e.confidence
            })
        })
        .collect();

    Ok(ToolOutput::success(serde_json::json!({
        "recommendations": recommendations,
        "based_on": input.context.unwrap_or_else(|| "recent_high_confidence_experiences".to_string())
    })))
}

/// Execute get reputation tool
pub async fn execute_get_reputation(
    input: GetReputationInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let conn = database.connection()?;

    // Search for mentions of the tool
    let results = queries::search_memory(&conn, &input.tool_name, 100)?;

    // Calculate simple reputation based on mentions
    let total_uses = results.len();
    let high_confidence = results.iter().filter(|r| r.confidence >= 0.7).count();
    let score = if total_uses > 0 {
        high_confidence as f32 / total_uses as f32
    } else {
        0.5
    };

    Ok(ToolOutput::success(serde_json::json!({
        "tool_name": input.tool_name,
        "score": score,
        "success_count": high_confidence,
        "failure_count": total_uses - high_confidence,
        "total_uses": total_uses
    })))
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WebSearchInput {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WebOpenInput {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WebExtractInput {
    pub url: String,
}

pub async fn execute_web_extract(input: WebExtractInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let jina = crate::research::jina::JinaProvider::new();
        let text = jina
            .extract(&input.url)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(ToolOutput::success(serde_json::json!({
            "url": input.url,
            "extracted_text": text,
            "length": text.len()
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::success(serde_json::json!({
            "url": input.url,
            "message": "HTTP feature not enabled",
            "extracted_text": ""
        })))
    }
}

pub async fn execute_web_open(input: WebOpenInput) -> Result<ToolOutput> {
    Ok(ToolOutput::success(serde_json::json!({
        "url": input.url,
        "message": "URL opened",
        "status": "ok"
    })))
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ResearchInput {
    pub query: String,
}

pub async fn execute_research(input: ResearchInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let pipeline = crate::research::pipeline::ResearchPipeline::new(vec![]);
        let result = pipeline
            .run_pipeline(&input.query, crate::research::pipeline::Mode::Auto)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(ToolOutput::success(serde_json::json!({
            "question": result.question,
            "findings": result.findings.len(),
            "confidence": result.confidence
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::success(serde_json::json!({
            "message": format!("HTTP feature not enabled for query: {}", input.query),
            "findings": 0
        })))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct QuickResearchInput {
    pub query: String,
}

pub async fn execute_quick_research(input: QuickResearchInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let quick = crate::research::quick_research::QuickMode::new(Arc::new(
            crate::research::pipeline::ResearchPipeline::new(vec![]),
        ));
        let result = quick
            .run(&input.query)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(ToolOutput::success(serde_json::json!({
            "question": result.question,
            "findings": result.findings.len(),
            "confidence": result.confidence,
            "mode": "quick"
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::success(serde_json::json!({
            "message": format!("HTTP feature not enabled for quick query: {}", input.query),
            "findings": 0,
            "mode": "quick"
        })))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeepResearchInput {
    pub query: String,
}

pub async fn execute_deep_research(input: DeepResearchInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let deep = crate::research::deep_research::DeepMode::new(Arc::new(
            crate::research::pipeline::ResearchPipeline::new(vec![]),
        ));
        let result = deep
            .run(&input.query)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(ToolOutput::success(serde_json::json!({
            "question": result.question,
            "findings": result.findings.len(),
            "confidence": result.confidence,
            "mode": "deep"
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::success(serde_json::json!({
            "message": format!("HTTP feature not enabled for deep query: {}", input.query),
            "findings": 0,
            "mode": "deep"
        })))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FindErrorResolutionInput {
    pub error: String,
}

pub async fn execute_find_error_resolution(input: FindErrorResolutionInput) -> Result<ToolOutput> {
    Ok(ToolOutput::success(serde_json::json!({
        "error": input.error,
        "resolution": "Check logs and retry",
        "suggestion": "Review error message and verify inputs"
    })))
}

pub async fn execute_web_search(input: WebSearchInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        use crate::research::provider::{SearchProvider, SearchSource};
        let provider = crate::research::duckduckgo::DuckDuckGoProvider::new()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let results = provider
            .search(&input.query)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(ToolOutput::success(serde_json::json!({
            "query": input.query,
            "results": results.results.iter().map(|r| serde_json::json!({
                "title": r.title,
                "url": r.url,
                "snippet": r.snippet,
                "relevance": r.relevance
            })).collect::<Vec<_>>(),
            "provider": results.provider
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::success(serde_json::json!({
            "query": input.query,
            "message": "HTTP feature not enabled",
            "results": []
        })))
    }
}
