//! Source reliability scoring (research_engine.md Tool Catalog: `score_reliability`).
//!
//! The spec describes this tool as *rule-based source quality scoring*, not as a
//! search. It therefore analyses the URLs it is given and returns a score with
//! the specific rules that fired, rather than performing a search and reporting
//! a fixed relevance number.
//!
//! The rules encode what can be established from a URL alone: the scheme, the
//! domain's apparent relationship to an academic or government registry, and
//! structural signals such as a DOI or a preprint identifier. A score is a
//! structural assessment of the citation, never a claim about a page's content,
//! and the output says so explicitly.

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResults, SearchSource};
use serde::Serialize;

const PROVIDER: &str = "score_reliability";

/// Domains whose host form indicates a scholarly or government registry.
///
/// A DOI in the path of a well-known registry host is strong evidence of a
/// registered, citable object; the same DOI string on an arbitrary host is not.
const REGISTRY_HOSTS: [&str; 12] = [
    "doi.org",
    "dx.doi.org",
    "arxiv.org",
    "biorxiv.org",
    "medrxiv.org",
    "zenodo.org",
    "figshare.com",
    "osf.io",
    "sec.gov",
    "europa.eu",
    "nih.gov",
    "ncbi.nlm.nih.gov",
];

/// A single rule that contributed to a score.
#[derive(Debug, Clone, Serialize)]
pub struct AppliedRule {
    /// Stable identifier for the rule.
    pub rule: String,
    /// Human-readable explanation of what the rule observed.
    pub observation: String,
    /// Signed contribution to the score.
    pub weight: f32,
}

/// The reliability assessment for one URL.
#[derive(Debug, Clone, Serialize)]
pub struct ReliabilityScore {
    /// The URL that was assessed.
    pub url: String,
    /// The host portion of the URL.
    pub host: String,
    /// Final score, clamped to 0.0-1.0.
    pub score: f32,
    /// Confidence that the structural rules applied cleanly.
    pub assessment_confidence: f32,
    /// The rules that fired, in the order they were applied.
    pub rules: Vec<AppliedRule>,
    /// What this score can and cannot establish.
    pub caveat: String,
}

/// Split a URL into host and path without a URL-parsing dependency.
fn split_url(url: &str) -> (String, String) {
    let without_scheme = url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    match without_scheme.split_once('/') {
        Some((host, path)) => (host.to_ascii_lowercase(), path.to_ascii_lowercase()),
        None => (without_scheme.to_ascii_lowercase(), String::new()),
    }
}

/// Whether a host sits under a registry domain, including subdomains.
fn is_registry_host(host: &str) -> bool {
    REGISTRY_HOSTS
        .iter()
        .any(|registry| host == *registry || host.ends_with(&format!(".{registry}")))
}

/// Apply the structural rules to one URL.
pub fn score_url(url: &str) -> ReliabilityScore {
    let (host, path) = split_url(url);
    let mut rules: Vec<AppliedRule> = Vec::new();
    let mut score = 0.5_f32;
    let mut applicable = 0usize;
    let mut total_rules = 0usize;

    // Rule 1: a registered-object identifier in the path is strong evidence the
    // citation points at a persistent, citable object.
    total_rules += 1;
    let has_doi = path.starts_with("10.") || host.contains("doi.org");
    let has_preprint_id = (host.contains("arxiv.org") && path.contains('/'))
        || path.contains("10.5281")
        || path.contains("10.5061");
    if has_doi {
        applicable += 1;
        rules.push(AppliedRule {
            rule: "persistent_identifier".to_string(),
            observation: "URL carries a DOI, which is a registered persistent identifier."
                .to_string(),
            weight: 0.2,
        });
        score += 0.2;
    } else if has_preprint_id {
        applicable += 1;
        rules.push(AppliedRule {
            rule: "repository_identifier".to_string(),
            observation: "URL points at a recognised preprint or repository record.".to_string(),
            weight: 0.15,
        });
        score += 0.15;
    } else {
        rules.push(AppliedRule {
            rule: "persistent_identifier".to_string(),
            observation: "URL carries no DOI or repository identifier.".to_string(),
            weight: -0.1,
        });
        score -= 0.1;
    }

    // Rule 2: a scholarly or government registry host is stronger than an
    // unclassified one, because such hosts apply editorial or registration
    // processes before a record is published.
    total_rules += 1;
    if is_registry_host(&host) {
        applicable += 1;
        rules.push(AppliedRule {
            rule: "registry_host".to_string(),
            observation: format!("{host} is a recognised scholarly or government registry."),
            weight: 0.2,
        });
        score += 0.2;
    } else {
        rules.push(AppliedRule {
            rule: "registry_host".to_string(),
            observation: format!(
                "{host} is not a recognised registry; editorial process is unknown."
            ),
            weight: 0.0,
        });
    }

    // Rule 3: HTTPS is the minimum for a citation that can be verified later.
    total_rules += 1;
    let https = url.trim().starts_with("https://");
    if https {
        applicable += 1;
        rules.push(AppliedRule {
            rule: "transport".to_string(),
            observation: "URL uses HTTPS, so the citation can be re-verified securely.".to_string(),
            weight: 0.1,
        });
        score += 0.1;
    } else {
        rules.push(AppliedRule {
            rule: "transport".to_string(),
            observation: "URL does not use HTTPS; the citation cannot be re-verified securely."
                .to_string(),
            weight: -0.1,
        });
        score -= 0.1;
    }

    // Rule 4: a bare host with no path is a landing page, not a specific object.
    total_rules += 1;
    if path.is_empty() {
        rules.push(AppliedRule {
            rule: "specificity".to_string(),
            observation: "URL points at a site root rather than a specific page.".to_string(),
            weight: -0.1,
        });
        score -= 0.1;
    } else {
        applicable += 1;
        rules.push(AppliedRule {
            rule: "specificity".to_string(),
            observation: "URL points at a specific path, so it resolves to one page.".to_string(),
            weight: 0.0,
        });
    }

    ReliabilityScore {
        url: url.to_string(),
        host,
        score: score.clamp(0.0, 1.0),
        assessment_confidence: if total_rules == 0 {
            0.0
        } else {
            applicable as f32 / total_rules as f32
        },
        rules,
        caveat: "This score reflects structural properties of the URL only (scheme, host, identifier). \
It does not assess the page's content, authorship, or editorial review; read the source before relying on it."
            .to_string(),
    }
}

pub struct ScoreReliabilityProvider;

impl ScoreReliabilityProvider {
    pub fn new() -> Self {
        Self
    }

    /// Score every supplied URL.
    pub fn score_all(&self, urls: &[String]) -> Vec<ReliabilityScore> {
        urls.iter()
            .filter(|u| !u.trim().is_empty())
            .map(|u| score_url(u))
            .collect()
    }
}

impl Default for ScoreReliabilityProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for ScoreReliabilityProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        Box::pin(async move {
            // The adapter analyses URLs; a free-text query is scored as a single
            // URL when it looks like one, and reported plainly otherwise.
            let trimmed = query.trim();
            if crate::research::sanitize::is_citable_url(trimmed) {
                let assessment = score_url(trimmed);
                let result = crate::research::sanitize::build_result(
                    &format!("Reliability assessment for {trimmed}"),
                    trimmed,
                    &serde_json::to_string(&assessment).unwrap_or_default(),
                    assessment.score,
                    SearchSource::Web,
                );
                return Ok(SearchResults {
                    results: vec![result],
                    provider: PROVIDER.to_string(),
                    query,
                    retrieved_at: chrono::Utc::now(),
                });
            }
            Err(ResearchError::ProviderError {
                provider: PROVIDER.to_string(),
                message: format!(
                    "score_reliability takes a list of URLs, not a free-text query (received: {query})"
                ),
            })
        })
    }

    fn name(&self) -> &str {
        PROVIDER
    }

    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
