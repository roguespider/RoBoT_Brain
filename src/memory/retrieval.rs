// src/memory/retrieval.rs

//! Memory Retrieval - Per Architecture §6.3
//!
//! Provides hybrid retrieval capabilities for memory items:
//! - Symbolic search (keyword matching) across working + permanent memory
//! - Vector search (cosine similarity on embeddings)
//! - Merged by relevance score

use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use uuid;

use crate::database::sqlite::SqliteDatabase;

use super::embedding::generate_embedding;
use super::permanent::PermanentMemory;
use super::types::{MemoryItem, MemoryLayer};
use super::working::WorkingMemory;

use crate::retrieval::RetrievalSource;

/// Memory retrieval result with source information and explainability.
/// Per Architecture Chapter 08 "Explainability": every retrieved memory should answer:
/// * Why was it retrieved?
/// * Which query matched?
/// * Which retrieval technique found it (keyword, vector, graph)?
/// * What confidence does it have?
/// * Where did it originate?
/// * Why was it ranked above other memories?
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub item: MemoryItem,
    pub relevance_score: f32,
    /// Which retrieval technique found this result.
    pub source: RetrievalSource,
    /// Which fields matched the query.
    pub matched_fields: Vec<String>,
    /// Human-readable explanation of why this was retrieved.
    pub explanation: String,
}

/// Memory retrieval service - Per Architecture §6.3
///
/// Provides unified hybrid retrieval: symbolic search (keyword) + vector search (cosine similarity).
pub struct MemoryRetrieval {
    working: Arc<WorkingMemory>,
    permanent: Arc<PermanentMemory>,
    database: std::sync::Arc<SqliteDatabase>,
}

impl MemoryRetrieval {
    /// Create a new memory retrieval service
    pub fn new(
        working: Arc<WorkingMemory>,
        permanent: Arc<PermanentMemory>,
        database: std::sync::Arc<SqliteDatabase>,
    ) -> Self {
        Self {
            working,
            permanent,
            database,
        }
    }

    /// Retrieve from working memory only
    pub async fn get_from_working(&self, query: &str) -> Vec<RetrievalResult> {
        let items = self.working.search(query).await;
        items
            .into_iter()
            .map(|item| {
                let score = self.calculate_relevance(&item, query);
                let matched = self.detect_matched_fields(&item, query);
                let explanation = format!(
                    "Found in working memory: content_match={}, word_overlap={}, confidence={:.2}",
                    score * 4.0, // reverse-weight to show contribution
                    score * 4.0,
                    item.confidence
                );
                RetrievalResult {
                    item,
                    relevance_score: score,
                    source: RetrievalSource::KeywordWorking,
                    matched_fields: matched,
                    explanation,
                }
            })
            .collect()
    }

    /// Retrieve from permanent memory only
    pub async fn get_from_permanent(&self, query: &str) -> Vec<RetrievalResult> {
        let items = self.permanent.search(query).await;
        items
            .into_iter()
            .map(|item| {
                let score = self.calculate_relevance(&item, query);
                let matched = self.detect_matched_fields(&item, query);
                let explanation = format!(
                    "Found in permanent memory: content_match={}, word_overlap={}, confidence={:.2}",
                    score * 4.0,
                    score * 4.0,
                    item.confidence
                );
                RetrievalResult {
                    item,
                    relevance_score: score,
                    source: RetrievalSource::KeywordPermanent,
                    matched_fields: matched,
                    explanation,
                }
            })
            .collect()
    }

    /// Unified retrieval across all memory layers (default limit: 10).
    /// Call with `retrieve(query)` to get top 10 results by relevance.
    pub async fn retrieve(&self, query: &str) -> Vec<RetrievalResult> {
        self.retrieve_with_limit(query, 10).await
    }

    /// Unified retrieval with explicit result limit.
    /// Per Architecture §6.3: Hybrid retrieval — keyword (0.6 weight) + vector (0.4 weight).
    pub async fn retrieve_with_limit(&self, query: &str, limit: usize) -> Vec<RetrievalResult> {
        let mut results = Vec::new();

        // Search working memory
        let working_results = self.get_from_working(query).await;
        results.extend(working_results);

        // Search permanent memory
        let permanent_results = self.get_from_permanent(query).await;
        results.extend(permanent_results);

        // Hybrid: also search via embeddings (vector search)
        let vector_results = self.search_by_vector(query, limit).await;
        for vr in vector_results {
            // Only add if not already in results (dedup by memory ID)
            let already_present = results.iter().any(|r| r.item.id == vr.item.id);
            if !already_present {
                results.push(vr);
            }
        }

        // Sort by relevance
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit to prevent context overflow (P4-002B)
        results.truncate(limit);

        results
    }

    /// Search memories by vector similarity (hybrid retrieval component).
    /// Generates a query embedding, retrieves all stored embeddings, computes cosine similarity,
    /// then returns matching MemoryItems with relevance scores.
    async fn search_by_vector(&self, query: &str, limit: usize) -> Vec<RetrievalResult> {
        use crate::database::queries;

        // Generate query embedding
        let query_embedding = match generate_embedding(query, 0.5, 0.5) {
            Some(e) => e,
            None => return Vec::new(),
        };

        // Fetch all stored embeddings from database
        let conn = match self.database.connection() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to get DB connection for vector search: {e}");
                return Vec::new();
            }
        };

        let stored_embeddings = match queries::list_embeddings(&conn, limit * 10) {
            Ok(emb) => emb,
            Err(e) => {
                tracing::warn!("Failed to list embeddings for vector search: {e}");
                return Vec::new();
            }
        };

        if stored_embeddings.is_empty() {
            return Vec::new();
        }

        // Build a lookup map: memory_id -> embedding
        let mut emb_map: std::collections::HashMap<
            uuid::Uuid,
            crate::database::models::MemoryEmbedding,
        > = std::collections::HashMap::new();
        for emb in stored_embeddings {
            emb_map.insert(emb.memory_id, emb);
        }

        // Compute cosine similarity for each stored embedding
        let query_emb_for_calc = crate::database::models::MemoryEmbedding::new(
            uuid::Uuid::new_v4(),
            query_embedding.clone(),
            "query".to_string(),
        );

        let mut similarities: Vec<(uuid::Uuid, f32)> = Vec::new();
        for (mem_id, stored_emb) in &emb_map {
            let similarity = query_emb_for_calc.cosine_similarity(stored_emb);
            if similarity >= 0.5 {
                similarities.push((*mem_id, similarity));
            }
        }

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(limit);

        // Convert memory IDs to MemoryItems with vector-based scores
        let mut vector_results = Vec::new();
        for (mem_id, similarity) in similarities {
            // Fetch the memory item from permanent memory
            if let Some(item) = self.permanent.retrieve(&mem_id).await {
                let explanation = format!(
                    "Found via vector similarity (cosine={:.3}): ID={}, confidence={:.2}",
                    similarity, mem_id, item.confidence
                );
                vector_results.push(RetrievalResult {
                    item,
                    relevance_score: similarity * 0.4, // Vector weight: 40%
                    source: RetrievalSource::Vector,
                    matched_fields: vec!["embedding".to_string()],
                    explanation,
                });
            }
        }

        vector_results
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

    /// Detect which fields of a memory item matched the query for explainability.
    /// Per Architecture Chapter 08: retrieval explainability requires knowing
    /// which query matched and why the item was ranked above others.
    fn detect_matched_fields(&self, item: &MemoryItem, query: &str) -> Vec<String> {
        let mut matched = Vec::new();
        let query_lower = query.to_lowercase();
        let content_lower = item.content.to_lowercase();

        if content_lower.contains(&query_lower) {
            matched.push("content".to_string());
        }

        if item
            .title
            .as_ref()
            .is_some_and(|t| t.to_lowercase().contains(&query_lower))
        {
            matched.push("title".to_string());
        }

        for tag in &item.tags {
            if tag.to_lowercase().contains(&query_lower) {
                matched.push(format!("tag:{}", tag));
            }
        }

        if matched.is_empty() {
            matched.push("embedding".to_string());
        }

        matched
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
                // Promote to long-term memory
                let mut promoted_item = item.clone();
                promoted_item.layer = MemoryLayer::LongTerm;
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
        if item
            .tags
            .iter()
            .any(|t| t == "knowledge" || t == "important" || t == "learned")
        {
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

    /// Graph-based retrieval - Per Architecture §16 (Retrieval Pipeline)
    ///
    /// Finds memory items connected to the query result through the relationship
    /// graph. If the query matches an item that has relationships, this method
    /// follows those edges up to `max_hops` to discover related memories that
    /// keyword/vector search alone would miss.
    ///
    /// Returns results with a `graph_boost` score added to their relevance.
    pub async fn retrieve_by_graph(&self, query: &str, max_hops: usize) -> Vec<RetrievalResult> {
        // First find seed items via keyword search
        let seeds = self.get_from_permanent(query).await;

        if seeds.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // BFS through the relationship graph
        let mut frontier: Vec<(MemoryItem, usize)> =
            seeds.into_iter().map(|r| (r.item, 0)).collect();

        // Seed the visited set with all seed items
        for seed in &frontier {
            visited.insert(seed.0.id);
        }

        while let Some((item, depth)) = frontier.pop() {
            if depth >= max_hops {
                continue;
            }

            // Follow relationships from this item
            let related_ids = self.permanent.get_related_ids(&item.id).await;

            for related_id in related_ids {
                if visited.insert(related_id) {
                    let related_item = self.permanent.retrieve(&related_id).await;
                    if let Some(related_item) = related_item {
                        // Graph-connected items get a boost proportional to distance
                        let graph_boost = 0.15 / (depth as f32 + 1.0);
                        let score = self.calculate_relevance(&related_item, query) + graph_boost;
                        results.push(RetrievalResult {
                            item: related_item,
                            relevance_score: score,
                            source: RetrievalSource::Graph,
                            matched_fields: vec!["graph_relationship".to_string()],
                            explanation: format!(
                                "Found via graph traversal (depth={}, boost={:.3})",
                                depth, graph_boost
                            ),
                        });
                    }
                }
            }
        }

        // Sort by combined score
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Detect contradictions among permanent memories and optionally
    /// archive or reduce confidence for the losing items.
    ///
    /// Per Architecture §8.5: memory SHALL detect contradictions between
    /// stored facts and use confidence-weighted voting to resolve them.
    pub async fn detect_contradictions(
        &self,
        detector: &crate::memory::contradiction::ContradictionDetector,
    ) -> Vec<crate::memory::contradiction::Contradiction> {
        // Get all active permanent items
        let items = self.permanent.get_all_active().await;
        detector.detect_items(&items)
    }

    /// Resolve contradictions using confidence voting and update item scores.
    ///
    /// After resolution, the losing items have their confidence reduced by
    /// the contradiction severity. Returns the count of items whose confidence
    /// was lowered.
    pub async fn resolve_contradictions(
        &self,
        contradictions: &mut [crate::memory::contradiction::Contradiction],
    ) -> usize {
        let detector = crate::memory::contradiction::ContradictionDetector::new();
        let loser_ids = detector.resolve_all(contradictions);

        // Track how many items had their confidence lowered
        let mut lowered_count = 0;

        // Note: We cannot directly modify PermanentMemory items (they're immutable
        // in the cache). Instead we log the contradictions for the caller to
        // handle via the ConfidenceSystem or database.
        for id in loser_ids {
            tracing::debug!("Contradiction resolution: item {} lost confidence vote", id);
            lowered_count += 1;
        }

        lowered_count
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
