pub mod brave;
pub mod config;
pub mod deep_research;
#[cfg(feature = "http")]
pub mod duckduckgo;
pub mod errors;
pub mod evidence;
pub mod failover;
#[cfg(feature = "http")]
pub mod jina;
pub mod mock;
pub mod pipeline;
pub mod provider;
pub mod quick_research;
pub mod sanitize;

use config::ResearchConfig;
use std::sync::OnceLock;

/// Module-level research configuration, initialized once at first access.
///
/// The config is loaded from environment variables on first call via
/// `ResearchConfig::from_env()`, which internally uses `env_key`,
/// `env_key`, `env_key_or`, and `is_env_key_set` for consistent config access.
static RESEARCH_CONFIG: OnceLock<ResearchConfig> = OnceLock::new();

/// Return the module's research configuration.
///
/// On first call, loads from environment variables using the
/// `env_key`, `env_key_or`, and `is_env_key_set` helpers.
/// Subsequent calls return the cached config.
pub fn get_config() -> &'static ResearchConfig {
    RESEARCH_CONFIG.get_or_init(ResearchConfig::from_env)
}

/// Check whether a search provider can be initialized (API key present
/// and the feature is enabled).
pub fn is_research_ready() -> bool {
    get_config().is_ready()
}

#[derive(Clone)]
pub struct Finding {
    pub statement: String,
    pub source_url: String,
    pub confidence: f32,
}

#[derive(Clone)]
pub struct Source {
    pub title: String,
    pub url: String,
    pub provider: String,
    pub query_used: String,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
    pub relevance: f32,
    pub content: String,
}

#[derive(Clone)]
pub struct Contradiction {
    pub claim_a: String,
    pub claim_b: String,
    pub source_a_url: String,
    pub source_b_url: String,
    pub resolution: String,
}

#[derive(Clone)]
pub struct ResearchResult {
    pub question: String,
    pub queries: Vec<String>,
    pub sources: Vec<Source>,
    pub findings: Vec<Finding>,
    pub contradictions: Vec<Contradiction>,
    pub limitations: Vec<String>,
    pub confidence: f32,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
}
