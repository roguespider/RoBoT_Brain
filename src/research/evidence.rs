//! Module for research evidence types.
//!
//! Per Architecture Chapter 16 - Retrieval Pipeline.
//!
//! Note: Source, Finding, Contradiction, and ResearchResult are defined
//! in mod.rs and constructed in reference_research_contracts().
//! This module provides serialization helpers for the evidence packet.

/// Serialize a ResearchResult to JSON for MCP tool return.
pub fn serialize_result(result: &crate::research::ResearchResult) -> String {
    serde_json::to_string(result).unwrap_or_default()
}

/// Deserialize a ResearchResult from JSON.
pub fn deserialize_result(json: &str) -> Option<crate::research::ResearchResult> {
    serde_json::from_str(json).ok()
}

/// Build an evidence packet summary for LLM consumption.
pub fn build_evidence_summary(result: &crate::research::ResearchResult) -> String {
    let mut summary = format!(
        "Question: {}\nConfidence: {:.2}\nSources: {}\nFindings: {}\nContradictions: {}\n",
        result.question,
        result.confidence,
        result.sources.len(),
        result.findings.len(),
        result.contradictions.len()
    );
    for finding in &result.findings {
        summary.push_str(&format!(
            "- {} (confidence: {:.2}, source: {})\n",
            finding.statement, finding.confidence, finding.source_url
        ));
    }
    for contradiction in &result.contradictions {
        summary.push_str(&format!(
            "- CONTRADICTION: {} vs {} (resolution: {})\n",
            contradiction.claim_a,
            contradiction.claim_b,
            contradiction.resolution.as_deref().unwrap_or("pending")
        ));
    }
    summary
}
