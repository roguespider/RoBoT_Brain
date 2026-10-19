// src/bridge/tools/search/web.rs
// Web interaction functions: web_search, web_open, web_extract.

use anyhow::Result;

use crate::bridge::tools::ToolOutput;
use crate::research;

/// Execute web search tool
pub async fn execute_web_search(input: super::types::WebSearchInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        // The tool is provider-agnostic: it queries the configured chain
        // (DuckDuckGo primary, Brave secondary when keyed) rather than naming a
        // provider, and reports the chain outcome honestly (R14, Rule 4).
        let providers = research::adapter_registry::all_providers();
        let results = research::failover::try_providers(&providers, &input.query)
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
        Ok(ToolOutput::error(super::http_feature_unavailable_for(
            "web_search",
            &input.query,
        )))
    }
}

/// Execute web open tool
pub async fn execute_web_open(input: super::types::WebOpenInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        // The page is fetched and reduced to readable text. Raw HTML is never
        // returned across the MCP boundary (Rule 9), and the content is framed
        // as untrusted data so it cannot be mistaken for an instruction.
        let body = research::http::get_text("web_open", &input.url).await?;
        let (content, truncated) = research::sanitize::truncate_with_marker(
            &research::sanitize::sanitize_text(&body),
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
            "content": research::sanitize::as_quoted_data("PAGE", &content),
            "content_length": content.chars().count(),
            "truncated": truncated,
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(super::http_feature_unavailable_for(
            "web_open", &input.url,
        )))
    }
}

/// Execute web extract tool
pub async fn execute_web_extract(input: super::types::WebExtractInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let jina = research::jina::JinaProvider::new();
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
        let text = research::errors::with_timeout(extraction, std::time::Duration::from_secs(30))
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))??;
        // Extracted page text is untrusted web data and is framed as such
        // before it crosses the MCP boundary (Rule 9).
        let (content, truncated) = research::sanitize::truncate_with_marker(
            &research::sanitize::sanitize_text(&text),
            8000,
        );
        Ok(ToolOutput::success(serde_json::json!({
            "url": input.url,
            "provider": "jina",
            "extracted_text": research::sanitize::as_quoted_data("PAGE", &content),
            "length": content.chars().count(),
            "truncated": truncated,
        })))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(super::http_feature_unavailable_for(
            "web_extract",
            &input.url,
        )))
    }
}
