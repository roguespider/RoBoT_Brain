// src/knowledge/query.rs
//! Knowledge querying for reasoning and planning


use super::types::{KnowledgeItem, KnowledgeStatus, KnowledgeType};
use serde::{Deserialize, Serialize};

/// Query parameters for searching knowledge
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeQuery {
    /// Text search in statements
    pub text: Option<String>,
    
    /// Filter by knowledge type
    pub knowledge_type: Option<KnowledgeType>,
    
    /// Filter by status
    pub status: Option<KnowledgeStatus>,
    
    /// Filter by minimum confidence
    pub min_confidence: Option<f32>,
    
    /// Filter by tags (any match)
    pub tags: Option<Vec<String>>,
    
    /// Include only mature knowledge
    pub mature_only: bool,
    
    /// Include related knowledge
    pub include_related: bool,
    
    /// Maximum results to return
    pub limit: Option<usize>,
}

/// Result of a knowledge query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeResult {
    /// Matching knowledge items
    pub items: Vec<KnowledgeItem>,
    
    /// Total matches (before limit)
    pub total_matches: usize,
    
    /// Query that produced these results
    pub query: KnowledgeQuery,
}

impl KnowledgeResult {
    /// Create from filtered items
    pub fn new(items: Vec<KnowledgeItem>, query: KnowledgeQuery) -> Self {
        let total_matches = items.len();
        let limit = query.limit.unwrap_or(usize::MAX);
        let items: Vec<_> = items.into_iter().take(limit).collect();
        
        Self {
            items,
            total_matches,
            query,
        }
    }

    /// Check if any results found
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the best (highest confidence) result
    pub fn best(&self) -> Option<&KnowledgeItem> {
        self.items.first()
    }
}

/// Apply query filters to knowledge items
pub fn apply_query(items: &[KnowledgeItem], query: &KnowledgeQuery) -> Vec<KnowledgeItem> {
    items
        .iter()
        .filter(|item| matches_filter(item, query))
        .cloned()
        .collect()
}

/// Check if item matches query filters
fn matches_filter(item: &KnowledgeItem, query: &KnowledgeQuery) -> bool {
    // Text filter
    if let Some(ref text) = query.text {
        if !item.statement.to_lowercase().contains(&text.to_lowercase()) {
            return false;
        }
    }
    
    // Type filter
    if let Some(ref ktype) = query.knowledge_type {
        if &item.knowledge_type != ktype {
            return false;
        }
    }
    
    // Status filter
    if let Some(ref status) = query.status {
        if &item.status != status {
            return false;
        }
    }
    
    // Confidence filter
    if let Some(min) = query.min_confidence {
        if item.overall_confidence() < min {
            return false;
        }
    }
    
    // Tags filter (any match)
    if let Some(ref tags) = query.tags {
        if !tags.is_empty() && !tags.iter().any(|t| item.tags.contains(t)) {
            return false;
        }
    }
    
    // Mature only filter
    if query.mature_only && !item.is_mature() {
        return false;
    }
    
    true
}

/// Rank items by relevance to query
pub fn rank_items(items: Vec<KnowledgeItem>, query: &KnowledgeQuery) -> Vec<KnowledgeItem> {
    let mut ranked: Vec<_> = items.into_iter().map(|item| {
        let score = calculate_relevance(&item, query);
        (score, item)
    }).collect();
    
    ranked.sort_by(|(a, _), (b, _)| b.partial_cmp(a).unwrap());
    ranked.into_iter().map(|(_, item)| item).collect()
}

/// Calculate relevance score for an item given a query
fn calculate_relevance(item: &KnowledgeItem, query: &KnowledgeQuery) -> f32 {
    let mut score = item.overall_confidence();
    
    // Boost for text match
    if let Some(ref text) = query.text {
        let text_lower = text.to_lowercase();
        if item.statement.to_lowercase().contains(&text_lower) {
            score += 0.1;
        }
    }
    
    // Boost for matching type
    if let Some(ref ktype) = query.knowledge_type {
        if &item.knowledge_type == ktype {
            score += 0.15;
        }
    }
    
    // Boost for matching tags
    if let Some(ref tags) = query.tags {
        let matching: usize = tags.iter()
            .filter(|t| item.tags.contains(t))
            .count();
        score += 0.05 * matching as f32;
    }
    
    // Boost for active status
    if item.status == KnowledgeStatus::Active {
        score += 0.1;
    }
    
    // Penalize items needing review
    if item.needs_review() {
        score -= 0.2;
    }
    
    score.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::KnowledgeSource;
    use uuid::Uuid;
    use chrono::Utc;

    fn make_test_item(statement: &str, confidence: f32, status: KnowledgeStatus) -> KnowledgeItem {
        KnowledgeItem {
            id: Uuid::new_v4(),
            statement: statement.to_string(),
            knowledge_type: KnowledgeType::Fact,
            confidence: super::super::types::KnowledgeConfidence::new(confidence),
            status,
            source: KnowledgeSource::User,
            supporting_evidence: vec![],
            contradicting_evidence: vec![],
            relations: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            success_count: 0,
            failure_count: 0,
            tags: vec!["test".to_string()],
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_text_filter() {
        let items = vec![
            make_test_item("Rust is fast", 0.8, KnowledgeStatus::Active),
            make_test_item("Python is easy", 0.7, KnowledgeStatus::Active),
        ];
        
        let query = KnowledgeQuery {
            text: Some("rust".to_string()),
            ..Default::default()
        };
        
        let filtered = apply_query(&items, &query);
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].statement.contains("Rust"));
    }

    #[test]
    fn test_confidence_filter() {
        let items = vec![
            make_test_item("High confidence", 0.9, KnowledgeStatus::Active),
            make_test_item("Low confidence", 0.3, KnowledgeStatus::Active),
        ];
        
        let query = KnowledgeQuery {
            min_confidence: Some(0.7),
            ..Default::default()
        };
        
        let filtered = apply_query(&items, &query);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_ranking() {
        let items = vec![
            make_test_item("Low match", 0.3, KnowledgeStatus::Active),
            make_test_item("High match", 0.9, KnowledgeStatus::Active),
        ];
        
        let query = KnowledgeQuery {
            text: Some("match".to_string()),
            min_confidence: Some(0.2),
            ..Default::default()
        };
        
        let ranked = rank_items(items, &query);
        assert_eq!(ranked[0].statement, "High match");
    }
}
