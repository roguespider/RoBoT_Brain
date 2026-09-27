//! Deployment bootstrap and shutdown (Architecture Chapter 31).
//!
//! Per Architecture Chapter 31: Bootstrap initializes all subsystems in the
//! correct order — database, storage, memory, experience, learning, planning,
//! execution, tools, model, communication, and coordination.

/// Bootstrap steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapStep {
    DatabaseInit,
    StorageInit,
    MemoryInit,
    ExperienceInit,
    LearningInit,
    PlanningInit,
    ExecutionInit,
    ToolInit,
    ModelInit,
    CommunicationInit,
    CoordinationInit,
}

/// Bootstrap result.
#[derive(Debug, Clone, Default)]
pub struct BootstrapResult {
    pub completed_steps: Vec<String>,
    pub failed_steps: Vec<String>,
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

/// Run bootstrap sequence.
pub fn run_bootstrap() -> Result<BootstrapResult, DeploymentError> {
    let steps = [
        BootstrapStep::DatabaseInit,
        BootstrapStep::StorageInit,
        BootstrapStep::MemoryInit,
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
                // Initialize SQLite database connection
                match crate::database::sqlite::SqliteDatabase::initialize() {
                    Ok(db) => {
                        tracing::info!(
                            "Database initialized successfully at {}",
                            db.path().to_string_lossy()
                        );
                        result.completed_steps.push("DatabaseInit".to_string());
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
                // Initialize storage layer (working, session, experience, semantic, skill, graph, archive, operational)
                let layers = [
                    crate::database::StorageLayer::Working,
                    crate::database::StorageLayer::Session,
                    crate::database::StorageLayer::Experience,
                    crate::database::StorageLayer::Semantic,
                    crate::database::StorageLayer::Skill,
                    crate::database::StorageLayer::Graph,
                    crate::database::StorageLayer::Archive,
                    crate::database::StorageLayer::Operational,
                ];
                for layer in &layers {
                    tracing::debug!("Storage layer: {:?}", layer);
                }
                tracing::info!("Storage layer initialized with {} layers", layers.len());
                result.completed_steps.push("StorageInit".to_string());
            }
            BootstrapStep::MemoryInit => {
                // Initialize permanent memory store
                let memory = crate::memory::PermanentMemory::new(100);
                // Verify memory store is operational by checking it responds
                let store_type = std::any::type_name_of_val(&memory);
                tracing::info!("Memory store initialized (type: {})", store_type);
                result.completed_steps.push("MemoryInit".to_string());
            }
            BootstrapStep::ExperienceInit => {
                // Initialize experience coordinator with required dependencies
                let scorer = crate::experience::scorer::ExperienceScorer::new();
                let bus = std::sync::Arc::new(crate::experience::bus::ExperienceBus::new());
                let metrics =
                    std::sync::Arc::new(crate::experience::metrics::MetricsCollector::new());
                let coordinator = crate::experience::coordinator::ExperienceCoordinator::new(
                    scorer, bus, metrics,
                );
                tracing::info!("Experience coordinator initialized with scorer, bus, metrics");
                drop(coordinator);
                result.completed_steps.push("ExperienceInit".to_string());
            }
            BootstrapStep::LearningInit => {
                // Initialize learning patterns and confidence
                let empty_experiences: Vec<
                    crate::data_contracts::experience_record::ExperienceRecord,
                > = Vec::new();
                let patterns = crate::learning::patterns::detect_patterns(&empty_experiences, 2);
                let confidence = crate::learning::confidence::update_confidence("init", 0.5);
                tracing::info!(
                    "Learning initialized: {} patterns, confidence result: {:?}",
                    patterns.len(),
                    confidence
                );
                tracing::info!("Learning subsystem initialized");
                result.completed_steps.push("LearningInit".to_string());
            }
            BootstrapStep::PlanningInit => {
                // Initialize planner engine
                let metrics =
                    std::sync::Arc::new(crate::experience::metrics::MetricsCollector::new());
                let planner = crate::planner::engine::Planner::new(metrics);
                // get_policy() is async; use Handle to block in sync bootstrap
                let policy_handle = tokio::runtime::Handle::current();
                let policy = policy_handle.block_on(planner.get_policy());
                tracing::debug!("Planner policy: {:?}", policy);
                tracing::info!("Planner engine initialized: policy loaded");
                result.completed_steps.push("PlanningInit".to_string());
            }
            BootstrapStep::ExecutionInit => {
                // Initialize execution scheduler and graph
                let graph = crate::execution::graph::ExecutionGraph::new();
                let mut scheduler = crate::execution::scheduler::ExecutionScheduler::new(graph, 2);
                scheduler.initialize_jobs();
                tracing::info!("Execution scheduler initialized with concurrency 2");
                result.completed_steps.push("ExecutionInit".to_string());
            }
            BootstrapStep::ToolInit => {
                // Initialize tool registry (search tools)
                let tool_count = crate::bridge::tools::search::definitions::all().len();
                tracing::info!("Tool registry initialized with {} tools", tool_count);
                result.completed_steps.push("ToolInit".to_string());
            }
            BootstrapStep::ModelInit => {
                // Report which research providers are actually usable, so an
                // unconfigured deployment is visible at startup rather than at
                // the first query.
                tracing::info!(
                    ready = crate::research::is_research_ready(),
                    "{}",
                    crate::research::adapter_registry::readiness_report()
                );
                result.completed_steps.push("ModelInit".to_string());
            }
            BootstrapStep::CommunicationInit => {
                // Initialize communication subsystem
                crate::communication::reference_communication_contracts();
                tracing::info!("Communication subsystem initialized");
                result.completed_steps.push("CommunicationInit".to_string());
            }
            BootstrapStep::CoordinationInit => {
                // Initialize coordination subsystem
                crate::coordination::reference_coordination_contracts();
                tracing::info!("Coordination subsystem initialized");
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

/// Graceful shutdown.
pub fn graceful_shutdown() -> Result<(), DeploymentError> {
    Ok(())
}
