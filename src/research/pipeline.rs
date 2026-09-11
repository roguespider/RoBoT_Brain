use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::jina::JinaProvider;
use crate::research::provider::SearchProvider;
use std::sync::Arc;
use std::time::Duration;

pub enum Mode {
    Quick,
    Deep,
    Auto,
}

pub struct ResearchPipeline {
    providers: Vec<Arc<dyn SearchProvider>>,
}

impl ResearchPipeline {
    pub fn new(providers: Vec<Arc<dyn SearchProvider>>) -> Self {
        Self { providers }
    }

    pub async fn run_pipeline(
        &self,
        query: &str,
        mode: Mode,
    ) -> Result<crate::research::ResearchResult, ResearchError> {
        // Use mode to select timeout: quick=5s, deep=30s, auto=10s
        let timeout_secs = match mode {
            Mode::Quick => 5,
            Mode::Deep => 30,
            Mode::Auto => 10,
        };
        let search_future = async {
            let mut all_results = Vec::new();
            for provider in &self.providers {
                let result = provider.search(query).await?;
                for r in result.results.into_iter().take(10) {
                    all_results.push(r);
                }
            }
            // Rank by relevance (higher first) and cap at 5
            all_results.sort_by(|a, b| {
                b.relevance
                    .partial_cmp(&a.relevance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let (top_results, _) = crate::research::sanitize::cap_and_truncate(all_results, 5);
            // Build Source entries for the evidence packet
            let sources: Vec<crate::research::Source> = top_results
                .iter()
                .map(|r| crate::research::Source {
                    title: r.title.clone(),
                    url: r.url.clone(),
                    provider: "pipeline".to_string(),
                    query_used: query.to_string(),
                    retrieved_at: chrono::Utc::now(),
                    relevance: r.relevance,
                    content: r.snippet.clone(),
                })
                .collect();
            // Extract content via Jina for findings
            let findings: Vec<crate::research::Finding> = {
                #[cfg(feature = "http")]
                {
                    let jina = JinaProvider::new();
                    let mut findings = Vec::new();
                    for result in &top_results[..top_results.len().min(5)] {
                        if let Ok(text) = jina.extract(&result.url).await {
                            let cleaned = crate::research::sanitize::strip_html(&text);
                            let cleaned = crate::research::sanitize::strip_control_chars(&cleaned);
                            findings.push(crate::research::Finding {
                                statement: cleaned,
                                source_url: result.url.clone(),
                                confidence: result.relevance,
                            });
                        }
                    }
                    findings
                }
                #[cfg(not(feature = "http"))]
                {
                    Vec::new()
                }
            };

            Ok::<
                (
                    Vec<crate::research::provider::SearchResult>,
                    Vec<crate::research::Source>,
                    Vec<crate::research::Finding>,
                ),
                ResearchError,
            >((top_results, sources, findings))
        };
        let timeout_result =
            tokio::time::timeout(Duration::from_secs(timeout_secs), search_future).await;
        let (top_results, sources, findings) = if let Ok(inner) = timeout_result {
            inner?
        } else {
            return Err(ResearchError::Timeout {
                query: query.to_string(),
                elapsed: Duration::from_secs(timeout_secs),
            });
        };
        // Use source count for confidence adjustment
        let source_count = top_results.len();
        let confidence = if source_count >= 5 {
            0.9
        } else if source_count >= 3 {
            0.7
        } else {
            0.5
        };
        Ok(crate::research::ResearchResult {
            question: query.to_string(),
            queries: vec![query.to_string()],
            sources,
            findings,
            contradictions: Vec::new(),
            limitations: Vec::new(),
            confidence,
            retrieved_at: chrono::Utc::now(),
        })
    }
}
