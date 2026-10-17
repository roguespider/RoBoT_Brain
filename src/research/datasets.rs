//! Dataset discovery adapter (research_engine.md Tool Catalog: `search_datasets`).
//!
//! Backed by Zenodo's public REST API, which needs no API key and returns
//! structured JSON for registered datasets. Every result is a real record with a
//! real DOI and a citable landing page, so a dataset citation resolves to the
//! record it describes (Rule 3 "Source Provenance").

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://zenodo.org/api/records";
const PROVIDER: &str = "datasets";

#[derive(Debug, Deserialize)]
struct ZenodoResponse {
    #[serde(default)]
    hits: ZenodoHits,
}

#[derive(Debug, Deserialize, Default)]
struct ZenodoHits {
    #[serde(default)]
    hits: Vec<ZenodoRecord>,
}

#[derive(Debug, Deserialize)]
struct ZenodoRecord {
    #[serde(default)]
    id: Option<i64>,
    #[serde(default)]
    doi: Option<String>,
    #[serde(default, rename = "type")]
    record_type: Option<String>,
    #[serde(default)]
    metadata: Option<ZenodoMetadata>,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct ZenodoMetadata {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    publication_date: Option<String>,
    #[serde(default)]
    creators: Option<Vec<ZenodoCreator>>,
    #[serde(default)]
    keywords: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
struct ZenodoCreator {
    #[serde(default)]
    name: Option<String>,
}

/// The citable landing page for a record.
///
/// The DOI is preferred because it is the persistent identifier; the record URL
/// is the fallback when a record has not been assigned one.
fn record_url(record: &ZenodoRecord) -> Option<String> {
    if let Some(doi) = record.doi.as_deref().filter(|d| !d.trim().is_empty()) {
        return Some(format!("https://doi.org/{doi}"));
    }
    record
        .id
        .map(|id| format!("https://zenodo.org/records/{id}"))
}

/// Compose a snippet from the fields the record genuinely provides.
fn snippet_for(record: &ZenodoRecord, metadata: &ZenodoMetadata) -> String {
    let mut parts: Vec<String> = Vec::new();
    let description = crate::research::sanitize::truncate_with_marker(
        &crate::research::sanitize::sanitize_text(
            &metadata.description.clone().unwrap_or_default(),
        ),
        800,
    )
    .0;
    if !description.trim().is_empty() {
        parts.push(description);
    }
    let mut meta: Vec<String> = Vec::new();
    let creators: Vec<String> = metadata
        .creators
        .as_ref()
        .map(|list| {
            list.iter()
                .filter_map(|c| c.name.clone())
                .filter(|n| !n.trim().is_empty())
                .collect()
        })
        .unwrap_or_default();
    if !creators.is_empty() {
        meta.push(format!("Creators: {}", creators.join(", ")));
    }
    if let Some(date) = metadata
        .publication_date
        .as_deref()
        .filter(|d| !d.trim().is_empty())
    {
        meta.push(format!("Published: {date}"));
    }
    if let Some(kind) = record
        .record_type
        .as_deref()
        .filter(|t| !t.trim().is_empty())
    {
        meta.push(format!("Record type: {kind}"));
    }
    if let Some(keywords) = metadata.keywords.as_ref().filter(|k| !k.is_empty()) {
        meta.push(format!("Keywords: {}", keywords.join(", ")));
    }
    if !meta.is_empty() {
        parts.push(meta.join(" - "));
    }
    if parts.is_empty() {
        return "Dataset record returned without metadata.".to_string();
    }
    parts.join("\n")
}

pub struct DatasetsProvider;

impl DatasetsProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let url = format!(
            "{ENDPOINT}?q={}&size={}&sort=bestmatch",
            urlencoding::encode(query),
            max_results.clamp(1, 100)
        );
        let parsed: ZenodoResponse = crate::research::http::get_json(PROVIDER, &url).await?;
        let records = parsed.hits.hits;
        let total = records.len();
        let mut accepted = 0usize;
        let results: Vec<SearchResult> = records
            .into_iter()
            .filter_map(|record| {
                let url = record_url(&record)?;
                let metadata = record.metadata.clone().unwrap_or_default();
                let title = metadata.title.clone().filter(|t| !t.trim().is_empty())?;
                // Zenodo sorts by best match but exposes no score, so the
                // accepted sequence order carries the rank (R5).
                let relevance = crate::research::sanitize::positional_relevance(accepted, total);
                accepted += 1;
                Some(crate::research::sanitize::build_result(
                    &title,
                    &url,
                    &snippet_for(&record, &metadata),
                    relevance,
                    SearchSource::Web,
                ))
            })
            .collect();
        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for DatasetsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for DatasetsProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = DatasetsProvider::new();
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
        source == SearchSource::Web
    }
}
