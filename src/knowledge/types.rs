// src/knowledge/types.rs
//! Core types for the Knowledge System

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// ============================================================================
/// KNOWLEDGE ITEM
/// ============================================================================
/// A piece of knowledge that has gained sufficient confidence to influence reasoning.
///
/// Knowledge is distinct from information:
/// - Information = raw data, observations
/// - Knowledge = information with evidence, confidence, and validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    /// Unique identifier
    pub id: Uuid,

    /// Human-readable statement of knowledge
    pub statement: String,

    /// Type/category of knowledge
    pub knowledge_type: KnowledgeType,

    /// Multi-dimensional confidence score
    pub confidence: KnowledgeConfidence,

    /// Current status
    pub status: KnowledgeStatus,

    /// Source of this knowledge (what created it)
    pub source: KnowledgeSource,

    /// Evidence supporting this knowledge
    pub supporting_evidence: Vec<Uuid>,
    
    /// Evidence contradicting this knowledge
    pub contradicting_evidence: Vec<Uuid>,

    /// Related knowledge items
    pub relations: Vec<KnowledgeRelation>,

    /// When this knowledge was created
    pub created_at: DateTime<Utc>,

    /// When this knowledge was last updated
    pub updated_at: DateTime<Utc>,

    /// Number of times this knowledge was successfully applied
    pub success_count: u32,

    /// Number of times this knowledge failed
    pub failure_count: u32,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Arbitrary metadata
    pub metadata: HashMap<String, String>,
}

impl KnowledgeItem {
    /// Create a new knowledge item from reflection output
    pub fn from_reflection(
        insight: &str,
        confidence: f32,
        source_experience: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            statement: insight.to_string(),
            knowledge_type: KnowledgeType::Insight,
            confidence: KnowledgeConfidence::new(confidence),
            status: KnowledgeStatus::New,
            source: KnowledgeSource::Reflection(source_experience),
            supporting_evidence: vec![source_experience],
            contradicting_evidence: Vec::new(),
            relations: Vec::new(),
            created_at: now,
            updated_at: now,
            success_count: 0,
            failure_count: 0,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Get overall confidence (weighted average of dimensions)
    pub fn overall_confidence(&self) -> f32 {
        self.confidence.overall()
    }

    /// Check if knowledge is mature enough to use
    pub fn is_mature(&self) -> bool {
        self.confidence.overall() >= 0.7 && self.status == KnowledgeStatus::Active
    }

    /// Check if knowledge should be questioned
    pub fn needs_review(&self) -> bool {
        self.failure_count > self.success_count || self.confidence.overall() < 0.5
    }

    /// Record successful application of this knowledge
    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.confidence.adjust_source_reliability(0.01);
        self.updated_at = Utc::now();
    }

    /// Record failed application of this knowledge
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.confidence.adjust_source_reliability(-0.02);
        self.confidence.adjust_historical_accuracy(-0.02);
        self.updated_at = Utc::now();
    }
}

/// ============================================================================
/// KNOWLEDGE TYPE
/// ============================================================================
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KnowledgeType {
    /// Factual knowledge (tested and verified)
    Fact,
    /// Procedure/workflow knowledge
    Procedure,
    /// Causal relationship
    Causality,
    /// Pattern recognition
    Pattern,
    /// Insight from reflection
    Insight,
    /// Rule or constraint
    Rule,
    /// Concept or definition
    Concept,
    /// Custom type
    Custom(String),
}

impl Default for KnowledgeType {
    fn default() -> Self {
        KnowledgeType::Fact
    }
}

/// ============================================================================
/// KNOWLEDGE STATUS
/// ============================================================================
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KnowledgeStatus {
    /// Newly created, needs validation
    New,
    /// Being validated
    Validating,
    /// Active and available for use
    Active,
    /// Suspended temporarily
    Suspended,
    /// Disproven or invalidated
    Disproven,
    /// Merged with other knowledge
    Merged,
}

impl Default for KnowledgeStatus {
    fn default() -> Self {
        KnowledgeStatus::New
    }
}

/// ============================================================================
/// KNOWLEDGE CONFIDENCE
//  ============================================================================
/// Multi-dimensional confidence tracking per architecture #3.
/// Confidence is not a simple yes or no value.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeConfidence {
    /// Dimensions of confidence
    pub dimensions: ConfidenceDimensions,
}

impl KnowledgeConfidence {
    /// Create new confidence with overall score
    pub fn new(overall: f32) -> Self {
        Self {
            dimensions: ConfidenceDimensions {
                source_reliability: overall,
                evidence_strength: overall,
                recency: 1.0, // New knowledge has full recency
                frequency: 0.5, // Default middle frequency
                context_relevance: 0.5,
                historical_accuracy: overall,
            },
        }
    }

    /// Create from reflection with evidence strength
    pub fn from_reflection(evidence_strength: f32) -> Self {
        Self {
            dimensions: ConfidenceDimensions {
                source_reliability: evidence_strength,
                evidence_strength,
                recency: 1.0,
                frequency: 0.5,
                context_relevance: 0.7,
                historical_accuracy: evidence_strength,
            },
        }
    }

    /// Calculate weighted overall confidence
    /// Weights based on importance per architecture
    pub fn overall(&self) -> f32 {
        let d = &self.dimensions;
        // Weighted average emphasizing reliability and evidence
        0.25 * d.source_reliability
            + 0.25 * d.evidence_strength
            + 0.15 * d.recency
            + 0.10 * d.frequency
            + 0.10 * d.context_relevance
            + 0.15 * d.historical_accuracy
    }

    /// Adjust source reliability (e.g., from reputation)
    pub fn adjust_source_reliability(&mut self, delta: f32) {
        self.dimensions.source_reliability = 
            (self.dimensions.source_reliability + delta).clamp(0.0, 1.0);
    }

    /// Adjust historical accuracy (e.g., from success/failure)
    pub fn adjust_historical_accuracy(&mut self, delta: f32) {
        self.dimensions.historical_accuracy = 
            (self.dimensions.historical_accuracy + delta).clamp(0.0, 1.0);
    }

    /// Adjust frequency (e.g., from repeated confirmation)
    pub fn adjust_frequency(&mut self, delta: f32) {
        self.dimensions.frequency = 
            (self.dimensions.frequency + delta).clamp(0.0, 1.0);
    }

    /// Update recency based on time since last update
    pub fn update_recency(&mut self, last_update: DateTime<Utc>) {
        let age_hours = (Utc::now() - last_update).num_hours() as f32;
        // Decay over 30 days
        self.dimensions.recency = (1.0 - (age_hours / (30.0 * 24.0))).clamp(0.1, 1.0);
    }
}

/// Individual confidence dimensions per architecture #3
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfidenceDimensions {
    /// How reliable is the source (per architecture #12)
    pub source_reliability: f32,

    /// Strength of supporting evidence
    pub evidence_strength: f32,

    /// How recent is this knowledge
    pub recency: f32,

    /// Frequency of confirmation
    pub frequency: f32,

    /// Relevance to current context
    pub context_relevance: f32,

    /// Historical accuracy of this knowledge
    pub historical_accuracy: f32,
}

/// ============================================================================
/// KNOWLEDGE SOURCE
//  ============================================================================
/// Where knowledge originated (per architecture #12: Reputation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Created from user input
    User,
    /// Created from tool execution
    Tool,
    /// Created from planning
    Planner,
    /// Created from reflection (per architecture #10)
    Reflection(Uuid), // experience_id
    /// Created from hypothesis validation
    Hypothesis(Uuid), // hypothesis_id
    /// Discovered through exploration
    Exploration(Uuid), // exploration_id
    /// Learned from external source
    External(String), // source_name
}

impl Default for KnowledgeSource {
    fn default() -> Self {
        KnowledgeSource::User
    }
}

/// ============================================================================
/// KNOWLEDGE RELATION
//  ============================================================================
/// Relationships between knowledge items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRelation {
    /// ID of related knowledge
    pub target_id: Uuid,
    /// Type of relationship
    pub relation_type: RelationType,
    /// Strength of the relationship
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationType {
    /// Supports/strengthens this knowledge
    Supports,
    /// Contradicts this knowledge
    Contradicts,
    /// General relatedness
    Related,
    /// Specialization
    Specializes,
    /// Generalization
    Generalizes,
    /// Prerequisite
    Prerequisite,
}

impl Default for RelationType {
    fn default() -> Self {
        RelationType::Related
    }
}
