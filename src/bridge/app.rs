// src/bridge/app.rs
// Root application container per Architecture §03

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
use crate::experience::reputation::decay::ReputationDecay;
use crate::experience::scheduler::{Scheduler, TaskSchedule, TaskType};
use crate::experience::scorer::ExperienceScorer;
use crate::experience::worker_manager::WorkerManager;
use crate::knowledge::KnowledgeStore;
use crate::memory::{MemoryRetrieval, PermanentMemory, WorkingMemory as MemWorkingMemory};
use crate::personality::{Personality, PersonalityTraits};
use crate::planner::{Planner, PolicyEngine};
use crate::skills::registry::SkillRegistry;
use crate::tools;
use crate::workflows::engine::WorkflowEngine;

/// Root application container.
///
/// Owns long-running services required by RoBoT.
pub struct App {
    /// Persistent database layer.
    database: Arc<SqliteDatabase>,

    /// Event bus for pub/sub.
    bus: Arc<ExperienceBus>,

    /// Worker manager for background job processing (Architecture §22).
    worker_manager: Arc<WorkerManager>,

    /// Experience system coordinator.
    coordinator: Arc<ExperienceCoordinator>,

    /// Hypothesis engine for belief management.
    hypothesis_engine: Arc<std::sync::Mutex<HypothesisEngine>>,

    /// Experience recorder for structured experience creation.
    experience_recorder: Arc<ExperienceRecorder>,

    /// Reflection pipeline for processing experiences into insights.
    reflection_pipeline: Arc<ReflectionPipeline>,

    /// Background task scheduler.
    scheduler: Arc<Scheduler>,

    /// Memory pipeline for working→permanent consolidation.
    memory_pipeline: Arc<crate::memory::pipeline::MemoryPipeline>,

    /// MCP context shared with bridge - owns all subsystems.
    mcp_context: Arc<McpContext>,

    /// Personality system for behavioral characteristics.
    personality: Arc<std::sync::Mutex<Personality>>,

    /// ACP router for inter-agent communication.
    acp_router: Arc<AcpRouter>,
}

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
        let scheduler = Self::setup_scheduler(database.clone()).await?;

        // Register task handlers with access to all required engines
        Self::register_task_handlers(
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
        tools::register_tools(&mcp_context);

        // Create MCP client for external connections and initialize globally
        crate::tools::agent::init_mcp_client(Arc::new(McpClient::new()));

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
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<Mutex<HypothesisEngine>>,
        evolution_engine: Arc<EvolutionEngine>,
        metrics: Arc<MetricsCollector>,
        database: Arc<SqliteDatabase>,
    ) {
        use crate::experience::scheduler::TaskType;

        // Reflection task handler - analyzes recent experiences for patterns
        let reflection_engine_clone = reflection_engine.clone();
        let database_reflect = database.clone();
        scheduler
            .register_handler(
                TaskType::Reflection,
                Box::new(move || {
                    let reflection_engine = reflection_engine_clone.clone();
                    let database = database_reflect.clone();
                    Box::pin(async move {
                        tracing::info!("Executing scheduled reflection task");

                        // Load recent experiences from database
                        let experiences = crate::database::queries::list_experiences(&database.connection()?, 100)
                            .unwrap_or_default();

                        if experiences.len() >= 3 {
                            // Analyze experiences for patterns
                            match reflection_engine.analyze_experiences(&experiences).await {
                                Ok(report) => {
                                    tracing::info!(
                                        "Reflection analysis complete: {} patterns, {} themes found",
                                        report.patterns.len(),
                                        report.themes.len()
                                    );
                                }
                                Err(e) => {
                                    tracing::error!("Reflection analysis failed: {}", e);
                                }
                            }
                        }

                        // Archive old reflections
                        let archived = reflection_engine.archive_old(30).await.unwrap_or(0);
                        if archived > 0 {
                            tracing::info!("Archived {} old reflections", archived);
                        }

                        Ok(())
                    })
                }),
            )
            .await;

        // Hypothesis evaluation handler - evaluates and updates hypothesis confidence
        let hypothesis_engine_clone = hypothesis_engine.clone();
        scheduler
            .register_handler(
                TaskType::HypothesisEvaluation,
                Box::new(move || {
                    let hypothesis_engine = hypothesis_engine_clone.clone();
                    Box::pin(async move {
                        tracing::info!("Executing scheduled hypothesis evaluation");

                        // Perform hypothesis maintenance
                        let engine_result = hypothesis_engine.lock();
                        match engine_result {
                            Ok(mut engine) => {
                                if let Err(e) = engine.maintenance() {
                                    tracing::error!("Hypothesis evaluation failed: {}", e);
                                }
                            }
                            Err(poisoned) => {
                                tracing::error!("Hypothesis engine mutex poisoned");
                                if let Err(e) = poisoned.into_inner().maintenance() {
                                    tracing::error!("Hypothesis evaluation failed on recovered mutex: {}", e);
                                }
                            }
                        }

                        Ok(())
                    })
                }),
            )
            .await;

        // Metrics collection handler - aggregates and reports metrics
        let metrics_clone = metrics.clone();
        scheduler
            .register_handler(
                TaskType::MetricsCollection,
                Box::new(move || {
                    let metrics = metrics_clone.clone();
                    Box::pin(async move {
                        tracing::debug!("Executing scheduled metrics collection");

                        // Get metrics summary
                        let summary = metrics.summary().await;
                        tracing::debug!(
                            "Metrics: {} counters, {} gauges, {} metrics tracked",
                            summary.counters.len(),
                            summary.gauges.len(),
                            summary.metrics.len()
                        );

                        // Clear old metrics (older than 24 hours)
                        metrics.clear_old(24).await;

                        Ok(())
                    })
                }),
            )
            .await;

        // Evolution maintenance handler - promotes/demotes behaviors based on performance
        let evolution_engine_clone = evolution_engine.clone();
        scheduler
            .register_handler(
                TaskType::EvolutionMaintenance,
                Box::new(move || {
                    let evolution_engine = evolution_engine_clone.clone();
                    Box::pin(async move {
                        tracing::info!("Executing scheduled evolution maintenance");

                        // Evaluate and maintain all behaviors for promotion/demotion
                        match evolution_engine.evaluate_and_maintain().await {
                            Ok(summary) => {
                                tracing::info!(
                                    "Evolution maintenance: {} total, {} promoted, {} deprecated, {} integrated",
                                    summary.total_behaviors,
                                    summary.promoted,
                                    summary.deprecated,
                                    summary.integrated
                                );
                            }
                            Err(e) => {
                                tracing::error!("Evolution maintenance failed: {}", e);
                            }
                        }

                        // Archive deprecated behaviors
                        let archived = evolution_engine.archive_deprecated().await.unwrap_or(0);
                        if archived > 0 {
                            tracing::info!("Archived {} deprecated behaviors", archived);
                        }

                        Ok(())
                    })
                }),
            )
            .await;

        // Exploration analysis handler - analyzes exploration results and patterns
        scheduler
            .register_handler(
                TaskType::ExplorationAnalysis,
                Box::new(|| {
                    Box::pin(async move {
                        tracing::debug!("Executing scheduled exploration analysis");

                        // Exploration analysis would analyze exploration results
                        // For now, log that the task executed
                        tracing::debug!("Exploration analysis completed");

                        Ok(())
                    })
                }),
            )
            .await;

        // Cleanup handler - removes old/stale data
        scheduler
            .register_handler(
                TaskType::Cleanup,
                Box::new(move || {
                    Box::pin(async move {
                        tracing::info!("Executing scheduled cleanup");

                        // Clean up old events from database
                        let cutoff = chrono::Utc::now() - chrono::Duration::days(7);

                        // This would call cleanup queries if implemented
                        tracing::info!("Cleanup task executed (older than {})", cutoff);

                        Ok(())
                    })
                }),
            )
            .await;

        // Reputation decay handler - applies time-based reputation decay
        let database_reput = database.clone();
        scheduler
            .register_handler(
                TaskType::ReputationDecay,
                Box::new(move || {
                    let database = database_reput.clone();
                    Box::pin(async move {
                        tracing::debug!("Executing scheduled reputation decay");

                        // Load reputations and apply decay
                        let reputations =
                            crate::database::queries::list_reputations(&database.connection()?)
                                .unwrap_or_default();

                        let mut decayed_count = 0;
                        for mut reputation in reputations {
                            let original_score = reputation.score;
                            reputation.score =
                                ReputationDecay::apply(reputation.score, reputation.updated_at);

                            if (original_score - reputation.score).abs() > 0.001 {
                                decayed_count += 1;
                                // Save updated reputation
                                if let Err(e) = crate::database::queries::insert_reputation(
                                    &database.connection()?,
                                    &reputation,
                                ) {
                                    tracing::warn!(
                                        "Failed to update reputation {}: {}",
                                        reputation.id,
                                        e
                                    );
                                }
                            }
                        }

                        if decayed_count > 0 {
                            tracing::debug!("Applied decay to {} reputations", decayed_count);
                        }

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

    // =========================================================================
    // Personality Methods (designed for future use)
    // =========================================================================
    /// Get reference to personality system
    pub fn personality(&self) -> Arc<std::sync::Mutex<Personality>> {
        self.personality.clone()
    }

    /// Get current personality traits
    pub fn get_personality_traits(&self) -> PersonalityTraits {
        match self.personality.lock() {
            Ok(guard) => guard.get_traits().clone(),
            Err(poisoned) => {
                tracing::error!("Personality mutex poisoned, recovering");
                poisoned.into_inner().get_traits().clone()
            }
        }
    }

    /// Set personality traits
    pub fn set_personality_traits(&self, traits: PersonalityTraits) {
        if let Err(poisoned) = self.personality.lock() {
            tracing::error!("Personality mutex poisoned during set_traits, recovering");
            poisoned.into_inner().set_traits(traits);
        } else {
            // Lock succeeded and will be released when scope ends
        }
    }

    /// Apply a personality preset (balanced, analytical, creative, cautious, bold)
    pub fn apply_personality_preset(&self, preset: &str) -> bool {
        match self.personality.lock() {
            Ok(mut guard) => guard.apply_preset(preset),
            Err(poisoned) => {
                tracing::error!("Personality mutex poisoned during apply_preset, recovering");
                poisoned.into_inner().apply_preset(preset)
            }
        }
    }

    /// Get available personality presets
    pub fn list_personality_presets(&self) -> Vec<String> {
        match self.personality.lock() {
            Ok(guard) => guard.list_presets(),
            Err(poisoned) => {
                tracing::error!("Personality mutex poisoned during list_presets, recovering");
                poisoned.into_inner().list_presets()
            }
        }
    }

    /// Get current personality preset name
    pub fn get_personality_preset(&self) -> String {
        match self.personality.lock() {
            Ok(guard) => guard.get_current_preset().to_string(),
            Err(poisoned) => {
                tracing::error!("Personality mutex poisoned during get_current_preset, recovering");
                poisoned.into_inner().get_current_preset().to_string()
            }
        }
    }

    /// Adapt personality based on experience outcome
    pub fn adapt_personality(&self, success: bool, risk_taken: bool) {
        if let Err(poisoned) = self.personality.lock() {
            tracing::error!("Personality mutex poisoned during adapt, recovering");
            poisoned.into_inner().adapt_from_experience(success, risk_taken);
        }
    }

    /// Get communication style based on personality verbosity
    pub fn get_communication_style(&self) -> crate::personality::CommunicationStyle {
        match self.personality.lock() {
            Ok(guard) => guard.get_communication_style(),
            Err(poisoned) => {
                tracing::error!("Personality mutex poisoned, returning default communication style");
                poisoned.into_inner().get_communication_style()
            }
        }
    }

    /// Decide if system should explore new approaches
    pub fn should_explore(&self, confidence: f32) -> bool {
        match self.personality.lock() {
            Ok(guard) => guard.should_explore(confidence),
            Err(poisoned) => {
                tracing::error!("Personality mutex poisoned, defaulting to exploration");
                poisoned.into_inner().should_explore(confidence)
            }
        }
    }

    /// Decide if system should take a risk
    pub fn should_take_risk(&self, potential_gain: f32, potential_loss: f32) -> bool {
        match self.personality.lock() {
            Ok(guard) => guard.should_take_risk(potential_gain, potential_loss),
            Err(poisoned) => {
                tracing::error!("Personality mutex poisoned, defaulting risk assessment");
                poisoned.into_inner().should_take_risk(potential_gain, potential_loss)
            }
        }
    }

    /// Decide if a creative approach should be used for planning.
    /// Uses personality creativity trait combined with problem complexity
    /// to determine whether to explore unconventional solutions.
    pub fn should_use_creativity(&self, problem_complexity: f32) -> bool {
        match self.personality.lock() {
            Ok(guard) => guard.should_use_creativity(problem_complexity),
            Err(poisoned) => {
                tracing::error!("Personality mutex poisoned, defaulting creativity");
                poisoned.into_inner().should_use_creativity(problem_complexity)
            }
        }
    }

    /// Get patience-based timeout
    pub fn get_personality_timeout(&self, base_timeout_secs: u64) -> u64 {
        match self.personality.lock() {
            Ok(guard) => guard.get_timeout(base_timeout_secs),
            Err(poisoned) => {
                tracing::error!("Personality mutex poisoned, returning base timeout");
                poisoned.into_inner().get_timeout(base_timeout_secs)
            }
        }
    }

    /// Get personality success rate
    pub fn get_personality_success_rate(&self) -> f32 {
        match self.personality.lock() {
            Ok(guard) => guard.success_rate(),
            Err(poisoned) => {
                tracing::error!("Personality mutex poisoned, returning 0.0 success rate");
                poisoned.into_inner().success_rate()
            }
        }
    }

    // =========================================================================
    // ACP (Agent Communication Protocol) Methods (designed for future use)
    // =========================================================================
    /// Get reference to ACP router
    pub fn acp_router(&self) -> Arc<AcpRouter> {
        self.acp_router.clone()
    }

    /// Get ACP registry for agent registration
    pub fn acp_registry(&self) -> Arc<AcpRegistry> {
        self.acp_router.registry()
    }

    /// Route an ACP message to the appropriate agent
    pub fn route_acp_message(
        &self,
        message: crate::bridge::acp::AcpMessage,
    ) -> Result<Option<crate::bridge::acp::AcpMessage>> {
        self.acp_router.route(message)
    }

    /// List all registered ACP agents
    pub fn list_acp_agents(&self) -> Result<Vec<crate::bridge::acp::AcpAgentId>> {
        self.acp_router.registry().list_agents()
    }

    /// Get count of registered ACP agents
    pub fn acp_agent_count(&self) -> usize {
        self.acp_router.registry().count()
    }
}
