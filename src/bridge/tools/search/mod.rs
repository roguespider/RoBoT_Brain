// src/tools/search/mod.rs
// Search-related MCP tools

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::bridge::tools::ToolOutput;
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;

/// Build the structured error returned when the `http` cargo feature is off.
///
/// research_engine.md Rule 6 requires provider failures to surface as
/// structured errors. Returning `success: true` with an empty payload would be
/// strictly worse: the calling LLM reads a hollow success as a real answer and
/// hallucinates from the absence of data. Every http-gated tool therefore
/// reports `success: false` with this message instead.
///
/// `subject` is the query/URL/channel that was requested. It is echoed back so
/// the caller can tell which request was refused, and so the parameter is used
/// meaningfully on the no-http build rather than being dropped.
#[cfg(not(feature = "http"))]
fn http_feature_unavailable_for(tool: &str, subject: &str) -> String {
    let requested = match subject.is_empty() {
        true => String::new(),
        false => format!(" Request was: {subject}."),
    };
    format!(
        "Research tool '{tool}' is unavailable.{requested} This build of robot_brain was \
         compiled without the 'http' cargo feature, so no external research provider is \
         loaded. Rebuild with the default features (which include 'http') to enable it. \
         No evidence was retrieved -- do not treat this as an answer."
    )
}

/// Variant for per-source adapters, which do not echo the request subject.
#[cfg(not(feature = "http"))]
fn http_feature_unavailable(tool: &str) -> String {
    http_feature_unavailable_for(tool, "")
}

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
            // Per-source adapter tools (Architecture §R11 / adapter registry)
            crate::bridge::mcp::McpTool {
                name: "get_wikipedia".to_string(),
                description: desc!("Search Wikipedia for encyclopedic facts"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Wikipedia search query" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_news".to_string(),
                description: desc!("Search current news sources"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "News search query" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_reddit".to_string(),
                description: desc!("Search Reddit discussions"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Reddit search query" },
                        "subreddit": { "type": "string", "description": "Optional subreddit filter" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_hackernews".to_string(),
                description: desc!("Search Hacker News tech community"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "HN search query" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_youtube".to_string(),
                description: desc!("Search YouTube videos (optional transcript)"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "YouTube search query" },
                        "include_transcript": { "type": "boolean", "description": "Include video transcript" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_substack".to_string(),
                description: desc!("Search Substack newsletter publications"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "publications": { "type": "string", "description": "Publication names" },
                        "max_posts": { "type": "number", "description": "Maximum posts to return" }
                    },
                    "required": ["publications"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_bluesky".to_string(),
                description: desc!("Search Bluesky social posts"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Bluesky search query" },
                        "sort": { "type": "string", "description": "Sort order (top/recent)" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_telegram".to_string(),
                description: desc!("Search Telegram public channel messages"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "channel": { "type": "string", "description": "Channel name" },
                        "max_messages": { "type": "number", "description": "Maximum messages" }
                    },
                    "required": ["channel"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_mastodon".to_string(),
                description: desc!("Search Mastodon fediverse posts"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Mastodon search query" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_vk".to_string(),
                description: desc!("Search VK social network"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "VK search query" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_preprints".to_string(),
                description: desc!("Search arXiv/bioRxiv/medRxiv preprints"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Preprint search query" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_datasets".to_string(),
                description: desc!("Search Zenodo/Figshare/OSF datasets"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Dataset search query" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_osm".to_string(),
                description: desc!("Search OpenStreetMap geographic data"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "OSM search query" },
                        "location": { "type": "string", "description": "Location filter" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "search_sec_filings".to_string(),
                description: desc!("Search SEC EDGAR financial filings"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "SEC filing search query" },
                        "filing_type": { "type": "string", "description": "Filing type (10-K, 10-Q, etc.)" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "resurrect_dead_link".to_string(),
                description: desc!("Recover broken links via Wayback Machine"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "Dead URL to resurrect" }
                    },
                    "required": ["url"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "verify_citations".to_string(),
                description: desc!("Verify citations via Crossref/OpenAlex"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "references": { "type": "array", "description": "Reference list with DOI" }
                    },
                    "required": ["references"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "find_counter_arguments".to_string(),
                description: desc!("Find opposing academic arguments"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Topic to find counter-arguments for" }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "validate_bibliography".to_string(),
                description: desc!("Validate an entire reference list"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "bibliography": { "type": "array", "description": "Bibliography entries" }
                    },
                    "required": ["bibliography"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "format_citations".to_string(),
                description: desc!("Format citations (BibTeX/APA/MLA/Chicago/RIS)"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "doi": { "type": "string", "description": "DOI to format" },
                        "format": { "type": "string", "description": "Output format" }
                    },
                    "required": ["doi", "format"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "detect_trends".to_string(),
                description: desc!("Detect trending topics across platforms"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "platforms": { "type": "array", "items": {"type":"string"}, "description": "Platforms to scan" }
                    },
                    "required": ["platforms"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: "score_reliability".to_string(),
                description: desc!("Score source reliability (rule-based)"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "urls": { "type": "array", "items": {"type":"string"}, "description": "URLs to score" }
                    },
                    "required": ["urls"]
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
        if !jina.is_configured() {
            // Report the missing credential rather than an opaque failure, so
            // the operator can see this is a configuration gap (Rule 6).
            return Ok(ToolOutput::error(format!(
                "web_extract is not configured: set JINA_API_KEY to enable hosted content extraction. Request was: {}",
                input.url
            )));
        }
        // The extraction deadline is enforced here so a slow page cannot hold
        // the tool open past its budget (R9 Rule 5).
        let extraction = jina.extract(&input.url);
        let text =
            crate::research::errors::with_timeout(extraction, std::time::Duration::from_secs(30))
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))??;
        // Extracted page text is untrusted web data and is framed as such
        // before it crosses the MCP boundary (Rule 9).
        let (content, truncated) = crate::research::sanitize::truncate_with_marker(
            &crate::research::sanitize::sanitize_text(&text),
            8000,
        );
        Ok(ToolOutput::success(serde_json::json!({
            "url": input.url,
            "provider": "jina",
            "extracted_text": crate::research::sanitize::as_quoted_data("PAGE", &content),
            "length": content.chars().count(),
            "truncated": truncated,
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(http_feature_unavailable_for(
            "web_extract",
            &input.url,
        )))
    }
}

pub async fn execute_web_open(input: WebOpenInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        // The page is fetched and reduced to readable text. Raw HTML is never
        // returned across the MCP boundary (Rule 9), and the content is framed
        // as untrusted data so it cannot be mistaken for an instruction.
        let body = crate::research::http::get_text("web_open", &input.url).await?;
        let (content, truncated) = crate::research::sanitize::truncate_with_marker(
            &crate::research::sanitize::sanitize_text(&body),
            4000,
        );
        if content.trim().is_empty() {
            return Ok(ToolOutput::error(format!(
                "No readable text was extracted from {}. The page may be empty, script-rendered, or blocked automated access.",
                input.url
            )));
        }
        Ok(ToolOutput::success(serde_json::json!({
            "url": input.url,
            "provider": "web_open",
            "content": crate::research::sanitize::as_quoted_data("PAGE", &content),
            "content_length": content.chars().count(),
            "truncated": truncated,
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(http_feature_unavailable_for(
            "web_open", &input.url,
        )))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ResearchInput {
    pub query: String,
}

/// Render a completed research result as the MCP payload for a research tool.
///
/// The payload is the bounded evidence packet the LLM reasons over: findings,
/// sources with full provenance, contradictions and limitations. Raw provider
/// HTML never crosses this boundary (Rule 2, Rule 9). The accompanying summary
/// is explicitly marked as third-party data so the reasoning step has an
/// unambiguous trust boundary.
/// Build a research pipeline wired to the memory repository.
///
/// Validated findings must be durable, not merely constructed (R12), so every
/// entry point that can run research builds its pipeline through this helper
/// rather than constructing one directly.
#[cfg(feature = "http")]
fn pipeline_with_memory(
    database: &Arc<SqliteDatabase>,
) -> crate::research::pipeline::ResearchPipeline {
    use crate::memory::repository::SqliteMemoryRepository;
    let providers = crate::research::adapter_registry::all_providers();
    let repository = Arc::new(SqliteMemoryRepository::new((**database).clone()));
    crate::research::pipeline::ResearchPipeline::new(providers).with_memory_repository(repository)
}

#[cfg(feature = "http")]
fn research_payload(result: &crate::research::ResearchResult, mode: &str) -> ToolOutput {
    let packet = result.to_evidence_packet();
    // The packet is serialized once to confirm it round-trips across the MCP
    // boundary (R8). A result that cannot be serialized is reported as an error
    // rather than returned as a partially-formed payload.
    let serialized = crate::research::evidence::serialize_result(result);
    let round_trips = crate::research::evidence::deserialize_result(&serialized).is_some();
    ToolOutput::success(serde_json::json!({
        "mode": mode,
        "question": packet.query,
        "findings": packet.findings,
        "sources": packet.sources,
        "contradictions": packet.contradictions,
        "limitations": packet.limitations,
        "confidence": packet.confidence,
        "queries": result.queries,
        "retrieved_at": packet.retrieved_at,
        "findings_count": packet.findings.len(),
        "sources_count": packet.sources.len(),
        "contradictions_count": packet.contradictions.len(),
        "evidence_summary": crate::research::evidence::build_evidence_summary(result),
        "evidence_packet_serialized": serialized,
        "evidence_packet_round_trips": round_trips,
    }))
}

pub async fn execute_research(
    input: ResearchInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        if !crate::research::is_research_ready() {
            return Ok(ToolOutput::error(format!(
                "research is not available. Request was: {}. {}",
                input.query,
                crate::research::adapter_registry::readiness_report()
            )));
        }
        let pipeline = pipeline_with_memory(database);
        let result = pipeline
            .run_pipeline(&input.query, crate::research::pipeline::Mode::Auto)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(research_payload(&result, "auto"))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(http_feature_unavailable_for(
            "research",
            &input.query,
        )))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct QuickResearchInput {
    pub query: String,
}

pub async fn execute_quick_research(
    input: QuickResearchInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let quick = crate::research::quick_research::QuickMode::new(Arc::new(
            pipeline_with_memory(database),
        ));
        let result = quick
            .run(&input.query)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(research_payload(&result, "quick"))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(http_feature_unavailable_for(
            "quick_research",
            &input.query,
        )))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeepResearchInput {
    pub query: String,
}

pub async fn execute_deep_research(
    input: DeepResearchInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let deep =
            crate::research::deep_research::DeepMode::new(Arc::new(pipeline_with_memory(database)));
        let result = deep
            .run(&input.query)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(research_payload(&result, "deep"))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(http_feature_unavailable_for(
            "deep_research",
            &input.query,
        )))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FindErrorResolutionInput {
    pub error: String,
}

pub async fn execute_find_error_resolution(
    input: FindErrorResolutionInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let pipeline = pipeline_with_memory(database);
        let query = format!("error resolution for: {}", input.error);
        match pipeline
            .run_pipeline(&query, crate::research::pipeline::Mode::Quick)
            .await
        {
            Ok(result) => {
                let mut payload = research_payload(&result, "quick");
                if let Some(data) = payload.data.as_object_mut() {
                    data.insert("error_text".to_string(), serde_json::json!(input.error));
                }
                Ok(payload)
            }
            Err(e) => {
                // Do NOT fabricate a resolution. Returning success here with
                // "Check logs and retry" tells the LLM that a search happened
                // and produced a verdict, which invites it to invent a fix.
                // Propagate the real cause instead (Rule 6, structured errors).
                tracing::warn!(error = %e, "error-resolution research pipeline failed");
                Err(anyhow::anyhow!(
                    "error-resolution research failed for '{}': {}",
                    input.error,
                    e
                ))
            }
        }
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(http_feature_unavailable_for(
            "find_error_resolution",
            &input.error,
        )))
    }
}

pub async fn execute_web_search(input: WebSearchInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        // The tool is provider-agnostic: it queries the configured chain
        // (DuckDuckGo primary, Brave secondary when keyed) rather than naming a
        // provider, and reports the chain outcome honestly (R14, Rule 4).
        let providers = crate::research::adapter_registry::all_providers();
        let results = crate::research::failover::try_providers(&providers, &input.query)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(ToolOutput::success(serde_json::json!({
            "query": input.query,
            "provider": results.provider,
            "retrieved_at": results.retrieved_at,
            "results": results.results.iter().map(|r| serde_json::json!({
                "title": r.title,
                "url": r.url,
                "snippet": r.snippet,
                "relevance": r.relevance,
            })).collect::<Vec<_>>(),
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(http_feature_unavailable_for(
            "web_search",
            &input.query,
        )))
    }
}

// Per-source adapters (research_engine.md Tool Catalog, R11).
//
// Each handler returns the adapter's ACTUAL results with full provenance, never
// a result count. Returning a count is worse than returning nothing: the calling
// LLM reads a non-zero count as evidence that the search succeeded and invents
// the content that was never returned.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterQueryInput {
    pub query: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterUrlInput {
    pub url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterReferencesInput {
    pub references: Vec<serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterBibliographyInput {
    pub bibliography: Vec<serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterPlatformsInput {
    pub platforms: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterUrlsInput {
    pub urls: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterSubredditInput {
    pub query: String,
    pub subreddit: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterYouTubeInput {
    pub query: String,
    pub include_transcript: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterSubstackInput {
    pub publications: String,
    pub max_posts: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterBlueskyInput {
    pub query: String,
    pub sort: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterTelegramInput {
    pub channel: String,
    pub max_messages: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterOsmInput {
    pub query: String,
    pub location: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterSecFilingsInput {
    pub query: String,
    pub filing_type: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterCitationsInput {
    pub doi: String,
    pub format: String,
}

/// Render a provider result set as the MCP payload for a per-source adapter.
///
/// The payload carries every result with its title, URL, snippet and relevance,
/// so the caller can read the evidence and cite it. A note is attached when the
/// result count exceeds what is returned, so the cap is never silent.
#[cfg(feature = "http")]
fn adapter_payload(
    provider: &'static str,
    query: &str,
    results: Vec<crate::research::provider::SearchResult>,
    limit: usize,
    extra: Vec<(&str, serde_json::Value)>,
) -> ToolOutput {
    let total = results.len();
    let returned: Vec<_> = results.into_iter().take(limit).collect();
    let dropped = total.saturating_sub(returned.len());
    let mut payload = serde_json::json!({
        "provider": provider,
        "query": query,
        "result_count": returned.len(),
        "results": returned.iter().map(|r| serde_json::json!({
            "title": r.title,
            "url": r.url,
            "snippet": r.snippet,
            "relevance": r.relevance,
        })).collect::<Vec<_>>(),
    });
    if dropped > 0 {
        payload["truncation_notice"] = serde_json::json!(format!(
            "{dropped} further results were retrieved and are not shown; raise the limit to see them."
        ));
    }
    for (key, value) in extra {
        payload[key] = value;
    }
    ToolOutput::success(payload)
}

/// Guard for the no-http build: report the tool as unavailable, never as empty.
#[cfg(not(feature = "http"))]
fn adapter_unavailable(tool: &str, subject: &str) -> Result<ToolOutput> {
    Ok(ToolOutput::error(http_feature_unavailable_for(
        tool, subject,
    )))
}

pub async fn execute_get_wikipedia(input: AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::wikipedia::WikipediaProvider::new();
        let results = provider
            .fetch(&input.query, 10)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "wikipedia",
            &input.query,
            results,
            10,
            Vec::new(),
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("get_wikipedia", &input.query)
    }
}

pub async fn execute_search_news(input: AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::news::NewsProvider::new();
        let results = provider
            .fetch(&input.query, 25)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "google_news",
            &input.query,
            results,
            15,
            Vec::new(),
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_news", &input.query)
    }
}

pub async fn execute_search_reddit(input: AdapterSubredditInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::reddit::RedditProvider::new();
        let results = provider
            .fetch(&input.query, input.subreddit.as_deref(), 25)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "reddit",
            &input.query,
            results,
            15,
            vec![("subreddit", serde_json::json!(input.subreddit))],
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_reddit", &input.query)
    }
}

pub async fn execute_search_hackernews(input: AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::hackernews::HackerNewsProvider::new();
        let results = provider
            .fetch(&input.query, 20)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "hackernews",
            &input.query,
            results,
            15,
            Vec::new(),
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_hackernews", &input.query)
    }
}

pub async fn execute_search_youtube(input: AdapterYouTubeInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::youtube::YouTubeProvider::new();
        let results = provider
            .fetch(&input.query, input.include_transcript.unwrap_or(false), 10)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "youtube",
            &input.query,
            results,
            10,
            Vec::new(),
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_youtube", &input.query)
    }
}

pub async fn execute_search_substack(input: AdapterSubstackInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::substack::SubstackProvider::new();
        let limit = input.max_posts.unwrap_or(20).clamp(1, 50);
        let results = provider
            .fetch(&input.publications, limit)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "substack",
            &input.publications,
            results,
            limit,
            vec![("max_posts", serde_json::json!(limit))],
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_substack", &input.publications)
    }
}

pub async fn execute_search_bluesky(input: AdapterBlueskyInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::bluesky::BlueskyProvider::new();
        let results = provider
            .fetch(&input.query, input.sort.as_deref(), 25)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "bluesky",
            &input.query,
            results,
            15,
            vec![("sort", serde_json::json!(input.sort))],
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_bluesky", &input.query)
    }
}

pub async fn execute_search_telegram(input: AdapterTelegramInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::telegram::TelegramProvider::new();
        let limit = input.max_messages.unwrap_or(20).clamp(1, 100);
        let results = provider
            .fetch(&input.channel, limit)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "telegram",
            &input.channel,
            results,
            limit,
            Vec::new(),
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_telegram", &input.channel)
    }
}

pub async fn execute_search_mastodon(input: AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::mastodon::MastodonProvider::new();
        let results = provider
            .fetch(&input.query, 20)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "mastodon",
            &input.query,
            results,
            15,
            Vec::new(),
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_mastodon", &input.query)
    }
}

pub async fn execute_search_vk(input: AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::vk::VkProvider::new();
        let results = provider
            .fetch(&input.query, 20)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload("vk", &input.query, results, 15, Vec::new()))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_vk", &input.query)
    }
}

pub async fn execute_search_preprints(input: AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::preprints::PreprintsProvider::new();
        let results = provider
            .fetch(&input.query, 20)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "arxiv",
            &input.query,
            results,
            15,
            Vec::new(),
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_preprints", &input.query)
    }
}

pub async fn execute_search_datasets(input: AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::datasets::DatasetsProvider::new();
        let results = provider
            .fetch(&input.query, 20)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "zenodo",
            &input.query,
            results,
            15,
            Vec::new(),
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_datasets", &input.query)
    }
}

pub async fn execute_search_osm(input: AdapterOsmInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::osm::OsmProvider::new();
        let results = provider
            .fetch(&input.query, input.location.as_deref(), 15)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "openstreetmap",
            &input.query,
            results,
            15,
            vec![("location", serde_json::json!(input.location))],
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_osm", &input.query)
    }
}

pub async fn execute_search_sec_filings(input: AdapterSecFilingsInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::sec_filings::SecFilingsProvider::new();
        let results = provider
            .fetch(&input.query, input.filing_type.as_deref(), 20)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "sec_edgar",
            &input.query,
            results,
            15,
            vec![("filing_type", serde_json::json!(input.filing_type))],
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("search_sec_filings", &input.query)
    }
}

pub async fn execute_resurrect_dead_link(input: AdapterUrlInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::wayback::WaybackProvider::new();
        let snapshot = provider
            .resurrect(&input.url)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(ToolOutput::success(serde_json::json!({
            "provider": "wayback_machine",
            "original_url": input.url,
            "archived_url": snapshot.url,
            "title": snapshot.title,
            "snapshot": snapshot.snippet,
            "retrieved_at": chrono::Utc::now(),
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("resurrect_dead_link", &input.url)
    }
}

pub async fn execute_verify_citations(input: AdapterReferencesInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        if input.references.is_empty() {
            return Ok(ToolOutput::error(
                "verify_citations requires at least one reference to verify.",
            ));
        }
        let provider = crate::research::crossref::CrossrefProvider::new();
        let mut verified = Vec::new();
        let mut unverified = Vec::new();
        for reference in &input.references {
            let doi = match reference.as_str() {
                Some(text) if !text.trim().is_empty() => text.trim().to_string(),
                Some(_) => continue,
                None => match reference.get("DOI").or_else(|| reference.get("doi")) {
                    Some(value) if value.is_string() => {
                        value.as_str().unwrap_or_default().trim().to_string()
                    }
                    _ => continue,
                },
            };
            if doi.is_empty() {
                continue;
            }
            match provider.verify_doi(&doi).await {
                Ok(record) => verified.push(serde_json::json!({
                    "doi": doi,
                    "title": record.title,
                    "url": record.url,
                    "snippet": record.snippet,
                })),
                Err(e) => unverified.push(serde_json::json!({
                    "doi": doi,
                    "reason": e.to_string(),
                })),
            }
        }
        if verified.is_empty() && unverified.is_empty() {
            return Ok(ToolOutput::error(
                "verify_citations found no DOI in the supplied references; nothing could be verified.",
            ));
        }
        Ok(ToolOutput::success(serde_json::json!({
            "provider": "crossref",
            "verified": verified,
            "unverified": unverified,
            "verified_count": verified.len(),
            "unverified_count": unverified.len(),
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("verify_citations", "")
    }
}

pub async fn execute_find_counter_arguments(input: AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::counter_arguments::CounterArgumentsProvider::new();
        let results = provider
            .fetch(&input.query, 15)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(adapter_payload(
            "semantic_scholar",
            &input.query,
            results,
            15,
            Vec::new(),
        ))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("find_counter_arguments", &input.query)
    }
}

pub async fn execute_validate_bibliography(input: AdapterBibliographyInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        if input.bibliography.is_empty() {
            return Ok(ToolOutput::error(
                "validate_bibliography requires at least one reference to validate.",
            ));
        }
        let provider = crate::research::validate_bibliography::ValidateBibliographyProvider::new();
        let statuses = provider.validate(&input.bibliography).await;
        let verified_count = statuses.iter().filter(|s| s.verified).count();
        Ok(ToolOutput::success(serde_json::json!({
            "provider": "crossref",
            "entries": statuses,
            "verified_count": verified_count,
            "unverified_count": statuses.len() - verified_count,
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("validate_bibliography", "")
    }
}

pub async fn execute_format_citations(input: AdapterCitationsInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = crate::research::format_citations::FormatCitationsProvider::new();
        let citation = provider
            .format(&input.doi, &input.format)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(ToolOutput::success(serde_json::json!({
            "provider": "crossref",
            "doi": input.doi,
            "format": input.format,
            "citation": citation,
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("format_citations", &input.doi)
    }
}

pub async fn execute_detect_trends(input: AdapterPlatformsInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        if input.platforms.is_empty() {
            return Ok(ToolOutput::error(
                "detect_trends requires at least one platform (supported: hackernews, reddit).",
            ));
        }
        let provider = crate::research::detect_trends::DetectTrendsProvider::new();
        let (results, reports) = provider.fetch(&input.platforms, 10).await;
        if results.is_empty() {
            // No platform produced a trend. Report why, so the caller can tell an
            // unreachable platform from one with no matching items.
            let reasons: Vec<String> = reports.iter().filter_map(|r| r.reason.clone()).collect();
            return Ok(ToolOutput::error(format!(
                "detect_trends retrieved no trending items. {}",
                if reasons.is_empty() {
                    "Every queried platform returned an empty result.".to_string()
                } else {
                    reasons.join("; ")
                }
            )));
        }
        Ok(ToolOutput::success(serde_json::json!({
            "platforms": input.platforms,
            "result_count": results.len(),
            "results": results.iter().map(|r| serde_json::json!({
                "title": r.title,
                "url": r.url,
                "snippet": r.snippet,
                "relevance": r.relevance,
            })).collect::<Vec<_>>(),
            "platform_reports": reports,
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        adapter_unavailable("detect_trends", "")
    }
}

pub async fn execute_score_reliability(input: AdapterUrlsInput) -> Result<ToolOutput> {
    // This tool is rule-based and performs no network call, so it works on every
    // build rather than being gated behind the `http` feature.
    if input.urls.is_empty() {
        return Ok(ToolOutput::error(
            "score_reliability requires at least one URL to assess.",
        ));
    }
    let provider = crate::research::score_reliability::ScoreReliabilityProvider::new();
    let assessments = provider.score_all(&input.urls);
    if assessments.is_empty() {
        return Ok(ToolOutput::error(
            "score_reliability received no non-empty URLs to assess.",
        ));
    }
    Ok(ToolOutput::success(serde_json::json!({
        "provider": "rule_based",
        "method": "structural URL analysis (scheme, host, persistent identifier, path specificity)",
        "assessments": assessments,
        "assessment_count": assessments.len(),
    })))
}
