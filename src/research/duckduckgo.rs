#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct DuckDuckGoProvider {
    client: reqwest::Client,
}

#[cfg(feature = "http")]
impl DuckDuckGoProvider {
    pub fn new() -> Result<Self, ResearchError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| ResearchError::ProviderError {
                provider: "duckduckgo".into(),
                message: e.to_string(),
            })?;
        Ok(Self { client })
    }

    async fn fetch_html(&self, query: &str) -> Result<String, ResearchError> {
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );
        let resp =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| ResearchError::ProviderError {
                    provider: "duckduckgo".into(),
                    message: e.to_string(),
                })?;
        let html = resp
            .text()
            .await
            .map_err(|e| ResearchError::ProviderError {
                provider: "duckduckgo".into(),
                message: e.to_string(),
            })?;
        Ok(html)
    }

    fn parse_results(html: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let html_lower = html.to_lowercase();
        for line in html.lines() {
            if line.contains("<a") && line.contains("href=") {
                let url_start = line.find("href=\"").map(|i| i + 6).unwrap_or(0);
                let url_end = line[url_start..]
                    .find("\"")
                    .map(|i| url_start + i)
                    .unwrap_or(url_start);
                let url = line[url_start..url_end].to_string();
                let title_start = line.find("<h2>").map(|i| i + 4).unwrap_or(0);
                let title_end = line[title_start..]
                    .find("</h2>")
                    .map(|i| title_start + i)
                    .unwrap_or(title_start);
                let title = line[title_start..title_end].to_string();
                let snippet_start = line.find("<span>").map(|i| i + 6).unwrap_or(0);
                let snippet_end = line[snippet_start..]
                    .find("</span>")
                    .map(|i| snippet_start + i)
                    .unwrap_or(snippet_start);
                let snippet = line[snippet_start..snippet_end].to_string();
                if !url.is_empty() || !title.is_empty() {
                    results.push(SearchResult {
                        title,
                        url,
                        snippet,
                        relevance: 0.5,
                        source: SearchSource::Web,
                    });
                }
            }
        }
        results
    }
}

#[cfg(feature = "http")]
impl SearchProvider for DuckDuckGoProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let provider = self;
        let query = query.to_string();
        Box::pin(async move {
            let html = provider.fetch_html(&query).await?;
            let results = Self::parse_results(&html);
            Ok(SearchResults {
                results,
                provider: provider.name().to_string(),
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
