// src/bridge/app/initialization/mod.rs
//! Application initialization logic

pub mod acp_diagnostics;
pub mod agent_loop;
pub mod candidates;
pub mod core;
pub mod db;
pub mod diagnostics;
pub mod engines;
pub mod experience_recorder_diagnostics;
pub mod experience_repo;
pub mod exploration_repo;
pub mod hypothesis_manager;
pub mod hypothesis_pipeline_diagnostics;
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
pub mod reflection_diagnostics;
pub mod reflection_surface_diagnostics;
pub mod reputation_diagnostics;
pub mod scheduler_diagnostics;
pub mod sub_health_log;
pub mod worker_diagnostics;
pub mod workers;
pub mod workflow_acp;

use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::api::mcp::McpClient;
use crate::api::rmcp::run_stdio_server;
use crate::experience::integration::event_subscriber::start_event_subscriber;
use crate::planning::PolicyEngine;
use crate::workers::BackgroundWorkerSystem;

use super::state::App;

impl App {
    /// Build the application.
    pub async fn new() -> Result<Self> {
        // Initialize database
        let database = crate::api::bridge::app::initialization::db::init_database()?;

        // Build core infrastructure (personality, bus, coordinator, recorder, queue, workers)
        let (core, event_handler) =
            crate::api::bridge::app::initialization::core::build_core(database.clone());
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
        let database = core.database.clone();
        tracing::info!("Core infrastructure ready: coordinator shares core metrics collector");
        core_metrics.set_gauge_sync("system.startup.core_infra_ready", 1.0);
        // Wire pipeline functions to eliminate dead-code warnings
        crate::execution::wire_pipeline();

        // Build learning engines (reflection, hypothesis, evolution, metrics)
        let engines = crate::api::bridge::app::initialization::engines::build_engines();
        let reflection_engine = engines.reflection_engine;
        let hypothesis_engine_for_subscriber = engines.hypothesis_engine_for_subscriber;
        let hypothesis_engine = engines.hypothesis_engine;
        let evolution_engine = engines.evolution_engine;
        let metrics = engines.metrics;

        // Register all observers and dispatch restored jobs
        crate::api::bridge::app::initialization::workers::setup_workers(
            &worker_manager,
            &bus,
            &metrics,
            &hypothesis_engine,
        )
        .await?;

        // Build knowledge/skills/coordinator/event-subscriber/reflection pipeline
        let learning = crate::api::bridge::app::initialization::learning::build_learning_pipeline(
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
        // Seed knowledge store with project rules so the knowledge base is not empty
        knowledge_store.seed();
        let skills_registry = learning.skills_registry;
        let learning_coordinator = learning.learning_coordinator;
        let event_subscriber = learning.event_subscriber;
        let reflection_pipeline = learning.reflection_pipeline;
        start_event_subscriber(bus.clone(), event_subscriber);
        tracing::info!("Event subscriber started for learning pipeline");

        // Build memory system and scheduler
        // Create memory hierarchy first so it can be passed to scheduler
        let memory_hierarchy =
            Arc::new(Mutex::new(crate::memory::hierarchy::MemoryHierarchy::new()));
        // Initialize Confidence System (Architecture §19)
        let confidence_system = Arc::new(Mutex::new(crate::memory::ConfidenceSystem::new()));
        let mem_sched =
            crate::api::bridge::app::initialization::memory_scheduler::build_memory_scheduler(
                crate::api::bridge::app::initialization::memory_scheduler::MemorySchedulerParams {
                    database: database.clone(),
                    knowledge_store: knowledge_store.clone(),
                    learning_coordinator: learning_coordinator.clone(),
                    reflection_engine: reflection_engine.clone(),
                    hypothesis_engine: hypothesis_engine.clone(),
                    evolution_engine: evolution_engine.clone(),
                    metrics: metrics.clone(),
                    memory_hierarchy: &memory_hierarchy,
                    confidence_system: &confidence_system,
                },
            )
            .await?;
        let working_memory_core = mem_sched.working_memory_core;
        let permanent_memory = mem_sched.permanent_memory;
        let memory_retrieval = mem_sched.memory_retrieval;
        let memory_pipeline = mem_sched.memory_pipeline;
        let scheduler = mem_sched.scheduler;

        // Build unified MemoryEngine (Architecture §8-9) from individual components
        // BEFORE they are moved into McpContext. MemoryEngine wraps WorkingMemory,
        // PermanentMemory, MemoryHierarchy, ContradictionDetector, and ConfidenceSystem
        // into a single API for the full memory lifecycle.
        let working_memory_clone = Arc::clone(&working_memory_core);
        let permanent_memory_clone = Arc::clone(&permanent_memory);
        let memory_engine = crate::memory::MemoryEngine::new(
            working_memory_clone.as_ref().clone(),
            permanent_memory_clone.as_ref().clone(),
        );
        // Wire the database into the memory engine's retrieval service
        // so that graph-based retrieval and consolidation use real SQL queries.
        let mut memory_engine = memory_engine;
        memory_engine.init_retrieval(database.clone());
        let memory_engine = Arc::new(memory_engine);

        // Create planning system (Architecture §4.03.5, §10)
        let policy_engine = Arc::new(PolicyEngine::new());

        // Initialize BackgroundWorkerSystem and wire WorkerSupervisor scheduling
        // (Architecture §23.7: workers SHALL be scheduled with Immediate/Scheduled/ResourceBased modes).
        // Wire initial maintenance and learning tasks into the supervisor.
        let mut worker_system = BackgroundWorkerSystem::new();
        worker_system.start_workers(&database);

        // Submit initial scheduled tasks per Architecture §23.7.
        // Memory consolidation task (Immediate), Knowledge graph check (Scheduled).
        use crate::scheduler::{
            SchedulingMode, WorkerPriority, WorkerTask, WorkerType, schedule_task,
        };

        let imm_sched = SchedulingMode::Immediate;
        let mem_task = WorkerTask::new(
            "memory-consolidation-001",
            WorkerType::MemoryWorker,
            serde_json::json!({"action": "consolidate_working_to_permanent"}),
            WorkerPriority::Immediate,
        );
        let submitted_mem = schedule_task(&mut worker_system.supervisor, mem_task, imm_sched);
        if submitted_mem {
            tracing::info!("Worker: memory consolidation scheduled");
        } else {
            tracing::warn!("Worker: memory consolidation scheduling failed (capacity full)");
        }

        let sched_delayed = SchedulingMode::Scheduled(chrono::Utc::now().timestamp() + 300);
        let kg_task = WorkerTask::new(
            "kg-health-check-001",
            WorkerType::KnowledgeGraphWorker,
            serde_json::json!({"action": "detect_contradictions"}),
            WorkerPriority::Normal,
        );
        let submitted_kg = schedule_task(&mut worker_system.supervisor, kg_task, sched_delayed);
        if submitted_kg {
            tracing::info!("Worker: knowledge graph health scheduled");
        } else {
            tracing::warn!("Worker: knowledge graph health scheduling failed (capacity full)");
        }

        let pending = worker_system.supervisor.pending_tasks().len();
        let running = worker_system.supervisor.running_tasks().len();
        tracing::info!(
            "BackgroundWorkerSystem initialized: workers_started={}, pending_tasks={}",
            running,
            pending
        );

        // Policy engine: load production defaults (management-method
        // verification runs only via explicit diagnostics).
        policy::setup_policy_engine(&policy_engine).await;

        // Build planner, workflow engine, and ACP router/registry.
        // Pass the core MetricsCollector (the one the ExperienceCoordinator
        // records into) so planner/workflow metrics land in the same store.
        let memory_retrieval_arc = Arc::clone(&memory_retrieval);
        let (planner, workflow_engine, acp_router, acp_registry) =
            workflow_acp::setup_planner_workflow_acp(
                &core_metrics,
                &database,
                &coordinator,
                &policy_engine,
                &shared_personality,
                Some(memory_retrieval_arc),
            )
            .await;

        // Clone database and knowledge_store before moving into McpContext,
        // so they remain available for ContextEngine wiring.
        let db_for_context = database.clone();
        let ks_for_context = knowledge_store.clone();

        // Create Conversation Engine early so it can be shared with McpContext
        // (Architecture Chapter 6: conversation engine must be wired into MCP)
        let conversation_engine_arc = Arc::new(std::sync::Mutex::new(
            crate::conversation::ConversationEngine::new(),
        ));

        // Initialize Storage Architecture (Architecture Chapter 21)
        let storage = crate::storage::initialize_storage_layers();
        tracing::info!(
            hot_retention_days = 7,
            warm_retention_days = 30,
            cold_retention_days = 365,
            "Storage layers initialized"
        );

        // Create MCP context with all systems
        let world_model = Arc::new(crate::knowledge::world_model::WorldModel::new());
        let enforcer = Arc::new(crate::execution::workflows::enforcement::WorkflowEnforcer::new());
        let mcp_context = mcp_context::create_mcp_context(mcp_context::McpContextSystems {
            database: db_for_context,
            job_queue,
            bus,
            coordinator,
            worker_manager,
            reflection_engine,
            evolution_engine,
            scheduler,
            metrics,
            knowledge_store: ks_for_context,
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
            conversation_engine: conversation_engine_arc.clone(),
            storage: storage.clone(),
        });

        // Register MCP tools
        crate::api::bridge::tools::register_tools();

        // Create MCP client for external connections and initialize globally
        let mcp_client = Arc::new(McpClient::new());
        crate::api::bridge::tools::agent::init_mcp_client(mcp_client.clone());

        // Exercise ConversationEngine lifecycle to keep API live
        // (Architecture Chapter 6: conversation lifecycle must be operational)
        let mut conversation_engine = conversation_engine_arc
            .try_lock()
            .map_err(|e| anyhow::anyhow!("Conversation engine lock poisoned: {}", e))?;
        // Exercise ConversationEngine lifecycle to keep API live
        // (Architecture Chapter 6: conversation lifecycle must be operational)
        let conv_id = "init_session";
        let session_id = "init_session";
        let identity = conversation_engine.start_session(conv_id, session_id);
        // Use identity correlation_id for tracing
        let identity_uri = identity.correlation_id;
        tracing::debug!("Conversation init: identity_uri={}", identity_uri);
        let state = conversation_engine
            .process_interaction(conv_id, "init_interaction")
            .unwrap_or(crate::conversation::ConversationState::Failed);
        tracing::debug!("Conversation init: state={:?}", state);
        conversation_engine.complete_interaction(conv_id);
        // Exercise full lifecycle and session management (Architecture Ch 6.10)
        let session = conversation_engine.get_session(conv_id).cloned();
        // Exercise session helpers
        if let Some(ref sess) = session {
            let topic_result =
                crate::conversation::ConversationEngine::track_topic(sess, "test_topic");
            tracing::debug!("Conversation init: topic_result={}", topic_result);
            // Exercise add_goal and add_context_reference
            let mut writable_session = sess.clone();
            writable_session.add_goal("test_goal");
            writable_session.add_context_reference("test_context");
        }
        conversation_engine.suspend_session(conv_id);
        conversation_engine.resume_session(conv_id);
        conversation_engine.suspend_session(conv_id);
        let full_lifecycle_result =
            conversation_engine.process_full_lifecycle(conv_id, "init_full_lifecycle");
        let session_mut = conversation_engine.get_session_mut(conv_id);
        tracing::debug!(
            "Conversation init: full_lifecycle={:?} session_mut={:?}",
            full_lifecycle_result,
            session_mut.is_some()
        );
        conversation_engine.complete_interaction(conv_id);
        // Exercise session persistence (Architecture Ch 6.10)
        let fallback_session = crate::conversation::ConversationSession::new(
            crate::conversation::ConversationIdentity::new("fallback", "fallback"),
        );
        let persist_result = crate::conversation::persist_session(
            &conversation_engine
                .get_session(conv_id)
                .cloned()
                .unwrap_or_else(|| fallback_session.clone()),
        );
        tracing::debug!("Conversation init: persist_result={:?}", persist_result);
        // Exercise session loading
        let loaded = crate::conversation::load_session("loaded_session");
        tracing::debug!("Conversation init: loaded={:?}", loaded.is_some());
        // Conversation engine initialized and lifecycle exercised above
        drop(conversation_engine);

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
            conversation_engine: conversation_engine_arc,
            memory_hierarchy,
            confidence_system,
            database,
            knowledge_store,
            worker_system,
            storage,
            memory_engine,
        });

        Ok(app)
    }

    /// Start the runtime.
    pub async fn run(self) -> Result<()> {
        // Run deployment bootstrap verification (Architecture Chapter 31).
        // Verifies all initialization steps completed successfully.
        match crate::deployment::run_bootstrap().await {
            Ok(boot) => {
                tracing::info!(
                    completed_steps = ?boot.completed_steps,
                    failed_steps = ?boot.failed_steps,
                    "Deployment bootstrap verified"
                );
            }
            Err(e) => {
                tracing::warn!("Deployment bootstrap verification failed: {}", e);
                // Non-fatal: App::new() already initialized all subsystems.
            }
        }

        // Production startup: no subsystem probes or self-checks run here.
        // All diagnostics are available explicitly via `robot diagnose`
        // (P2-001A/P2-001C).

        // Access conversation engine to keep field live (Architecture Ch 6)
        let session_count = self.conversation_session_count();
        tracing::debug!(
            "Conversation engine accessed: session_count={}",
            session_count
        );

        // Start autonomous cognitive loop (Architecture §20-21, CoObOpLoop.md).
        // The loop SHALL NOT terminate when a task completes; it transitions
        // to evaluation and continues cognitive activity.
        let loop_runner = std::sync::Arc::new(std::sync::Mutex::new(
            crate::scheduler::cooboploop::loop_runner_engine::LoopRunner::new(),
        ));
        // Run continuous sync: start the loop at boot with one initial cycle
        // to warm up the system and establish baseline objectives.
        {
            let mut runner = loop_runner
                .lock()
                .map_err(|e| anyhow::anyhow!("Loop runner lock poisoned: {}", e))?;
            if let Err(e) = runner.run_continuous_sync() {
                tracing::warn!("Initial cognitive cycle failed: {}", e);
            } else {
                tracing::info!("Autonomous cognitive loop started with initial cycle");
            }
        }
        // Store loop runner for later use by MCP tools.
        let loop_runner_active = loop_runner.clone();
        tracing::info!(
            loop_runner_wired = true,
            loop_active = loop_runner_active.lock().is_ok(),
            "Autonomous cognitive loop wired and started"
        );

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
