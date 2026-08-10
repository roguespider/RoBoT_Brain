// src/bridge/mcp/context.rs

// MCP context for sharing state across handlers

use std::sync::Arc;

use crate::bridge::acp::{AcpRegistry, AcpRouter};
use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::metrics::MetricsCollector;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::scheduler::Scheduler;
use crate::experience::worker_manager::WorkerManager;
use crate::knowledge::KnowledgeStore;
use crate::memory::events::MemoryEventBus;
use crate::memory::{MemoryRetrieval, PermanentMemory, WorkingMemory};
use crate::personality::Personality;
use crate::planner::{Planner, PolicyEngine};
use crate::skills::registry::{SkillExecutor, SkillRegistry};
use crate::workflows::engine::WorkflowEngine;
use crate::agent::SafetyGate;

use std::sync::Mutex;

use super::types::{McpCapabilities, McpEmpty, McpResourcesCapability, McpServerInfo};

/// McpBridge context shared across handlers
pub struct McpContext {
    /// Database layer
    pub database: Arc<SqliteDatabase>,

    /// Event bus
    pub bus: Arc<ExperienceBus>,

    /// Experience coordinator (used by experience tools)
    pub coordinator: Arc<ExperienceCoordinator>,

    /// Background worker manager (per Architecture §22)
    pub worker_manager: Arc<WorkerManager>,

    /// Reflection engine (used by reflection tools)
    pub reflection: Arc<ReflectionEngine>,

    /// Evolution engine
    pub evolution: Arc<EvolutionEngine>,

    /// Background scheduler
    pub scheduler: Arc<Scheduler>,

    /// Metrics collector
    pub metrics: Arc<MetricsCollector>,

    /// Knowledge system - manages validated knowledge (used by knowledge tools)
    pub knowledge: Arc<KnowledgeStore>,

    /// Planner - task decomposition and execution (used by planner tools)
    pub planner: Arc<Planner>,

    /// Policy engine - decision-making rules
    pub policy: Arc<PolicyEngine>,

    /// Working memory - short-term memory layer
    pub working_memory: Arc<WorkingMemory>,

    /// Permanent memory - long-term memory layer
    pub permanent_memory: Arc<PermanentMemory>,

    /// Memory retrieval - unified retrieval across layers
    pub memory_retrieval: Arc<MemoryRetrieval>,

    /// Workflow engine - structured workflow execution
    pub workflow_engine: Arc<WorkflowEngine>,

    /// Skill registry - manages reusable capabilities (per Architecture §15)
    pub skills: Arc<SkillRegistry>,

    /// Skill executor - executes skills and tracks execution metrics
    /// (per Architecture §15 "Skill::track_execution_metrics()")
    pub skill_executor: Arc<SkillExecutor>,

    /// ACP router for inter-agent communication
    pub acp_router: Arc<AcpRouter>,

    /// ACP registry for agent registration
    pub acp_registry: Arc<AcpRegistry>,

    /// Memory event bus - per Architecture §35 (short-term: in-memory event bus)
    pub memory_event_bus: Arc<MemoryEventBus>,

    /// Personality — shared behavioral model (Architecture §13). Provides
    /// emotional weighting for the agent loop and other subsystems.
    pub personality: Arc<Mutex<Personality>>,

    /// Safety gate for the agent loop (Architecture §16, TASK-V2-07).
    pub safety_gate: Arc<SafetyGate>,

    /// World Model — typed entity-relationship graph (Architecture §14,
    /// TASK-V2-06). Stores understanding of how the world works.
    pub world_model: Arc<crate::world_model::WorldModel>,

    /// Server info
    pub server_info: McpServerInfo,

    /// Server capabilities
    pub capabilities: McpCapabilities,
}

impl McpContext {
    pub fn new(
        database: Arc<SqliteDatabase>,
        bus: Arc<ExperienceBus>,
        coordinator: Arc<ExperienceCoordinator>,
        worker_manager: Arc<WorkerManager>,
        reflection: Arc<ReflectionEngine>,
        evolution: Arc<EvolutionEngine>,
        scheduler: Arc<Scheduler>,
        metrics: Arc<MetricsCollector>,
        knowledge: Arc<KnowledgeStore>,
        planner: Arc<Planner>,
        policy: Arc<PolicyEngine>,
        working_memory: Arc<WorkingMemory>,
        permanent_memory: Arc<PermanentMemory>,
        memory_retrieval: Arc<MemoryRetrieval>,
        workflow_engine: Arc<WorkflowEngine>,
        skills: Arc<SkillRegistry>,
        acp_router: Arc<AcpRouter>,
        acp_registry: Arc<AcpRegistry>,
        memory_event_bus: Arc<MemoryEventBus>,
        personality: Arc<Mutex<Personality>>,
        safety_gate: Arc<SafetyGate>,
        world_model: Arc<crate::world_model::WorldModel>,
    ) -> Self {
        let skill_executor = Arc::new(SkillExecutor::new(skills.clone()));
        Self {
            database,
            bus,
            coordinator,
            worker_manager,
            reflection,
            evolution,
            scheduler,
            metrics,
            knowledge,
            planner,
            policy,
            working_memory,
            permanent_memory,
            memory_retrieval,
            workflow_engine,
            skills,
            skill_executor,
            acp_router,
            acp_registry,
            memory_event_bus,
            personality,
            safety_gate,
            world_model,
            server_info: McpServerInfo {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: McpCapabilities {
                tools: Some(McpEmpty),
                resources: Some(McpResourcesCapability {
                    subscribe: Some(true),
                    list_changed: Some(true),
                }),
                prompts: Some(McpEmpty),
                logging: Some(McpEmpty),
            },
        }
    }
}
