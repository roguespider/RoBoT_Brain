//! Memory Hierarchy — Organized memory layers (Architecture Chapter 17).
//!
//! Wiring: `memory_hierarchy/` -> `memory/` -> `database/` (hierarchy tables)

/// Memory layer types per Architecture Chapter 17.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
}

/// A memory record in the hierarchy with promotion/demotion tracking.
#[derive(Debug, Clone, PartialEq)]
pub struct HierarchyRecord {
    /// Record identifier.
    pub id: String,
    /// Memory layer.
    pub layer: MemoryLayer,
    /// Content summary.
    pub content: String,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Importance score.
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
    /// Records by layer.
    records: std::collections::HashMap<String, HierarchyRecord>,
}

impl MemoryHierarchy {
    /// Create a new memory hierarchy.
    pub fn new() -> Self {
        Self {
            records: std::collections::HashMap::new(),
        }
    }

    /// Add a record.
    pub fn add_record(&mut self, record: HierarchyRecord) {
        self.records.insert(record.id.clone(), record);
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

    /// Promote a record based on access count and confidence.
    pub fn promote_record(&mut self, id: &str) -> Option<MemoryLayer> {
        if let Some(record) = self.get_record_mut(id)
            && record.access_count > 10
            && record.confidence > 0.8
            && !record.archived
            && let Some(new_layer) = record.layer.promote_to()
        {
            record.promote(new_layer.clone());
            Some(new_layer)
        } else {
            None
        }
    }

    /// Archive a record.
    pub fn archive_record(&mut self, id: &str) -> bool {
        if let Some(record) = self.get_record_mut(id) {
            record.archive();
            return true;
        }
        false
    }
}

/// Promotion gate thresholds per Architecture Chapter 17.5.
///
/// Controls when a memory record is eligible for promotion to the next layer.
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
///
/// Per Architecture Chapter 17.5 — promotion pipeline evaluates whether a
/// memory record meets the age, confidence, and access thresholds required
/// to advance to the next memory layer.
pub fn should_promote(memory: &HierarchyRecord, gate: &PromotionGate) -> bool {
    let age_hours = (chrono::Utc::now().timestamp() - memory.created_at) as u64 / 3600;
    age_hours >= gate.min_age_hours
        && memory.confidence >= gate.min_confidence
        && memory.access_count >= gate.min_access_count
        && !memory.archived
}
