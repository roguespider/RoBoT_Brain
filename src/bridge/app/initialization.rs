// src/bridge/app/initialization.rs
//! Application initialization logic

use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;

use crate::bridge::acp::{AcpRegistry, AcpRouter};
use crate::bridge::mcp::McpClient;
use crate::bridge::mcp::McpContext;
use crate::bridge::rmcp::run_stdio_server;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::encounter_recorder::ExperienceRecorder;
use crate::experience::event_handler::EventHandler;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::integration::event_subscriber::{start_event_subscriber, EventSubscriber};
use crate::experience::integration::reflection_pipeline::ReflectionPipeline;
use crate::experience::metrics::MetricsCollector;
use crate::experience::observer::{HypothesisObserver, MetricsObserver, ReputationObserver};
use crate::experience::reflection::ReflectionEngine;
use crate::experience::scorer::ExperienceScorer;
use crate::experience::worker_manager::WorkerManager;
use crate::knowledge::KnowledgeStore;
use crate::memory::{MemoryRetrieval, PermanentMemory, WorkingMemory as MemWorkingMemory};
use crate::personality::Personality;
use crate::planner::{Planner, PolicyEngine};
use crate::skills::registry::SkillRegistry;
use crate::workflows::engine::WorkflowEngine;

use super::state::App;
use super::scheduler;

impl App {
    /// Build the application.
    pub async fn new() -> Result<Self> {
        // Initialize database
        let database = Arc::new(SqliteDatabase::initialize()?);

        // Create shared personality instance (used by both App and planner)
        let shared_personality = Arc::new(std::sync::Mutex::new(Personality::new()));

        // Create core systems
        let bus = Arc::new(ExperienceBus::new());
        let metrics = Arc::new(MetricsCollector::new());
        let scorer = ExperienceScorer::new();
        let coordinator = Arc::new(ExperienceCoordinator::new(
            scorer,
            bus.clone(),
            metrics.clone(),
        ));

        // Create experience recorder for structured experience creation (Architecture §07)
        let experience_recorder = Arc::new(ExperienceRecorder::new(database.clone()));

        // Start event handler to process events from the bus
        let event_handler = EventHandler::new(bus.clone());
        event_handler.start();
        tracing::info!("Event handler started");

        // Create learning engines first (needed for observers)
        let reflection_engine = Arc::new(ReflectionEngine::new());
        let hypothesis_engine_for_subscriber = Arc::new(HypothesisEngine::new());
        let hypothesis_engine = Arc::new(Mutex::new(HypothesisEngine::new()));
        let evolution_engine = Arc::new(EvolutionEngine::new());
        let metrics = Arc::new(MetricsCollector::new());

        // Create WorkerManager for background job processing per Architecture §22
        // Design: Experience → Recorder → Bus → Job Queue → Workers → Observers
        let worker_manager = Arc::new(WorkerManager::new(bus.clone()));

        // Register all observers with WorkerManager per Architecture §22
        // Each observer runs in its own dedicated worker

        // 1. ExperienceScorer - scores experiences
        let scorer = Arc::new(ExperienceScorer::new())
            as Arc<dyn crate::experience::observer::ExperienceObserver>;
        worker_manager.register_observer(scorer).await?;
        tracing::info!("ExperienceScorer registered with WorkerManager");

        // 2. ReputationObserver - updates entity reputations
        let reputation_observer = Arc::new(ReputationObserver::new())
            as Arc<dyn crate::experience::observer::ExperienceObserver>;
        worker_manager
            .register_observer(reputation_observer)
            .await?;
        tracing::info!("ReputationObserver registered with WorkerManager");

        // 3. HypothesisObserver - generates and evaluates hypotheses
        let hypothesis_observer = Arc::new(HypothesisObserver::new(hypothesis_engine.clone()))
            as Arc<dyn crate::experience::observer::ExperienceObserver>;
        worker_manager
            .register_observer(hypothesis_observer)
            .await?;
        tracing::info!("HypothesisObserver registered with WorkerManager");

        // 4. MetricsObserver - collects metrics from all events
        let metrics_observer = Arc::new(MetricsObserver::new(metrics.clone()))
            as Arc<dyn crate::experience::observer::ExperienceObserver>;
        worker_manager.register_observer(metrics_observer).await?;
        tracing::info!("MetricsObserver registered with WorkerManager");

        // Start worker manager background task - subscribes to bus and enqueues jobs
        let manager_clone = worker_manager.clone();
        let manager_bus = bus.clone();
        tokio::spawn(async move {
            let mut receiver = manager_bus.subscribe();
            tracing::info!("Worker manager started, listening for events");
            tracing::debug!(
                "Event bus subscriber count: {}",
                manager_bus.subscriber_count()
            );
            while let Ok(event) = receiver.recv().await {
                // Broadcast to all workers - they filter based on accepts()
                if let Err(e) = manager_clone.broadcast_event(event).await {
                    tracing::error!("Worker manager broadcast error: {}", e);
                }
            }
            manager_bus.unsubscribe();
        });
        tracing::info!(
            "Worker manager subscribed to bus (total subscribers: {})",
            bus.subscriber_count()
        );

        // Create working memory, lineage tracker, and knowledge store
        let knowledge_store = Arc::new(KnowledgeStore::new(10000));

        // Create skills registry - manages reusable capabilities (Architecture §15)
        let skills_registry = Arc::new(SkillRegistry::new());
        skills_registry.load_defaults().await;
        tracing::info!("Skills registry initialized with default skills");

        // Create event subscriber for the learning pipeline
        // Per Architecture §4.04: Experience → Reflection → Hypothesis → Knowledge → Reputation
        let event_subscriber = Arc::new(EventSubscriber::with_coordinator(
            coordinator.clone(),
            metrics.clone(),
            reflection_engine.clone(),
            hypothesis_engine_for_subscriber.clone(),
            evolution_engine.clone(),
            knowledge_store.clone(),
        ));

        // Create reflection pipeline for processing experiences into insights
        let reflection_pipeline = Arc::new(ReflectionPipeline::new(
            reflection_engine.clone(),
            bus.clone(),
        ));

        // Start the event subscriber background task
        start_event_subscriber(bus.clone(), event_subscriber);
        tracing::info!("Event subscriber started for learning pipeline");

        // Create memory system - Working and Permanent Memory (Architecture §6.3)
        let working_memory_core = Arc::new(MemWorkingMemory::new(1000));
        let permanent_memory = Arc::new(PermanentMemory::new(10000));
        let memory_retrieval = Arc::new(MemoryRetrieval::new(
            working_memory_core.clone(),
            permanent_memory.clone(),
        ));

        // Create memory pipeline for working→permanent consolidation (Architecture §6.3, §07)
        let memory_pipeline = Arc::new(crate::memory::pipeline::MemoryPipeline::new(
            database.clone(),
        ));

        // Load memories from database into in-memory caches on startup
        // This restores the caches from persistent storage
        if let Err(e) = working_memory_core.load_from_database(&database).await {
            tracing::warn!("Failed to load working memory from database: {}", e);
        }
        if let Err(e) = permanent_memory.load_from_database(&database).await {
            tracing::warn!("Failed to load permanent memory from database: {}", e);
        }
        tracing::info!(
            "Memory system initialized and loaded from database (Working: 1000, Permanent: 10000)"
        );

        // Create scheduler with background tasks (metrics already created above)
        let scheduler = scheduler::setup_scheduler(database.clone()).await?;

        // Register task handlers with access to all required engines
        scheduler::register_task_handlers(
            scheduler.clone(),
            memory_retrieval.clone(),
            reflection_engine.clone(),
            hypothesis_engine.clone(),
            evolution_engine.clone(),
            metrics.clone(),
            database.clone(),
        )
        .await;

        // Start scheduler background loop
        let scheduler_clone = scheduler.clone();
        tokio::spawn(async move {
            if let Err(e) = scheduler_clone.run().await {
                tracing::error!("Scheduler error: {}", e);
            }
        });
        tracing::info!("Scheduler background loop started");

        // Create planning system (Architecture §4.03.5, §10)
        let mut planner = Planner::new(metrics.clone());
        let policy_engine = Arc::new(PolicyEngine::new());

        // Load default policy rules
        policy_engine.load_defaults().await;
        tracing::info!("Policy engine loaded with default rules");

        // Wire personality creativity into planner for decision-making
        let shared_personality_clone = shared_personality.clone();
        planner.set_creativity_check(move |complexity: f32| {
            match shared_personality_clone.lock() {
                Ok(guard) => guard.should_use_creativity(complexity),
                Err(poisoned) => {
                    tracing::error!("Personality mutex poisoned in creativity check");
                    poisoned.into_inner().should_use_creativity(complexity)
                }
            }
        });
        let planner = Arc::new(planner);

        // Create workflow engine with database access and coordinator for event integration
        // This ensures workflow experiences flow to WorkerManager and EventSubscriber
        let workflow_engine = Arc::new(WorkflowEngine::with_database_and_coordinator(
            metrics.clone(),
            database.clone(),
            coordinator.clone(),
        ));
        tracing::info!("Workflow engine initialized with coordinator");

        // Create MCP context with all systems
        let mcp_context = Arc::new(McpContext::new(
            database.clone(),
            bus.clone(),
            coordinator.clone(),
            worker_manager.clone(),
            reflection_engine.clone(),
            evolution_engine.clone(),
            scheduler.clone(),
            metrics.clone(),
            knowledge_store.clone(),
            planner.clone(),
            policy_engine.clone(),
            working_memory_core.clone(),
            permanent_memory.clone(),
            memory_retrieval.clone(),
            workflow_engine.clone(),
            skills_registry.clone(),
        ));

        // Register MCP tools
        crate::bridge::tools::register_tools();

        // Create MCP client for external connections and initialize globally
        crate::bridge::tools::agent::init_mcp_client(Arc::new(McpClient::new()));

        tracing::info!("RoBoT initialized successfully");

        Ok(Self {
            database,
            bus,
            worker_manager,
            coordinator,
            hypothesis_engine,
            experience_recorder,
            reflection_pipeline,
            scheduler,
            memory_pipeline,
            mcp_context,
            personality: shared_personality,
            acp_router: Arc::new(AcpRouter::new(Arc::new(AcpRegistry::new()))),
        })
    }

    /// Start the runtime.
    pub async fn run(self) -> Result<()> {
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
