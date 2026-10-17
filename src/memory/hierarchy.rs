//! Memory Hierarchy — Organized memory layers per Architecture §14.
//!
//! Implements:
//! - Memory layers (Sensory, Working, Experience, Episodic, Semantic, Procedural, Graph, Archive)
//! - Memory promotion pipeline
//! - Memory index cards
//! - Confidence model
//! - Importance scoring
//! - Memory compression
//! - Forgetting policy
//! - Statistics
//! - Autonomous restructuring

use serde::{Deserialize, Serialize};

/// Memory layer types per Architecture §14.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryLayer {
    /// Active working memory buffer.
    Working,
    /// Short-term working buffer.
    ShortTerm,
    /// Long-term integrated knowledge.
    LongTerm,
    /// Semantic memory (concepts, facts).
    Semantic,
    /// Episodic memory (specific events).
    Episodic,
    /// Procedural memory (skills, procedures).
    Procedural,
    /// Graph-based memory (relationships, knowledge graph).
    Graph,
    /// Archived historical memory.
    Archive,
}

impl MemoryLayer {
    /// Return the promotion target for this layer, if any.
    pub fn promote_to(&self) -> Option<MemoryLayer> {
        match self {
            MemoryLayer::Working => Some(MemoryLayer::ShortTerm),
            MemoryLayer::ShortTerm => Some(MemoryLayer::LongTerm),
            MemoryLayer::LongTerm => Some(MemoryLayer::Semantic),
            MemoryLayer::Semantic => Some(MemoryLayer::Episodic),
            MemoryLayer::Episodic => Some(MemoryLayer::Procedural),
            MemoryLayer::Procedural => Some(MemoryLayer::Graph),
            MemoryLayer::Graph => Some(MemoryLayer::Archive),
            MemoryLayer::Archive => None,
        }
    }

    /// Return the demotion target for this layer, if any.
    pub fn demote_to(&self) -> Option<MemoryLayer> {
        match self {
            MemoryLayer::ShortTerm => Some(MemoryLayer::Working),
            MemoryLayer::LongTerm => Some(MemoryLayer::ShortTerm),
            MemoryLayer::Semantic => Some(MemoryLayer::LongTerm),
            MemoryLayer::Episodic => Some(MemoryLayer::Semantic),
            MemoryLayer::Procedural => Some(MemoryLayer::Episodic),
            MemoryLayer::Graph => Some(MemoryLayer::Procedural),
            MemoryLayer::Archive => Some(MemoryLayer::Graph),
            MemoryLayer::Working => None,
        }
    }

    /// Return a human-readable name for the layer.
    pub fn name(&self) -> &'static str {
        match self {
            MemoryLayer::Working => "working",
            MemoryLayer::ShortTerm => "short_term",
            MemoryLayer::LongTerm => "long_term",
            MemoryLayer::Semantic => "semantic",
            MemoryLayer::Episodic => "episodic",
            MemoryLayer::Procedural => "procedural",
            MemoryLayer::Graph => "graph",
            MemoryLayer::Archive => "archive",
        }
    }
}

/// Memory index card — lightweight representation per Architecture §14.407.
/// Contains metadata rather than complete content for fast retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryIndexCard {
    /// Card identifier.
    pub id: String,
    /// Title of the memory.
    pub title: String,
    /// Memory layer.
    pub layer: MemoryLayer,
    /// Type of memory.
    pub memory_type: MemoryType,
    /// Summary (not full content).
    pub summary: String,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Importance score (0.0-10.0).
    pub importance: f32,
    /// Source reference.
    pub source: String,
    /// Timestamp.
    pub timestamp: i64,
    /// Related memory IDs.
    pub relationships: Vec<String>,
    /// Embedding vector reference.
    pub embedding_ref: Option<String>,
    /// Storage location (where full content is stored).
    pub storage_location: String,
}

impl MemoryIndexCard {
    /// Create a new memory index card.
    pub fn new(id: &str, title: &str, layer: MemoryLayer, summary: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            layer,
            memory_type: MemoryType::Generic,
            summary: summary.to_string(),
            confidence: 0.5,
            importance: 0.5,
            source: String::new(),
            timestamp: chrono::Utc::now().timestamp(),
            relationships: Vec::new(),
            embedding_ref: None,
            storage_location: String::new(),
        }
    }
}

/// Type of memory content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    /// Factual knowledge.
    Fact,
    /// Specific event or experience.
    Experience,
    /// Episode (group of related experiences).
    Episode,
    /// Skill or procedure.
    Skill,
    /// Concept or relationship.
    Relationship,
    /// Question or uncertainty.
    Question,
    /// Generic/unknown type.
    Generic,
    /// Custom type.
    Custom(String),
}

/// Confidence model per Architecture §14.444.
/// Every memory maintains confidence values that change with experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceModel {
    /// Fact confidence (0.0-1.0).
    pub fact_confidence: f32,
    /// Relationship confidence (0.0-1.0).
    pub relationship_confidence: f32,
    /// Skill confidence (0.0-1.0).
    pub skill_confidence: f32,
    /// Workflow confidence (0.0-1.0).
    pub workflow_confidence: f32,
}

impl Default for ConfidenceModel {
    fn default() -> Self {
        Self {
            fact_confidence: 0.5,
            relationship_confidence: 0.5,
            skill_confidence: 0.5,
            workflow_confidence: 0.5,
        }
    }
}

impl ConfidenceModel {
    /// Update fact confidence based on new evidence.
    pub fn update_fact_confidence(&mut self, new_confidence: f32, evidence_strength: f32) {
        // Weighted average of old and new confidence
        let weight = evidence_strength.clamp(0.0, 1.0);
        self.fact_confidence = self.fact_confidence * (1.0 - weight) + new_confidence * weight;
        self.fact_confidence = self.fact_confidence.clamp(0.0, 1.0);
    }

    /// Decay confidence over time.
    pub fn decay(&mut self, decay_rate: f32, days_elapsed: i64) {
        let decay = 1.0 / (1.0 + decay_rate * days_elapsed as f32);
        self.fact_confidence *= decay;
        self.relationship_confidence *= decay;
        self.skill_confidence *= decay;
        self.workflow_confidence *= decay;
        // Don't let confidence go below 0.01
        self.fact_confidence = self.fact_confidence.max(0.01);
        self.relationship_confidence = self.relationship_confidence.max(0.01);
        self.skill_confidence = self.skill_confidence.max(0.01);
        self.workflow_confidence = self.workflow_confidence.max(0.01);
    }

    /// Get the minimum confidence across all domains.
    pub fn min_confidence(&self) -> f32 {
        [
            self.fact_confidence,
            self.relationship_confidence,
            self.skill_confidence,
            self.workflow_confidence,
        ]
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min)
    }
}

/// Importance scorer per Architecture §14.474.
/// Calculates importance based on multiple factors.
#[derive(Debug, Clone)]
pub struct ImportanceScorer {
    /// Weight for frequency of use.
    pub frequency_weight: f32,
    /// Weight for planner requests.
    pub planner_weight: f32,
    /// Weight for user corrections.
    pub correction_weight: f32,
    /// Weight for success rate.
    pub success_weight: f32,
    /// Weight for age (decaying).
    pub age_weight: f32,
    /// Weight for novelty.
    pub novelty_weight: f32,
    /// Weight for connectedness.
    pub connectedness_weight: f32,
}

impl Default for ImportanceScorer {
    fn default() -> Self {
        Self {
            frequency_weight: 0.25,
            planner_weight: 0.2,
            correction_weight: 0.15,
            success_weight: 0.15,
            age_weight: 0.1,
            novelty_weight: 0.1,
            connectedness_weight: 0.05,
        }
    }
}

/// Parameters for importance calculation.
#[derive(Clone, Copy)]
pub struct ImportanceParams {
    pub access_count: u32,
    pub planner_requests: u32,
    pub user_corrections: u32,
    pub success_rate: f32,
    pub age_hours: u64,
    pub is_novel: bool,
    pub relationship_count: u32,
    pub max_access: u32,
}

impl ImportanceScorer {
    /// Calculate importance score for a memory record.
    pub fn calculate_importance(&self, params: ImportanceParams) -> f32 {
        let freq_score = (params.access_count as f32 / (params.max_access.max(1) as f32))
            .clamp(0.0, 1.0)
            * self.frequency_weight;
        let planner_score = (params.planner_requests as f32
            / (params.planner_requests.max(1) as f32))
            .clamp(0.0, 1.0)
            * self.planner_weight;
        let correction_score = (params.user_corrections as f32
            / ((params.user_corrections.max(1) + 1) as f32))
            .clamp(0.0, 1.0)
            * self.correction_weight;
        let success_score = params.success_rate.clamp(0.0, 1.0) * self.success_weight;
        // Older memories get slightly lower score, but not below 0.3
        let age_score = (1.0 - (params.age_hours as f32 / 720.0).clamp(0.0, 1.0) * 0.7).max(0.3)
            * self.age_weight;
        let novelty_score = if params.is_novel { 1.0 } else { 0.0 } * self.novelty_weight;
        let connected_score = (params.relationship_count as f32
            / (params.relationship_count.max(1) as f32))
            .clamp(0.0, 1.0)
            * self.connectedness_weight;

        freq_score
            + planner_score
            + correction_score
            + success_score
            + age_score
            + novelty_score
            + connected_score
    }
}

/// Memory compression per Architecture §14.550.
/// Compresses repeated information into summaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedMemory {
    /// Original memory IDs that were compressed.
    pub original_ids: Vec<String>,
    /// Compressed summary.
    pub summary: String,
    /// Compressed confidence.
    pub confidence: f32,
    /// Compression timestamp.
    pub compressed_at: i64,
    /// Number of original memories compressed.
    pub compression_count: u32,
}

impl CompressedMemory {
    /// Create a new compressed memory.
    pub fn new(
        original_ids: Vec<String>,
        summary: &str,
        confidence: f32,
        compression_count: u32,
    ) -> Self {
        Self {
            original_ids,
            summary: summary.to_string(),
            confidence,
            compressed_at: chrono::Utc::now().timestamp(),
            compression_count,
        }
    }
}

/// Forgetting policy per Architecture §14.535.
/// Controls which memories are forgotten/compressed/archived.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingPolicy {
    /// Minimum importance to avoid forgetting.
    pub min_importance_to_preserve: f32,
    /// Maximum age in hours before consideration for forgetting.
    pub max_age_hours: u64,
    /// Minimum confidence to preserve.
    pub min_confidence_to_preserve: f32,
    /// Whether to compress before forgetting.
    pub compress_before_forget: bool,
}

impl Default for ForgettingPolicy {
    fn default() -> Self {
        Self {
            min_importance_to_preserve: 0.3,
            max_age_hours: 720, // 30 days
            min_confidence_to_preserve: 0.2,
            compress_before_forget: true,
        }
    }
}

impl ForgettingPolicy {
    /// Check if a memory should be forgotten.
    pub fn should_forget(&self, importance: f32, age_hours: u64, confidence: f32) -> bool {
        importance < self.min_importance_to_preserve
            && age_hours > self.max_age_hours
            && confidence < self.min_confidence_to_preserve
    }
}

/// Memory statistics for monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    /// Total records.
    pub total_records: u32,
    /// Records per layer.
    pub records_by_layer: std::collections::HashMap<String, u32>,
    /// Average confidence.
    pub avg_confidence: f32,
    /// Average importance.
    pub avg_importance: f32,
    /// Number of archived records.
    pub archived_count: u32,
    /// Number of compressed memories.
    pub compressed_count: u32,
}

impl Default for MemoryStatistics {
    fn default() -> Self {
        Self {
            total_records: 0,
            records_by_layer: std::collections::HashMap::new(),
            avg_confidence: 0.0,
            avg_importance: 0.0,
            archived_count: 0,
            compressed_count: 0,
        }
    }
}

/// A memory record in the hierarchy with promotion/demotion tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HierarchyRecord {
    /// Record identifier.
    pub id: String,
    /// Memory layer.
    pub layer: MemoryLayer,
    /// Content summary.
    pub content: String,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Importance score (0.0-10.0).
    pub importance: f32,
    /// Source references.
    pub sources: Vec<String>,
    /// Created timestamp.
    pub created_at: i64,
    /// Last updated timestamp.
    pub updated_at: i64,
    /// Access count.
    pub access_count: u32,
    /// Whether archived.
    pub archived: bool,
}

impl HierarchyRecord {
    /// Create a new hierarchy record.
    pub fn new(id: &str, layer: MemoryLayer, content: &str) -> Self {
        Self {
            id: id.to_string(),
            layer,
            content: content.to_string(),
            confidence: 0.5,
            importance: 0.5,
            sources: Vec::new(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            access_count: 0,
            archived: false,
        }
    }

    /// Update access count.
    pub fn access(&mut self) {
        self.access_count += 1;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Update confidence.
    pub fn update_confidence(&mut self, confidence: f32) {
        self.confidence = confidence;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Update importance.
    pub fn update_importance(&mut self, importance: f32) {
        self.importance = importance;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Archive the record.
    pub fn archive(&mut self) {
        self.archived = true;
        self.layer = MemoryLayer::Archive;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Promote to a higher layer.
    pub fn promote(&mut self, new_layer: MemoryLayer) {
        self.layer = new_layer;
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

/// The memory hierarchy organizes knowledge into specialized layers.
#[derive(Debug, Clone, Default)]
pub struct MemoryHierarchy {
    /// Records by ID.
    records: std::collections::HashMap<String, HierarchyRecord>,
    /// Index cards.
    index_cards: std::collections::HashMap<String, MemoryIndexCard>,
    /// Compressed memories.
    compressed: Vec<CompressedMemory>,
    /// Confidence models.
    confidence_models: std::collections::HashMap<String, ConfidenceModel>,
    /// Forgetting policy.
    forgetting_policy: ForgettingPolicy,
    /// Importance scorer.
    importance_scorer: ImportanceScorer,
}

impl MemoryHierarchy {
    /// Create a new memory hierarchy.
    pub fn new() -> Self {
        Self {
            records: std::collections::HashMap::new(),
            index_cards: std::collections::HashMap::new(),
            compressed: Vec::new(),
            confidence_models: std::collections::HashMap::new(),
            forgetting_policy: ForgettingPolicy::default(),
            importance_scorer: ImportanceScorer::default(),
        }
    }

    /// Add a record.
    pub fn add_record(&mut self, record: HierarchyRecord) {
        self.records.insert(record.id.clone(), record.clone());

        // Create index card
        let card = MemoryIndexCard {
            id: record.id.clone(),
            title: record.content.clone(),
            layer: record.layer.clone(),
            memory_type: MemoryType::Generic,
            summary: record.content.clone(),
            confidence: record.confidence,
            importance: record.importance,
            source: record.sources.first().cloned().unwrap_or_default(),
            timestamp: record.created_at,
            relationships: Vec::new(),
            embedding_ref: None,
            storage_location: record.id.clone(),
        };
        self.index_cards.insert(record.id, card);
    }

    /// Get a record by ID.
    pub fn get_record(&self, id: &str) -> Option<&HierarchyRecord> {
        self.records.get(id)
    }

    /// Get mutable record by ID.
    pub fn get_record_mut(&mut self, id: &str) -> Option<&mut HierarchyRecord> {
        self.records.get_mut(id)
    }

    /// Get records by layer.
    pub fn get_by_layer(&self, layer: MemoryLayer) -> Vec<&HierarchyRecord> {
        self.records.values().filter(|r| r.layer == layer).collect()
    }

    /// Get an index card by ID.
    pub fn get_index_card(&self, id: &str) -> Option<&MemoryIndexCard> {
        self.index_cards.get(id)
    }

    /// Promote a record based on access count and confidence.
    pub fn promote_record(&mut self, id: &str) -> Option<MemoryLayer> {
        let promotion = self
            .records
            .get_mut(id)
            .filter(|r| r.access_count >= 10 && r.confidence >= 0.8 && !r.archived)
            .and_then(|record| {
                record
                    .layer
                    .promote_to()
                    .inspect(|nl| record.promote(nl.clone()))
            });
        if let Some(new_layer) = promotion {
            // Update index card
            let card_result = self.index_cards.get_mut(id).map(|card| {
                card.layer = new_layer.clone();
                card.confidence = self.records.get(id).map(|r| r.confidence).unwrap_or(0.0);
                card.importance = self.records.get(id).map(|r| r.importance).unwrap_or(0.0);
            });
            if card_result.is_some() {
                return Some(new_layer);
            }
        }
        None
    }

    /// Archive a record.
    pub fn archive_record(&mut self, id: &str) -> bool {
        if let Some(record) = self.records.get_mut(id) {
            record.archive();
            if let Some(card) = self.index_cards.get_mut(id) {
                card.layer = MemoryLayer::Archive;
            }
            true
        } else {
            false
        }
    }

    /// Compress similar memories into a summary.
    pub fn compress_memories(
        &mut self,
        ids: Vec<String>,
        summary: &str,
    ) -> Option<CompressedMemory> {
        if ids.len() < 2 {
            return None;
        }
        let mut total_confidence = 0.0;
        let mut count = 0;
        let mut original_ids = Vec::new();

        for id in &ids {
            if let Some(record) = self.records.get(id) {
                total_confidence += record.confidence;
                count += 1;
                original_ids.push(id.clone());
            }
        }

        if count == 0 {
            return None;
        }

        let avg_confidence = total_confidence / count as f32;
        let result = CompressedMemory::new(original_ids, summary, avg_confidence, ids.len() as u32);
        self.compressed.push(result.clone());
        Some(result)
    }

    /// Check which memories should be forgotten and archive them.
    pub fn apply_forgetting(&mut self) -> Vec<String> {
        let now = chrono::Utc::now().timestamp();
        let mut forgotten = Vec::new();

        let ids: Vec<String> = self.records.keys().cloned().collect();
        for id in &ids {
            if let Some(record) = self.records.get(id) {
                let age_hours = ((now - record.updated_at) as u64) / 3600;
                if self.forgetting_policy.should_forget(
                    record.importance,
                    age_hours,
                    record.confidence,
                ) {
                    // Archive instead of delete
                    if self.archive_record(id) {
                        forgotten.push(id.clone());
                    }
                }
            }
        }
        forgotten
    }

    /// Calculate importance for all records.
    pub fn calculate_importances(&mut self) {
        let ids: Vec<String> = self.records.keys().cloned().collect();
        let max_access = ids
            .iter()
            .filter_map(|id| self.records.get(id))
            .map(|r| r.access_count)
            .max()
            .unwrap_or(1);

        for id in ids {
            if let Some(record) = self.records.get(&id) {
                let age_hours =
                    ((chrono::Utc::now().timestamp() - record.created_at) as u64) / 3600;
                let importance = self
                    .importance_scorer
                    .calculate_importance(ImportanceParams {
                        access_count: record.access_count,
                        planner_requests: 0,
                        user_corrections: 0,
                        success_rate: record.confidence,
                        age_hours,
                        is_novel: false,
                        relationship_count: record.sources.len() as u32,
                        max_access,
                    });
                if let Some(record) = self.records.get_mut(&id) {
                    record.importance = importance;
                    if let Some(card) = self.index_cards.get_mut(&id) {
                        card.importance = importance;
                    }
                }
            }
        }
    }

    /// Update confidence for all records.
    pub fn decay_confidence(&mut self, decay_rate: f32) {
        let now = chrono::Utc::now().timestamp();
        let ids: Vec<String> = self.records.keys().cloned().collect();
        for id in &ids {
            if let Some(record) = self.records.get(id) {
                let days_elapsed = ((now - record.updated_at) as f32) / 86400.0;
                if let Some(model) = self.confidence_models.get_mut(id) {
                    model.decay(decay_rate, days_elapsed as i64);
                }
            }
        }
    }

    /// Get statistics for the memory hierarchy.
    pub fn statistics(&self) -> MemoryStatistics {
        let total = self.records.len() as u32;
        let mut records_by_layer = std::collections::HashMap::new();
        let mut total_confidence = 0.0;
        let mut total_importance = 0.0;
        let mut archived_count = 0;

        for record in self.records.values() {
            *records_by_layer
                .entry(record.layer.name().to_string())
                .or_insert(0) += 1;
            total_confidence += record.confidence;
            total_importance += record.importance;
            if record.archived {
                archived_count += 1;
            }
        }

        let avg_confidence = if total > 0 {
            total_confidence / total as f32
        } else {
            0.0
        };
        let avg_importance = if total > 0 {
            total_importance / total as f32
        } else {
            0.0
        };

        MemoryStatistics {
            total_records: total,
            records_by_layer,
            avg_confidence,
            avg_importance,
            archived_count,
            compressed_count: self.compressed.len() as u32,
        }
    }

    /// Get the number of records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Check if the hierarchy is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Promotion gate thresholds per Architecture §14.
/// Controls when a memory record is eligible for promotion.
pub struct PromotionGate {
    /// Minimum age in hours before promotion is considered.
    pub min_age_hours: u64,
    /// Minimum confidence score for promotion.
    pub min_confidence: f32,
    /// Minimum access count for promotion.
    pub min_access_count: u32,
}

impl PromotionGate {
    /// Create a new promotion gate with the given thresholds.
    pub fn new(min_age_hours: u64, min_confidence: f32, min_access_count: u32) -> Self {
        Self {
            min_age_hours,
            min_confidence,
            min_access_count,
        }
    }
}

impl Default for PromotionGate {
    fn default() -> Self {
        Self {
            min_age_hours: 1,
            min_confidence: 0.5,
            min_access_count: 3,
        }
    }
}

/// Determine if a memory record should be promoted based on gate thresholds.
/// Per Architecture §14 — promotion pipeline evaluates whether a
/// memory record meets the age, confidence, and access thresholds required
/// to advance to the next memory layer.
pub fn should_promote(memory: &HierarchyRecord, gate: &PromotionGate) -> bool {
    let age_hours = ((chrono::Utc::now().timestamp() - memory.created_at) as u64) / 3600;
    age_hours >= gate.min_age_hours
        && memory.confidence >= gate.min_confidence
        && memory.access_count >= gate.min_access_count
        && !memory.archived
}

/// Run autonomous memory restructuring per Architecture §14.
/// Performs promotion, compression, forgetting, and importance recalculation.
pub fn restructure_hierarchy(hierarchy: &mut MemoryHierarchy) {
    tracing::debug!("Running autonomous memory restructuring");

    // 1. Recalculate importance scores
    hierarchy.calculate_importances();

    // 2. Promote high-confidence, high-access records
    let ids: Vec<String> = hierarchy.records.keys().cloned().collect();
    let mut promoted = 0;
    for id in ids {
        if hierarchy.promote_record(&id).is_some() {
            promoted += 1;
        }
    }

    // 3. Compress similar low-priority memories
    let working_records: Vec<String> = hierarchy
        .get_by_layer(MemoryLayer::Working)
        .iter()
        .filter(|r| r.importance < 0.3)
        .map(|r| r.id.clone())
        .collect();

    if working_records.len() >= 3 {
        let summary = format!(
            "Compressed {} working memories into a single summary",
            working_records.len()
        );
        hierarchy.compress_memories(working_records, &summary);
    }

    // 4. Apply forgetting policy
    let forgotten = hierarchy.apply_forgetting();

    tracing::info!(
        promoted = promoted,
        forgotten = forgotten.len(),
        "Memory restructuring completed"
    );
}
