// src/agent/context.rs
//! Dependency container for the agent loop.
//!
//! `AgentDeps` bundles the existing subsystem handles the agent composes, so
//! the loop has a single injection point. It borrows `Arc` clones from the
//! `McpContext` / `App` and adds nothing of its own.

use std::sync::{Arc, Mutex};

use crate::database::sqlite::SqliteDatabase;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::knowledge::KnowledgeStore;
use crate::memory::retrieval::MemoryRetrieval;
use crate::personality::Personality;
use crate::planner::Planner;

use super::safety_gate::SafetyGate;

/// All subsystems the goal-driven agent loop composes (Architecture §5.7).
///
/// Every field is a shared handle (`Arc`) to an existing, already-initialized
/// subsystem. The agent does not construct or own any business logic.
pub struct AgentDeps {
    /// Decomposes a goal into actionable steps (Architecture §2.8).
    pub planner: Arc<Planner>,
    /// Unified retrieval across working + permanent memory (Architecture §3).
    pub memory_retrieval: Arc<MemoryRetrieval>,
    /// Validated knowledge store (Architecture §2.3).
    pub knowledge_store: Arc<KnowledgeStore>,
    /// Experience coordinator — used to publish the outcome event that drives
    /// the §4.04 learning spine. `process()` scores the experience and
    /// publishes `ExperienceRecorded` once (P0 V2-02).
    pub coordinator: Arc<ExperienceCoordinator>,
    /// Direct database handle for persisting processed experiences.
    pub database: Arc<SqliteDatabase>,
    /// Safety gate that may block an action before execution (TASK-V2-07).
    pub safety_gate: Arc<SafetyGate>,
    /// Personality — provides emotional weighting that nudges confidence and
    /// the action threshold (Architecture §13, TASK-V2-08). Emotion does not
    /// override evidence-based confidence; it biases it. Shared with the App
    /// and the planner behind a mutex.
    pub personality: Arc<Mutex<Personality>>,
}

impl AgentDeps {
    pub fn new(
        planner: Arc<Planner>,
        memory_retrieval: Arc<MemoryRetrieval>,
        knowledge_store: Arc<KnowledgeStore>,
        coordinator: Arc<ExperienceCoordinator>,
        database: Arc<SqliteDatabase>,
        safety_gate: Arc<SafetyGate>,
        personality: Arc<Mutex<Personality>>,
    ) -> Self {
        Self {
            planner,
            memory_retrieval,
            knowledge_store,
            coordinator,
            database,
            safety_gate,
            personality,
        }
    }
}
