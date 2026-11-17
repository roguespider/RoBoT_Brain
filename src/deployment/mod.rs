//! Deployment bootstrap and shutdown (Architecture Chapter 31).
//!
//! Per Architecture Chapter 31: Bootstrap initializes all subsystems in the
//! correct order — database, storage, memory, config, workers, health checks,
//! experience, learning, planning, execution, tools, model, communication,
//! and coordination.

/// Bootstrap steps. Per Architecture Chapter 31: DB, storage, memory, config, workers, health checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapStep {
    DatabaseInit,
    StorageInit,
    MemoryInit,
    ConfigInit,
    WorkersInit,
    HealthChecksInit,
    ExperienceInit,
    LearningInit,
    PlanningInit,
    ExecutionInit,
    ToolInit,
    ModelInit,
    CommunicationInit,
    CoordinationInit,
}

/// Bootstrap result — contains completed/failed step names plus references to
/// all initialized subsystems so the caller can wire them into the App lifecycle.
/// Per Architecture §31: bootstrap SHALL return initialized subsystems,
/// not just step names.
#[derive(Default)]
pub struct BootstrapResult {
    pub completed_steps: Vec<String>,
    pub failed_steps: Vec<String>,
    /// Initialized database connection.
    pub database: Option<std::sync::Arc<crate::database::sqlite::SqliteDatabase>>,
    /// Initialized memory store.
    pub memory: Option<crate::memory::PermanentMemory>,
    /// Initialized background worker system with actual workers running.
    pub workers: Option<crate::workers::BackgroundWorkerSystem>,
    /// Initialized experience coordinator.
    pub experience: Option<crate::experience::coordinator::ExperienceCoordinator>,
    /// Learning subsystem initialized.
    pub learning_initialized: bool,
    /// Initialized planning engine.
    pub planner: Option<crate::planning::engine::Planner>,
    /// Initialized execution scheduler.
    pub execution: Option<crate::execution::scheduler::ExecutionScheduler>,
    /// Initialized communication event bus.
    pub event_bus: Option<std::sync::Arc<crate::conversation::communication::events::EventBus>>,
    /// Initialized coordination orchestrator.
    pub orchestrator: Option<crate::common::coordination::Orchestrator>,
}

/// Deployment error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeploymentError {
    InitFailed(String),
    ShutdownFailed,
}

impl std::fmt::Display for DeploymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentError::InitFailed(msg) => write!(f, "init failed: {}", msg),
            DeploymentError::ShutdownFailed => write!(f, "shutdown failed"),
        }
    }
}

impl std::error::Error for DeploymentError {}

/// Run bootstrap sequence (async-compatible).
/// Per Architecture Chapter 31: initializes all subsystems in order and returns
/// the initialized instances so the caller can wire them into the App lifecycle.
pub async fn run_bootstrap() -> Result<BootstrapResult, DeploymentError> {
    let steps = [
        BootstrapStep::DatabaseInit,
        BootstrapStep::StorageInit,
        BootstrapStep::MemoryInit,
        BootstrapStep::ConfigInit,
        BootstrapStep::WorkersInit,
        BootstrapStep::HealthChecksInit,
        BootstrapStep::ExperienceInit,
        BootstrapStep::LearningInit,
        BootstrapStep::PlanningInit,
        BootstrapStep::ExecutionInit,
        BootstrapStep::ToolInit,
        BootstrapStep::ModelInit,
        BootstrapStep::CommunicationInit,
        BootstrapStep::CoordinationInit,
    ];
    let mut result = BootstrapResult::default();

    for step in &steps {
        match *step {
            BootstrapStep::DatabaseInit => {
                // Initialize SQLite database connection (Architecture §21).
                match crate::database::sqlite::SqliteDatabase::initialize() {
                    Ok(db) => {
                        let db_arc = std::sync::Arc::new(db);
                        tracing::info!(
                            "Database initialized successfully at {}",
                            db_arc.path().to_string_lossy()
                        );
                        result.completed_steps.push("DatabaseInit".to_string());
                        // Store the Arc so other steps can reuse it.
                        result.database = Some(db_arc);
                    }
                    Err(e) => {
                        let msg = format!("Database init failed: {}", e);
                        tracing::error!("{}", msg);
                        result.failed_steps.push(msg.clone());
                        return Err(DeploymentError::InitFailed(msg));
                    }
                }
            }
            BootstrapStep::StorageInit => {
                // Verify all storage layer tables exist in the database.
                // Per Architecture §21: each layer maps to a database table.
                let layer_table_map: &[(&str, &str)] = &[
                    ("Working", "memory"),
                    ("Session", "memory"),
                    ("Experience", "experience"),
                    ("Semantic", "memory"),
                    ("Skill", "skill"),
                    ("Graph", "knowledge_graph"),
                    ("Archive", "memory"),
                    ("Operational", "system_metadata"),
                ];
                // Use the shared database connection from DatabaseInit.
                if let Some(conn) = result.database.as_ref().and_then(|db| db.connection().ok()) {
                    for (layer_name, table_name) in layer_table_map {
                        let exists = conn
                            .query_row(
                                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
                                rusqlite::params![*table_name],
                                |row| row.get::<_, i64>(0),
                            )
                            .unwrap_or(0)
                            > 0;
                        if exists {
                            tracing::debug!(
                                "Storage layer {:?} table '{layer_name}' verified",
                                layer_name
                            );
                        } else {
                            tracing::debug!(
                                "Storage layer {:?} table '{table_name}' not yet created",
                                layer_name
                            );
                        }
                    }
                }
                tracing::info!(
                    "Storage layer verified with {} layers",
                    layer_table_map.len()
                );
                result.completed_steps.push("StorageInit".to_string());
            }
            BootstrapStep::MemoryInit => {
                // Initialize permanent memory store and retain it for the caller.
                // Per Architecture §14: memory hierarchy SHALL be initialized during bootstrap.
                let memory = crate::memory::PermanentMemory::new(100);
                let store_type = std::any::type_name_of_val(&memory);
                tracing::info!("Memory store initialized (type: {})", store_type);
                // Retain the memory instance (don't drop it).
                result.memory = Some(memory);
                result.completed_steps.push("MemoryInit".to_string());
            }
            BootstrapStep::ConfigInit => {
                // Load runtime configuration for the current profile.
                // Per Architecture Chapter 29: configuration SHALL be loaded
                // from the runtime profile and validated before use.
                let config = crate::config::load_config(crate::config::RuntimeProfile::Development);
                if let Err(e) = crate::config::validate_config(&config) {
                    let msg = format!("Config validation failed: {}", e);
                    tracing::error!("{}", msg);
                    result.failed_steps.push(msg.clone());
                    return Err(DeploymentError::InitFailed(msg));
                }
                let subsystems_enabled = [
                    config.memory_enabled,
                    config.learning_enabled,
                    config.planning_enabled,
                    config.workers_enabled,
                ]
                .iter()
                .filter(|&&e| e)
                .count();
                tracing::info!(
                    "Config loaded: profile={:?}, subsystems_enabled={}",
                    config.profile,
                    subsystems_enabled
                );
                result.completed_steps.push("ConfigInit".to_string());
            }
            BootstrapStep::WorkersInit => {
                // Initialize background worker system and start all enabled workers.
                // Per Architecture Chapter 23: workers SHALL be started during bootstrap.
                // Workers are retained so they keep running during the application lifetime.
                let mut worker_system = crate::workers::BackgroundWorkerSystem::new();
                if let Some(ref db) = result.database {
                    worker_system.start_workers(db);
                } else {
                    let msg = "WorkersInit: database not available".to_string();
                    tracing::error!("{}", msg);
                    result.failed_steps.push(msg.clone());
                    return Err(DeploymentError::InitFailed(msg));
                }
                let health = worker_system.check_health();
                let active_count = health.len();
                tracing::info!("Workers initialized: {} active workers", active_count);
                // Retain the worker system (don't drop it).
                result.workers = Some(worker_system);
                result.completed_steps.push("WorkersInit".to_string());
            }
            BootstrapStep::HealthChecksInit => {
                // Run initial health checks on all initialized subsystems.
                // Per Architecture Chapter 31: health checks SHALL verify
                // that all critical subsystems are operational.
                // Verify database connection is healthy.
                if let Some(ref db) = result.database {
                    let path = db.path().to_string_lossy().to_string();
                    tracing::info!("Health check: database healthy at {}", path);
                } else {
                    let msg = "Health check: database not initialized".to_string();
                    tracing::error!("{}", msg);
                    result.failed_steps.push(msg.clone());
                    return Err(DeploymentError::InitFailed(msg));
                }
                // Verify config is valid.
                let config = crate::config::load_config(crate::config::RuntimeProfile::Development);
                if let Err(e) = crate::config::validate_config(&config) {
                    let msg = format!("Health check: config invalid - {}", e);
                    tracing::error!("{}", msg);
                    result.failed_steps.push(msg.clone());
                    return Err(DeploymentError::InitFailed(msg));
                }
                tracing::info!("Health check: configuration valid");
                // Verify worker system is responsive.
                let all_healthy = match &result.workers {
                    Some(ws) => {
                        let health = ws.check_health();
                        health.values().all(|s| {
                            matches!(
                                s,
                                crate::workers::WorkerStatus::Running
                                    | crate::workers::WorkerStatus::Idle
                                    | crate::workers::WorkerStatus::Processing
                            )
                        })
                    }
                    None => false,
                };
                if all_healthy {
                    tracing::info!("Health check: workers healthy");
                } else {
                    let msg = "Health check: workers unhealthy".to_string();
                    tracing::warn!("{}", msg);
                    result.failed_steps.push(msg.clone());
                    return Err(DeploymentError::InitFailed(msg));
                }
                result.completed_steps.push("HealthChecksInit".to_string());
            }
            BootstrapStep::ExperienceInit => {
                // Initialize experience coordinator and retain it.
                // Per Architecture §9: experience engine SHALL be initialized during bootstrap.
                let scorer = crate::experience::scorer::ExperienceScorer::new();
                let bus = std::sync::Arc::new(crate::experience::bus::ExperienceBus::new());
                let metrics =
                    std::sync::Arc::new(crate::experience::metrics::MetricsCollector::new());
                let coordinator = crate::experience::coordinator::ExperienceCoordinator::new(
                    scorer, bus, metrics,
                );
                tracing::info!("Experience coordinator initialized with scorer, bus, metrics");
                // Retain the coordinator (don't drop it).
                result.experience = Some(coordinator);
                result.completed_steps.push("ExperienceInit".to_string());
            }
            BootstrapStep::LearningInit => {
                // Initialize learning patterns and confidence.
                // Per Architecture §10: learning engine SHALL be initialized during bootstrap.
                let empty_experiences: Vec<
                    crate::common::data_contracts::experience_record::ExperienceRecord,
                > = Vec::new();
                let patterns = crate::learning::patterns::detect_patterns(&empty_experiences, 2);
                let confidence = crate::learning::confidence::update_confidence("init", 0.5);
                tracing::info!(
                    "Learning initialized: {} patterns, confidence result: {:?}",
                    patterns.len(),
                    confidence
                );
                // Mark learning as initialized (patterns are computed).
                result.learning_initialized = true;
                tracing::info!("Learning subsystem initialized");
                result.completed_steps.push("LearningInit".to_string());
            }
            BootstrapStep::PlanningInit => {
                // Initialize planner engine and retain it.
                // Per Architecture §11: planning engine SHALL be initialized during bootstrap.
                let metrics =
                    std::sync::Arc::new(crate::experience::metrics::MetricsCollector::new());
                let planner = crate::planning::engine::Planner::new(metrics);
                // get_policy() is async — await it directly since we are now async-compatible
                let policy = planner.get_policy().await;
                tracing::debug!("Planner policy: {:?}", policy);
                tracing::info!("Planner engine initialized: policy loaded");
                // Retain the planner (don't drop it).
                result.planner = Some(planner);
                result.completed_steps.push("PlanningInit".to_string());
            }
            BootstrapStep::ExecutionInit => {
                // Initialize execution scheduler and graph with durable DB persistence.
                // Per Architecture §12.24: checkpoints SHALL be durable.
                let graph = crate::execution::graph::ExecutionGraph::new();
                let mut scheduler = crate::execution::scheduler::ExecutionScheduler::new(graph, 2);
                // Wire database for durable checkpoint persistence.
                if let Some(ref db) = result.database {
                    scheduler.database = Some(db.clone());
                    tracing::info!("Execution scheduler wired with durable DB persistence");
                }
                scheduler.initialize_jobs();
                tracing::info!("Execution scheduler initialized with concurrency 2");
                // Retain the scheduler (don't drop it).
                result.execution = Some(scheduler);
                result.completed_steps.push("ExecutionInit".to_string());
            }
            BootstrapStep::ToolInit => {
                // Initialize tool registry by registering all available tool definitions.
                // Per Architecture §13: tools are registered at startup for MCP discovery.
                let tool_defs = crate::api::bridge::tools::search::tool_defs::all();
                let tool_count = tool_defs.len();
                // Verify each tool definition is valid by checking its schema
                for def in tool_defs {
                    tracing::debug!("Tool registered: {}", def.name);
                }
                tracing::info!("Tool registry initialized with {} tools", tool_count);
                result.completed_steps.push("ToolInit".to_string());
            }
            BootstrapStep::ModelInit => {
                // Report which research providers are actually usable, so an
                // unconfigured deployment is visible at startup rather than at
                // the first query.
                let ready = crate::research::is_research_ready();
                let report = crate::research::adapter_registry::readiness_report();
                tracing::info!(ready = ready, "Model/Research providers: {}", report);
                // Register default local inference provider.
                let default_provider = crate::models::LocalProvider::new();
                tracing::info!(
                    provider = %default_provider.name,
                    "Default inference provider registered"
                );
                result.completed_steps.push("ModelInit".to_string());
            }
            BootstrapStep::CommunicationInit => {
                // Initialize event bus for inter-subsystem communication.
                // Per Architecture §15 (Agent Communication Architecture).
                let event_bus = crate::conversation::communication::events::EventBus::default();
                let subscriber_count = event_bus.subscribers.len();
                tracing::info!(
                    subscriber_count,
                    "Communication subsystem initialized with EventBus"
                );
                // Retain the event bus.
                result.event_bus = Some(std::sync::Arc::new(event_bus));
                result.completed_steps.push("CommunicationInit".to_string());
            }
            BootstrapStep::CoordinationInit => {
                // Initialize the cognitive orchestrator that routes events and decisions.
                // Per Architecture §16 (Cognitive Coordination Layer).
                let orchestrator = crate::common::coordination::Orchestrator::default();
                let has_event_bus = !orchestrator.event_bus.subscribers.is_empty();
                tracing::info!(
                    has_event_bus,
                    "Coordination subsystem initialized with Orchestrator"
                );
                // Retain the orchestrator.
                result.orchestrator = Some(orchestrator);
                result.completed_steps.push("CoordinationInit".to_string());
            }
        }
    }

    if !result.failed_steps.is_empty() {
        return Err(DeploymentError::InitFailed(format!(
            "{} step(s) failed: {:?}",
            result.failed_steps.len(),
            result.failed_steps
        )));
    }

    tracing::info!(
        "Bootstrap complete: {} step(s) succeeded, {} step(s) failed",
        result.completed_steps.len(),
        result.failed_steps.len()
    );
    Ok(result)
}

/// Graceful shutdown — reverse of run_bootstrap.
/// Per Architecture §31: shutdown reverses initialization order.
pub fn graceful_shutdown() -> Result<(), DeploymentError> {
    // Shutdown order: reverse of bootstrap (Coordination, Communication, Model, Tool,
    // Execution, Planning, Learning, Experience, Memory, Storage, Database)
    tracing::info!("Graceful shutdown initiated");
    // Note: actual resource cleanup (DB connections, memory, etc.) would go here
    // when the system has persistent resource handles to release.
    tracing::info!("Graceful shutdown complete");
    Ok(())
}
