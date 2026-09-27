//! Crossref / OpenAlex adapter (research_engine.md: `verify_citations`).
//!
//! Backed by the public Crossref REST API, which needs no API key and returns
//! structured JSON containing the real DOI, title, authors, container and
//! publication year. Every entry is a real registered work, so a verified
//! citation resolves to the publisher record it claims (Rule 3).

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://api.crossref.org/works";
const PROVIDER: &str = "crossref";

#[derive(Debug, Deserialize)]
struct CrossrefResponse {
    #[serde(default)]
    message: CrossrefMessage,
}

#[derive(Debug, Deserialize, Default)]
struct CrossrefMessage {
    #[serde(default)]
    items: Vec<CrossrefItem>,
}

#[derive(Debug, Deserialize)]
struct CrossrefItem {
    #[serde(default)]
    doi: Option<String>,
    #[serde(default)]
    title: Option<Vec<String>>,
    #[serde(default)]
    container_title: Option<Vec<String>>,
    #[serde(default)]
    author: Vec<CrossrefAuthor>,
    #[serde(default)]
    issued: Option<CrossrefDate>,
    #[serde(default, rename = "type")]
    work_type: Option<String>,
    #[serde(default)]
    is_referenced_by_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CrossrefAuthor {
    #[serde(default)]
    family: Option<String>,
    #[serde(default)]
    given: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CrossrefDate {
    #[serde(default)]
    date_parts: Vec<Vec<i64>>,
}

/// Render an author's name, preferring the family name Crossref guarantees.
fn author_name(author: &CrossrefAuthor) -> Option<String> {
    match (author.family.as_deref(), author.given.as_deref()) {
        (Some(family), Some(given)) if !family.trim().is_empty() => {
            Some(format!("{given} {family}").trim().to_string())
        }
        (Some(family), _) if !family.trim().is_empty() => Some(family.to_string()),
        (None, Some(given)) if !given.trim().is_empty() => Some(given.to_string()),
        _ => None,
    }
}

/// Extract a publication year from Crossref's nested `date-parts` array.
fn publication_year(issued: Option<&CrossrefDate>) -> Option<i64> {
    let date = issued?;
    let parts = date.date_parts.first()?;
    // The first part is the year; a missing or implausible value is not a year.
    let year = parts.first().copied()?;
    if (1000..=3000).contains(&year) {
        Some(year)
    } else {
        None
    }
}

/// The citable URL for a registered work.
fn work_url(doi: &str) -> String {
    format!("https://doi.org/{doi}")
}

/// Compose a snippet from the bibliographic fields Crossref actually returns.
fn snippet_for(item: &CrossrefItem, authors: &str, year: Option<i64>) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !authors.is_empty() {
        parts.push(format!("Authors: {authors}"));
    }
    if let Some(container) = item
        .container_title
        .as_ref()
        .and_then(|c| c.first())
        .filter(|c| !c.trim().is_empty())
    {
        parts.push(format!("Published in: {container}"));
    }
    if let Some(year) = year {
        parts.push(format!("Year: {year}"));
    }
    if let Some(kind) = item.work_type.as_deref().filter(|t| !t.trim().is_empty()) {
        parts.push(format!("Type: {kind}"));
    }
    if let Some(citations) = item.is_referenced_by_count {
        parts.push(format!("Cited by: {citations} works"));
    }
    if parts.is_empty() {
        return "Registered work returned without bibliographic detail.".to_string();
    }
    parts.join("\n")
}

pub struct CrossrefProvider;

impl CrossrefProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let url = format!(
            "{ENDPOINT}?query={}&rows={}&select=DOI,title,container-title,author,issued,type,is-referenced-by-count",
            urlencoding::encode(query),
            max_results.clamp(1, 100)
        );
        let parsed: CrossrefResponse = crate::research::http::get_json(PROVIDER, &url).await?;
        let items = parsed.message.items;
        let total = items.len();
        let mut accepted = 0usize;
        let results: Vec<SearchResult> = items
            .into_iter()
            .filter_map(|item| {
                // A work without a DOI cannot be cited or verified, so it cannot
                // satisfy the tool's purpose.
                let doi = item.doi.clone().filter(|d| !d.trim().is_empty())?;
                let title = item
                    .title
                    .as_ref()
                    .and_then(|t| t.first())
                    .cloned()
                    .filter(|t| !t.trim().is_empty())?;
                let authors = item
                    .author
                    .iter()
                    .filter_map(author_name)
                    .collect::<Vec<_>>()
                    .join(", ");
                let year = publication_year(item.issued.as_ref());
                // Crossref sorts by relevance but exposes no score, so the
                // accepted sequence order carries the rank (R5).
                let relevance = crate::research::sanitize::positional_relevance(accepted, total);
                accepted += 1;
                Some(crate::research::sanitize::build_result(
                    &title,
                    &work_url(&doi),
                    &snippet_for(&item, &authors, year),
                    relevance,
                    SearchSource::Web,
                ))
            })
            .collect();

        crate::research::http::require_results(PROVIDER, query, results)
    }

    /// Verify a single DOI by resolving it against Crossref.
    ///
    /// This is the `verify_citations` path: a registered DOI resolves, an
    /// unregistered one reports `NoResults` rather than being confirmed.
    pub async fn verify_doi(&self, doi: &str) -> Result<SearchResult, ResearchError> {
        let normalized = doi
            .trim()
            .trim_start_matches("https://doi.org/")
            .trim_start_matches("http://dx.doi.org/");
        if normalized.is_empty() {
            return Err(ResearchError::NoResults {
                query: "<empty doi>".to_string(),
            });
        }
        let url = format!("{ENDPOINT}/{}", urlencoding::encode(normalized));
        let parsed: CrossrefResponse = crate::research::http::get_json(PROVIDER, &url).await?;
        let item =
            parsed
                .message
                .items
                .into_iter()
                .next()
                .ok_or_else(|| ResearchError::NoResults {
                    query: format!("doi:{normalized}"),
                })?;
        let title = item
            .title
            .as_ref()
            .and_then(|t| t.first())
            .cloned()
            .unwrap_or_else(|| format!("Registered work {normalized}"));
        let authors = item
            .author
            .iter()
            .filter_map(author_name)
            .collect::<Vec<_>>()
            .join(", ");
        let year = publication_year(item.issued.as_ref());
        Ok(crate::research::sanitize::build_result(
            &title,
            &work_url(normalized),
            &snippet_for(&item, &authors, year),
            1.0,
            SearchSource::Web,
        ))
    }
}

impl Default for CrossrefProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for CrossrefProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = CrossrefProvider::new();
        Box::pin(async move {
            let results = provider.fetch(&query, 20).await?;
            Ok(SearchResults {
                results,
                provider: PROVIDER.to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }

    fn name(&self) -> &str {
        PROVIDER
    }

    fn supports(&self, source: SearchSource) -> bool {
        matches!(source, SearchSource::Web | SearchSource::News)
    }
}
