use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub title: String,
    pub url: String,
    pub provider: String,
    pub query_used: String,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
    pub relevance: f32,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub statement: String,
    pub source_url: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub claim_a: String,
    pub claim_b: String,
    pub source_a_url: String,
    pub source_b_url: String,
    pub resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub question: String,
    pub queries: Vec<String>,
    pub sources: Vec<Source>,
    pub findings: Vec<Finding>,
    pub contradictions: Vec<Contradiction>,
    pub limitations: Vec<String>,
    pub confidence: f32,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
}
