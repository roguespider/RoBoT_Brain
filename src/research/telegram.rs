//! Telegram adapter (research_engine.md Tool Catalog: `search_telegram`).
//!
//! Backed by the public web preview of a channel at `t.me/s/<channel>`, which
//! needs no API key and is server-rendered. The preview is parsed with the
//! `scraper` crate: the message text lives in `div.tgme_widget_message_text`
//! and each message links back to its own permalink, so every citation resolves
//! to the exact message it reports (Rule 3 "Source Provenance").

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use scraper::{Html, Selector};

const PROVIDER: &str = "telegram";

/// Build the public preview URL for a channel.
///
/// Channel handles are written `@name` on the public side and `name` in URLs;
/// the leading `@` is stripped and any path prefix is discarded.
fn preview_url(channel: &str) -> Option<String> {
    let handle = channel
        .trim()
        .trim_start_matches('@')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim();
    if handle.is_empty() {
        return None;
    }
    Some(format!("https://t.me/s/{}", urlencoding::encode(handle)))
}

/// Parse the channel preview into search results.
fn parse_preview(html: &str, channel: &str) -> Vec<SearchResult> {
    let document = Html::parse_document(html);
    // A malformed selector here would be a programming error in a static
    // string, so the parse result is matched rather than unwrapped.
    let (Ok(message_selector), Ok(text_selector), Ok(link_selector)) = (
        Selector::parse("div.tgme_widget_message"),
        Selector::parse("div.tgme_widget_message_text"),
        Selector::parse("a.tgme_widget_message_date"),
    ) else {
        return Vec::new();
    };

    let messages: Vec<_> = document.select(&message_selector).collect();
    let total = messages.len();
    let mut accepted = 0usize;
    let mut results = Vec::new();
    for message in messages {
        let url = message
            .select(&link_selector)
            .next()
            .and_then(|node| node.value().attr("href"))
            .map(|href| format!("https://t.me{href}"))
            .filter(|url| crate::research::sanitize::is_citable_url(url));
        let Some(url) = url else { continue };
        let body = message
            .select(&text_selector)
            .next()
            .map(|node| node.text().collect::<Vec<_>>().join(" "))
            .unwrap_or_default();
        let text = crate::research::sanitize::truncate_with_marker(
            &crate::research::sanitize::sanitize_text(&body),
            900,
        )
        .0;
        if text.trim().is_empty() {
            continue;
        }
        // The preview is ordered newest-first, so the accepted sequence order
        // carries the rank (R5).
        let relevance = crate::research::sanitize::positional_relevance(accepted, total);
        accepted += 1;
        let preview = crate::research::sanitize::truncate_with_marker(&text, 120).0;
        results.push(crate::research::sanitize::build_result(
            &format!("Message in {channel}: {preview}"),
            &url,
            &text,
            relevance,
            SearchSource::Web,
        ));
    }
    results
}

pub struct TelegramProvider;

impl TelegramProvider {
    pub fn new() -> Self {
        Self
    }

    /// Read the most recent public messages from a channel.
    pub async fn fetch(
        &self,
        channel: &str,
        max_messages: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let url = preview_url(channel).ok_or_else(|| ResearchError::NoResults {
            query: format!("telegram:{channel}"),
        })?;
        let html = crate::research::http::get_text(PROVIDER, &url).await?;
        let mut results = parse_preview(&html, channel.trim());
        // The preview ignores a message count, so the cap is applied locally.
        results.truncate(max_messages.clamp(1, 100));
        crate::research::http::require_results(PROVIDER, channel, results)
    }
}

impl Default for TelegramProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for TelegramProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = TelegramProvider::new();
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
