// src/agent/context.rs
//! Dependency container for the agent loop.
//!
//! `AgentDeps` bundles the existing subsystem handles the agent composes, so
//! the loop has a single injection point. It borrows `Arc` clones from the
//! `McpContext` / `App` and adds nothing of its own.

use std::sync::{Arc, Mutex};

use crate::bridge::mcp::McpContext;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::metrics::Metrics;
use crate::knowledge::KnowledgeStore;
use crate::memory::retrieval::MemoryRetrieval;
use crate::personality::Personality;
use crate::planner::Planner;

use super::safety_gate::SafetyGate;

/// Persistence-related subsystems used by the agent loop.
pub struct PersistenceDeps {
    /// Validated knowledge store (Architecture §2.3).
    pub knowledge_store: Arc<KnowledgeStore>,
    /// Experience coordinator — used to publish the outcome event that drives
    /// the §4.04 learning spine. `process()` scores the experience and
    /// publishes `ExperienceRecorded` once (P0 V2-02).
    pub coordinator: Arc<ExperienceCoordinator>,
    /// Direct database handle for persisting processed experiences.
    pub database: Arc<SqliteDatabase>,
}

/// All subsystems the goal-driven agent loop composes (Architecture §5.7).
///
/// Every field is a shared handle (`Arc`) to an existing, already-initialized
/// subsystem. The agent does not construct or own any business logic.
pub struct AgentDeps {
    /// Decomposes a goal into actionable steps (Architecture §2.8).
    pub planner: Arc<Planner>,
    /// Unified retrieval across working + permanent memory (Architecture §3).
    pub memory_retrieval: Arc<MemoryRetrieval>,
    /// Persistence-related subsystems (knowledge, coordinator, database).
    pub persistence: PersistenceDeps,
    /// Safety gate that may block an action before execution (TASK-V2-07).
    pub safety_gate: Arc<SafetyGate>,
    /// Personality — provides emotional weighting that nudges confidence and
    /// the action threshold (Architecture §13, TASK-V2-08). Emotion does not
    /// override evidence-based confidence; it biases it. Shared with the App
    /// and the planner behind a mutex.
    pub personality: Arc<Mutex<Personality>>,
    /// Metrics collection for loop-health monitoring (T1-13..T1-16).
    pub metrics: Arc<Metrics>,
}

impl AgentDeps {
    /// Compose agent dependencies from the shared MCP context.
    ///
    /// Every handle is cloned from the context; `personality` is passed
    /// separately because it is owned by the App layer, not the MCP context.
    pub fn from_context(context: &McpContext, personality: Arc<Mutex<Personality>>) -> Self {
        Self {
            planner: context.planner.clone(),
            memory_retrieval: context.memory_retrieval.clone(),
            persistence: PersistenceDeps {
                knowledge_store: context.knowledge.clone(),
                coordinator: context.coordinator.clone(),
                database: context.database.clone(),
            },
            safety_gate: context.safety_gate.clone(),
            personality,
            metrics: context.metrics.clone(),
        }
    }
}
