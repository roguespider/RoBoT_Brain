pub mod adapter_registry;
pub mod atom;
#[cfg(feature = "http")]
pub mod bluesky;
pub mod brave;
pub mod config;
pub mod counter_arguments;
pub mod crossref;
pub mod datasets;
pub mod deep_research;
pub mod detect_trends;
#[cfg(feature = "http")]
pub mod duckduckgo;
pub mod errors;
pub mod evidence;
pub mod failover;
pub mod format_citations;
#[cfg(feature = "http")]
pub mod hackernews;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "http")]
pub mod jina;
#[cfg(feature = "http")]
pub mod mastodon;
pub mod mock;
pub mod news;
pub mod osm;
pub mod pipeline;
pub mod preprints;
pub mod provider;
pub mod quick_research;
pub mod reddit;
pub mod sanitize;
pub mod score_reliability;
pub mod sec_filings;
pub mod substack;
pub mod telegram;
pub mod validate_bibliography;
pub mod vk;
pub mod wayback;
pub mod wikipedia;
pub mod youtube;

use config::ResearchConfig;
use std::sync::OnceLock;

/// Re-export the evidence packet contract types (R8).
///
/// The definitions live in `evidence.rs` per the spec's File Structure table;
/// they are re-exported here because `crate::research::Source` and friends are
/// the canonical paths used across the pipeline, memory promotion and the MCP
/// bridge.
pub use evidence::{Contradiction, Finding, ResearchResult, Source};

/// Return the module's research configuration.
///
/// Loaded from environment variables on first call; cached thereafter.
pub fn get_config() -> &'static ResearchConfig {
    static RESEARCH_CONFIG: OnceLock<ResearchConfig> = OnceLock::new();
    RESEARCH_CONFIG.get_or_init(ResearchConfig::from_env)
}

/// Check whether research can actually be performed right now.
///
/// Readiness is a runtime property, not a build-time one: the primary
/// DuckDuckGo adapter needs no API key, so a deployment with no credentials is
/// still research-capable. The dispatch path consults this before issuing a
/// query so a disabled or unconfigured subsystem is reported as
/// `ProviderUnavailable` rather than silently returning an empty packet.
pub fn is_research_ready() -> bool {
    get_config().is_enabled() && !available_providers().is_empty()
}

/// Names of the providers that are configured and able to serve requests.
///
/// This is the runtime readiness signal consulted by the research dispatch path
/// before a query is issued, so a misconfigured deployment is detected at the
/// point of use rather than after a silent empty result.
pub fn available_providers() -> Vec<String> {
    adapter_registry::available_provider_names()
}
