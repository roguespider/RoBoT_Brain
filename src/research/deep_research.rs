//! Deep research mode (research_engine.md R7).
//!
//! Derives genuinely distinct sub-questions from the query, searches each
//! within a shared deadline, merges the results, and compares findings across
//! iterations so contradictions between rounds surface.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::research::errors::ResearchError;
use crate::research::pipeline::{Mode, ResearchPipeline};

/// Overall budget for a deep research run (R7: up to 60 seconds).
pub const DEEP_TIMEOUT: Duration = Duration::from_secs(60);

/// Maximum sub-questions derived from one query (R7: 1-3 sub-questions).
pub const MAX_SUB_QUESTIONS: usize = 3;

/// Question forms used to decompose a query into distinct angles.
///
/// Each form is a genuine, different question about the same subject — a
/// definition, a mechanism, and an evaluation. They are applied only when the
/// base query is a bare topic, so a query that already asks a specific question
/// is decomposed by its own clauses instead.
const QUESTION_FORMS: [&str; 3] = [
    "definition and core properties of {subject}",
    "how {subject} works and why it behaves as it does",
    "evidence and limitations concerning {subject}",
];

pub struct DeepMode {
    pipeline: Arc<ResearchPipeline>,
}

impl DeepMode {
    pub fn new(pipeline: Arc<ResearchPipeline>) -> Self {
        Self { pipeline }
    }

    /// Derive distinct sub-questions from a query.
    ///
    /// Two strategies, chosen by the shape of the input:
    ///
    /// * A query that is already one or more questions is split on its
    ///   conjunctions, so each part is researched on its own.
    /// * A bare topic is expanded through the three question forms above,
    ///   producing genuinely different angles rather than a restatement.
    ///
    /// Duplicates are removed, so a sub-question is never searched twice.
    pub fn generate_sub_questions(&self, query: &str) -> Vec<String> {
        let base = query.trim();
        if base.is_empty() {
            return Vec::new();
        }

        let clauses: Vec<&str> = base
            .split(" and ")
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .collect();

        // A conjunctive multi-part query yields its parts directly: they are
        // already distinct questions.
        if clauses.len() > 1 {
            return dedupe(clauses.iter().map(|c| c.to_string()).collect());
        }

        // A query that is already a question is searched as asked.
        if base.ends_with('?') {
            return vec![base.to_string()];
        }

        dedupe(
            QUESTION_FORMS
                .iter()
                .map(|form| form.replace("{subject}", base))
                .collect(),
        )
    }

    /// Run deep research over the derived sub-questions.
    pub async fn run(&self, query: &str) -> Result<crate::research::ResearchResult, ResearchError> {
        let started = Instant::now();
        let sub_questions = self.generate_sub_questions(query);
        if sub_questions.is_empty() {
            return Err(ResearchError::NoResults {
                query: "<empty query>".to_string(),
            });
        }

        let mut merged: Vec<crate::research::ResearchResult> = Vec::new();
        let mut limitations: Vec<String> = Vec::new();
        let mut failed_sub_questions = 0usize;
        let mut cancelled = false;

        for sub in &sub_questions {
            let remaining = DEEP_TIMEOUT.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                limitations.push(format!(
                    "The 60-second deep-research budget was exhausted after {} of {} sub-questions.",
                    merged.len(),
                    sub_questions.len()
                ));
                break;
            }
            match tokio::time::timeout(remaining, self.pipeline.run_pipeline(sub, Mode::Deep)).await
            {
                Ok(Ok(result)) => merged.push(result),
                Ok(Err(ResearchError::Cancelled)) => {
                    // A cancelled sub-question ends the run: returning the rounds
                    // completed so far as a finished packet would misrepresent a
                    // partial search as a complete one (R9, Rule 5).
                    cancelled = true;
                    limitations.push(
                        "Deep research was cancelled before all sub-questions completed."
                            .to_string(),
                    );
                    break;
                }
                Ok(Err(e)) => {
                    // One failed sub-question must not discard the rounds that
                    // succeeded; the gap is reported as a limitation.
                    tracing::debug!(sub_question = %sub, error = %e, "Deep research sub-question failed");
                    failed_sub_questions += 1;
                    limitations.push(format!("Sub-question '{sub}' could not be researched: {e}"));
                }
                Err(_) => {
                    failed_sub_questions += 1;
                    limitations.push(format!(
                        "Sub-question '{sub}' exceeded the remaining deep-research budget."
                    ));
                    break;
                }
            }
        }

        if merged.is_empty() {
            return Err(ResearchError::NoResults {
                query: format!("deep:{query}"),
            });
        }
        if cancelled {
            return Err(ResearchError::Cancelled);
        }

        let result = self.merge(query, &sub_questions, merged);
        let mut result = result;
        result.limitations.extend(limitations);
        if failed_sub_questions == sub_questions.len() {
            result.limitations.push(
                "No sub-question returned evidence; this packet reflects no usable sources."
                    .to_string(),
            );
        }
        tracing::debug!(
            sources = result.sources.len(),
            findings = result.findings.len(),
            contradictions = result.contradictions.len(),
            elapsed_ms = started.elapsed().as_millis(),
            "Deep research completed"
        );
        Ok(result)
    }

    /// Merge per-sub-question results into one bounded packet.
    ///
    /// Sources are de-duplicated by URL and re-capped, so a three-round run
    /// cannot exceed the context cap just because it ran three searches.
    fn merge(
        &self,
        query: &str,
        sub_questions: &[String],
        rounds: Vec<crate::research::ResearchResult>,
    ) -> crate::research::ResearchResult {
        // Preserve the strongest sources across rounds, bounded by the cap.
        let mut sources: Vec<crate::research::Source> = Vec::new();
        let mut findings: Vec<crate::research::Finding> = Vec::new();
        let mut seen_urls: BTreeSet<String> = BTreeSet::new();
        let mut seen_findings: BTreeSet<String> = BTreeSet::new();

        for round in &rounds {
            for source in &round.sources {
                if seen_urls.insert(source.url.clone()) {
                    sources.push(source.clone());
                }
            }
            for finding in &round.findings {
                if seen_findings.insert(finding.statement.clone()) {
                    findings.push(finding.clone());
                }
            }
        }

        sources.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let dropped = sources
            .len()
            .saturating_sub(crate::research::sanitize::MAX_SOURCES);
        sources.truncate(crate::research::sanitize::MAX_SOURCES);
        findings.truncate(crate::research::sanitize::MAX_SOURCES);

        let mut limitations: Vec<String> = Vec::new();
        if dropped > 0 {
            limitations.push(format!(
                "Deep research merged sources from {} sub-questions; {dropped} lower-ranked sources were dropped by the context cap.",
                sub_questions.len()
            ));
        }

        let contradictions = crate::research::pipeline::detect_contradictions_for_packet(&findings);
        let confidence =
            crate::research::pipeline::confidence_for_packet(&sources, &findings, &contradictions);

        crate::research::ResearchResult {
            question: query.to_string(),
            queries: sub_questions.to_vec(),
            sources,
            findings,
            contradictions,
            limitations,
            confidence,
            retrieved_at: chrono::Utc::now(),
        }
    }
}

/// Remove duplicates while preserving order.
fn dedupe(items: Vec<String>) -> Vec<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out: Vec<String> = Vec::new();
    for item in items {
        let normalized = item.trim().to_string();
        if normalized.is_empty() {
            continue;
        }
        if seen.insert(normalized.to_lowercase()) {
            out.push(normalized);
        }
    }
    out.truncate(MAX_SUB_QUESTIONS);
    out
}
