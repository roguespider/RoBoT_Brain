//! Research pipeline orchestration (research_engine.md R5).
//!
//! Orchestrates: search -> failover -> rank -> cap -> extract -> compare -> evidence.
//!
//! Every stage is bounded (Rule 2 context protection, Rule 5 timeouts) and every
//! failure is reported as a structured `ResearchError` rather than as an empty
//! success (Rule 6). The output is an evidence packet the LLM reasons over; raw
//! provider content never crosses this boundary (Rule 9).

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::research::errors::{CancellationToken, ResearchError};
use crate::research::provider::SearchProvider;

/// Research mode, which selects the stage budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// 1-3 sources, under 5 seconds (R6).
    Quick,
    /// Sub-questions, multi-iteration, up to 60 seconds (R7).
    Deep,
    /// Heuristic selection between quick and deep.
    Auto,
}

impl Mode {
    /// The name recorded in the research experience.
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Quick => "quick",
            Mode::Deep => "deep",
            Mode::Auto => "auto",
        }
    }

    /// The overall wall-clock budget for this mode.
    fn budget(&self) -> Duration {
        match self {
            Mode::Quick => Duration::from_secs(5),
            Mode::Deep => Duration::from_secs(60),
            Mode::Auto => Duration::from_secs(15),
        }
    }

    /// The source cap for this mode (R6 caps quick mode at 3).
    ///
    /// The quick ceiling is the strictest applicable limit; a deployment that
    /// configures a lower global cap gets that lower cap.
    fn source_cap(&self) -> usize {
        let configured = crate::research::get_config().max_sources;
        match self {
            Mode::Quick => configured.min(crate::research::quick_research::QUICK_MAX_SOURCES),
            _ => configured,
        }
    }
}

/// The full provider chain the pipeline may query.
pub struct ResearchPipeline {
    providers: Vec<Arc<dyn SearchProvider>>,
    cancellation: Arc<CancellationToken>,
    started: Instant,
    /// Destination for promoted findings. A validated finding is written through
    /// this repository so the promotion is durable rather than transient (R12).
    memory_repository: Option<Arc<dyn crate::memory::repository::MemoryRepository>>,
}

impl ResearchPipeline {
    pub fn new(providers: Vec<Arc<dyn SearchProvider>>) -> Self {
        Self {
            providers,
            cancellation: Arc::new(CancellationToken::new()),
            started: Instant::now(),
            memory_repository: None,
        }
    }

    /// Attach the memory repository that promoted findings are written to.
    ///
    /// Without one, findings that pass the promotion gate are logged and
    /// counted but not stored; the limitation is surfaced in the packet so the
    /// gap is visible rather than silent.
    pub fn with_memory_repository(
        mut self,
        repository: Arc<dyn crate::memory::repository::MemoryRepository>,
    ) -> Self {
        self.memory_repository = Some(repository);
        self
    }

    /// Whether cancellation has been requested.
    ///
    /// Cancellation is signalled internally on timeout; this reports the token
    /// state so a caller can distinguish a cancelled run from a failed one.
    fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    /// The elapsed wall-clock time since this pipeline was constructed.
    ///
    /// Used as the duration recorded on the research experience, so the
    /// experience log reflects how long research actually took (R13).
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// Query the provider chain, applying failover (R14).
    ///
    /// The first provider that returns results wins. Every failure encountered
    /// along the way is preserved in the returned error when all providers fail,
    /// so the cause is not lost (Rule 6).
    async fn search_with_failover(
        &self,
        query: &str,
        token: &CancellationToken,
    ) -> Result<Vec<crate::research::provider::SearchResult>, ResearchError> {
        if self.providers.is_empty() {
            return Err(ResearchError::ProviderUnavailable {
                provider:
                    "none configured (set RESEARCH_ENABLED=true and provide a reachable provider)"
                        .to_string(),
            });
        }
        let mut failures: Vec<String> = Vec::new();
        for provider in &self.providers {
            if token.is_cancelled() {
                return Err(ResearchError::Cancelled);
            }
            match provider.search(query).await {
                Ok(results) if !results.results.is_empty() => return Ok(results.results),
                Ok(_) => failures.push(format!("{}: returned no results", provider.name())),
                Err(e) => failures.push(format!("{}: {e}", provider.name())),
            }
        }
        // Every provider failed: name each one and its cause rather than
        // collapsing the chain into an opaque "all" (T3-7 / Rule 6).
        Err(ResearchError::ProviderUnavailable {
            provider: format!("all providers failed: {}", failures.join("; ")),
        })
    }

    /// Run the full research pipeline.
    pub async fn run_pipeline(
        &self,
        query: &str,
        mode: Mode,
    ) -> Result<crate::research::ResearchResult, ResearchError> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Err(ResearchError::NoResults {
                query: "<empty query>".to_string(),
            });
        }
        if !crate::research::get_config().is_enabled() {
            return Err(ResearchError::ProviderUnavailable {
                provider: "research is disabled (RESEARCH_ENABLED is set to a false value)"
                    .to_string(),
            });
        }

        let budget = mode.budget();
        let started = Instant::now();
        // The token is moved into the search future so that a timeout can
        // cancel it: dropping the future stops the in-flight provider fetch,
        // which is what Rule 5 ("cancellation must stop all in-flight fetches")
        // requires. A bare `tokio::time::timeout` would abandon the future
        // without signalling the providers.
        let token = Arc::clone(&self.cancellation);
        let search = async move {
            let results = self.search_with_failover(trimmed, &token).await;
            (results, token)
        };

        let outcome = tokio::time::timeout(budget, search).await;
        let (raw_results, _token) = match outcome {
            Ok(pair) => pair,
            Err(_) => {
                // Signal cancellation so any provider work that is still
                // unwinding observes it, then record the failed research so the
                // pattern analysis can see which queries time out (R13).
                self.cancellation.cancel();
                self.record_experience(trimmed, mode, Vec::new(), "failed", started.elapsed());
                return Err(ResearchError::Timeout {
                    query: trimmed.to_string(),
                    elapsed: started.elapsed(),
                });
            }
        };
        let raw_results = raw_results?;

        if self.is_cancelled() {
            self.record_experience(trimmed, mode, Vec::new(), "failed", started.elapsed());
            return Err(ResearchError::Cancelled);
        }

        // Rank by relevance, then apply the context cap. When Jina is configured
        // its reranker performs the ordering (R4); otherwise the pipeline's own
        // relevance ordering applies. The truncation marker is carried into the
        // packet's limitations so the LLM can see what was dropped rather than
        // believing the list is exhaustive (R15).
        let ranked = {
            #[cfg(feature = "http")]
            {
                let jina = crate::research::jina::JinaProvider::new();
                if jina.is_configured() {
                    jina.rerank(raw_results)
                } else {
                    sort_by_relevance(raw_results)
                }
            }
            #[cfg(not(feature = "http"))]
            {
                sort_by_relevance(raw_results)
            }
        };
        let (top_results, truncation_marker) =
            crate::research::sanitize::cap_and_truncate(ranked, mode.source_cap());

        let sources: Vec<crate::research::Source> = top_results
            .iter()
            .map(|r| self.build_source(r, trimmed))
            .collect();

        let findings = self.build_findings(&sources);
        let contradictions = detect_contradictions_for_packet(&findings);

        let mut limitations: Vec<String> = Vec::new();
        if !truncation_marker.is_empty() {
            limitations.push(truncation_marker);
        }

        // Context protection: measure the packet that will actually be sent
        // (R15). The budget is derived from real content length, so the limit
        // fires when the packet genuinely grows past it.
        let token_estimate = crate::research::sanitize::packet_token_estimate(&sources, &findings);
        const TOKEN_BUDGET: usize = 3000;
        if token_estimate > TOKEN_BUDGET {
            limitations.push(format!(
                "Evidence packet estimated at ~{token_estimate} tokens, above the {TOKEN_BUDGET}-token budget; consider narrowing the query."
            ));
        }

        let confidence = confidence_for_packet(&sources, &findings, &contradictions);
        let outcome_label = if confidence >= 0.7 {
            "solved"
        } else {
            "partial"
        };
        if confidence < 0.7 {
            limitations.push(format!(
                "Confidence {confidence:.2} is below the 0.70 threshold, so this evidence will not be promoted to permanent memory."
            ));
        }

        let source_urls: Vec<String> = sources.iter().map(|s| s.url.clone()).collect();
        // The pipeline's own elapsed time is the true research duration, which
        // includes every provider round rather than just the last stage (R13).
        self.record_experience(trimmed, mode, source_urls, outcome_label, self.elapsed());

        // Memory promotion gate (R12, Rule 8): promote to permanent memory
        // ONLY if experience validates it (confidence >= 0.7 AND outcome = solved).
        if confidence >= 0.7 && outcome_label == "solved" {
            let promoted = self.promote_findings(&findings, &sources, confidence, outcome_label);
            tracing::debug!(
                promoted,
                candidates = findings.len(),
                "Research promotion gate evaluated"
            );
        }

        Ok(crate::research::ResearchResult {
            question: trimmed.to_string(),
            queries: vec![trimmed.to_string()],
            sources,
            findings,
            contradictions,
            limitations,
            confidence,
            retrieved_at: chrono::Utc::now(),
        })
    }

    /// Build a `Source` with provenance from a ranked result.
    fn build_source(
        &self,
        result: &crate::research::provider::SearchResult,
        query: &str,
    ) -> crate::research::Source {
        // The provider that produced this result is identified by the result's
        // own source kind, with the pipeline's primary as the fallback for the
        // general web case.
        let provider = match &result.source {
            crate::research::provider::SearchSource::Web => self
                .providers
                .first()
                .map(|p| p.name().to_string())
                .unwrap_or_else(|| "web".to_string()),
            other => format!("{other:?}").to_ascii_lowercase(),
        };
        crate::research::Source {
            title: result.title.clone(),
            url: result.url.clone(),
            provider,
            query_used: query.to_string(),
            retrieved_at: chrono::Utc::now(),
            relevance: result.relevance,
            content: result.snippet.clone(),
        }
    }

    /// Derive findings from the retrieved sources.
    ///
    /// A finding is a source's own claim about the question. The statement is the
    /// sanitized content, and the source URL is the finding's provenance, so a
    /// promoted memory always carries the text it was derived from.
    fn build_findings(&self, sources: &[crate::research::Source]) -> Vec<crate::research::Finding> {
        sources
            .iter()
            .filter(|s| !s.content.trim().is_empty())
            .map(|s| crate::research::Finding {
                statement: crate::research::sanitize::as_quoted_data("SOURCE", &s.content),
                source_url: s.url.clone(),
                confidence: s.relevance,
            })
            .collect()
    }

    /// Record the research experience (R13, Rule 7).
    fn record_experience(
        &self,
        query: &str,
        mode: Mode,
        source_urls: Vec<String>,
        outcome: &str,
        duration: Duration,
    ) {
        crate::experience::record_research(
            query.to_string(),
            source_urls,
            mode.as_str().to_string(),
            duration,
            outcome.to_string(),
        );
    }

    /// Promote validated findings to permanent memory (R12, Rule 8).
    ///
    /// Each promoted memory carries the finding's statement together with its
    /// source URL, provider and timestamp, so a fact promoted from the web always
    /// traces back to the page it came from. Returns the count actually stored so
    /// a promotion gap is visible to the caller.
    fn promote_findings(
        &self,
        findings: &[crate::research::Finding],
        sources: &[crate::research::Source],
        confidence: f32,
        outcome: &str,
    ) -> usize {
        let Some(repository) = self.memory_repository.as_deref() else {
            tracing::warn!(
                findings = findings.len(),
                "Research findings passed the promotion gate but no memory repository is attached, so nothing was stored"
            );
            return 0;
        };

        let mut promoted = 0usize;
        for finding in findings {
            let Some(source) = sources.iter().find(|s| s.url == finding.source_url) else {
                continue;
            };
            let provenance = crate::memory::types::ResearchProvenance {
                url: source.url.clone(),
                provider: source.provider.clone(),
                timestamp: source.retrieved_at,
                query: source.query_used.clone(),
            };
            match crate::memory::promote_research_finding(
                finding, confidence, outcome, provenance, repository,
            ) {
                Ok(memory_id) => {
                    promoted += 1;
                    tracing::info!(
                        memory_id = %memory_id,
                        source_url = %source.url,
                        "Research finding promoted to permanent memory"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        error = ?e,
                        source_url = %source.url,
                        "Memory promotion did not store a research finding"
                    );
                }
            }
        }
        promoted
    }
}

/// Order results by descending relevance, highest first (R5).
fn sort_by_relevance(
    results: Vec<crate::research::provider::SearchResult>,
) -> Vec<crate::research::provider::SearchResult> {
    let mut sorted = results;
    sorted.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted
}

/// Derive overall confidence from the evidence actually gathered.
///
/// Confidence rises with independent sources that carry content and falls when
/// sources disagree, so a single weak source cannot clear the promotion gate.
pub fn confidence_for_packet(
    sources: &[crate::research::Source],
    findings: &[crate::research::Finding],
    contradictions: &[crate::research::Contradiction],
) -> f32 {
    if sources.is_empty() {
        return 0.0;
    }
    let distinct_urls = sources
        .iter()
        .map(|s| s.url.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    let with_content = findings.len();

    // Agreement ratio across independently-sourced findings.
    let pairs = if findings.is_empty() {
        0
    } else {
        findings.len() * findings.len().saturating_sub(1) / 2
    };
    let conflicts = contradictions.len();
    let agreement = if pairs == 0 {
        1.0
    } else {
        1.0 - (conflicts as f32 / pairs as f32)
    };

    // Corroboration: more distinct sources with content means more support.
    let corroboration = (distinct_urls.min(5) as f32 / 5.0).clamp(0.0, 1.0);
    let content_ratio = if sources.is_empty() {
        0.0
    } else {
        with_content as f32 / sources.len() as f32
    };

    let score = 0.5 * corroboration + 0.3 * agreement + 0.2 * content_ratio;
    score.clamp(0.0, 1.0)
}

/// Detect genuine contradictions between high-confidence findings.
///
/// Two findings contradict only when they make opposing claims about the same
/// subject. Requiring an explicit opposition signal — a shared subject plus a
/// negation or a stance marker — is what stops two agreeing sources phrased
/// differently from being reported as a conflict (DoD 8).
///
/// Exposed so deep mode can run the same detection across its merged rounds
/// instead of applying a second, weaker rule.
pub fn detect_contradictions_for_packet(
    findings: &[crate::research::Finding],
) -> Vec<crate::research::Contradiction> {
    let mut contradictions = Vec::new();
    for (i, a) in findings.iter().enumerate() {
        for b in findings.iter().skip(i + 1) {
            if a.source_url == b.source_url {
                continue;
            }
            if a.confidence < 0.7 || b.confidence < 0.7 {
                continue;
            }
            let Some(subject) = shared_subject(&a.statement, &b.statement) else {
                continue;
            };
            let Some(opposition) = detect_opposition(&a.statement, &b.statement, &subject) else {
                continue;
            };
            contradictions.push(crate::research::Contradiction {
                claim_a: a.statement.clone(),
                claim_b: b.statement.clone(),
                source_a_url: a.source_url.clone(),
                source_b_url: b.source_url.clone(),
                resolution: Some(opposition),
            });
        }
    }
    contradictions
}

/// The longest run of shared significant words between two statements.
///
/// A shared subject is the precondition for a contradiction: two claims can only
/// disagree if they are about the same thing.
fn shared_subject(a: &str, b: &str) -> Option<String> {
    let a_words = significant_words(a);
    let b_words = significant_words(b);
    let mut best: Option<String> = None;
    for word in &a_words {
        if b_words.iter().any(|w| w == word) && word.chars().count() >= 5 {
            let candidate = word.clone();
            if best.as_ref().is_none_or(|b| candidate.len() > b.len()) {
                best = Some(candidate);
            }
        }
    }
    best
}

/// Lowercase content words of at least four characters, minus stop words.
fn significant_words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 4)
        .filter(|w| !STOP_WORDS.contains(w))
        .map(|w| w.to_string())
        .collect()
}

/// Words that carry no topical information.
const STOP_WORDS: [&str; 24] = [
    "about", "after", "again", "against", "along", "also", "been", "being", "between", "both",
    "could", "does", "from", "have", "into", "more", "most", "other", "over", "same", "should",
    "such", "than", "there",
];

/// Negation and stance markers that indicate opposing claims.
const OPPOSITION_MARKERS: [&str; 8] = [
    "not",
    "never",
    "no evidence",
    "contradicts",
    "disproves",
    "refutes",
    "however",
    "but",
];

/// Decide whether two statements about a shared subject actually disagree.
///
/// Returns a human-readable description of the opposition, or `None` when the
/// statements do not conflict. The comparison is deliberately conservative:
/// reporting no contradiction is correct, whereas a false contradiction hides
/// agreement and misleads the reasoning step.
fn detect_opposition(a: &str, b: &str, subject: &str) -> Option<String> {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let a_negated = has_negation(&a_lower);
    let b_negated = has_negation(&b_lower);
    let a_numeric = numbers_in(&a_lower);
    let b_numeric = numbers_in(&b_lower);

    // Opposing polarity on the same subject: one affirms, one denies.
    if a_negated != b_negated {
        return Some(format!(
            "Sources disagree on whether '{subject}' holds; the claims use opposing polarity and require manual review."
        ));
    }

    // The same subject with different measured values is a real conflict.
    if let (Some(a_num), Some(b_num)) = (a_numeric, b_numeric) {
        if a_num != b_num {
            return Some(format!(
                "Sources report different values for '{subject}' ({a_num} vs {b_num}); the figures require manual reconciliation."
            ));
        }
    }

    None
}

/// Whether a statement contains an explicit negation or stance marker.
fn has_negation(text: &str) -> bool {
    OPPOSITION_MARKERS
        .iter()
        .any(|marker| text.contains(marker))
}

/// The first numeric token in a statement, if any.
fn numbers_in(text: &str) -> Option<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '.')
        .find(|token| !token.is_empty() && token.chars().any(|c| c.is_ascii_digit()))
        .map(|t| t.to_string())
}
