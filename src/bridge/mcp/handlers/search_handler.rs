// src/bridge/tools/handlers/search_handler.rs
// Search tools handler - handles global search, recommendations, reputation,
// research pipeline (quick/deep/auto), web search/open/extract, error resolution,
// and all per-source adapter tools.

use crate::bridge::mcp::McpContext;
use crate::bridge::mcp::handlers::{
    HandlerError, HandlerInitError, HandlerInitResult, ToolHandler,
};
use crate::bridge::tools::search;
use std::sync::Arc;

/// Handler for search-related tools
#[derive(Clone)]
pub struct SearchToolsHandler {
    context: Arc<McpContext>,
}

impl SearchToolsHandler {
    /// Create a new search tools handler
    pub fn new(context: Arc<McpContext>) -> HandlerInitResult<Self> {
        // Validate that required dependencies exist
        if context.database.connection().is_err() {
            return Err(HandlerInitError::new(
                "search",
                "Database connection not available",
            ));
        }

        Ok(Self { context })
    }

    /// Execute global search across all data
    pub async fn execute_global_search(
        &self,
        input: search::GlobalSearchInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        search::execute_global_search(input, &self.context.database).await
    }

    /// Get recommendations based on patterns
    pub async fn execute_get_recommendations(
        &self,
        input: search::GetRecommendationsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        search::execute_get_recommendations(input, &self.context.database).await
    }

    /// Get reputation score for a tool
    pub async fn execute_get_reputation(
        &self,
        input: search::GetReputationInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        search::execute_get_reputation(input, &self.context.database).await
    }
}

impl ToolHandler for SearchToolsHandler {
    fn category(&self) -> &str {
        "search"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "global_search".to_string(),
            "get_recommendations".to_string(),
            "get_reputation".to_string(),
            "web_search".to_string(),
            "web_open".to_string(),
            "web_extract".to_string(),
            "research".to_string(),
            "quick_research".to_string(),
            "deep_research".to_string(),
            "find_error_resolution".to_string(),
            "get_wikipedia".to_string(),
            "search_news".to_string(),
            "search_reddit".to_string(),
            "search_hackernews".to_string(),
            "search_youtube".to_string(),
            "search_substack".to_string(),
            "search_bluesky".to_string(),
            "search_telegram".to_string(),
            "search_mastodon".to_string(),
            "search_vk".to_string(),
            "search_preprints".to_string(),
            "search_datasets".to_string(),
            "search_osm".to_string(),
            "search_sec_filings".to_string(),
            "resurrect_dead_link".to_string(),
            "verify_citations".to_string(),
            "find_counter_arguments".to_string(),
            "validate_bibliography".to_string(),
            "format_citations".to_string(),
            "detect_trends".to_string(),
            "score_reliability".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        self.context.database.connection().is_ok()
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        let mut tools = vec![
            rmcp::model::Tool::new(
                "global_search",
                "Search across all memories, experiences, and knowledge",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query" },
                        "limit": { "type": "number", "description": "Maximum results per category" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("Global Search"),
            rmcp::model::Tool::new(
                "get_recommendations",
                "Get recommendations based on patterns and history",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "category": { "type": "string", "description": "Recommendation category" }
                    }
                })),
            )
            .with_title("Get Recommendations"),
            rmcp::model::Tool::new(
                "get_reputation",
                "Get reputation/quality score for a tool or approach",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "tool_name": { "type": "string", "description": "Tool name to check" }
                    },
                    "required": ["tool_name"]
                })),
            )
            .with_title("Get Reputation"),
        ];

        // Research pipeline tools
        tools.push(
            rmcp::model::Tool::new(
                "research",
                "Run research pipeline (auto-selects quick/deep)",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Research query" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("Research"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "quick_research",
                "Run quick research (fast mode, 1-3 sources, <5s)",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Quick research query" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("Quick Research"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "deep_research",
                "Run deep research (thorough mode, multi-iteration, comparison)",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Deep research query" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("Deep Research"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "find_error_resolution",
                "Find error resolution via external search",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "error": { "type": "string", "description": "Error message" }
                    },
                    "required": ["error"]
                })),
            )
            .with_title("Find Error Resolution"),
        );

        // Web interaction tools
        tools.push(
            rmcp::model::Tool::new(
                "web_search",
                "Search the web using DuckDuckGo",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Web search query" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("Web Search"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "web_open",
                "Open a web URL and retrieve content",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "URL to open" }
                    },
                    "required": ["url"]
                })),
            )
            .with_title("Web Open"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "web_extract",
                "Extract clean text from URL via Jina",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "URL to extract content from" }
                    },
                    "required": ["url"]
                })),
            )
            .with_title("Web Extract"),
        );

        // Per-source adapter tools
        tools.push(
            rmcp::model::Tool::new(
                "get_wikipedia",
                "Search Wikipedia for encyclopedic facts",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Wikipedia search query" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("Wikipedia"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "search_news",
                "Search current news sources",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "News search query" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("News Search"),
        );

        tools.push(rmcp::model::Tool::new(
            "search_reddit",
            "Search Reddit discussions",
            json_to_schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Reddit search query" },
                    "subreddit": { "type": "string", "description": "Optional subreddit filter" }
                },
                "required": ["query"]
            })),
        ).with_title("Reddit Search"));

        tools.push(
            rmcp::model::Tool::new(
                "search_hackernews",
                "Search Hacker News tech community",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "HN search query" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("Hacker News Search"),
        );

        tools.push(rmcp::model::Tool::new(
            "search_youtube",
            "Search YouTube videos (optional transcript)",
            json_to_schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "YouTube search query" },
                    "include_transcript": { "type": "boolean", "description": "Include video transcript" }
                },
                "required": ["query"]
            })),
        ).with_title("YouTube Search"));

        tools.push(
            rmcp::model::Tool::new(
                "search_substack",
                "Search Substack newsletter publications",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "publications": { "type": "string", "description": "Publication names" },
                        "max_posts": { "type": "number", "description": "Maximum posts to return" }
                    },
                    "required": ["publications"]
                })),
            )
            .with_title("Substack Search"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "search_bluesky",
                "Search Bluesky social posts",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Bluesky search query" },
                        "sort": { "type": "string", "description": "Sort order (top/recent)" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("Bluesky Search"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "search_telegram",
                "Search Telegram public channel messages",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "channel": { "type": "string", "description": "Channel name" },
                        "max_messages": { "type": "number", "description": "Maximum messages" }
                    },
                    "required": ["channel"]
                })),
            )
            .with_title("Telegram Search"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "search_mastodon",
                "Search Mastodon fediverse posts",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Mastodon search query" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("Mastodon Search"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "search_vk",
                "Search VK social network",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "VK search query" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("VK Search"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "search_preprints",
                "Search arXiv/bioRxiv/medRxiv preprints",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Preprint search query" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("Preprints Search"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "search_datasets",
                "Search Zenodo/Figshare/OSF datasets",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Dataset search query" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("Datasets Search"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "search_osm",
                "Search OpenStreetMap geographic data",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "OSM search query" },
                        "location": { "type": "string", "description": "Location filter" }
                    },
                    "required": ["query"]
                })),
            )
            .with_title("OSM Search"),
        );

        tools.push(rmcp::model::Tool::new(
            "search_sec_filings",
            "Search SEC EDGAR financial filings",
            json_to_schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "SEC filing search query" },
                    "filing_type": { "type": "string", "description": "Filing type (10-K, 10-Q, etc.)" }
                },
                "required": ["query"]
            })),
        ).with_title("SEC Filings Search"));

        tools.push(
            rmcp::model::Tool::new(
                "resurrect_dead_link",
                "Recover broken links via Wayback Machine",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "Dead URL to resurrect" }
                    },
                    "required": ["url"]
                })),
            )
            .with_title("Resurrect Dead Link"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "verify_citations",
                "Verify citations via Crossref/OpenAlex",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "references": { "type": "array", "description": "Reference list with DOI" }
                    },
                    "required": ["references"]
                })),
            )
            .with_title("Verify Citations"),
        );

        tools.push(rmcp::model::Tool::new(
            "find_counter_arguments",
            "Find opposing academic arguments",
            json_to_schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Topic to find counter-arguments for" }
                },
                "required": ["query"]
            })),
        ).with_title("Find Counter Arguments"));

        tools.push(
            rmcp::model::Tool::new(
                "validate_bibliography",
                "Validate an entire reference list",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "bibliography": { "type": "array", "description": "Bibliography entries" }
                    },
                    "required": ["bibliography"]
                })),
            )
            .with_title("Validate Bibliography"),
        );

        tools.push(
            rmcp::model::Tool::new(
                "format_citations",
                "Format citations (BibTeX/APA/MLA/Chicago/RIS)",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "doi": { "type": "string", "description": "DOI to format" },
                        "format": { "type": "string", "description": "Output format" }
                    },
                    "required": ["doi", "format"]
                })),
            )
            .with_title("Format Citations"),
        );

        tools.push(rmcp::model::Tool::new(
            "detect_trends",
            "Detect trending topics across platforms",
            json_to_schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "platforms": { "type": "array", "items": {"type":"string"}, "description": "Platforms to scan" }
                },
                "required": ["platforms"]
            })),
        ).with_title("Detect Trends"));

        tools.push(rmcp::model::Tool::new(
            "score_reliability",
            "Score source reliability (rule-based)",
            json_to_schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "urls": { "type": "array", "items": {"type":"string"}, "description": "URLs to score" }
                },
                "required": ["urls"]
            })),
        ).with_title("Score Reliability"));

        tools
    }

    async fn execute_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<crate::bridge::tools::ToolOutput, HandlerError> {
        match name {
            "global_search" => {
                let input: search::GlobalSearchInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                self.execute_global_search(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "get_recommendations" => {
                let input: search::GetRecommendationsInput =
                    serde_json::from_value(args).unwrap_or_default();
                self.execute_get_recommendations(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "get_reputation" => {
                let input: search::GetReputationInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                self.execute_get_reputation(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            // Research pipeline tools
            "research" => {
                let input: search::ResearchInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_research(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "quick_research" => {
                let input: search::QuickResearchInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_quick_research(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "deep_research" => {
                let input: search::DeepResearchInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_deep_research(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "find_error_resolution" => {
                let input: search::FindErrorResolutionInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_find_error_resolution(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            // Web interaction tools
            "web_search" => {
                let input: search::WebSearchInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_web_search(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "web_open" => {
                let input: search::WebOpenInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_web_open(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "web_extract" => {
                let input: search::WebExtractInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_web_extract(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            // Per-source adapter tools
            "get_wikipedia" => {
                let input: search::AdapterQueryInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_get_wikipedia(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_news" => {
                let input: search::AdapterQueryInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_news(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_reddit" => {
                let input: search::AdapterSubredditInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_reddit(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_hackernews" => {
                let input: search::AdapterQueryInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_hackernews(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_youtube" => {
                let input: search::AdapterYouTubeInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_youtube(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_substack" => {
                let input: search::AdapterSubstackInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_substack(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_bluesky" => {
                let input: search::AdapterBlueskyInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_bluesky(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_telegram" => {
                let input: search::AdapterTelegramInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_telegram(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_mastodon" => {
                let input: search::AdapterQueryInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_mastodon(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_vk" => {
                let input: search::AdapterQueryInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_vk(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_preprints" => {
                let input: search::AdapterQueryInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_preprints(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_datasets" => {
                let input: search::AdapterQueryInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_datasets(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_osm" => {
                let input: search::AdapterOsmInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_osm(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "search_sec_filings" => {
                let input: search::AdapterSecFilingsInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_search_sec_filings(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "resurrect_dead_link" => {
                let input: search::AdapterUrlInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_resurrect_dead_link(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "verify_citations" => {
                let input: search::AdapterReferencesInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_verify_citations(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "find_counter_arguments" => {
                let input: search::AdapterQueryInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_find_counter_arguments(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "validate_bibliography" => {
                let input: search::AdapterBibliographyInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_validate_bibliography(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "format_citations" => {
                let input: search::AdapterCitationsInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_format_citations(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "detect_trends" => {
                let input: search::AdapterPlatformsInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_detect_trends(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            "score_reliability" => {
                let input: search::AdapterUrlsInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                search::execute_score_reliability(input)
                    .await
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
            }
            _ => Err(HandlerError::ToolNotFound(name.to_string())),
        }
    }
}
