//! Adapter registry — collects all registered SearchProvider implementations.
//!
//! Per Architecture §R3-R4, §R14: provider-agnostic abstraction.

use crate::research::provider::SearchProvider;
use std::sync::Arc;

/// Build the full provider list for the pipeline.
/// Includes primary (DuckDuckGo), optional fallback (Brave),
/// and all per-source adapters.
#[cfg(feature = "http")]
pub fn all_providers() -> Vec<Arc<dyn SearchProvider>> {
    let mut providers: Vec<Arc<dyn SearchProvider>> = Vec::new();
    // Primary
    if let Ok(ddg) = crate::research::duckduckgo::DuckDuckGoProvider::new() {
        providers.push(Arc::new(ddg));
    }
    // Optional fallback
    if let Some(key) = crate::research::config::env_key("BRAVE_API_KEY") {
        if !key.is_empty() {
            providers.push(Arc::new(crate::research::brave::BraveProvider::new()));
        }
    }
    // Per-source adapters
    providers.push(Arc::new(
        crate::research::wikipedia::WikipediaProvider::new(),
    ));
    providers.push(Arc::new(crate::research::reddit::RedditProvider::new()));
    providers.push(Arc::new(
        crate::research::hackernews::HackerNewsProvider::new(),
    ));
    providers.push(Arc::new(crate::research::youtube::YouTubeProvider::new()));
    providers.push(Arc::new(crate::research::substack::SubstackProvider::new()));
    providers.push(Arc::new(crate::research::bluesky::BlueskyProvider::new()));
    providers.push(Arc::new(crate::research::telegram::TelegramProvider::new()));
    providers.push(Arc::new(crate::research::mastodon::MastodonProvider::new()));
    providers.push(Arc::new(crate::research::vk::VkProvider::new()));
    providers.push(Arc::new(
        crate::research::preprints::PreprintsProvider::new(),
    ));
    providers.push(Arc::new(crate::research::datasets::DatasetsProvider::new()));
    providers.push(Arc::new(crate::research::osm::OsmProvider::new()));
    providers.push(Arc::new(
        crate::research::sec_filings::SecFilingsProvider::new(),
    ));
    providers.push(Arc::new(crate::research::wayback::WaybackProvider::new()));
    providers.push(Arc::new(crate::research::crossref::CrossrefProvider::new()));
    providers.push(Arc::new(crate::research::news::NewsProvider::new()));
    providers.push(Arc::new(
        crate::research::counter_arguments::CounterArgumentsProvider::new(),
    ));
    providers.push(Arc::new(
        crate::research::validate_bibliography::ValidateBibliographyProvider::new(),
    ));
    providers.push(Arc::new(
        crate::research::format_citations::FormatCitationsProvider::new(),
    ));
    providers.push(Arc::new(
        crate::research::detect_trends::DetectTrendsProvider::new(),
    ));
    providers.push(Arc::new(
        crate::research::score_reliability::ScoreReliabilityProvider::new(),
    ));
    providers
}

#[cfg(not(feature = "http"))]
pub fn all_providers() -> Vec<Arc<dyn SearchProvider>> {
    Vec::new()
}
