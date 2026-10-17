// src/bridge/tools/search/definitions.rs
// Search tool definitions and schema

pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
    all_tools()
}

fn all_tools() -> Vec<crate::bridge::mcp::McpTool> {
    macro_rules! desc {
        ($s:expr) => {
            format!("[WORKFLOW: get_workflow + search_memory first] {}", $s)
        };
    }
    vec![
        crate::bridge::mcp::McpTool {
            name: "global_search".to_string(),
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
            name: "get_recommendations".to_string(),
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
            name: "get_reputation".to_string(),
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
            name: "web_extract".to_string(),
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
            name: "research".to_string(),
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
            name: "quick_research".to_string(),
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
            name: "deep_research".to_string(),
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
            name: "find_error_resolution".to_string(),
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
            name: "web_open".to_string(),
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
            name: "web_search".to_string(),
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
        // Per-source adapter tools (Architecture R11 / adapter registry)
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
