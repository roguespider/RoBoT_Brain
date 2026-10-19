// src/bridge/tools/search/mod.rs
// Search-related MCP tools - split into focused submodules.
//
// Modules:
//   - `types`:     Input struct definitions
//   - `definitions`: Tool name constants and McpTool list
//   - `search`:    Core search (global_search, recommendations, reputation)
//   - `web`:       Web interaction (web_search, web_open, web_extract)
//   - `research`:  Research pipeline (research, quick_research, deep_research)
//   - `adapters`:  Per-source adapter execute functions

mod adapters;
mod research;
pub mod search_core;
pub mod tool_defs;
pub mod types;
mod web;

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
pub(crate) fn http_feature_unavailable_for(tool: &str, subject: &str) -> String {
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

// Re-export public types for backward compatibility
pub use types::{
    AdapterBibliographyInput, AdapterBlueskyInput, AdapterCitationsInput, AdapterOsmInput,
    AdapterPlatformsInput, AdapterQueryInput, AdapterReferencesInput, AdapterSecFilingsInput,
    AdapterSubredditInput, AdapterSubstackInput, AdapterTelegramInput, AdapterUrlInput,
    AdapterUrlsInput, AdapterYouTubeInput, DeepResearchInput, FindErrorResolutionInput,
    GetRecommendationsInput, GetReputationInput, GlobalSearchInput, QuickResearchInput,
    ResearchInput, WebExtractInput, WebOpenInput, WebSearchInput,
};

// Re-export core search functions
pub use search_core::{execute_get_recommendations, execute_get_reputation, execute_global_search};

// Re-export web functions
pub use web::{execute_web_extract, execute_web_open, execute_web_search};

// Re-export research functions
pub use research::{
    execute_deep_research, execute_find_error_resolution, execute_quick_research, execute_research,
};

// Re-export adapter functions
pub use adapters::{
    execute_detect_trends, execute_find_counter_arguments, execute_format_citations,
    execute_get_wikipedia, execute_resurrect_dead_link, execute_score_reliability,
    execute_search_bluesky, execute_search_datasets, execute_search_hackernews,
    execute_search_mastodon, execute_search_news, execute_search_osm, execute_search_preprints,
    execute_search_reddit, execute_search_sec_filings, execute_search_substack,
    execute_search_telegram, execute_search_vk, execute_search_youtube,
    execute_validate_bibliography, execute_verify_citations,
};
