// src/bridge/tools/search/adapters.rs
// Per-source adapter execute functions.
//
// Per-source adapters (research_engine.md Tool Catalog, R11).
//
// Each handler returns the adapter's ACTUAL results with full provenance, never
// a result count. Returning a count is worse than returning nothing: the calling
// LLM reads a non-zero count as evidence that the search succeeded and invents
// the content that was never returned.

use anyhow::Result;

use crate::bridge::tools::ToolOutput;
use crate::research;

/// Render a provider result set as the MCP payload for a per-source adapter.
///
/// The payload carries every result with its title, URL, snippet and relevance,
/// so the caller can read the evidence and cite it. A note is attached when the
/// result count exceeds what is returned, so the cap is never silent.
#[cfg(feature = "http")]
fn adapter_payload(
    provider: &'static str,
    query: &str,
    results: Vec<research::provider::SearchResult>,
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
    Ok(ToolOutput::error(super::http_feature_unavailable_for(
        tool, subject,
    )))
}

pub async fn execute_get_wikipedia(input: super::types::AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::wikipedia::WikipediaProvider::new();
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

pub async fn execute_search_news(input: super::types::AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::news::NewsProvider::new();
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

pub async fn execute_search_reddit(
    input: super::types::AdapterSubredditInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::reddit::RedditProvider::new();
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

pub async fn execute_search_hackernews(
    input: super::types::AdapterQueryInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::hackernews::HackerNewsProvider::new();
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

pub async fn execute_search_youtube(
    input: super::types::AdapterYouTubeInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::youtube::YouTubeProvider::new();
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

pub async fn execute_search_substack(
    input: super::types::AdapterSubstackInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::substack::SubstackProvider::new();
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

pub async fn execute_search_bluesky(
    input: super::types::AdapterBlueskyInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::bluesky::BlueskyProvider::new();
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

pub async fn execute_search_telegram(
    input: super::types::AdapterTelegramInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::telegram::TelegramProvider::new();
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

pub async fn execute_search_mastodon(input: super::types::AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::mastodon::MastodonProvider::new();
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

pub async fn execute_search_vk(input: super::types::AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::vk::VkProvider::new();
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

pub async fn execute_search_preprints(
    input: super::types::AdapterQueryInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::preprints::PreprintsProvider::new();
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

pub async fn execute_search_datasets(input: super::types::AdapterQueryInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::datasets::DatasetsProvider::new();
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

pub async fn execute_search_osm(input: super::types::AdapterOsmInput) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::osm::OsmProvider::new();
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

pub async fn execute_search_sec_filings(
    input: super::types::AdapterSecFilingsInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::sec_filings::SecFilingsProvider::new();
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

pub async fn execute_resurrect_dead_link(
    input: super::types::AdapterUrlInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::wayback::WaybackProvider::new();
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

pub async fn execute_verify_citations(
    input: super::types::AdapterReferencesInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        if input.references.is_empty() {
            return Ok(ToolOutput::error(
                "verify_citations requires at least one reference to verify.",
            ));
        }
        let provider = research::crossref::CrossrefProvider::new();
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

pub async fn execute_find_counter_arguments(
    input: super::types::AdapterQueryInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::counter_arguments::CounterArgumentsProvider::new();
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

pub async fn execute_validate_bibliography(
    input: super::types::AdapterBibliographyInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        if input.bibliography.is_empty() {
            return Ok(ToolOutput::error(
                "validate_bibliography requires at least one reference to validate.",
            ));
        }
        let provider = research::validate_bibliography::ValidateBibliographyProvider::new();
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

pub async fn execute_format_citations(
    input: super::types::AdapterCitationsInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let provider = research::format_citations::FormatCitationsProvider::new();
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

pub async fn execute_detect_trends(
    input: super::types::AdapterPlatformsInput,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        if input.platforms.is_empty() {
            return Ok(ToolOutput::error(
                "detect_trends requires at least one platform (supported: hackernews, reddit).",
            ));
        }
        let provider = research::detect_trends::DetectTrendsProvider::new();
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

pub async fn execute_score_reliability(
    input: super::types::AdapterUrlsInput,
) -> Result<ToolOutput> {
    // This tool is rule-based and performs no network call, so it works on every
    // build rather than being gated behind the `http` feature.
    if input.urls.is_empty() {
        return Ok(ToolOutput::error(
            "score_reliability requires at least one URL to assess.",
        ));
    }
    let provider = research::score_reliability::ScoreReliabilityProvider::new();
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
