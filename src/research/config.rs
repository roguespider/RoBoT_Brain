use std::env;

/// Returns the value of an environment variable if it exists, otherwise None.
/// Used throughout the research module for configuration lookups.
pub fn env_key(name: &str) -> Option<String> {
    env::var(name).ok()
}

/// Returns the value of an environment variable or a default value if not set.
/// Commonly used for configuration with sensible defaults.
pub fn env_key_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

/// Returns true if an environment variable is set and not empty.
pub fn is_env_key_set(name: &str) -> bool {
    env::var(name)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

/// Research module configuration loaded from environment variables.
///
/// Uses the `env_key`, `env_key_or`, and `is_env_key_set` helpers for
/// consistent, testable environment-variable access across the module.
#[derive(Debug, Clone)]
pub struct ResearchConfig {
    /// API key for the search provider (e.g. SERPAPI_KEY).
    pub api_key: Option<String>,
    /// Base URL for the search provider.
    pub base_url: String,
    /// Maximum concurrent requests.
    pub max_concurrent: u32,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Whether web search is enabled.
    pub enabled: bool,
    /// Default language for searches.
    pub default_language: Option<String>,
}

impl ResearchConfig {
    /// Load configuration from environment variables.
    ///
    /// Uses `env_key_or` for values with defaults,
    /// `env_key` for optional values, and `is_env_key_set`
    /// for feature toggles.
    pub(crate) fn from_env() -> Self {
        Self {
            api_key: env_key("RESEARCH_API_KEY"),
            base_url: env_key_or("RESEARCH_BASE_URL", "https://default-api.example.com"),
            max_concurrent: env_key_or("RESEARCH_MAX_CONCURRENT", "3")
                .parse()
                .unwrap_or(3),
            timeout_secs: env_key_or("RESEARCH_TIMEOUT_SECS", "30")
                .parse()
                .unwrap_or(30),
            enabled: is_env_key_set("RESEARCH_ENABLED")
                || env_key("RESEARCH_API_KEY")
                    .map(|key| !key.is_empty())
                    .unwrap_or(false),
            default_language: env_key("RESEARCH_DEFAULT_LANGUAGE"),
        }
    }

    /// Return `true` when a usable search configuration exists
    /// (i.e. an API key is available and the feature is enabled).
    pub fn is_ready(&self) -> bool {
        self.enabled && self.api_key.is_some()
    }
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: "https://default-api.example.com".to_string(),
            max_concurrent: 3,
            timeout_secs: 30,
            enabled: false,
            default_language: None,
        }
    }
}
