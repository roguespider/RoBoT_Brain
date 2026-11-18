// src/bridge/app/state.rs
//! Application state and struct definition

use std::sync::Arc;
use std::sync::Mutex;

use crate::api::acp::AcpRouter;
use crate::api::bridge::app::agent::AgentLoop;
use crate::api::mcp::McpContext;
use crate::common::personality::Personality;
use crate::context::ContextEngine;
use crate::conversation::ConversationEngine;
use crate::experience::encounter_recorder::ExperienceRecorder;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::integration::reflection_pipeline::ReflectionPipeline;
use crate::knowledge::world_model::WorldModel;
use crate::memory::ConfidenceSystem;
use crate::memory::MemoryEngine;
use crate::memory::hierarchy::MemoryHierarchy;
use crate::memory::pipeline::MemoryPipeline;
use crate::storage::StorageArchitecture;
use crate::workers::BackgroundWorkerSystem;

/// Root application container.
///
/// Owns long-running services required by RoBoT.
pub struct App {
    /// Hypothesis engine for belief management.
    pub(crate) hypothesis_engine: Arc<Mutex<HypothesisEngine>>,

    /// Experience recorder for structured experience creation.
    pub(crate) experience_recorder: Arc<ExperienceRecorder>,

    /// Reflection pipeline for processing experiences into insights.
    pub(crate) reflection_pipeline: Arc<ReflectionPipeline>,

    /// Memory pipeline for working→permanent consolidation.
    pub(crate) memory_pipeline: Arc<MemoryPipeline>,

    /// MCP context shared with bridge - owns all subsystems.
    pub(crate) mcp_context: Arc<McpContext>,

    /// Personality system for behavioral characteristics.
    pub(crate) personality: Arc<Mutex<Personality>>,

    /// ACP router for inter-agent communication.
    pub(crate) acp_router: Arc<AcpRouter>,

    /// Goal-driven agent loop (Architecture §5.7, TASK-V2-04). Closes the
    /// cognitive loop: Goal → Plan → Retrieve → Decide → Act → Record.
    pub(crate) agent_loop: Arc<AgentLoop>,

    /// World Model: typed entity-relationship graph (Architecture §14,
    /// TASK-V2-06). "Memory stores facts. World Model stores understanding."
    pub(crate) world_model: Arc<WorldModel>,

    /// Conversation Engine: orchestrates user interaction lifecycle
    /// (Architecture Chapter 6). Coordinates interaction without owning
    /// memory, knowledge, experience, or execution logic.
    pub(crate) conversation_engine: Arc<Mutex<ConversationEngine>>,

    /// Context Engine: assembles temporary cognitive environments from
    /// conversation, planner, memory, experience, and knowledge
    /// (Architecture Chapter 7). Context is derived, disposable, assembled on demand.
    pub(crate) context_engine: std::sync::Mutex<ContextEngine>,

    /// Memory Hierarchy: manages promotion pipeline between memory layers
    /// (Architecture §14). Consolidates and promotes records based on access
    /// count, confidence, and importance.
    pub(crate) memory_hierarchy: Arc<Mutex<MemoryHierarchy>>,

    /// Confidence System: tracks confidence scores with decay over time
    /// (Architecture §19). Used for knowledge, skill, and relationship
    /// confidence evaluation.
    pub(crate) confidence_system: Arc<Mutex<ConfidenceSystem>>,

    /// Storage Architecture: manages storage layers, policies, and audit trails
    /// (Architecture Chapter 21). Coordinates persistence through hot/warm/cold/frozen
    /// layers with retention, promotion, and provenance tracking.
    pub(crate) storage: std::sync::Mutex<StorageArchitecture>,

    /// Background Worker System: schedules and executes asynchronous cognitive
    /// tasks (Memory, Experience, Learning, KnowledgeGraph, Maintenance).
    /// Per Architecture §23.7: workers SHALL be scheduled via Supervisor
    /// with Immediate/Scheduled/ResourceBased modes.
    pub(crate) worker_system: BackgroundWorkerSystem,

    /// Unified Memory Engine: wraps WorkingMemory, PermanentMemory,
    /// MemoryHierarchy, ContradictionDetector, and ConfidenceSystem into
    /// a single API for the full memory lifecycle (retention, retrieval,
    /// promotion, consolidation, contradiction, decay, archival).
    /// Per Architecture §8-9: memory SHALL support all seven lifecycle ops.
    pub(crate) memory_engine: std::sync::Arc<MemoryEngine>,
}

impl App {
    /// Get the Memory Hierarchy.
    pub fn memory_hierarchy(&self) -> &std::sync::Mutex<MemoryHierarchy> {
        &self.memory_hierarchy
    }

    /// Get the Confidence System.
    pub fn confidence_system(&self) -> &std::sync::Mutex<crate::memory::ConfidenceSystem> {
        &self.confidence_system
    }

    /// Get the Conversation Engine's session count.
    pub fn conversation_session_count(&self) -> usize {
        match self.conversation_engine.lock() {
            Ok(engine) => engine.session_count(),
            Err(e) => {
                tracing::error!(error = %e, "Conversation engine lock poisoned");
                0
            }
        }
    }

    /// Get a reference to the Context Engine.
    pub fn context_engine(&self) -> &std::sync::Mutex<ContextEngine> {
        &self.context_engine
    }

    /// Get the BackgroundWorkerSystem for health/status checks.
    pub fn worker_system(&self) -> &BackgroundWorkerSystem {
        &self.worker_system
    }

    /// Get the Storage Architecture for audit/policy queries.
    /// (Architecture Chapter 21).
    pub fn storage(&self) -> &std::sync::Mutex<StorageArchitecture> {
        &self.storage
    }

    /// Get the unified Memory Engine (Architecture §8-9).
    pub fn memory_engine(&self) -> &Arc<crate::memory::MemoryEngine> {
        &self.memory_engine
    }
}
