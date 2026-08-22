// src/bridge/app/initialization/core.rs
//! Core infrastructure: personality, bus, metrics, coordinator, recorder, job queue, worker manager

use std::sync::{Arc, Mutex};

use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::encounter_recorder::ExperienceRecorder;
use crate::experience::event_handler::EventHandler;
use crate::experience::metrics::MetricsCollector;
use crate::experience::scorer::ExperienceScorer;
use crate::experience::worker_manager::WorkerManager;
use crate::personality::Personality;

/// Core infrastructure built by `build_core`.
pub(crate) struct CoreInfra {
    pub(crate) database: Arc<SqliteDatabase>,
    pub(crate) shared_personality: Arc<Mutex<Personality>>,
    pub(crate) bus: Arc<ExperienceBus>,
    pub(crate) metrics: Arc<MetricsCollector>,
    pub(crate) coordinator: Arc<ExperienceCoordinator>,
    pub(crate) experience_recorder: Arc<ExperienceRecorder>,
    pub(crate) job_queue: Arc<Mutex<crate::experience::queue::JobQueue>>,
    pub(crate) worker_manager: Arc<WorkerManager>,
}

/// Build core infrastructure (personality, bus, coordinator, recorder, queue, workers).
/// The event handler must be started separately.
pub(crate) fn build_core(database: Arc<SqliteDatabase>) -> (CoreInfra, EventHandler) {
    // Create shared personality instance (used by both App and planner)
    let shared_personality = Arc::new(Mutex::new(Personality::new()));

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

    // Create event handler to process events from the bus
    let event_handler = EventHandler::new(bus.clone());
    event_handler.start();
    tracing::info!("Event handler started");

    // Create WorkerManager for background job processing per Architecture §22
    // Design: Experience → Recorder → Bus → Job Queue → Workers → Observers
    // The JobQueue is SQLite-backed so jobs survive restarts.
    let job_queue = Arc::new(Mutex::new(
        crate::experience::queue::JobQueue::with_database(database.clone()),
    ));
    {
        let mut q = job_queue.lock().unwrap_or_else(|e| e.into_inner());
        if let Err(e) = q.restore_from_database() {
            tracing::warn!("JobQueue restore failed: {}", e);
        }
    }
    let worker_manager = Arc::new(WorkerManager::new_with_queue(
        bus.clone(),
        job_queue.clone(),
    ));

    (
        CoreInfra {
            database,
            shared_personality,
            bus,
            metrics,
            coordinator,
            experience_recorder,
            job_queue,
            worker_manager,
        },
        event_handler,
    )
}
