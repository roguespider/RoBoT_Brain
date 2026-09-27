//! Research module configuration (research_engine.md R4, R9, R14).
//!
//! Every tunable the adapters and the pipeline actually read lives here, sourced
//! from the environment so a deployment is configured without a rebuild. The
//! config is consulted at the point of use: the dispatch path checks readiness
//! before issuing a query, and each adapter reads its own key from this struct.

use std::env;

/// Returns the value of an environment variable if it exists, otherwise None.
///
/// The single entry point for environment access in the research module, so key
/// lookup and trimming behave identically for every adapter.
pub fn env_key(name: &str) -> Option<String> {
    env::var(name).ok().and_then(|v| {
        let trimmed = v.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

/// Returns true if an environment variable is set to a non-empty value.
///
/// Used by adapters that gate an optional provider on a credential being
/// present, so a blank key is treated as absent rather than as a usable value.
pub fn is_env_key_set(name: &str) -> bool {
    env_key(name).is_some()
}

/// Research configuration resolved from the environment.
///
/// Only fields the running system actually reads are kept here; a setting that
/// nothing consults is worse than no setting, because it looks configurable
/// while having no effect.
#[derive(Debug, Clone)]
pub struct ResearchConfig {
    /// Optional Brave Search API key, enabling the secondary provider (R14).
    pub brave_api_key: Option<String>,
    /// Optional Jina API key, enabling hosted content extraction (R4).
    pub jina_api_key: Option<String>,
    /// Whether research is permitted to make outbound calls at all.
    ///
    /// Set `RESEARCH_ENABLED=false` to hard-disable the subsystem; unset means
    /// enabled, because the primary provider needs no credential and a
    /// default-off switch would silently remove the last line of defence
    /// against hallucination.
    pub enabled: bool,
    /// Hard cap on sources per evidence packet (R15).
    pub max_sources: usize,
}

impl ResearchConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            brave_api_key: env_key("BRAVE_API_KEY"),
            jina_api_key: env_key("JINA_API_KEY"),
            enabled: !matches!(
                env::var("RESEARCH_ENABLED")
                    .ok()
                    .as_deref()
                    .map(str::trim)
                    .map(str::to_ascii_lowercase)
                    .as_deref(),
                Some("false") | Some("0") | Some("off") | Some("no")
            ),
            max_sources: if is_env_key_set("RESEARCH_MAX_SOURCES") {
                env_key("RESEARCH_MAX_SOURCES")
                    .and_then(|v| v.parse::<usize>().ok())
                    .map(|v| v.clamp(1, crate::research::sanitize::MAX_SOURCES))
                    .unwrap_or(crate::research::sanitize::MAX_SOURCES)
            } else {
                crate::research::sanitize::MAX_SOURCES
            },
        }
    }

    /// Whether the module may issue outbound research calls.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Whether the secondary (Brave) provider is configured (R14).
    pub fn has_brave(&self) -> bool {
        self.brave_api_key.is_some()
    }

    /// Whether hosted Jina extraction is configured (R4).
    pub fn has_jina(&self) -> bool {
        self.jina_api_key.is_some()
    }
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            brave_api_key: None,
            jina_api_key: None,
            enabled: true,
            max_sources: crate::research::sanitize::MAX_SOURCES,
        }
    }
}
