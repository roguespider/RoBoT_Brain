//! Adapter registry — collects all registered `SearchProvider` implementations.
//!
//! Per research_engine.md R3-R4 and R14 (provider-agnostic abstraction), the
//! pipeline never names a concrete provider. It receives a `Vec<Arc<dyn
//! SearchProvider>>` and applies failover across it.
//!
//! Two distinct lists are maintained here, and the distinction matters:
//!
//! * [`primary_providers`] — the general web-search chain the pipeline actually
//!   queries. Only adapters that can answer an arbitrary query belong here.
//! * [`source_adapters`] — the per-source adapters bound to their own MCP tool
//!   (`search_reddit`, `search_osm`, ...). Folding these into the general chain
//!   would send a Reddit query to the OSM geocoder, so they are kept separate.

use crate::research::provider::SearchProvider;
use std::sync::Arc;

/// Adapters that can answer an arbitrary general-web query.
///
/// Order encodes preference: the first entry is primary, the rest are failover
/// targets (R14). Only key-free or key-optional adapters qualify.
#[cfg(feature = "http")]
pub fn primary_providers() -> Vec<Arc<dyn SearchProvider>> {
    let mut providers: Vec<Arc<dyn SearchProvider>> = Vec::new();
    // Primary: DuckDuckGo, no API key required (R3).
    if let Ok(ddg) = crate::research::duckduckgo::DuckDuckGoProvider::new() {
        providers.push(Arc::new(ddg));
    }
    // Secondary: Brave Search, requires BRAVE_API_KEY (R14).
    if crate::research::get_config().has_brave() {
        providers.push(Arc::new(crate::research::brave::BraveProvider::new()));
    }
    providers
}

/// The full provider list handed to `ResearchPipeline`.
///
/// This is the general web-search chain. Per-source adapters are deliberately
/// excluded: they are selected by their own MCP tools, not by the pipeline.
#[cfg(feature = "http")]
pub fn all_providers() -> Vec<Arc<dyn SearchProvider>> {
    primary_providers()
}

/// The general web-search chain on a build without the `http` feature.
#[cfg(not(feature = "http"))]
pub fn all_providers() -> Vec<Arc<dyn SearchProvider>> {
    Vec::new()
}

/// Names of the providers that can actually serve a request right now.
///
/// This is the runtime readiness signal the dispatch path consults before
/// issuing a query, so a misconfigured deployment is reported as
/// `ProviderUnavailable` rather than silently returning an empty packet.
pub fn available_provider_names() -> Vec<String> {
    if !crate::research::get_config().is_enabled() {
        return Vec::new();
    }
    #[cfg(feature = "http")]
    {
        primary_providers()
            .iter()
            .map(|p| p.name().to_string())
            .collect()
    }
    #[cfg(not(feature = "http"))]
    {
        Vec::new()
    }
}

/// A human-readable readiness report for startup logging.
///
/// Names the live general-web providers and states which optional processing
/// services are configured, so an operator can see at boot what research will
/// and will not be able to do.
pub fn readiness_report() -> String {
    let config = crate::research::get_config();
    let live = available_provider_names();
    let providers = if live.is_empty() {
        "none".to_string()
    } else {
        live.join(", ")
    };
    format!(
        "general_web=[{providers}] brave_fallback={} jina_extraction={} max_sources={}",
        if config.has_brave() {
            "configured"
        } else {
            "unset"
        },
        if config.has_jina() {
            "configured"
        } else {
            "unset"
        },
        config.max_sources
    )
}
