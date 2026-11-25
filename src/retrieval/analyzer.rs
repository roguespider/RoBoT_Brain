//! Query analysis per Architecture Chapter 16.
//! Analyzes queries for intent, complexity, and retrieval strategy.

use crate::retrieval::RetrievalRequest;

/// Analysis result for a retrieval query.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryAnalysis {
    /// Query complexity score (0.0 - 1.0).
    pub complexity: f32,
    /// Suggested retrieval sources.
    pub suggested_sources: Vec<String>,
    /// Whether semantic search is recommended.
    pub semantic_recommended: bool,
    /// Whether graph search is recommended.
    pub graph_recommended: bool,
    /// Whether experience search is recommended.
    pub experience_recommended: bool,
}

impl QueryAnalysis {
    /// Analyze a retrieval request and produce analysis.
    pub fn analyze(request: &RetrievalRequest) -> Self {
        let query = &request.query;
        let lower = query.to_lowercase();
        let word_count = query.split_whitespace().count();

        let complexity = (word_count as f32 / 20.0).min(1.0);

        let mut suggested_sources = Vec::new();
        let mut semantic_recommended = false;
        let mut graph_recommended = false;
        let mut experience_recommended = false;

        if lower.contains("how") || lower.contains("why") || lower.contains("explain") {
            semantic_recommended = true;
            suggested_sources.push("semantic".to_string());
        }

        if lower.contains("relationship")
            || lower.contains("connected")
            || lower.contains("related")
        {
            graph_recommended = true;
            suggested_sources.push("graph".to_string());
        }

        if lower.contains("previous") || lower.contains("before") || lower.contains("past") {
            experience_recommended = true;
            suggested_sources.push("experience".to_string());
        }

        // Default to semantic if nothing specific matched
        if !semantic_recommended && !graph_recommended && !experience_recommended {
            semantic_recommended = true;
            suggested_sources.push("semantic".to_string());
        }

        Self {
            complexity,
            suggested_sources,
            semantic_recommended,
            graph_recommended,
            experience_recommended,
        }
    }
}
