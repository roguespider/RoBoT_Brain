// src/bridge/app/state.rs
//! Application state and struct definition

use std::sync::Arc;
use std::sync::Mutex;

use crate::bridge::acp::AcpRouter;
use crate::bridge::mcp::McpContext;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::encounter_recorder::ExperienceRecorder;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::integration::reflection_pipeline::ReflectionPipeline;
use crate::experience::scheduler::Scheduler;
use crate::memory::pipeline::MemoryPipeline;
use crate::personality::Personality;
use crate::agent::AgentLoop;
use crate::world_model::WorldModel;

/// Root application container.
///
/// Owns long-running services required by RoBoT.
pub struct App {
    /// Persistent database layer.
    pub(crate) database: Arc<SqliteDatabase>,

    /// Event bus for pub/sub.
    pub(crate) bus: Arc<crate::experience::bus::ExperienceBus>,

    /// Worker manager for background job processing (Architecture §22).
    pub(crate) worker_manager: Arc<crate::experience::worker_manager::WorkerManager>,

    /// Experience system coordinator.
    pub(crate) coordinator: Arc<ExperienceCoordinator>,

    /// Hypothesis engine for belief management.
    pub(crate) hypothesis_engine: Arc<Mutex<HypothesisEngine>>,

    /// Experience recorder for structured experience creation.
    pub(crate) experience_recorder: Arc<ExperienceRecorder>,

    /// Reflection pipeline for processing experiences into insights.
    pub(crate) reflection_pipeline: Arc<ReflectionPipeline>,

    /// Background task scheduler.
    pub(crate) scheduler: Arc<Scheduler>,

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
}
