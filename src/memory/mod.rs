// src/memory/mod.rs

//! Memory System - Per Architecture §6.3
//!
//! Memory provides storage and retrieval capabilities.
//!
//! Memory contains multiple layers:
//! - Working Memory: Temporary information used during active tasks
//! - Permanent Memory: Curated knowledge retained after evaluation
//!
//! Per Architecture §6.3:
//! - Working Memory: Short lifespan, high volatility, context focused
//! - Permanent Memory: Indexed, connected, confidence weighted, relationship aware
use crate::memory::types::ResearchProvenance;

pub mod confidence;
pub mod consolidation;
pub mod contradiction;
pub mod dedup;
pub mod embedding;
pub mod hierarchy;
pub mod integrity;
pub mod lifecycle;
pub mod permanent;
pub mod pipeline;
pub mod procedural;
pub mod promotion;
pub mod ranking;
pub mod repository;
pub mod retrieval;
pub mod semantic;
pub mod summarize;
pub mod types;
pub mod working;

pub use confidence::ConfidenceSystem;
pub use consolidation::MemoryConsolidator;
pub use contradiction::{ContradictionDetector, check_new_item_contradictions, detect_and_resolve};
pub use dedup::merge_duplicates;
pub use embedding::generate_embedding;
pub use hierarchy::{MemoryHierarchy, restructure_hierarchy};
pub use integrity::{check_integrity, compute_checksum, detect_duplicates, validate_schema};
pub use lifecycle::{MemoryLifecycle, PromotionGate, can_promote, is_terminal};
pub use permanent::PermanentMemory;
pub use procedural::{ForgettingPolicy, ProceduralMemoryStore, ProceduralRecord};
pub use ranking::rank_score;
pub use ranking::ranked_search;
pub use repository::MemoryRepository;
pub use repository::SqliteMemoryRepository;
pub use retrieval::MemoryRetrieval;
pub use semantic::SemanticMemoryStore;
pub use semantic::SemanticRecord;
pub use summarize::summarize;
pub use working::WorkingMemory;

/// Build the memory item a validated research finding promotes to.
///
/// Kept separate from the gate and the write so both can be reasoned about
/// independently: the gate decides *whether* a finding is worth keeping, this
/// decides *what* is kept, and the repository decides whether it landed.
fn build_research_memory(
    finding: &crate::research::Finding,
    confidence: f32,
    provenance: &ResearchProvenance,
) -> crate::memory::types::MemoryItem {
    // The stored content carries the finding's statement AND its provenance, so
    // the memory is self-describing and independently verifiable: a reader can
    // tell where the claim came from without a second lookup (R12, Rule 3).
    let content = format!(
        "{}\n\n[provenance] url={} provider={} query={} retrieved_at={}",
        finding.statement.trim(),
        provenance.url,
        provenance.provider,
        provenance.query,
        provenance.timestamp
    );
    let mut memory = crate::memory::types::MemoryItem::new(
        crate::memory::types::MemoryLayer::LongTerm,
        crate::memory::types::MemoryType::Knowledge,
        content,
        "research".to_string(),
    );
    memory.confidence = confidence;
    memory.add_tag("research".to_string());
    memory.add_tag("web".to_string());
    memory.add_tag(provenance.provider.clone());
    memory
}

/// Promote a research finding to permanent memory and persist it.
///
/// research_engine.md Rule 8 and R12: research-derived knowledge is NOT stored
/// automatically. It is promoted only when experience validates it, meaning
/// confidence >= 0.7 AND outcome = solved. The gate is enforced here rather than
/// by the caller, so no call site can bypass it.
///
/// The item is written through the same `MemoryRepository` that `store_memory`
/// uses, which makes the promotion durable and retrievable by `search_memory`
/// and `list_memories`. A failed write is reported as `StorageFailed` rather
/// than being logged and dropped: a promotion that never persisted is data loss
/// the caller must be able to see.
pub fn promote_research_finding(
    finding: &crate::research::Finding,
    confidence: f32,
    outcome: &str,
    provenance: ResearchProvenance,
    repository: &dyn crate::memory::repository::MemoryRepository,
) -> Result<String, crate::memory::types::MemoryError> {
    // Gate: confidence >= 0.7 AND outcome = solved
    if confidence < 0.7 {
        return Err(crate::memory::types::MemoryError::InsufficientConfidence);
    }
    let outcome_lower = outcome.to_lowercase();
    if !outcome_lower.contains("solved") && !outcome_lower.contains("success") {
        return Err(crate::memory::types::MemoryError::InsufficientConfidence);
    }

    let memory = build_research_memory(finding, confidence, &provenance);
    let memory_id = memory.id.to_string();

    repository.store(&memory).map_err(|e| {
        // The write failure is logged before it is mapped, because the
        // storage error carries the cause (constraint violation, locked
        // database) that the unit variant alone would discard.
        tracing::error!(
            memory_id = %memory_id,
            provider = %provenance.provider,
            error = %e,
            "Failed to persist a promoted research finding"
        );
        crate::memory::types::MemoryError::StorageFailed
    })?;

    tracing::info!(
        memory_id = %memory_id,
        provider = %provenance.provider,
        source_url = %provenance.url,
        "Research finding promoted to permanent memory"
    );

    Ok(memory_id)
}

/// Integration: retrieve memory for context and optionally promote.
/// Uses `retrieve_for_context` and `promote_to_permanent`.
///
/// Returns (records, promoted_id) where `promoted_id` is `Some` if the item
/// was successfully promoted and `None` if it was not promoted (low confidence
/// or storage error). Errors are logged but do not prevent context retrieval.
pub async fn retrieve_and_promote(
    permanent: &PermanentMemory,
    query: &str,
    budget: usize,
    item: crate::memory::types::MemoryItem,
) -> (
    Vec<crate::common::data_contracts::memory_record::MemoryRecord>,
    Option<String>,
) {
    let records = retrieve_for_context(permanent, query, budget).await;
    let promoted = if item.confidence >= 0.5 {
        match permanent.promote_to_permanent(item).await {
            Ok(id) => {
                tracing::debug!(memory_id = %id, "Memory promoted to permanent storage");
                Some(id)
            }
            Err(e) => {
                tracing::warn!(
                    error_kind = ?e,
                    "Failed to promote memory to permanent storage"
                );
                None
            }
        }
    } else {
        None
    };
    (records, promoted)
}

/// Retrieve memory items for context construction.
/// Returns up to `budget` items matching the query, sorted by relevance.
pub async fn retrieve_for_context(
    permanent: &PermanentMemory,
    query: &str,
    budget: usize,
) -> Vec<crate::common::data_contracts::memory_record::MemoryRecord> {
    let items = permanent.search(query).await;
    items
        .into_iter()
        .take(budget)
        .map(|item| {
            let kind = match item.lifecycle {
                crate::memory::lifecycle::MemoryLifecycle::Working => {
                    crate::common::data_contracts::memory_record::MemoryKind::Working
                }
                crate::memory::lifecycle::MemoryLifecycle::Candidate => {
                    crate::common::data_contracts::memory_record::MemoryKind::Candidate
                }
                crate::memory::lifecycle::MemoryLifecycle::Accepted => {
                    crate::common::data_contracts::memory_record::MemoryKind::Accepted
                }
                crate::memory::lifecycle::MemoryLifecycle::Permanent => {
                    crate::common::data_contracts::memory_record::MemoryKind::Permanent
                }
                crate::memory::lifecycle::MemoryLifecycle::Archived => {
                    crate::common::data_contracts::memory_record::MemoryKind::Archived
                }
            };
            crate::common::data_contracts::memory_record::MemoryRecord {
                id: item.id,
                memory_type: item.memory_type.to_string(),
                kind,
                title: item.title.unwrap_or_default(),
                content: item.content,
                summary: String::new(),
                embedding: None,
                confidence: item.confidence,
                created_at: item.created_at.timestamp(),
                updated_at: item.modified_at.timestamp(),
                relationships: item.related_ids,
                tags: item.tags,
                source: item.source,
                version: "1.0.0".to_string(),
                importance: item.importance,
                access_count: item.access_count,
                metadata: crate::common::data_contracts::metadata::Metadata::default(),
            }
        })
        .collect()
}

// ============================================================
// MEMORY ENGINE — Production wiring layer (FIX 00)
// ============================================================

/// The MemoryEngine is the single production entry point for the memory subsystem.
///
/// It wires together WorkingMemory, PermanentMemory, MemoryRetrieval,
/// MemoryHierarchy, ContradictionDetector, and ConfidenceSystem so that
/// the full memory lifecycle (store → retrieve → promote → consolidate →
/// detect contradictions → decay → archive) is available through one
/// struct with clear method boundaries.
///
/// Per Architecture §8-9: memory SHALL support retention, retrieval,
/// promotion, consolidation, contradiction, decay, and archival.
pub struct MemoryEngine {
    /// Temporary volatile storage.
    pub working: std::sync::Arc<WorkingMemory>,
    /// Curated long-term storage.
    pub permanent: std::sync::Arc<PermanentMemory>,
    /// Database handle (optional — needed for vector search).
    pub database: Option<std::sync::Arc<crate::database::sqlite::SqliteDatabase>>,
    /// Hybrid retrieval service (keyword + vector + graph). Lazily created.
    pub retrieval: Option<std::sync::Arc<MemoryRetrieval>>,
    /// Lifecycle management (promotion, compression, forgetting).
    pub hierarchy: std::sync::Mutex<MemoryHierarchy>,
    /// Contradiction detection and resolution.
    pub contradiction_detector: std::sync::Arc<ContradictionDetector>,
    /// Confidence tracking across domains (wrapped in Mutex for interior mutability).
    pub confidence_system: std::sync::Mutex<ConfidenceSystem>,
    /// Deduplication helper.
    pub dedup: fn(
        Vec<crate::common::data_contracts::memory_record::MemoryRecord>,
    ) -> Vec<crate::common::data_contracts::memory_record::MemoryRecord>,
    /// Summarization helper.
    pub summarize_fn: fn(
        Vec<crate::common::data_contracts::memory_record::MemoryRecord>,
    ) -> crate::common::data_contracts::memory_record::MemoryRecord,
    /// Memory consolidation (merge duplicates, summarize low-confidence).
    pub consolidator: crate::memory::consolidation::MemoryConsolidator,
}

impl MemoryEngine {
    /// Create a new MemoryEngine with all subsystems wired together.
    pub fn new(working: WorkingMemory, permanent: PermanentMemory) -> Self {
        let working_arc = std::sync::Arc::new(working);
        let permanent_arc = std::sync::Arc::new(permanent);

        let engine = Self {
            working: working_arc.clone(),
            permanent: permanent_arc.clone(),
            database: None,
            retrieval: None,
            hierarchy: std::sync::Mutex::new(MemoryHierarchy::new()),
            contradiction_detector: std::sync::Arc::new(ContradictionDetector::new()),
            confidence_system: std::sync::Mutex::new(ConfidenceSystem::new()),
            dedup: merge_duplicates,
            summarize_fn: summarize,
            consolidator: crate::memory::consolidation::MemoryConsolidator::default_config(),
        };

        tracing::info!("MemoryEngine initialized with all lifecycle subsystems wired");
        engine
    }

    /// Merge duplicate memories and summarize low-confidence items.
    pub fn consolidate_items(
        &self,
        items: &[crate::memory::types::MemoryItem],
    ) -> Vec<crate::memory::types::MemoryItem> {
        self.consolidator.merge_duplicates(items)
    }

    /// Wire the database into the retrieval service.
    /// Call this after the database is initialized.
    pub fn init_retrieval(
        &mut self,
        database: std::sync::Arc<crate::database::sqlite::SqliteDatabase>,
    ) {
        self.database = Some(database.clone());
        self.retrieval = Some(std::sync::Arc::new(MemoryRetrieval::new(
            self.working.clone(),
            self.permanent.clone(),
            database.clone(),
        )));
    }

    /// Get the retrieval service, initializing it if not yet done.
    fn get_retrieval(&self) -> Option<&std::sync::Arc<MemoryRetrieval>> {
        self.retrieval.as_ref()
    }

    /// Store a memory item through the full lifecycle:
    /// 1. Check for contradictions against existing memories
    /// 2. Store in working memory
    /// 3. Record confidence
    pub async fn store(
        &self,
        item: crate::memory::types::MemoryItem,
    ) -> Result<String, crate::memory::types::MemoryError> {
        // Check for contradictions
        let existing = self.permanent.get_all_active().await;
        let contradictions = self.contradiction_detector.check_new_item(&item, &existing);
        if !contradictions.is_empty() {
            tracing::warn!(
                item_id = %item.id,
                contradiction_count = contradictions.len(),
                "New memory item has contradictions with existing memories"
            );
        }

        // Store in working memory
        self.working.store(item.clone()).await;

        // Record confidence
        let score = crate::memory::confidence::ConfidenceScore::new(
            crate::memory::confidence::ConfidenceDomain::Fact,
            item.confidence,
            "memory_store",
        );
        if let Ok(mut cs) = self.confidence_system.lock() {
            cs.record("memory_store", score);
        } else {
            tracing::error!("Failed to lock confidence system for recording");
        }

        Ok(item.id.to_string())
    }

    /// Retrieve memories using hybrid search across all layers.
    /// Returns up to `limit` results sorted by relevance.
    /// Returns empty vec if retrieval service is not initialized.
    pub async fn retrieve(&self, query: &str, limit: usize) -> Vec<retrieval::RetrievalResult> {
        match self.get_retrieval() {
            Some(r) => r.retrieve_with_limit(query, limit).await,
            None => {
                tracing::debug!("Retrieval service not initialized; returning empty results");
                Vec::new()
            }
        }
    }

    /// Retrieve memories using graph-based traversal from seed items.
    /// Follows relationships up to `max_hops` from items matching the query.
    /// Returns empty vec if retrieval service is not initialized.
    pub async fn retrieve_by_graph(
        &self,
        query: &str,
        max_hops: usize,
    ) -> Vec<retrieval::RetrievalResult> {
        match self.get_retrieval() {
            Some(r) => r.retrieve_by_graph(query, max_hops).await,
            None => {
                tracing::debug!("Retrieval service not initialized; returning empty results");
                Vec::new()
            }
        }
    }

    /// Promote a record through the memory lifecycle.
    /// Returns the new layer if promoted, None if thresholds not met.
    pub fn promote_record(&self, id: &str) -> Option<hierarchy::MemoryLayer> {
        if let Ok(mut hierarchy) = self.hierarchy.lock() {
            hierarchy.promote_record(id)
        } else {
            tracing::error!("Failed to lock hierarchy for promotion");
            None
        }
    }

    /// Archive a record by ID.
    pub fn archive_record(&self, id: &str) -> bool {
        if let Ok(mut hierarchy) = self.hierarchy.lock() {
            hierarchy.archive_record(id)
        } else {
            tracing::error!("Failed to lock hierarchy for archival");
            false
        }
    }

    /// Consolidate working memories into permanent storage.
    /// High-value items (confidence >= 0.7, importance >= 0.8, access >= 5)
    /// are promoted automatically.
    /// Returns default stats if retrieval service is not initialized.
    pub async fn consolidate(&self) -> retrieval::ConsolidationStats {
        match self.get_retrieval() {
            Some(r) => r.consolidate().await,
            None => retrieval::ConsolidationStats::default(),
        }
    }

    /// Detect contradictions among all active permanent memories.
    /// Returns a list of detected contradictions.
    /// Returns empty vec if retrieval service is not initialized.
    pub async fn detect_contradictions(&self) -> Vec<contradiction::Contradiction> {
        match self.get_retrieval() {
            Some(r) => r.detect_contradictions(&self.contradiction_detector).await,
            None => Vec::new(),
        }
    }

    /// Resolve detected contradictions using confidence voting.
    /// Returns the count of items whose confidence was lowered.
    /// Returns 0 if retrieval service is not initialized.
    pub async fn resolve_contradictions(
        &self,
        contradictions: &mut [contradiction::Contradiction],
    ) -> usize {
        match self.get_retrieval() {
            Some(r) => r.resolve_contradictions(contradictions).await,
            None => 0,
        }
    }

    /// Apply confidence decay to all memories.
    /// Per Architecture §19: confidence decays over time based on staleness.
    pub fn decay_confidence(&self, decay_rate: f32) {
        if let Ok(mut hierarchy) = self.hierarchy.lock() {
            hierarchy.decay_confidence(decay_rate);
        } else {
            tracing::error!("Failed to lock hierarchy for decay");
        }

        // Also decay confidence system scores
        let hours_since_last_decay = 24; // decay once per day
        if let Ok(mut cs) = self.confidence_system.lock() {
            cs.confidence_decay(decay_rate, hours_since_last_decay);
        } else {
            tracing::error!("Failed to lock confidence system for decay");
        }
    }

    /// Apply forgetting policy: archive low-value, aged memories.
    /// Returns the IDs of forgotten records.
    pub fn apply_forgetting(&self) -> Vec<String> {
        if let Ok(mut hierarchy) = self.hierarchy.lock() {
            hierarchy.apply_forgetting()
        } else {
            tracing::error!("Failed to lock hierarchy for forgetting");
            Vec::new()
        }
    }

    /// Run a full autonomous restructuring cycle:
    /// 1. Recalculate importance scores
    /// 2. Promote high-confidence records
    /// 3. Compress similar low-priority memories
    /// 4. Apply forgetting policy
    ///
    /// Returns the number of records promoted.
    pub fn restructure(&self) -> usize {
        if let Ok(mut hierarchy) = self.hierarchy.lock() {
            restructure_hierarchy(&mut hierarchy)
        } else {
            tracing::error!("Failed to lock hierarchy for restructuring");
            0
        }
    }

    /// Get memory statistics for monitoring.
    pub fn statistics(&self) -> hierarchy::MemoryStatistics {
        if let Ok(hierarchy) = self.hierarchy.lock() {
            hierarchy.statistics()
        } else {
            tracing::error!("Failed to lock hierarchy for statistics");
            hierarchy::MemoryStatistics::default()
        }
    }

    /// Retrieve all active permanent memories as MemoryRecords.
    pub async fn list_memories(
        &self,
    ) -> Vec<crate::common::data_contracts::memory_record::MemoryRecord> {
        let items = self.permanent.get_all_active().await;
        items
            .into_iter()
            .map(|item| {
                let kind = match item.lifecycle {
                    crate::memory::lifecycle::MemoryLifecycle::Working => {
                        crate::common::data_contracts::memory_record::MemoryKind::Working
                    }
                    crate::memory::lifecycle::MemoryLifecycle::Candidate => {
                        crate::common::data_contracts::memory_record::MemoryKind::Candidate
                    }
                    crate::memory::lifecycle::MemoryLifecycle::Accepted => {
                        crate::common::data_contracts::memory_record::MemoryKind::Accepted
                    }
                    crate::memory::lifecycle::MemoryLifecycle::Permanent => {
                        crate::common::data_contracts::memory_record::MemoryKind::Permanent
                    }
                    crate::memory::lifecycle::MemoryLifecycle::Archived => {
                        crate::common::data_contracts::memory_record::MemoryKind::Archived
                    }
                };
                crate::common::data_contracts::memory_record::MemoryRecord {
                    id: item.id,
                    memory_type: item.memory_type.to_string(),
                    kind,
                    title: item.title.unwrap_or_default(),
                    content: item.content,
                    summary: String::new(),
                    embedding: None,
                    confidence: item.confidence,
                    created_at: item.created_at.timestamp(),
                    updated_at: item.modified_at.timestamp(),
                    relationships: item.related_ids,
                    tags: item.tags,
                    source: item.source,
                    version: "1.0.0".to_string(),
                    importance: item.importance,
                    access_count: item.access_count,
                    metadata: crate::common::data_contracts::metadata::Metadata::default(),
                }
            })
            .collect()
    }
}
