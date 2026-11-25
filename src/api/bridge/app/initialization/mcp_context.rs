// src/bridge/app/initialization/mcp_context.rs
//! Create the shared McpContext holding all subsystem references.

use std::sync::Arc;

use crate::api::acp::{AcpRegistry, AcpRouter};
use crate::api::mcp::McpContext;
use crate::conversation::ConversationEngine;
use crate::database::sqlite::SqliteDatabase;
use crate::execution::skills::SkillRegistry;
use crate::execution::workflows::enforcement::WorkflowEnforcer;
use crate::execution::workflows::engine::WorkflowEngine;
use crate::experience::bus::ExperienceBus;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::metrics::Metrics;
use crate::experience::queue::JobQueue;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::scheduler::Scheduler;
use crate::experience::worker_manager::WorkerManager;
use crate::knowledge::KnowledgeStore;
use crate::knowledge::world_model::WorldModel;
use crate::memory::MemoryRetrieval;
use crate::memory::PermanentMemory;
use crate::memory::WorkingMemory as MemWorkingMemory;
use crate::planning::Planner;
use crate::planning::PolicyEngine;
use crate::storage::StorageArchitecture;

/// Grouped subsystem handles for building the shared `McpContext`.
pub struct McpContextSystems {
    pub database: Arc<SqliteDatabase>,
    pub job_queue: Arc<std::sync::Mutex<JobQueue>>,
    pub bus: Arc<ExperienceBus>,
    pub coordinator: Arc<ExperienceCoordinator>,
    pub worker_manager: Arc<WorkerManager>,
    pub reflection_engine: Arc<ReflectionEngine>,
    pub evolution_engine: Arc<EvolutionEngine>,
    pub scheduler: Arc<Scheduler>,
    pub metrics: Arc<Metrics>,
    pub knowledge_store: Arc<KnowledgeStore>,
    pub planner: Arc<Planner>,
    pub policy_engine: Arc<PolicyEngine>,
    pub working_memory_core: Arc<MemWorkingMemory>,
    pub permanent_memory: Arc<PermanentMemory>,
    pub memory_retrieval: Arc<MemoryRetrieval>,
    pub workflow_engine: Arc<WorkflowEngine>,
    pub skills_registry: Arc<SkillRegistry>,
    pub acp_router: Arc<AcpRouter>,
    pub acp_registry: Arc<AcpRegistry>,
    pub shared_personality: Arc<std::sync::Mutex<crate::common::personality::Personality>>,
    pub world_model: Arc<WorldModel>,
    pub enforcer: Arc<WorkflowEnforcer>,
    pub conversation_engine: Arc<std::sync::Mutex<ConversationEngine>>,
    /// Storage Architecture: storage layers, policies, audit trails (Architecture Chapter 21).
    pub storage: StorageArchitecture,
}

/// Create the shared McpContext holding all subsystem references.
///
/// Builds the WorldModel and WorkflowEnforcer, then wires all 21 dependencies
/// into the McpContext.
pub fn create_mcp_context(systems: McpContextSystems) -> Arc<McpContext> {
    Arc::new(McpContext::new(systems))
}
