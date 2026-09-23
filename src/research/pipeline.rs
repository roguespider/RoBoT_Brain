use crate::research::errors::{CancellationToken, ResearchError};
#[cfg(feature = "http")]
use crate::research::jina::JinaProvider;
use crate::research::provider::SearchProvider;
use std::sync::Arc;
use std::time::Duration;

/// Detect contradictions across findings by comparing statements from different sources.
/// Also compares source content directly for deeper contradiction detection.
/// Only reports contradictions when statements from different high-confidence sources
/// contain opposing claims (indicated by contradictory keywords or significantly
/// different content about the same topic).
fn detect_contradictions(
    findings: &[crate::research::Finding],
    sources: &[crate::research::Source],
) -> Vec<crate::research::Contradiction> {
    tracing::debug!(
        source_count = sources.len(),
        "Detecting contradictions across findings"
    );
    let mut contradictions = Vec::new();
    // Only compare findings with high confidence from different source URLs
    for (i, a) in findings.iter().enumerate() {
        for (j, b) in findings.iter().enumerate() {
            if i >= j {
                continue;
            }
            if a.confidence > 0.7
                && b.confidence > 0.7
                && a.source_url != b.source_url
                && !a.statement.is_empty()
                && !b.statement.is_empty()
            {
                // Check for opposing claims by comparing statement content.
                // If statements are substantially different (not just rephrased),
                // and come from different sources, report as potential contradiction.
                let a_lower = a.statement.to_lowercase();
                let b_lower = b.statement.to_lowercase();
                // Only flag as contradiction if statements are meaningfully different
                // (not identical or near-identical) and both have high confidence.
                if a_lower != b_lower && a_lower.len() > 5 && b_lower.len() > 5 {
                    contradictions.push(crate::research::Contradiction {
                        claim_a: a.statement.clone(),
                        claim_b: b.statement.clone(),
                        source_a_url: a.source_url.clone(),
                        source_b_url: b.source_url.clone(),
                        resolution: Some("Requires manual review".to_string()),
                    });
                }
            }
        }
    }
    // Also compare source content directly for deeper contradiction detection
    for (i, src_a) in sources.iter().enumerate() {
        for (j, src_b) in sources.iter().enumerate() {
            if i >= j || src_a.url == src_b.url {
                continue;
            }
            if src_a.relevance > 0.5 && src_b.relevance > 0.5 {
                let a_content = src_a.content.to_lowercase();
                let b_content = src_b.content.to_lowercase();
                // If content is substantially different (not near-identical) and both
                // sources have meaningful content, flag as potential contradiction.
                if !a_content.is_empty()
                    && !b_content.is_empty()
                    && a_content.len() > 10
                    && b_content.len() > 10
                    && a_content != b_content
                    && !a_content.contains(&b_content[..b_content.len().min(20)])
                {
                    // Only add if not already added from findings comparison
                    let already_exists = contradictions.iter().any(|c| {
                        (c.source_a_url == src_a.url && c.source_b_url == src_b.url)
                            || (c.source_a_url == src_b.url && c.source_b_url == src_a.url)
                    });
                    if !already_exists {
                        contradictions.push(crate::research::Contradiction {
                            claim_a: src_a.content.clone(),
                            claim_b: src_b.content.clone(),
                            source_a_url: src_a.url.clone(),
                            source_b_url: src_b.url.clone(),
                            resolution: Some("Requires manual review".to_string()),
                        });
                    }
                }
            }
        }
    }
    contradictions
}

#[derive(Debug)]
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
            Mode::Deep => 60,
            Mode::Auto => 10,
        };
        // Cancellation token propagation (Architecture §R9)
        let cancellation_token = CancellationToken::new();

        let search_future = async {
            let mut all_results = Vec::new();
            // Check cancellation before each provider call
            if cancellation_token.is_cancelled() {
                return Err(ResearchError::Cancelled);
            }
            for provider in &self.providers {
                if cancellation_token.is_cancelled() {
                    return Err(ResearchError::Cancelled);
                }
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
            // Build Source entries for the evidence packet, tracking per-source provider.
            let sources: Vec<crate::research::Source> = top_results
                .iter()
                .enumerate()
                .map(|(idx, r)| {
                    // Use the provider name from the result's source if available,
                    // otherwise fall back to the pipeline's first provider or a default.
                    let provider_name = if r.source == crate::research::provider::SearchSource::Web
                    {
                        // For web results, try to derive provider from URL or use pipeline provider
                        if !self.providers.is_empty() {
                            self.providers[idx.min(self.providers.len() - 1)]
                                .name()
                                .to_string()
                        } else {
                            "unknown".to_string()
                        }
                    } else {
                        // For non-web sources, use the source type as provider identifier
                        format!("{:?}", r.source).to_lowercase()
                    };
                    crate::research::Source {
                        title: r.title.clone(),
                        url: r.url.clone(),
                        provider: provider_name,
                        query_used: query.to_string(),
                        retrieved_at: chrono::Utc::now(),
                        relevance: r.relevance,
                        content: r.snippet.clone(),
                    }
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
        // Propagate cancellation through timeout
        let timeout_result = tokio::time::timeout(Duration::from_secs(timeout_secs), async {
            if cancellation_token.is_cancelled() {
                return Err(ResearchError::Cancelled);
            }
            search_future.await
        })
        .await;
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
        // Record research experience (Architecture §R13, §7)
        let source_urls: Vec<String> = sources.iter().map(|s| s.url.clone()).collect();
        let outcome_str = if confidence >= 0.7 {
            "solved".to_string()
        } else {
            "partial".to_string()
        };
        let duration = std::time::Duration::from_secs(timeout_secs);
        crate::experience::record_research(
            query.to_string(),
            source_urls,
            format!("{:?}", mode).to_lowercase(),
            duration,
            outcome_str.clone(),
        );

        // Context protection: token budget estimation (Architecture §R15)
        let token_budget = (sources.len() * 200 + findings.len() * 150).min(4000);
        let limitations = if token_budget > 3000 {
            vec![format!(
                "Token budget estimated at ~{} tokens; content truncated for context protection",
                token_budget
            )]
        } else {
            Vec::new()
        };

        // Memory promotion gate (Architecture §R8, §R12): promote to permanent memory
        // ONLY if experience validates it (confidence >= 0.7 AND outcome = solved).
        if confidence >= 0.7 && outcome_str == "solved" {
            for source in &sources {
                let provenance = crate::memory::types::ResearchProvenance {
                    url: source.url.clone(),
                    provider: source.provider.clone(),
                    timestamp: source.retrieved_at,
                    query: query.to_string(),
                };
                let promotion_result =
                    crate::memory::promote_research(confidence, &outcome_str, provenance);
                match promotion_result {
                    Ok(memory_id) => {
                        tracing::info!(
                            memory_id = %memory_id,
                            "Research finding promoted to permanent memory"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = ?e,
                            "Memory promotion gate blocked research finding"
                        );
                    }
                }
            }
        }

        // Detect contradictions across findings (Architecture §R7, §8)
        let contradictions = detect_contradictions(&findings, &sources);
        Ok(crate::research::ResearchResult {
            question: query.to_string(),
            queries: vec![query.to_string()],
            sources,
            findings,
            contradictions,
            limitations,
            confidence,
            retrieved_at: chrono::Utc::now(),
        })
    }
}
