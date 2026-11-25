// src/bridge/app/initialization/agent_loop.rs
//! Create the goal-driven agent loop and App struct.

use std::sync::Arc;

use crate::api::acp::AcpRouter;
use crate::api::bridge::app::state::App;
use crate::api::developer_interface::full::{
    ExplorerTarget, explore_cognitive_system, inspect_background_workers, inspect_memory_by_layer,
    inspect_strategic_learning,
};
use crate::api::developer_interface::{
    ControlCommand, ControlPlaneArchitecture, DeveloperInterface,
};
use crate::api::mcp::McpContext;

use crate::experience::encounter_recorder::ExperienceRecorder;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::integration::reflection_pipeline::ReflectionPipeline;
use crate::knowledge::KnowledgeStore;
use crate::knowledge::world_model::WorldModel;
use crate::memory::MemoryEngine;
use crate::memory::pipeline::MemoryPipeline;
use crate::storage::StorageArchitecture;
use crate::workers::BackgroundWorkerSystem;

/// Grouped subsystem handles passed to `create_app`.
pub struct AgentLoopSystems {
    pub mcp_context: Arc<McpContext>,
    pub shared_personality: Arc<std::sync::Mutex<crate::common::personality::Personality>>,
    pub hypothesis_engine: Arc<std::sync::Mutex<HypothesisEngine>>,
    pub experience_recorder: Arc<ExperienceRecorder>,
    pub reflection_pipeline: Arc<ReflectionPipeline>,
    pub memory_pipeline: Arc<MemoryPipeline>,
    pub acp_router: Arc<AcpRouter>,
    pub world_model: Arc<WorldModel>,
    pub conversation_engine: Arc<std::sync::Mutex<crate::conversation::ConversationEngine>>,
    /// Memory Hierarchy: manages promotion pipeline between memory layers
    /// (Architecture §14).
    pub memory_hierarchy: Arc<std::sync::Mutex<crate::memory::hierarchy::MemoryHierarchy>>,
    /// Confidence System: tracks confidence scores with decay (Architecture §19).
    pub confidence_system: Arc<std::sync::Mutex<crate::memory::ConfidenceSystem>>,
    /// Database handle for ContextEngine wiring.
    pub database: Arc<crate::database::sqlite::SqliteDatabase>,
    /// Knowledge store for ContextEngine wiring.
    pub knowledge_store: Arc<KnowledgeStore>,
    /// Background Worker System for async task scheduling (Architecture §23.7).
    pub worker_system: BackgroundWorkerSystem,
    /// Storage Architecture: storage layers, policies, audit (Architecture Chapter 21).
    pub storage: StorageArchitecture,
    /// Unified Memory Engine: wraps all memory subsystems (Architecture §8-9).
    pub memory_engine: Arc<MemoryEngine>,
}

/// Create the goal-driven agent loop and the App struct.
///
/// Composes the already-initialized planner, memory retrieval, knowledge store,
/// coordinator and database into a single cognitive loop (Architecture §5.7),
/// then returns the App with all subsystems.
pub fn create_app(systems: AgentLoopSystems) -> App {
    let AgentLoopSystems {
        mcp_context,
        shared_personality,
        hypothesis_engine,
        experience_recorder,
        reflection_pipeline,
        memory_pipeline,
        acp_router,
        world_model,
        conversation_engine,
        memory_hierarchy,
        confidence_system,
        database,
        knowledge_store,
        worker_system,
        storage,
        memory_engine,
    } = systems;
    // Context Engine: assemble temporary cognitive environments
    // (Architecture Chapter 7). Wire database and knowledge store.
    let mut context_engine = crate::context::ContextEngine::new();
    context_engine.wire(database.clone(), knowledge_store.clone());
    // Clone the engine for the agent loop (ContextEngine implements Clone).
    // Both copies share the same Arc<Database> and Arc<KnowledgeStore> internally.
    let agent_context_engine = context_engine.clone();

    // Goal-driven agent loop (Architecture §5.7, TASK-V2-04).
    let agent_deps = crate::api::bridge::app::agent::AgentDeps::from_context_with_engine(
        &mcp_context,
        shared_personality.clone(),
        std::sync::Mutex::new(agent_context_engine),
    );
    let agent_loop = Arc::new(crate::api::bridge::app::agent::AgentLoop::new(agent_deps));

    // Developer Interface: control plane for visibility and debugging
    // (Architecture Chapter 28).
    let developer_interface = Arc::new(std::sync::Mutex::new(DeveloperInterface::new()));

    // Startup exercise: construct all 7 pub structs and call inspect functions
    // to verify the developer interface is wired end-to-end (Architecture Chapter 28).
    if let Ok(mut di) = developer_interface.lock() {
        let cmd = ControlCommand::new("init");
        di.execute_command(cmd);
    }
    ControlPlaneArchitecture::default();
    explore_cognitive_system(ExplorerTarget::Memory);
    inspect_memory_by_layer("test");
    inspect_strategic_learning();
    inspect_background_workers();

    App {
        hypothesis_engine,
        experience_recorder,
        reflection_pipeline,
        memory_pipeline,
        mcp_context,
        personality: shared_personality,
        acp_router,
        agent_loop,
        world_model,
        conversation_engine,
        memory_hierarchy,
        confidence_system,
        developer_interface,
        context_engine: std::sync::Mutex::new(context_engine),
        storage: std::sync::Mutex::new(storage),
        worker_system,
        memory_engine,
    }
}
