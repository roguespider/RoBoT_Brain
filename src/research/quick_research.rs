//! Quick research mode (research_engine.md R6).
//!
//! The bounded fast path: at most 3 sources, a 5-second total budget,
//! relevance-only ranking, and direct construction of the evidence packet.
//!
//! This is a genuine mode rather than a timeout wrapper: it applies its own
//! source cap, skips the sub-question machinery of deep mode, and reports the
//! tighter budget it actually enforced.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::research::errors::ResearchError;
use crate::research::pipeline::{Mode, ResearchPipeline};

/// The hard source ceiling for quick mode (R6: "1-3 sources maximum").
pub const QUICK_MAX_SOURCES: usize = 3;

/// The total wall-clock budget for a quick lookup (R6: under 5 seconds).
pub const QUICK_TIMEOUT: Duration = Duration::from_secs(5);

pub struct QuickMode {
    pipeline: Arc<ResearchPipeline>,
}

impl QuickMode {
    pub fn new(pipeline: Arc<ResearchPipeline>) -> Self {
        Self { pipeline }
    }

    /// Run a bounded quick lookup.
    ///
    /// The pipeline applies the source cap and the mode budget; this wrapper
    /// enforces the outer deadline so a provider that ignores its own timeout
    /// still cannot hold the caller past the quick-mode contract.
    pub async fn run(&self, query: &str) -> Result<crate::research::ResearchResult, ResearchError> {
        let started = Instant::now();
        let run = self.pipeline.run_pipeline(query, Mode::Quick);
        match tokio::time::timeout(QUICK_TIMEOUT, run).await {
            Ok(result) => {
                let mut result = result?;
                // Defence in depth: enforce the quick-mode cap even if the
                // pipeline's own cap is ever relaxed, so this tool's contract
                // holds regardless of the pipeline's configuration.
                if result.sources.len() > QUICK_MAX_SOURCES {
                    let (kept, marker) = truncate_sources(&mut result.sources, QUICK_MAX_SOURCES);
                    result.sources = kept;
                    if !marker.is_empty() && !result.limitations.contains(&marker) {
                        result.limitations.push(marker);
                    }
                }
                let elapsed = started.elapsed();
                tracing::debug!(
                    sources = result.sources.len(),
                    findings = result.findings.len(),
                    elapsed_ms = elapsed.as_millis(),
                    "Quick research completed"
                );
                Ok(result)
            }
            Err(_) => {
                crate::experience::record_research(
                    query.to_string(),
                    Vec::new(),
                    Mode::Quick.as_str().to_string(),
                    started.elapsed(),
                    "failed".to_string(),
                );
                Err(ResearchError::Timeout {
                    query: query.to_string(),
                    elapsed: started.elapsed(),
                })
            }
        }
    }
}

/// Truncate the source list to `max`, producing the R15 truncation marker.
fn truncate_sources(
    sources: &mut Vec<crate::research::Source>,
    max: usize,
) -> (Vec<crate::research::Source>, String) {
    let dropped = sources.len().saturating_sub(max);
    sources.truncate(max);
    let marker = if dropped > 0 {
        format!(
            "Quick mode retained the top {max} sources; {dropped} additional sources were dropped by the quick-mode context cap."
        )
    } else {
        String::new()
    };
    (sources.clone(), marker)
}
