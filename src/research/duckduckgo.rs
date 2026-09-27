//! DuckDuckGo provider — primary web search adapter (research_engine.md R3).
//!
//! Scrapes the no-JavaScript endpoint at `html.duckduckgo.com`, which returns
//! a stable server-rendered result list. Parsing uses the `scraper` crate
//! rather than line-based string slicing: the result anchors, their headings
//! and their snippets live on separate lines in the real response, so any
//! line-oriented parse yields empty titles.
//!
//! DuckDuckGo wraps outbound links in a redirect of the form
//! `//duckduckgo.com/l/?uddg=<urlencoded-target>`. Those must be unwrapped to
//! recover the real source URL, otherwise every citation points at a
//! DuckDuckGo redirect and violates Rule 3 "Source Provenance".

#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
#[cfg(feature = "http")]
use scraper::{Html, Selector};

#[cfg(feature = "http")]
const ENDPOINT: &str = "https://html.duckduckgo.com/html/";

#[cfg(feature = "http")]
pub struct DuckDuckGoProvider {
    client: reqwest::Client,
}

#[cfg(feature = "http")]
impl DuckDuckGoProvider {
    pub fn new() -> Result<Self, ResearchError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent(concat!(
                "Mozilla/5.0 (compatible; robot_brain/0.0.2; ",
                "+https://github.com/roguespider/robot_brain)"
            ))
            .build()
            .map_err(|e| ResearchError::ProviderError {
                provider: "duckduckgo".into(),
                message: e.to_string(),
            })?;
        Ok(Self { client })
    }

    /// Parse the result page into structured results.
    ///
    /// Only entries that yield BOTH a usable absolute URL and a non-empty
    /// title are kept. A result missing either field cannot satisfy Rule 3
    /// (provenance) and must not reach the evidence packet.
    fn parse_results(html: &str) -> Vec<SearchResult> {
        let document = Html::parse_document(html);
        let result_selector =
            Selector::parse("div.result, div.web-result").expect("static result selector");
        let link_selector = Selector::parse("a.result__a").expect("static result link selector");
        let snippet_selector =
            Selector::parse(".result__snippet").expect("static result snippet selector");

        let mut results = Vec::new();
        for element in document.select(&result_selector) {
            let link = match element.select(&link_selector).next() {
                Some(link) => link,
                None => continue,
            };
            let href = match link.value().attr("href") {
                Some(href) => href,
                None => continue,
            };
            let url = match resolve_result_url(href) {
                Some(url) => url,
                None => continue,
            };
            let title = collapse_whitespace(&link.text().collect::<String>());
            if title.is_empty() {
                continue;
            }
            let snippet = element
                .select(&snippet_selector)
                .next()
                .map(|node| collapse_whitespace(&node.text().collect::<String>()))
                .unwrap_or_default();
            results.push(SearchResult {
                title,
                url,
                snippet,
                relevance: 0.5,
                source: SearchSource::Web,
            });
        }
        results
    }
}

/// Unwrap a DuckDuckGo redirect and reject anything that is not absolute HTTP(S).
///
/// Handles the three shapes the endpoint emits:
///   * `//duckduckgo.com/l/?uddg=<encoded>` — protocol-relative redirect wrapper
///   * `/l/?uddg=<encoded>`                — path-relative redirect wrapper
///   * `https://example.com/page`          — already a direct link
fn resolve_result_url(href: &str) -> Option<String> {
    let raw = match href.split_once("uddg=") {
        Some((_, encoded)) => urlencoding::decode(encoded)
            .map(|decoded| decoded.into_owned())
            .unwrap_or_else(|_| encoded.to_string()),
        None => href.to_string(),
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Reject anything that is not an absolute http(s) URL: a relative path
    // cannot be cited and would break the provenance guarantee.
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return None;
    }
    Some(trimmed.to_string())
}

/// Collapse runs of whitespace so multi-line HTML text becomes one clean line.
#[cfg(feature = "http")]
fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(feature = "http")]
impl SearchProvider for DuckDuckGoProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let client = self.client.clone();
        let query = query.to_string();
        Box::pin(async move {
            let url = format!("{ENDPOINT}?q={}", urlencoding::encode(&query));
            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|e| ResearchError::ProviderError {
                    provider: "duckduckgo".into(),
                    message: e.to_string(),
                })?;
            if !resp.status().is_success() {
                return Err(ResearchError::ProviderError {
                    provider: "duckduckgo".into(),
                    message: format!("HTTP status {}", resp.status().as_u16()),
                });
            }
            let html = resp
                .text()
                .await
                .map_err(|e| ResearchError::ProviderError {
                    provider: "duckduckgo".into(),
                    message: e.to_string(),
                })?;
            let results = Self::parse_results(&html);
            if results.is_empty() {
                // DuckDuckGo answers a challenged or rate-limited request with a
                // page that contains no result entries. Reporting `NoResults`
                // here is honest: the provider returned nothing usable, and the
                // pipeline will try the next provider rather than the agent
                // treating this as "the web has nothing on the subject".
                return Err(ResearchError::NoResults {
                    query: format!(
                        "{query} (duckduckgo returned no parseable results; the endpoint may be rate-limiting this client)"
                    ),
                });
            }
            Ok(SearchResults {
                results,
                provider: "duckduckgo".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }

    fn name(&self) -> &str {
        "duckduckgo"
    }

    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
