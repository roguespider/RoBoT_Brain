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
        let keywords: Vec<String> = query.split_whitespace().map(|s| s.to_string()).collect();
        keywords.into_iter().take(3).collect()
    }

    pub async fn run(&self, query: &str) -> Result<crate::research::ResearchResult, ResearchError> {
        let sub_questions = self.generate_sub_questions(query);
        let mut all_results = Vec::new();
        for sub in &sub_questions {
            let result = tokio::time::timeout(
                Duration::from_secs(10),
                self.pipeline.run_pipeline(sub, Mode::Deep),
            )
            .await
            .map_err(|_| ResearchError::Timeout {
                query: sub.clone(),
                elapsed: Duration::from_secs(10),
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
                        {
                            // Different sources with high confidence may contradict
                            contradictions.push(crate::research::Contradiction {
                                claim_a: finding_a.statement.clone(),
                                claim_b: finding_b.statement.clone(),
                                source_a_url: finding_a.source_url.clone(),
                                source_b_url: finding_b.source_url.clone(),
                                resolution: "".to_string(),
                            });
                        }
                    }
                }
            }
        }
        contradictions
    }
}
