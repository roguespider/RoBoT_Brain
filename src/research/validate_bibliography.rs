//! Bibliography validation adapter (research_engine.md: `validate_bibliography`).
//!
//! Validates each reference against Crossref by resolving its DOI. A reference
//! whose DOI resolves is reported as verified; one that does not is reported as
//! unverified with the reason. Nothing is confirmed without a live lookup
//! (Rule 3, Rule 6 "Graceful Degradation").

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResults, SearchSource};
use serde_json::Value;

const PROVIDER: &str = "validate_bibliography";

/// The outcome of checking one bibliography entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BibliographyEntryStatus {
    /// Index of the entry within the submitted bibliography.
    pub index: usize,
    /// The reference as submitted.
    pub reference: Value,
    /// Whether a registered record was found.
    pub verified: bool,
    /// The DOI that was checked, when the entry carried one.
    pub doi: Option<String>,
    /// Why the entry could not be verified, when it could not be.
    pub reason: Option<String>,
    /// The registered title, when the entry was verified.
    pub registered_title: Option<String>,
}

/// Extract a DOI from a bibliography entry.
///
/// A DOI may appear as a bare string or under one of the common field names.
/// Returning `None` means the entry carries no resolvable identifier, which is
/// a different condition from a DOI that fails to resolve.
fn extract_doi(entry: &Value) -> Option<String> {
    // A bare DOI string.
    if let Some(text) = entry.as_str() {
        let trimmed = text.trim();
        if trimmed.starts_with("10.") {
            return Some(trimmed.to_string());
        }
        return None;
    }
    let object = entry.as_object()?;
    for key in ["DOI", "doi", "Doi"] {
        if let Some(value) = object.get(key).and_then(|v| v.as_str()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_ascii_lowercase());
            }
        }
    }
    // Fall back to scanning any string field for an embedded `10.` prefix.
    for value in object.values() {
        if let Some(text) = value.as_str()
            && let Some(position) = text.find("10.")
        {
            let candidate: String = text[position..]
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '"' && *c != ',')
                .collect();
            if candidate.len() > 8 {
                return Some(candidate.to_ascii_lowercase());
            }
        }
    }
    None
}

/// Read the registered title from a Crossref work record.
fn registered_title(record: &Value) -> Option<String> {
    let message = record.get("message")?;
    let titles = message.get("title")?.as_array()?;
    let first = titles.first()?.as_str()?;
    if first.trim().is_empty() {
        None
    } else {
        Some(first.trim().to_string())
    }
}

pub struct ValidateBibliographyProvider;

impl ValidateBibliographyProvider {
    pub fn new() -> Self {
        Self
    }

    /// Validate every entry, returning one status per entry.
    ///
    /// Entries are checked in input order so the caller can map each status back
    /// to the reference it describes.
    pub async fn validate(&self, bibliography: &[Value]) -> Vec<BibliographyEntryStatus> {
        let mut statuses = Vec::with_capacity(bibliography.len());
        for (index, entry) in bibliography.iter().enumerate() {
            let status = match extract_doi(entry) {
                None => BibliographyEntryStatus {
                    index,
                    reference: entry.clone(),
                    verified: false,
                    doi: None,
                    reason: Some(
                        "No DOI found in this reference; it cannot be verified against Crossref."
                            .to_string(),
                    ),
                    registered_title: None,
                },
                Some(doi) => match self.verify(&doi).await {
                    Ok(title) => BibliographyEntryStatus {
                        index,
                        reference: entry.clone(),
                        verified: true,
                        doi: Some(doi),
                        reason: None,
                        registered_title: Some(title),
                    },
                    Err(e) => BibliographyEntryStatus {
                        index,
                        reference: entry.clone(),
                        verified: false,
                        doi: Some(doi),
                        reason: Some(format!("Crossref did not resolve this DOI: {e}")),
                        registered_title: None,
                    },
                },
            };
            statuses.push(status);
        }
        statuses
    }

    /// Resolve a single DOI against Crossref, returning the registered title.
    async fn verify(&self, doi: &str) -> Result<String, ResearchError> {
        let url = format!(
            "https://api.crossref.org/works/{}",
            urlencoding::encode(doi)
        );
        let record: Value = crate::research::http::get_json(PROVIDER, &url).await?;
        registered_title(&record).ok_or_else(|| ResearchError::NoResults {
            query: format!("doi:{doi}"),
        })
    }
}

impl Default for ValidateBibliographyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for ValidateBibliographyProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        Box::pin(async move {
            // The adapter takes a structured bibliography rather than a query
            // string, so the trait path reports that plainly instead of
            // fabricating a result from the query text.
            Err(ResearchError::ProviderError {
                provider: PROVIDER.to_string(),
                message: format!(
                    "validate_bibliography takes a bibliography argument, not a free-text query (received: {query})"
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
