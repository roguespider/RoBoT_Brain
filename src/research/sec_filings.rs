//! SEC EDGAR adapter (research_engine.md Tool Catalog: `search_sec_filings`).
//!
//! Backed by EDGAR's public full-text search endpoint, which needs no API key
//! but does require a descriptive User-Agent identifying the requester (supplied
//! by the shared HTTP client). Every result is a real filing with a real
//! accession number and a citable filing index (Rule 3 "Source Provenance").

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://efts.sec.gov/LATEST/search-index";
const PROVIDER: &str = "sec_filings";

#[derive(Debug, Deserialize)]
struct EdgarResponse {
    #[serde(default)]
    hits: EdgarHits,
}

#[derive(Debug, Deserialize, Default)]
struct EdgarHits {
    #[serde(default)]
    hits: Vec<EdgarHit>,
}

#[derive(Debug, Deserialize)]
struct EdgarHit {
    #[serde(default)]
    _id: Option<String>,
    #[serde(default)]
    _source: EdgarSource,
}

#[derive(Debug, Deserialize, Default)]
struct EdgarSource {
    #[serde(default)]
    file_date: Option<String>,
    #[serde(default)]
    display_names: Option<Vec<String>>,
    #[serde(default)]
    form: Option<String>,
    #[serde(default)]
    accession_no: Option<String>,
    #[serde(default)]
    file_num: Option<String>,
}

/// Build the citable EDGAR filing-index URL from an accession number.
///
/// EDGAR accession numbers are written `0001234567-24-000123` but their index
/// lives under the dash-free form `000123456724000123`.
fn filing_index_url(accession: &str) -> Option<String> {
    let digits: String = accession.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 18 {
        return None;
    }
    // The central index is derived from the first ten digits of the filer CIK.
    let cik = &digits[..10];
    Some(format!(
        "https://www.sec.gov/Archives/edgar/data/{cik}/{digits}/{}-index.htm",
        &digits[10..18]
    ))
}

/// Compose a snippet from the fields EDGAR genuinely returns.
fn snippet_for(hit: &EdgarHit) -> String {
    let source = &hit._source;
    let mut parts: Vec<String> = Vec::new();
    if let Some(names) = source.display_names.as_ref().filter(|n| !n.is_empty()) {
        parts.push(format!("Filer: {}", names.join(", ")));
    }
    if let Some(form) = source.form.as_deref().filter(|f| !f.is_empty()) {
        parts.push(format!("Form type: {form}"));
    }
    if let Some(date) = source.file_date.as_deref().filter(|d| !d.is_empty()) {
        parts.push(format!("Filed: {date}"));
    }
    if let Some(file_num) = source.file_num.as_deref().filter(|f| !f.is_empty()) {
        parts.push(format!("File number: {file_num}"));
    }
    if let Some(accession) = source.accession_no.as_deref().filter(|a| !a.is_empty()) {
        parts.push(format!("Accession: {accession}"));
    }
    if parts.is_empty() {
        return "SEC filing returned without filer metadata.".to_string();
    }
    parts.join("\n")
}

pub struct SecFilingsProvider;

impl SecFilingsProvider {
    pub fn new() -> Self {
        Self
    }

    /// Search EDGAR full-text, optionally restricted to a filing form.
    pub async fn fetch(
        &self,
        query: &str,
        filing_type: Option<&str>,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let form_filter = filing_type
            .map(str::trim)
            .filter(|f| !f.is_empty())
            .map(|f| format!("&forms={}", urlencoding::encode(f)));
        let url = format!(
            "{ENDPOINT}?q={}&dateRange=custom&startdt=2000-01-01{}",
            urlencoding::encode(query),
            form_filter.unwrap_or_default()
        );
        let parsed: EdgarResponse = crate::research::http::get_json(PROVIDER, &url).await?;
        let hits = parsed.hits.hits;
        let total = hits.len();
        let mut accepted = 0usize;
        let results: Vec<SearchResult> = hits
            .into_iter()
            .filter_map(|hit| {
                let accession = hit
                    ._source
                    .accession_no
                    .clone()
                    .filter(|a| !a.trim().is_empty())?;
                // A filing without a resolvable index URL cannot be cited.
                let url = filing_index_url(&accession)?;
                let filer = hit
                    ._source
                    .display_names
                    .clone()
                    .unwrap_or_default()
                    .join(", ");
                let form = hit
                    ._source
                    .form
                    .clone()
                    .unwrap_or_else(|| "filing".to_string());
                let title = if filer.is_empty() {
                    format!("SEC {form} filing ({accession})")
                } else {
                    format!("{filer} - SEC {form} filing")
                };
                // EDGAR returns matches in relevance order without a score, so
                // the accepted sequence order carries the rank (R5).
                let relevance = crate::research::sanitize::positional_relevance(accepted, total);
                accepted += 1;
                Some(crate::research::sanitize::build_result(
                    &title,
                    &url,
                    &snippet_for(&hit),
                    relevance,
                    SearchSource::Web,
                ))
            })
            .collect();
        let mut results = results;
        results.truncate(max_results.clamp(1, 100));
        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for SecFilingsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for SecFilingsProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = SecFilingsProvider::new();
        Box::pin(async move {
            let results = provider.fetch(&query, None, 20).await?;
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
