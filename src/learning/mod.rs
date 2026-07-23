// src/learning/mod.rs
//! Learning module for experience-based learning
//!
//! Per Architecture §9 - Learning Pipeline:
//! Input → Observation → Memory → Experience → Knowledge → Planning → Decision → Action → Reflection

#![allow(dead_code, unused_imports)]

pub mod working_memory;
pub mod hypothesis;
pub mod candidates;
pub mod lineage;
pub mod pipeline;

pub use working_memory::{
    WorkingMemory, 
    WorkingMemoryItem, 
    MemoryItemType, 
    MemoryStats,
    MemoryState,
    StateTransition,
    StateTransitionRecord,
    PromotionPolicy,
    PromotionEvaluation,
};
pub use hypothesis::{Hypothesis, HypothesisEvidence, HypothesisStatus};
pub use candidates::{Candidate, CandidateGenerator, CandidateScore};
pub use lineage::{
    MemoryLineage,
    LineageTracker,
    LineageSummary,
    EvidenceRef,
    EvidenceType,
    ObservationRef,
    ObservationType,
    ObservationOutcome,
    Refinement,
    RefinementType,
    Contradiction,
    ContradictionResolution,
    Confirmation,
    ConfirmationSource,
};
pub use pipeline::{LearningPipeline, PipelineStage, PipelineRecord, PipelineStats};
