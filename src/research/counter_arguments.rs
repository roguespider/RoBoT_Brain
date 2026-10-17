//! Counter-argument adapter (research_engine.md Tool Catalog: `find_counter_arguments`).
//!
//! Backed by the public Semantic Scholar Graph API, which needs no API key for
//! basic search and returns structured JSON for real papers. The adapter finds
//! real works that engage with a topic, so a "counter-argument" result is a real
//! publication the reader can open, never an invented opposing claim (Rule 3).

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://api.semanticscholar.org/graph/v1/paper/search";
const PROVIDER: &str = "counter_arguments";

#[derive(Debug, Deserialize)]
struct SemanticScholarResponse {
    #[serde(default)]
    data: Vec<SemanticScholarPaper>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarPaper {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    abstract_text: Option<String>,
    #[serde(default)]
    year: Option<i64>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    external_ids: Option<SemanticScholarExternalIds>,
    #[serde(default)]
    citation_count: Option<u32>,
    #[serde(default)]
    authors: Option<Vec<SemanticScholarAuthor>>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarExternalIds {
    #[serde(default)]
    doi: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarAuthor {
    #[serde(default)]
    name: Option<String>,
}

/// The citable URL for a paper, preferring the publisher DOI.
fn paper_url(paper: &SemanticScholarPaper) -> Option<String> {
    if let Some(doi) = paper
        .external_ids
        .as_ref()
        .and_then(|e| e.doi.as_deref())
        .filter(|d| !d.trim().is_empty())
    {
        return Some(format!("https://doi.org/{doi}"));
    }
    paper
        .url
        .clone()
        .filter(|u| crate::research::sanitize::is_citable_url(u))
}

/// Compose a snippet from the fields Semantic Scholar genuinely returns.
fn snippet_for(paper: &SemanticScholarPaper) -> String {
    let mut parts: Vec<String> = Vec::new();
    let abstract_text = crate::research::sanitize::truncate_with_marker(
        &crate::research::sanitize::sanitize_text(&paper.abstract_text.clone().unwrap_or_default()),
        800,
    )
    .0;
    if !abstract_text.trim().is_empty() {
        parts.push(abstract_text);
    }
    let mut meta: Vec<String> = Vec::new();
    let authors: Vec<String> = paper
        .authors
        .as_ref()
        .map(|list| {
            list.iter()
                .filter_map(|a| a.name.clone())
                .filter(|n| !n.trim().is_empty())
                .collect()
        })
        .unwrap_or_default();
    if !authors.is_empty() {
        meta.push(format!("Authors: {}", authors.join(", ")));
    }
    if let Some(year) = paper.year {
        meta.push(format!("Year: {year}"));
    }
    if let Some(citations) = paper.citation_count {
        meta.push(format!("Cited by: {citations} papers"));
    }
    if !meta.is_empty() {
        parts.push(meta.join(" - "));
    }
    if parts.is_empty() {
        return "Paper record returned without an abstract.".to_string();
    }
    parts.join("\n")
}

pub struct CounterArgumentsProvider;

impl CounterArgumentsProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let url = format!(
            "{ENDPOINT}?query={}&limit={}&fields=title,abstract,year,url,externalIds,citationCount,authors",
            urlencoding::encode(query),
            max_results.clamp(1, 100)
        );
        let parsed: SemanticScholarResponse =
            crate::research::http::get_json(PROVIDER, &url).await?;
        let papers = parsed.data;
        let total = papers.len();
        let mut accepted = 0usize;
        let results: Vec<SearchResult> = papers
            .into_iter()
            .filter_map(|paper| {
                let url = paper_url(&paper)?;
                let title = paper.title.clone().filter(|t| !t.trim().is_empty())?;
                // Semantic Scholar ranks by relevance without exposing the score,
                // so the accepted sequence order carries the rank (R5).
                let relevance = crate::research::sanitize::positional_relevance(accepted, total);
                accepted += 1;
                Some(crate::research::sanitize::build_result(
                    &title,
                    &url,
                    &snippet_for(&paper),
                    relevance,
                    SearchSource::Web,
                ))
            })
            .collect();
        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for CounterArgumentsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for CounterArgumentsProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = CounterArgumentsProvider::new();
        Box::pin(async move {
            let results = provider.fetch(&query, 15).await?;
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
        source == SearchSource::Web
    }
}
