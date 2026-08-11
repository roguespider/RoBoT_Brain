// src/memory/retrieval.rs

//! Memory Retrieval - Per Architecture §6.3
//!
//! Provides retrieval capabilities for memory items across both
//! working and permanent memory layers.

use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;

use crate::database::sqlite::SqliteDatabase;

use super::permanent::PermanentMemory;
use super::types::{MemoryItem, MemoryLayer};
use super::working::WorkingMemory;

/// Memory retrieval result with source information
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub item: MemoryItem,
    pub relevance_score: f32,
}

/// Memory retrieval service - Per Architecture §6.3
///
/// Provides unified retrieval across working and permanent memory.
pub struct MemoryRetrieval {
    working: Arc<WorkingMemory>,
    permanent: Arc<PermanentMemory>,
}

impl MemoryRetrieval {
    /// Create a new memory retrieval service
    pub fn new(working: Arc<WorkingMemory>, permanent: Arc<PermanentMemory>) -> Self {
        Self { working, permanent }
    }

    /// Retrieve from working memory only
    pub async fn get_from_working(&self, query: &str) -> Vec<RetrievalResult> {
        let items = self.working.search(query).await;
        items
            .into_iter()
            .map(|item| RetrievalResult {
                relevance_score: self.calculate_relevance(&item, query),
                item,            })
            .collect()
    }

    /// Retrieve from permanent memory only
    pub async fn get_from_permanent(&self, query: &str) -> Vec<RetrievalResult> {
        let items = self.permanent.search(query).await;
        items
            .into_iter()
            .map(|item| RetrievalResult {
                relevance_score: self.calculate_relevance(&item, query),
                item,            })
            .collect()
    }

    /// Unified retrieval across all memory layers
    pub async fn retrieve(&self, query: &str) -> Vec<RetrievalResult> {
        let mut results = Vec::new();

        // Search working memory
        let working_results = self.get_from_working(query).await;
        results.extend(working_results);

        // Search permanent memory
        let permanent_results = self.get_from_permanent(query).await;
        results.extend(permanent_results);

        // Sort by relevance
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Get context from memory (recent working items)
    pub async fn get_context(&self, limit: usize) -> Vec<MemoryItem> {
        let mut items = self.working.get_all().await;
        items.sort_by_key(|b| std::cmp::Reverse(b.accessed_at));
        items.truncate(limit);
        items
    }

    /// Calculate relevance score for a memory item
    fn calculate_relevance(&self, item: &MemoryItem, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let content_lower = item.content.to_lowercase();

        // Base score from content match
        let content_match = if content_lower.contains(&query_lower) {
            1.0
        } else {
            0.0
        };

        // Word overlap score
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();
        
        let mut matches = 0.0;
        for qw in &query_words {
            for cw in &content_words {
                if cw.contains(qw) || qw.contains(cw) {
                    matches += 1.0;
                    break;
                }
            }
        }
        let word_score = if !query_words.is_empty() {
            matches / query_words.len() as f32
        } else {
            0.0
        };

        // Confidence contribution
        let confidence_score = item.confidence;

        // Importance contribution
        let importance_score = item.importance;

        // Access recency (more recent = higher score)
        let now = Utc::now();
        let age_hours = (now - item.accessed_at).num_hours() as f32;
        let recency_score = (1.0 / (1.0 + age_hours / 24.0)).min(1.0);

        // Weighted combination
        (content_match * 0.25)
            + (word_score * 0.25)
            + (confidence_score * 0.2)
            + (importance_score * 0.15)
            + (recency_score * 0.15)
    }

    /// Get reference to working memory
    pub fn working_memory(&self) -> &Arc<WorkingMemory> {
        &self.working
    }

    /// Get reference to permanent memory
    pub fn permanent_memory(&self) -> &Arc<PermanentMemory> {
        &self.permanent
    }

    /// Consolidate memories between working and permanent memory layers
    /// Per Architecture §6.3: Moves high-value memories from Working to Permanent Memory
    pub async fn consolidate(&self) -> ConsolidationStats {
        let mut stats = ConsolidationStats::default();
        
        // Get all items from working memory
        let working_items = self.working.get_all().await;
        
        for item in working_items {
            // Evaluate for promotion based on criteria
            let should_promote = self.should_promote(&item).await;
            
            if should_promote {
                // Promote to permanent memory
                let mut promoted_item = item.clone();
                promoted_item.layer = MemoryLayer::Permanent;
                promoted_item.last_consolidated = Some(Utc::now());
                
                self.permanent.store(promoted_item).await;
                self.working.remove(&item.id).await;
                stats.promoted += 1;
            } else {
                stats.kept += 1;
            }
        }
        
        stats
    }

    /// Check if a memory item should be promoted to permanent memory
    async fn should_promote(&self, item: &MemoryItem) -> bool {
        // Promote if high confidence (>= 0.7)
        if item.confidence >= 0.7 {
            return true;
        }
        
        // Promote if high importance (>= 0.8)
        if item.importance >= 0.8 {
            return true;
        }
        
        // Promote if frequently accessed (>= 5 accesses)
        if item.access_count >= 5 {
            return true;
        }
        
        // Promote if tagged as knowledge
        if item.tags.iter().any(|t| 
            t == "knowledge" || 
            t == "important" || 
            t == "learned"
        ) {
            return true;
        }
        
        false
    }

    /// Checkpoint all memories to database for persistence
    /// Per Architecture §6.3: SQLite is the final persistence layer
    pub async fn checkpoint_to_database(&self, db: &Arc<SqliteDatabase>) -> Result<()> {
        // Checkpoint working memory
        self.working.checkpoint_to_database(db).await?;
        
        // Checkpoint permanent memory  
        self.permanent.checkpoint_to_database(db).await?;
        
        Ok(())
    }
}

/// Statistics for memory consolidation
#[derive(Debug, Clone, Default)]
pub struct ConsolidationStats {
    pub promoted: usize,
    pub archived: usize,
    pub kept: usize,
    pub deleted: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_working() {
        let working = Arc::new(WorkingMemory::new(100));
        let permanent = Arc::new(PermanentMemory::new(100));
        let retrieval = MemoryRetrieval::new(working.clone(), permanent.clone());

        let item = MemoryItem::new(
            MemoryLayer::Working,
            MemoryType::Context,
            "Python is a great language".to_string(),
            "test".to_string(),
        );
        working.store(item).await;

        let results = retrieval.get_from_working("Python").await;
        assert_eq!(results.len(), 1);
        assert!(results[0].item.content.contains("Python"));
    }

    #[tokio::test]
    async fn test_retrieve_permanent() {
        let working = Arc::new(WorkingMemory::new(100));
        let permanent = Arc::new(PermanentMemory::new(100));
        let retrieval = MemoryRetrieval::new(working.clone(), permanent.clone());

        let mut item = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "Rust is a systems language".to_string(),
            "test".to_string(),
        );
        item.add_tag("rust");
        permanent.store(item).await;

        let results = retrieval.get_from_permanent("Rust").await;
        assert_eq!(results.len(), 1);
        assert!(results[0].item.content.contains("Rust"));
    }

    #[tokio::test]
    async fn test_unified_retrieve() {
        let working = Arc::new(WorkingMemory::new(100));
        let permanent = Arc::new(PermanentMemory::new(100));
        let retrieval = MemoryRetrieval::new(working.clone(), permanent.clone());

        let item1 = MemoryItem::new(
            MemoryLayer::Working,
            MemoryType::Context,
            "Temporary context".to_string(),
            "test".to_string(),
        );
        working.store(item1).await;

        let item2 = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "Knowledge about context".to_string(),
            "test".to_string(),
        );
        permanent.store(item2).await;

        let results = retrieval.retrieve("context").await;
        assert_eq!(results.len(), 2); // Both contain "context"
    }

    #[tokio::test]
    async fn test_confidence_filtering() {
        let working = Arc::new(WorkingMemory::new(100));
        let permanent = Arc::new(PermanentMemory::new(100));
        let retrieval = MemoryRetrieval::new(working.clone(), permanent.clone());

        let mut low_conf = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "Low confidence item".to_string(),
            "test".to_string(),
        );
        low_conf.update_confidence(0.2);
        permanent.store(low_conf).await;

        let mut high_conf = MemoryItem::new(
            MemoryLayer::Permanent,
            MemoryType::Knowledge,
            "High confidence item".to_string(),
            "test".to_string(),
        );
        high_conf.update_confidence(0.9);
        permanent.store(high_conf).await;

        let query = RetrievalQuery {
            query: "item".to_string(),
            memory_types: Vec::new(),
            min_confidence: Some(0.5),
            tags: Vec::new(),
            limit: 10,
        };

        let results = retrieval.retrieve_with_query(&query).await;
        assert_eq!(results.len(), 1);
        assert!(results[0].item.content.contains("High"));
    }
}
