use std::sync::Arc;
use std::time::Duration;
use crate::research::pipeline::{ResearchPipeline, Mode};
use crate::research::errors::ResearchError;

pub struct QuickMode {
    pipeline: Arc<ResearchPipeline>,
}

impl QuickMode {
    pub fn new(pipeline: Arc<ResearchPipeline>) -> Self {
        Self { pipeline }
    }

    pub async fn run(&self, query: &str) -> Result<crate::research::ResearchResult, ResearchError> {
        tokio::time::timeout(Duration::from_secs(5), self.pipeline.run_pipeline(query, Mode::Quick)).await.map_err(|_| ResearchError::Timeout { query: query.to_string(), elapsed: Duration::from_secs(5) })?
    }
}
