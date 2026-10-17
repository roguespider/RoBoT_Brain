//! Citation formatting adapter (research_engine.md: `format_citations`).
//!
//! Fetches the authoritative record for a DOI from Crossref and renders it in the
//! requested style. Because the metadata comes from the registration record
//! rather than from the caller's text, a formatted citation reflects what the
//! publisher actually registered (Rule 3 "Source Provenance").
//!
//! Supported styles: `bibtex`, `apa`, `mla`, `chicago`, `ris`.

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResults, SearchSource};
use serde::Deserialize;

const PROVIDER: &str = "format_citations";

/// The citation styles this adapter can render.
pub const SUPPORTED_FORMATS: [&str; 5] = ["bibtex", "apa", "mla", "chicago", "ris"];

#[derive(Debug, Deserialize)]
struct CrossrefWork {
    #[serde(default)]
    message: CrossrefMessage,
}

#[derive(Debug, Deserialize, Default)]
struct CrossrefMessage {
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
    volume: Option<String>,
    #[serde(default)]
    issue: Option<String>,
    #[serde(default)]
    page: Option<String>,
    #[serde(default)]
    publisher: Option<String>,
    #[serde(default)]
    doi: Option<String>,
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

/// Author rendered as `Family, Given`.
fn author_bibtex(author: &CrossrefAuthor) -> String {
    match (author.family.as_deref(), author.given.as_deref()) {
        (Some(family), Some(given)) if !family.is_empty() && !given.is_empty() => {
            format!("{family}, {given}")
        }
        (Some(family), _) if !family.is_empty() => family.to_string(),
        (None, Some(given)) if !given.is_empty() => given.to_string(),
        _ => "Anonymous".to_string(),
    }
}

/// Author rendered as `Given Family`.
fn author_plain(author: &CrossrefAuthor) -> String {
    match (author.family.as_deref(), author.given.as_deref()) {
        (Some(family), Some(given)) if !family.is_empty() && !given.is_empty() => {
            format!("{given} {family}")
        }
        (Some(family), _) if !family.is_empty() => family.to_string(),
        (None, Some(given)) if !given.is_empty() => given.to_string(),
        _ => "Anonymous".to_string(),
    }
}

/// The publication year from Crossref's nested `date-parts` array.
fn year_of(issued: Option<&CrossrefDate>) -> Option<i64> {
    let parts = issued?.date_parts.first()?;
    let year = parts.first().copied()?;
    if (1000..=3000).contains(&year) {
        Some(year)
    } else {
        None
    }
}

fn first_of(list: &Option<Vec<String>>) -> Option<String> {
    list.as_ref()
        .and_then(|v| v.first())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub struct FormatCitationsProvider;

impl FormatCitationsProvider {
    pub fn new() -> Self {
        Self
    }

    /// Fetch a work's registration record by DOI.
    async fn fetch_work(&self, doi: &str) -> Result<CrossrefWork, ResearchError> {
        let normalized = doi
            .trim()
            .trim_start_matches("https://doi.org/")
            .trim_start_matches("http://dx.doi.org/");
        if normalized.is_empty() {
            return Err(ResearchError::NoResults {
                query: "<empty doi>".to_string(),
            });
        }
        let url = format!(
            "https://api.crossref.org/works/{}",
            urlencoding::encode(normalized)
        );
        crate::research::http::get_json(PROVIDER, &url).await
    }

    /// Render a work in the requested citation style.
    pub async fn format(&self, doi: &str, format: &str) -> Result<String, ResearchError> {
        let style = format.trim().to_ascii_lowercase();
        if !SUPPORTED_FORMATS.contains(&style.as_str()) {
            return Err(ResearchError::ProviderError {
                provider: PROVIDER.to_string(),
                message: format!(
                    "unsupported citation format '{format}'; expected one of {}",
                    SUPPORTED_FORMATS.join(", ")
                ),
            });
        }
        let work = self.fetch_work(doi).await?.message;
        let title = first_of(&work.title).unwrap_or_else(|| "Untitled work".to_string());
        let container = first_of(&work.container_title);
        let year = year_of(work.issued.as_ref());
        let authors_bibtex: Vec<String> = work.author.iter().map(author_bibtex).collect();
        let authors_plain: Vec<String> = work.author.iter().map(author_plain).collect();
        let author_field = if authors_bibtex.is_empty() {
            "Anonymous".to_string()
        } else {
            authors_bibtex.join(" and ")
        };
        let year_str = year
            .map(|y| y.to_string())
            .unwrap_or_else(|| "n.d.".to_string());

        Ok(match style.as_str() {
            "bibtex" => {
                let entry_type = match work.work_type.as_deref() {
                    Some("journal-article") => "article",
                    Some("book-chapter") => "incollection",
                    Some("proceedings-article") => "inproceedings",
                    Some("book") | Some("monograph") | Some("reference-book") => "book",
                    _ => "misc",
                };
                let mut fields = vec![
                    format!("  title        = {{{title}}}"),
                    format!("  author       = {{{author_field}}}"),
                ];
                if let Some(container) = container.as_deref() {
                    fields.push(format!("  journal      = {{{container}}}"));
                }
                fields.push(format!("  year         = {{{year_str}}}"));
                if let Some(volume) = work.volume.as_deref().filter(|v| !v.is_empty()) {
                    fields.push(format!("  volume       = {{{volume}}}"));
                }
                if let Some(issue) = work.issue.as_deref().filter(|i| !i.is_empty()) {
                    fields.push(format!("  number       = {{{issue}}}"));
                }
                if let Some(page) = work.page.as_deref().filter(|p| !p.is_empty()) {
                    fields.push(format!("  pages        = {{{page}}}"));
                }
                let resolved_doi = work
                    .doi
                    .as_deref()
                    .map(|d| d.trim_start_matches("https://doi.org/"))
                    .unwrap_or(doi.trim());
                fields.push(format!("  doi          = {{{resolved_doi}}}"));
                format!(
                    "@{entry_type}{{{}\n{}\n}}",
                    cite_key(&title),
                    fields.join(",\n")
                )
            }
            "ris" => {
                let mut lines = vec!["TY  - JOUR".to_string()];
                for author in &authors_plain {
                    lines.push(format!("AU  - {author}"));
                }
                lines.push(format!("TI  - {title}"));
                if let Some(container) = container.as_deref() {
                    lines.push(format!("JO  - {container}"));
                }
                lines.push(format!("PY  - {year_str}"));
                if let Some(volume) = work.volume.as_deref().filter(|v| !v.is_empty()) {
                    lines.push(format!("VL  - {volume}"));
                }
                if let Some(issue) = work.issue.as_deref().filter(|i| !i.is_empty()) {
                    lines.push(format!("IS  - {issue}"));
                }
                if let Some(page) = work.page.as_deref().filter(|p| !p.is_empty()) {
                    lines.push(format!("SP  - {page}"));
                }
                let resolved_doi = work
                    .doi
                    .as_deref()
                    .map(|d| d.trim_start_matches("https://doi.org/"))
                    .unwrap_or(doi.trim());
                lines.push(format!("DO  - {resolved_doi}"));
                lines.push("ER  - ".to_string());
                lines.join("\n")
            }
            "mla" => {
                let author_text = if authors_plain.is_empty() {
                    "Anonymous".to_string()
                } else {
                    authors_plain.join(", ")
                };
                let mut parts = vec![format!("{author_text}. \"{title}.\"")];
                if let Some(container) = container.as_deref() {
                    parts.push(format!("{container}"));
                }
                if let Some(publisher) = work.publisher.as_deref().filter(|p| !p.is_empty()) {
                    parts.push(publisher.to_string());
                }
                parts.push(year_str.clone());
                parts.push(doi.trim().to_string());
                format!("{}.", parts.join(", "))
            }
            "chicago" => {
                let author_text = if authors_plain.is_empty() {
                    "Anonymous".to_string()
                } else {
                    authors_plain.join(", ")
                };
                let mut parts = vec![format!("{author_text}. \"{title}.\"")];
                if let Some(container) = container.as_deref() {
                    parts.push(format!("{container}"));
                }
                parts.push(year_str.clone());
                parts.push(format!("https://doi.org/{}", doi.trim()));
                format!("{}.", parts.join(". "))
            }
            _ => {
                // APA
                let author_text = if authors_plain.is_empty() {
                    "Anonymous".to_string()
                } else if authors_plain.len() == 1 {
                    authors_plain[0].clone()
                } else {
                    format!(
                        "{}, & {}",
                        authors_plain[..authors_plain.len() - 1].join(", "),
                        authors_plain[authors_plain.len() - 1]
                    )
                };
                let mut parts = vec![format!("{author_text} ({year_str}). {title}.")];
                if let Some(container) = container.as_deref() {
                    parts.push(format!("{container}."));
                }
                parts.push(format!("https://doi.org/{}", doi.trim()));
                parts.join(" ")
            }
        })
    }
}

/// A stable citation key derived from the title's first significant words.
fn cite_key(title: &str) -> String {
    let words: Vec<String> = title
        .split_whitespace()
        .filter(|w| w.chars().any(|c| c.is_alphanumeric()))
        .take(3)
        .map(|w| {
            w.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() {
        "untitled".to_string()
    } else {
        words.join("")
    }
}

impl Default for FormatCitationsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for FormatCitationsProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        Box::pin(async move {
            // The adapter takes a DOI and a style rather than a free-text query,
            // so the trait path reports that plainly instead of inventing a
            // result from the query text.
            Err(ResearchError::ProviderError {
                provider: PROVIDER.to_string(),
                message: format!(
                    "format_citations takes a DOI and a format argument, not a free-text query (received: {query})"
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
