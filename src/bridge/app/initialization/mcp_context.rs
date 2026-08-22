// src/bridge/app/initialization/mcp_context.rs
//! Create the shared McpContext holding all subsystem references.

use std::sync::Arc;

use crate::agent::SafetyGate;
use crate::bridge::acp::{AcpRegistry, AcpRouter};
use crate::bridge::mcp::McpContext;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::metrics::Metrics;
use crate::experience::queue::JobQueue;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::scheduler::Scheduler;
use crate::experience::worker_manager::WorkerManager;
use crate::knowledge::KnowledgeStore;
use crate::memory::MemoryRetrieval;
use crate::memory::PermanentMemory;
use crate::memory::WorkingMemory as MemWorkingMemory;
use crate::planner::Planner;
use crate::planner::PolicyEngine;
use crate::skills::registry::SkillRegistry;
use crate::workflows::enforcement::WorkflowEnforcer;
use crate::workflows::engine::WorkflowEngine;
use crate::world_model::WorldModel;

/// Create the shared McpContext holding all subsystem references.
///
/// Builds the WorldModel and WorkflowEnforcer, then wires all 21 dependencies
/// into the McpContext.
pub fn create_mcp_context(
    database: Arc<SqliteDatabase>,
    job_queue: Arc<std::sync::Mutex<JobQueue>>,
    bus: Arc<ExperienceBus>,
    coordinator: Arc<ExperienceCoordinator>,
    worker_manager: Arc<WorkerManager>,
    reflection_engine: Arc<ReflectionEngine>,
    evolution_engine: Arc<EvolutionEngine>,
    scheduler: Arc<Scheduler>,
    metrics: Arc<Metrics>,
    knowledge_store: Arc<KnowledgeStore>,
    planner: Arc<Planner>,
    policy_engine: Arc<PolicyEngine>,
    working_memory_core: Arc<MemWorkingMemory>,
    permanent_memory: Arc<PermanentMemory>,
    memory_retrieval: Arc<MemoryRetrieval>,
    workflow_engine: Arc<WorkflowEngine>,
    skills_registry: Arc<SkillRegistry>,
    acp_router: Arc<AcpRouter>,
    acp_registry: Arc<AcpRegistry>,
    shared_personality: Arc<std::sync::Mutex<crate::personality::Personality>>,
    world_model: Arc<WorldModel>,
    enforcer: Arc<WorkflowEnforcer>,
) -> Arc<McpContext> {
    Arc::new(McpContext::new(
        database,
        job_queue,
        bus,
        coordinator,
        worker_manager,
        reflection_engine,
        evolution_engine,
        scheduler,
        metrics,
        knowledge_store,
        planner,
        policy_engine,
        working_memory_core,
        permanent_memory,
        memory_retrieval,
        workflow_engine,
        skills_registry,
        acp_router,
        acp_registry,
        shared_personality,
        Arc::new(SafetyGate::new()),
        world_model,
        enforcer,
    ))
}
