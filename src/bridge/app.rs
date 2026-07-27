

// src/bridge/app.rs
// Root application container per Architecture §03

use std::sync::Arc;

use anyhow::Result;

use crate::bridge::mcp::McpClient;
use crate::bridge::mcp::McpContext;
use crate::bridge::rmcp::run_stdio_server;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::event_handler::EventHandler;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::integration::event_subscriber::{EventSubscriber, start_event_subscriber};
use crate::experience::metrics::MetricsCollector;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::scheduler::{Scheduler, TaskSchedule, TaskType};
use crate::experience::scorer::ExperienceScorer;
use crate::experience::worker::{ExperienceWorker, ObserverJob};
use crate::knowledge::KnowledgeStore;
use crate::learning::{LineageTracker, WorkingMemory};
use crate::memory::{MemoryRetrieval, PermanentMemory, WorkingMemory as MemWorkingMemory};
use crate::planner::{Planner, PolicyEngine};
use crate::tools;
use crate::workflows::engine::WorkflowEngine;
use tokio::sync::mpsc;

/// Root application container.
///
/// Owns long-running services required by RoBoT.
pub struct App {
    /// Persistent database layer.
    _database: Arc<SqliteDatabase>,

    /// Event bus for pub/sub.
    bus: Arc<ExperienceBus>,

    /// Experience system coordinator.
    coordinator: Arc<ExperienceCoordinator>,

    /// Background task scheduler.
    scheduler: Arc<Scheduler>,

    /// MCP context shared with bridge - owns all subsystems.
    mcp_context: Arc<McpContext>,
}

impl App {
    /// Build the application.
    pub async fn new() -> Result<Self> {
        // Initialize database
        let database = Arc::new(SqliteDatabase::initialize()?);

        // Create core systems
        let bus = Arc::new(ExperienceBus::new());
        let metrics = Arc::new(MetricsCollector::new());
        let scorer = ExperienceScorer::new();
        let coordinator = Arc::new(ExperienceCoordinator::new(scorer, bus.clone(), metrics.clone()));

        // Start event handler to process events from the bus
        let event_handler = EventHandler::new(bus.clone());
        event_handler.start();
        tracing::info!("Event handler started");

        // Wire ExperienceScorer as an observer that scores experiences
        // Create a separate scorer instance for the observer
        let scorer_for_observer = ExperienceScorer::new();
        let scorer_observer: Arc<dyn crate::experience::observer::ExperienceObserver> = Arc::new(scorer_for_observer);
        
        // Subscribe scorer to events (it will score experiences)
        let scorer_bus = bus.clone();
        tokio::spawn(async move {
            let mut receiver = scorer_bus.subscribe();
            tracing::info!("Experience scorer observer started, listening for events");
            while let Ok(event) = receiver.recv().await {
                if scorer_observer.accepts(&event) {
                    if let Err(e) = scorer_observer.observe(&event) {
                        tracing::error!("Scorer observer error: {}", e);
                    }
                }
            }
        });
        tracing::info!("Experience scorer observer subscribed to bus");

        // Create learning engines
        let reflection_engine = Arc::new(ReflectionEngine::new());
        let hypothesis_engine = Arc::new(HypothesisEngine::new());
        let evolution_engine = Arc::new(EvolutionEngine::new());

        // Create working memory, lineage tracker, and knowledge store
        let _working_memory = Arc::new(WorkingMemory::new(1000));
        let _lineage_tracker = Arc::new(LineageTracker::new());
        let knowledge_store = Arc::new(KnowledgeStore::new(10000));

        // Create event subscriber for the learning pipeline
        // Per Architecture §4.04: Experience → Reflection → Hypothesis → Knowledge → Reputation
        let event_subscriber = Arc::new(EventSubscriber::new(
            reflection_engine.clone(),
            hypothesis_engine.clone(),
            evolution_engine.clone(),
            knowledge_store.clone(),
        ));
        
        // Start the event subscriber background task
        let _event_subscriber_handle = start_event_subscriber(bus.clone(), event_subscriber);
        tracing::info!("Event subscriber started for learning pipeline");

        // Create memory system - Working and Permanent Memory (Architecture §6.3)
        let working_memory_core = Arc::new(MemWorkingMemory::new(1000));
        let permanent_memory = Arc::new(PermanentMemory::new(10000));
        let memory_retrieval = Arc::new(MemoryRetrieval::new(
            working_memory_core.clone(),
            permanent_memory.clone(),
        ));

        // Load memories from database into in-memory caches on startup
        // This restores the caches from persistent storage
        if let Err(e) = working_memory_core.load_from_database(&database).await {
            tracing::warn!("Failed to load working memory from database: {}", e);
        }
        if let Err(e) = permanent_memory.load_from_database(&database).await {
            tracing::warn!("Failed to load permanent memory from database: {}", e);
        }
        tracing::info!("Memory system initialized and loaded from database (Working: 1000, Permanent: 10000)");

        // Create scheduler with background tasks
        let scheduler = Self::setup_scheduler(database.clone()).await?;

        // Register task handlers with access to memory retrieval
        Self::register_task_handlers(
            scheduler.clone(),
            memory_retrieval.clone(),
            database.clone(),
        ).await;

        // Start scheduler background loop
        let scheduler_clone = scheduler.clone();
        tokio::spawn(async move {
            if let Err(e) = scheduler_clone.run().await {
                tracing::error!("Scheduler error: {}", e);
            }
        });
        tracing::info!("Scheduler background loop started");

        // Create metrics collector
        let metrics = Arc::new(MetricsCollector::new());

        // Create planning system (Architecture §4.03.5, §10)
        let planner = Arc::new(Planner::new(metrics.clone()));
        let policy_engine = Arc::new(PolicyEngine::new());

        // Load default policy rules
        policy_engine.load_defaults().await;
        tracing::info!("Policy engine loaded with default rules");

        // Create workflow engine with database access
        let workflow_engine = Arc::new(WorkflowEngine::with_database(
            metrics.clone(),
            database.clone(),
        ));
        tracing::info!("Workflow engine initialized");

        // Create MCP context with all systems
        let mcp_context = Arc::new(McpContext::new(
            database.clone(),
            bus.clone(),
            coordinator.clone(),
            reflection_engine.clone(),
            hypothesis_engine.clone(),
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
        ));

        // Register MCP tools
        tools::register_tools(&mcp_context);

        // Create MCP client for external connections and initialize globally
        crate::tools::agent::init_mcp_client(Arc::new(McpClient::new()));

        tracing::info!("RoBoT initialized successfully");

        Ok(Self {
            _database: database,
            bus,
            coordinator,
            scheduler,
            mcp_context,
        })
    }

    /// Setup background task scheduler with default tasks.
    async fn setup_scheduler(database: Arc<SqliteDatabase>) -> Result<Arc<Scheduler>> {
        let scheduler = Arc::new(Scheduler::new(database));

        // Schedule periodic reflection (every 30 minutes)
        scheduler
            .create_task(
                "periodic_reflection",
                TaskType::Reflection,
                TaskSchedule::Interval { seconds: 1800 },
            )
            .await?;

        // Schedule hypothesis evaluation (every hour)
        scheduler
            .create_task(
                "hypothesis_evaluation",
                TaskType::HypothesisEvaluation,
                TaskSchedule::Interval { seconds: 3600 },
            )
            .await?;

        // Schedule metrics collection (every 5 minutes)
        scheduler
            .create_task(
                "metrics_collection",
                TaskType::MetricsCollection,
                TaskSchedule::Interval { seconds: 300 },
            )
            .await?;

        // Schedule evolution maintenance (every day at midnight)
        scheduler
            .create_task(
                "evolution_maintenance",
                TaskType::EvolutionMaintenance,
                TaskSchedule::Daily { hour: 0, minute: 0 },
            )
            .await?;

        // Schedule memory consolidation (every hour)
        scheduler
            .create_task(
                "memory_consolidation",
                TaskType::MemoryConsolidation,
                TaskSchedule::Interval { seconds: 3600 },
            )
            .await?;

        // Schedule memory checkpoint (every 15 minutes)
        // Per Architecture §6.3: SQLite is the persistence layer
        scheduler
            .create_task(
                "memory_checkpoint",
                TaskType::MemoryCheckpoint,
                TaskSchedule::Interval { seconds: 900 },
            )
            .await?;

        Ok(scheduler)
    }

    /// Register task handlers for the scheduler
    async fn register_task_handlers(
        scheduler: Arc<Scheduler>,
        memory_retrieval: Arc<MemoryRetrieval>,
        database: Arc<SqliteDatabase>,
    ) {
        use crate::experience::scheduler::TaskType;

        // Reflection task handler
        scheduler
            .register_handler(
                TaskType::Reflection,
                Box::new(|| {
                    Box::pin(async move {
                        tracing::info!("Executing scheduled reflection task");
                        Ok(())
                    })
                }),
            )
            .await;

        // Hypothesis evaluation handler
        scheduler
            .register_handler(
                TaskType::HypothesisEvaluation,
                Box::new(|| {
                    Box::pin(async move {
                        tracing::info!("Executing scheduled hypothesis evaluation");
                        Ok(())
                    })
                }),
            )
            .await;

        // Metrics collection handler
        scheduler
            .register_handler(
                TaskType::MetricsCollection,
                Box::new(|| {
                    Box::pin(async move {
                        tracing::debug!("Executing scheduled metrics collection");
                        Ok(())
                    })
                }),
            )
            .await;

        // Evolution maintenance handler
        scheduler
            .register_handler(
                TaskType::EvolutionMaintenance,
                Box::new(|| {
                    Box::pin(async move {
                        tracing::info!("Executing scheduled evolution maintenance");
                        Ok(())
                    })
                }),
            )
            .await;

        // Exploration analysis handler
        scheduler
            .register_handler(
                TaskType::ExplorationAnalysis,
                Box::new(|| {
                    Box::pin(async move {
                        tracing::debug!("Executing scheduled exploration analysis");
                        Ok(())
                    })
                }),
            )
            .await;

        // Cleanup handler
        scheduler
            .register_handler(
                TaskType::Cleanup,
                Box::new(|| {
                    Box::pin(async move {
                        tracing::info!("Executing scheduled cleanup");
                        Ok(())
                    })
                }),
            )
            .await;

        // Reputation decay handler
        scheduler
            .register_handler(
                TaskType::ReputationDecay,
                Box::new(|| {
                    Box::pin(async move {
                        tracing::debug!("Executing scheduled reputation decay");
                        Ok(())
                    })
                }),
            )
            .await;

        // Memory consolidation handler
        // Per Architecture §6.3: Consolidates Working Memory to Permanent Memory
        let memory_retrieval_clone = memory_retrieval.clone();
        let database_clone = database.clone();
        scheduler
            .register_handler(
                TaskType::MemoryConsolidation,
                Box::new(move || {
                    let memory_retrieval = memory_retrieval_clone.clone();
                    let database = database_clone.clone();
                    Box::pin(async move {
                        tracing::info!("Executing scheduled memory consolidation");
                        
                        // Consolidate between in-memory caches (Architecture §6.3)
                        let stats = memory_retrieval.consolidate().await;
                        tracing::info!(
                            "Memory consolidation complete: {} promoted, {} archived, {} deleted, {} kept",
                            stats.promoted, stats.archived, stats.deleted, stats.kept
                        );
                        
                        // Checkpoint caches to database for persistence
                        if let Err(e) = memory_retrieval.checkpoint_to_database(&database).await {
                            tracing::error!("Failed to checkpoint memories to database: {}", e);
                        }
                        
                        Ok(())
                    })
                }),
            )
            .await;

        // Memory checkpoint handler
        // Per Architecture §6.3: Checkpoint in-memory caches to SQLite
        let memory_retrieval_checkpoint = memory_retrieval.clone();
        let database_checkpoint = database.clone();
        scheduler
            .register_handler(
                TaskType::MemoryCheckpoint,
                Box::new(move || {
                    let memory_retrieval = memory_retrieval_checkpoint.clone();
                    let database = database_checkpoint.clone();
                    Box::pin(async move {
                        tracing::debug!("Executing scheduled memory checkpoint");
                        
                        // Checkpoint caches to database for persistence
                        if let Err(e) = memory_retrieval.checkpoint_to_database(&database).await {
                            tracing::error!("Failed to checkpoint memories to database: {}", e);
                            return Err(e);
                        }
                        
                        tracing::debug!("Memory checkpoint completed successfully");
                        Ok(())
                    })
                }),
            )
            .await;

        tracing::info!("Registered {} task handlers", 9);
    }

    /// Start the runtime.
    pub async fn run(self) -> Result<()> {
        // Start background scheduler worker
        let scheduler = self.scheduler.clone();
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

    // === Accessor methods that wire up the bus and coordinator fields ===

    /// Get the event bus for monitoring/debugging
    pub fn event_bus(&self) -> &Arc<ExperienceBus> {
        &self.bus
    }

    /// Get the experience coordinator for testing/admin
    pub fn experience_coordinator(&self) -> &Arc<ExperienceCoordinator> {
        &self.coordinator
    }

    /// Get subscriber count on the event bus
    pub fn subscriber_count(&self) -> usize {
        self.bus.subscriber_count()
    }

    /// Record an experience through the coordinator
    pub fn process_experience(&self, experience: crate::experience::types::Experience) -> crate::experience::types::Experience {
        self.coordinator.process(experience)
    }
}
