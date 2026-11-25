//! Query understanding and expansion per Architecture Chapter 16.
//! Transforms raw user queries into structured retrieval queries.

use crate::retrieval::RetrievalQuery;

/// Understanding of a parsed query, including intent and key terms.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryUnderstanding {
    /// Original query text.
    pub original: String,
    /// Detected intent category.
    pub intent: String,
    /// Key terms extracted from the query.
    pub key_terms: Vec<String>,
    /// Entities mentioned in the query.
    pub entities: Vec<String>,
    /// Confidence in the understanding (0.0 - 1.0).
    pub confidence: f32,
}

impl QueryUnderstanding {
    /// Create a new understanding from a raw query.
    pub fn new(query: &str) -> Self {
        let key_terms = Self::extract_key_terms(query);
        let entities = Self::extract_entities(query);
        let confidence = Self::compute_confidence(&key_terms);
        Self {
            original: query.to_string(),
            intent: Self::detect_intent(query),
            key_terms: key_terms.clone(),
            entities,
            confidence,
        }
    }

    fn extract_key_terms(query: &str) -> Vec<String> {
        query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= 3)
            .map(|t| t.to_lowercase())
            .collect()
    }

    fn extract_entities(query: &str) -> Vec<String> {
        // Simple entity detection: capitalized words
        query
            .split_whitespace()
            .filter(|w| {
                let chars: Vec<char> = w.chars().collect();
                !chars.is_empty() && chars[0].is_uppercase()
            })
            .map(|w| w.to_string())
            .collect()
    }

    fn detect_intent(query: &str) -> String {
        let lower = query.to_lowercase();
        if lower.contains("how") || lower.contains("what") {
            "informational".to_string()
        } else if lower.contains("find") || lower.contains("search") {
            "navigational".to_string()
        } else {
            "general".to_string()
        }
    }

    fn compute_confidence(key_terms: &[String]) -> f32 {
        if key_terms.is_empty() {
            0.0
        } else {
            (key_terms.len() as f32 / 10.0).min(1.0)
        }
    }
}

/// Build a RetrievalQuery from query understanding.
pub fn build_retrieval_query(understanding: &QueryUnderstanding) -> RetrievalQuery {
    let mut rq = RetrievalQuery::new(&understanding.original);
    for term in &understanding.key_terms {
        rq = rq.with_expanded(vec![term.clone()]);
    }
    rq = rq.with_confidence_threshold(understanding.confidence);
    rq
}
