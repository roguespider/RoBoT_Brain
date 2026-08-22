// src/bridge/app/initialization/mod.rs
//! Application initialization logic

pub mod acp_diagnostics;
pub mod agent_loop;
pub mod candidates;
pub mod core;
pub mod db;
pub mod diagnostics;
pub mod engines;
pub mod experience_repo;
pub mod exploration_repo;
pub mod hypothesis_manager;
pub mod job_queue;
pub mod learning;
pub mod learning_diagnostics;
pub mod learning_pipeline;
pub mod lineage_tracker;
pub mod mcp_client_diagnostics;
pub mod mcp_context;
pub mod memory_scheduler;
pub mod personality_diagnostics;
pub mod policy;
pub mod scheduler_diagnostics;
pub mod sub_health_log;
pub mod worker_diagnostics;
pub mod workers;
pub mod workflow_acp;
pub mod working_memory;

use std::sync::Arc;

use anyhow::Result;

use crate::bridge::mcp::McpClient;
use crate::bridge::rmcp::run_stdio_server;
use crate::experience::integration::event_subscriber::start_event_subscriber;
use crate::planner::PolicyEngine;

use super::state::App;

impl App {
    /// Build the application.
    pub async fn new() -> Result<Self> {
        // Initialize database
        let database = crate::bridge::app::initialization::db::init_database()?;

        // Build core infrastructure (personality, bus, coordinator, recorder, queue, workers)
        let (core, event_handler) =
            crate::bridge::app::initialization::core::build_core(database.clone());
        tracing::info!(
            "Event handler running: {} bus subscriber(s)",
            event_handler.subscriber_count()
        );
        let shared_personality = core.shared_personality.clone();
        let bus = core.bus.clone();
        // Rebind the core MetricsCollector so the CoreInfra field stays live;
        // it is the same collector the ExperienceCoordinator records into.
        let core_metrics = core.metrics.clone();
        let experience_recorder = core.experience_recorder.clone();
        let coordinator = core.coordinator;
        let job_queue = core.job_queue.clone();
        let worker_manager = core.worker_manager.clone();
        // Rebind to the Arc held by CoreInfra so the field stays live and both
        // halves of initialization share the exact same database handle.
        let database = core.database;
        tracing::info!("Core infrastructure ready: coordinator shares core metrics collector");
        core_metrics.set_gauge_sync("system.startup.core_infra_ready", 1.0);

        // Build learning engines (reflection, hypothesis, evolution, metrics)
        let engines = crate::bridge::app::initialization::engines::build_engines();
        let reflection_engine = engines.reflection_engine;
        let hypothesis_engine_for_subscriber = engines.hypothesis_engine_for_subscriber;
        let hypothesis_engine = engines.hypothesis_engine;
        let evolution_engine = engines.evolution_engine;
        let metrics = engines.metrics;

        // Register all observers and dispatch restored jobs
        crate::bridge::app::initialization::workers::setup_workers(
            &worker_manager,
            &bus,
            &metrics,
            &hypothesis_engine,
        )
        .await?;

        // Build knowledge/skills/coordinator/event-subscriber/reflection pipeline
        let learning = crate::bridge::app::initialization::learning::build_learning_pipeline(
            &database,
            &reflection_engine,
            &hypothesis_engine_for_subscriber,
            &evolution_engine,
            &bus,
            &metrics,
            &experience_recorder,
        )
        .await;
        let knowledge_store = learning.knowledge_store;
        let skills_registry = learning.skills_registry;
        let learning_coordinator = learning.learning_coordinator;
        let event_subscriber = learning.event_subscriber;
        let reflection_pipeline = learning.reflection_pipeline;
        start_event_subscriber(bus.clone(), event_subscriber);
        tracing::info!("Event subscriber started for learning pipeline");

        // Build memory system and scheduler
        let mem_sched =
            crate::bridge::app::initialization::memory_scheduler::build_memory_scheduler(
                &database,
                &knowledge_store,
                &learning_coordinator,
                &reflection_engine,
                &hypothesis_engine,
                &evolution_engine,
                &metrics,
            )
            .await?;
        let working_memory_core = mem_sched.working_memory_core;
        let permanent_memory = mem_sched.permanent_memory;
        let memory_retrieval = mem_sched.memory_retrieval;
        let memory_pipeline = mem_sched.memory_pipeline;
        let scheduler = mem_sched.scheduler;

        // Create planning system (Architecture §4.03.5, §10)
        let policy_engine = Arc::new(PolicyEngine::new());

        // Policy engine: load production defaults (management-method
        // verification runs only via explicit diagnostics).
        policy::setup_policy_engine(&policy_engine).await;

        // Build planner, workflow engine, and ACP router/registry.
        // Pass the core MetricsCollector (the one the ExperienceCoordinator
        // records into) so planner/workflow metrics land in the same store.
        let (planner, workflow_engine, acp_router, acp_registry) =
            workflow_acp::setup_planner_workflow_acp(
                &core_metrics,
                &database,
                &coordinator,
                &policy_engine,
                &shared_personality,
            );

        // Create MCP context with all systems
        let world_model = Arc::new(crate::world_model::WorldModel::new());
        let enforcer = Arc::new(crate::workflows::enforcement::WorkflowEnforcer::new());
        let mcp_context = mcp_context::create_mcp_context(mcp_context::McpContextSystems {
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
            acp_router: Arc::clone(&acp_router),
            acp_registry,
            shared_personality: Arc::clone(&shared_personality),
            world_model: Arc::clone(&world_model),
            enforcer,
        });

        // Register MCP tools
        crate::bridge::tools::register_tools();

        // Create MCP client for external connections and initialize globally
        let mcp_client = Arc::new(McpClient::new());
        crate::bridge::tools::agent::init_mcp_client(mcp_client.clone());

        tracing::info!("RoBoT initialized successfully");

        // Goal-driven agent loop + App struct (Architecture §5.7)
        let app = agent_loop::create_app(agent_loop::AgentLoopSystems {
            mcp_context,
            shared_personality,
            hypothesis_engine,
            experience_recorder,
            reflection_pipeline,
            memory_pipeline,
            acp_router,
            world_model,
        });

        Ok(app)
    }

    /// Start the runtime.
    pub async fn run(self) -> Result<()> {
        // Production startup: no subsystem probes or self-checks run here.
        // All diagnostics are available explicitly via `robot diagnose`
        // (P2-001A/P2-001C).

        // Start background scheduler worker
        let scheduler = self.mcp_context.scheduler.clone();
        tokio::spawn(async move {
            if let Err(e) = scheduler.run().await {
                tracing::error!("Scheduler error: {}", e);
            }
        });

        // Run the MCP server with stdio transport
        run_stdio_server(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            self.mcp_context.clone(),
        )
        .await
    }
}
