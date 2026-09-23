use crate::research::errors::ResearchError;
use crate::research::pipeline::{Mode, ResearchPipeline};
use std::sync::Arc;
use std::time::Duration;

pub struct DeepMode {
    pipeline: Arc<ResearchPipeline>,
}

impl DeepMode {
    pub fn new(pipeline: Arc<ResearchPipeline>) -> Self {
        Self { pipeline }
    }

    fn generate_sub_questions(&self, query: &str) -> Vec<String> {
        // Derive 1-3 sub-questions from the main query for deep research.
        // Strategy: split on common question words and create focused sub-queries.
        let base = query.trim();
        let sub_queries = if base.contains("?")
            || base.contains("how")
            || base.contains("why")
            || base.contains("what")
        {
            // Complex query: generate focused sub-questions
            let parts: Vec<&str> = base.split_whitespace().collect();
            if parts.len() > 3 {
                vec![
                    format!("What is {}?", base),
                    format!("How does {} work?", base),
                    format!("Why is {} important?", base),
                ]
            } else {
                vec![
                    base.to_string(),
                    format!("Details about {}", base),
                    format!("Context for {}", base),
                ]
            }
        } else {
            // Simple query: expand with related angles
            vec![
                base.to_string(),
                format!("Overview of {}", base),
                format!("Latest on {}", base),
            ]
        };
        sub_queries
            .into_iter()
            .take(3)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub async fn run(&self, query: &str) -> Result<crate::research::ResearchResult, ResearchError> {
        // Overall timeout for deep research: 60 seconds (Architecture §R7)
        let overall_deadline = tokio::time::Instant::now() + Duration::from_secs(60);
        let sub_questions = self.generate_sub_questions(query);
        let mut all_results = Vec::new();
        for sub in &sub_questions {
            // Enforce overall 60-second deadline for deep research
            let remaining = overall_deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() || remaining.as_secs() < 1 {
                return Err(ResearchError::Timeout {
                    query: sub.clone(),
                    elapsed: Duration::from_secs(60),
                });
            }
            let sub_timeout = std::cmp::min(remaining, Duration::from_secs(60));
            let result =
                tokio::time::timeout(sub_timeout, self.pipeline.run_pipeline(sub, Mode::Deep))
                    .await
                    .map_err(|_| ResearchError::Timeout {
                        query: sub.clone(),
                        elapsed: Duration::from_secs(60),
                    })??;
            all_results.push(result.clone());
        }
        let all_findings: Vec<crate::research::Finding> = all_results
            .iter()
            .flat_map(|r| r.findings.clone())
            .collect();
        let contradictions = self.detect_contradictions(&all_results);
        let total_sources: usize = all_results.iter().map(|r| r.sources.len()).sum();
        let sources = if total_sources > 0 {
            all_results.iter().flat_map(|r| r.sources.clone()).collect()
        } else {
            Vec::new()
        };
        Ok(crate::research::ResearchResult {
            question: query.to_string(),
            queries: sub_questions,
            sources,
            findings: all_findings,
            contradictions,
            limitations: Vec::new(),
            confidence: 0.6,
            retrieved_at: chrono::Utc::now(),
        })
    }

    /// Detect contradictions across research results by comparing findings.
    /// Compares statements from different sources to find conflicting information.
    fn detect_contradictions(
        &self,
        results: &[crate::research::ResearchResult],
    ) -> Vec<crate::research::Contradiction> {
        let mut contradictions = Vec::new();
        // Simple contradiction detection: compare findings across results
        for (i, a) in results.iter().enumerate() {
            for (j, b) in results.iter().enumerate() {
                if i >= j {
                    continue;
                }
                for finding_a in &a.findings {
                    for finding_b in &b.findings {
                        // Check for potentially contradictory statements
                        // by comparing source URLs and finding statements
                        if finding_a.source_url != finding_b.source_url
                            && finding_a.confidence > 0.7
                            && finding_b.confidence > 0.7
                            && !finding_a.statement.is_empty()
                            && !finding_b.statement.is_empty()
                        {
                            // Different sources with high confidence and different content
                            // may contradict; only flag if statements are meaningfully different.
                            let a_lower = finding_a.statement.to_lowercase();
                            let b_lower = finding_b.statement.to_lowercase();
                            if a_lower != b_lower && a_lower.len() > 5 && b_lower.len() > 5 {
                                contradictions.push(crate::research::Contradiction {
                                    claim_a: finding_a.statement.clone(),
                                    claim_b: finding_b.statement.clone(),
                                    source_a_url: finding_a.source_url.clone(),
                                    source_b_url: finding_b.source_url.clone(),
                                    resolution: Some("Requires manual review".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
        contradictions
    }
}
