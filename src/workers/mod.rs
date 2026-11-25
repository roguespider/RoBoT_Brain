//! Background Workers — Per Architecture Chapter 23.
//!
//! Background workers provide non-blocking intelligence for long-running,
//! resource-intensive, and asynchronous operations.
//!
//! MaintenanceWorker is implemented in the `maintenance_worker` submodule with
//! real autonomous maintenance tasks: compaction, aging, checkpointing,
//! and pattern learning (Architecture §23.6.5).

pub mod experience_worker;
pub mod knowledge_worker;
pub mod learning_worker;
pub mod maintenance_worker;
pub mod memory_worker;
pub mod queue;
pub mod scheduler;
pub mod supervisor;

// Re-export worker types from submodules
pub use experience_worker::ExperienceWorker;
pub use knowledge_worker::KnowledgeGraphWorker;
pub use learning_worker::LearningWorker;
pub use maintenance_worker::{
    MaintenanceResult, MaintenanceScheduler, MaintenanceStats, MaintenanceTaskType,
};
pub use memory_worker::MemoryWorker;
pub use queue::BackgroundTask;
pub use scheduler::{SchedulingMode, WorkerPriority, WorkerTask, recover_worker, schedule_task};
pub use supervisor::{WorkerStatus, WorkerSupervisor};

use crate::database::sqlite::SqliteDatabase;
use crate::scheduler::WorkerType;

/// The background worker system manages asynchronous cognitive tasks.
#[derive(Debug, Clone, Default)]
pub struct BackgroundWorkerSystem {
    /// Supervisor for all workers.
    pub supervisor: WorkerSupervisor,
    /// Memory worker.
    pub memory_worker: MemoryWorker,
    /// Learning worker.
    pub learning_worker: LearningWorker,
    /// Experience worker.
    pub experience_worker: ExperienceWorker,
    /// Knowledge graph worker.
    pub knowledge_graph_worker: KnowledgeGraphWorker,
    /// Maintenance scheduler — runs autonomous maintenance tasks (Architecture §23.6.5).
    pub maintenance_scheduler: MaintenanceScheduler,
    /// Task queue.
    pub task_queue: Vec<BackgroundTask>,
}

impl BackgroundWorkerSystem {
    /// Create a new background worker system.
    pub fn new() -> Self {
        Self {
            supervisor: WorkerSupervisor::default(),
            memory_worker: MemoryWorker {
                enabled: true,
                priority: "normal".to_string(),
            },
            learning_worker: LearningWorker {
                enabled: true,
                priority: "low".to_string(),
            },
            experience_worker: ExperienceWorker {
                enabled: true,
                priority: "normal".to_string(),
            },
            knowledge_graph_worker: KnowledgeGraphWorker {
                enabled: true,
                priority: "low".to_string(),
            },
            maintenance_scheduler: MaintenanceScheduler::new(),
            task_queue: Vec::new(),
        }
    }

    /// Enqueue a new background task.
    pub fn enqueue_task(&mut self, task: BackgroundTask) {
        self.task_queue.push(task);
    }

    /// Get the number of pending tasks.
    pub fn pending_tasks(&self) -> usize {
        self.task_queue.len()
    }

    /// Process the next pending task.
    pub fn process_next(&mut self) -> Option<BackgroundTask> {
        if self.task_queue.is_empty() {
            return None;
        }
        Some(self.task_queue.remove(0))
    }

    /// Start all enabled workers by submitting initial tasks to the supervisor.
    /// Per Architecture §23.7: workers SHALL be started during bootstrap.
    pub fn start_workers(&mut self, db: &SqliteDatabase) {
        if self.memory_worker.enabled {
            let task = WorkerTask::new(
                "memory-worker-001",
                WorkerType::MemoryWorker,
                serde_json::json!({"action": "start"}),
                WorkerPriority::Immediate,
            );
            let submitted = schedule_task(
                &mut self.supervisor,
                task,
                crate::scheduler::SchedulingMode::Immediate,
            );
            if submitted {
                tracing::info!("Worker: memory scheduled");
            } else {
                tracing::warn!("Worker: memory scheduling failed (capacity full)");
            }
        }
        if self.learning_worker.enabled {
            let task = WorkerTask::new(
                "learning-worker-001",
                WorkerType::LearningWorker,
                serde_json::json!({"action": "start"}),
                WorkerPriority::Normal,
            );
            let submitted = schedule_task(
                &mut self.supervisor,
                task,
                crate::scheduler::SchedulingMode::Immediate,
            );
            if submitted {
                tracing::info!("Worker: learning scheduled");
            } else {
                tracing::warn!("Worker: learning scheduling failed (capacity full)");
            }
        }
        if self.experience_worker.enabled {
            let task = WorkerTask::new(
                "experience-worker-001",
                WorkerType::ExperienceWorker,
                serde_json::json!({"action": "start"}),
                WorkerPriority::Normal,
            );
            let submitted = schedule_task(
                &mut self.supervisor,
                task,
                crate::scheduler::SchedulingMode::Immediate,
            );
            if submitted {
                tracing::info!("Worker: experience scheduled");
            } else {
                tracing::warn!("Worker: experience scheduling failed (capacity full)");
            }
        }
        if self.knowledge_graph_worker.enabled {
            let task = WorkerTask::new(
                "kg-worker-001",
                crate::scheduler::WorkerType::KnowledgeGraphWorker,
                serde_json::json!({"action": "start"}),
                WorkerPriority::Normal,
            );
            let submitted = schedule_task(
                &mut self.supervisor,
                task,
                crate::scheduler::SchedulingMode::Immediate,
            );
            if submitted {
                tracing::info!("Worker: knowledge graph scheduled");
            } else {
                tracing::warn!("Worker: knowledge graph scheduling failed (capacity full)");
            }
        }
        if self.maintenance_scheduler.worker().enabled {
            let task = WorkerTask::new(
                "maintenance-worker-001",
                crate::scheduler::WorkerType::MaintenanceWorker,
                serde_json::json!({"action": "start"}),
                WorkerPriority::Low,
            );
            let submitted = schedule_task(
                &mut self.supervisor,
                task,
                crate::scheduler::SchedulingMode::Immediate,
            );
            if submitted {
                tracing::info!("Worker: maintenance scheduled");
            } else {
                tracing::warn!("Worker: maintenance scheduling failed (capacity full)");
            }
        }

        let results = self.process_tasks(db);
        for (task_id, success) in &results {
            tracing::info!(task_id, success, "Worker task executed");
        }
    }

    /// Process pending tasks from the supervisor and run scheduled maintenance.
    /// Per Architecture §23.7: workers execute tasks asynchronously with DB-backed results.
    pub fn process_tasks(&mut self, db: &SqliteDatabase) -> Vec<(String, bool)> {
        let results = self.supervisor.execute_pending();

        let maintenance_results = self.maintenance_scheduler.run_maintenance(db);
        match maintenance_results {
            Ok(results) => {
                for result in &results {
                    tracing::info!(
                        task = %result.task_type,
                        success = result.success,
                        records = result.records_affected,
                        duration_ms = result.duration_ms,
                        "Maintenance task executed"
                    );
                }
            }
            Err(e) => {
                tracing::error!("Maintenance failed: {}", e);
            }
        }

        results
    }

    /// Stop all workers by clearing pending tasks.
    pub fn stop_workers(&mut self) {
        self.supervisor.clear_tasks();
    }

    /// Check health of all workers.
    pub fn check_health(&self) -> std::collections::HashMap<String, WorkerStatus> {
        let mut health = std::collections::HashMap::new();
        for task in self.supervisor.pending_tasks() {
            health.insert(task.task_id.clone(), WorkerStatus::Idle);
        }
        for task in self.supervisor.running_tasks() {
            health.insert(task.task_id.clone(), WorkerStatus::Running);
        }
        health
    }

    /// Run autonomous memory promotion and consolidation cycle.
    /// Per Architecture §17: memory promotion and consolidation SHALL run autonomously.
    /// This wires MemoryHierarchy::promote_record() (via MemoryEngine::restructure)
    /// into the background worker system so promotion runs without external input.
    pub fn run_memory_maintenance(&self, engine: &crate::memory::MemoryEngine) {
        // Run full restructuring: promote high-confidence records, compress similar
        // memories, apply forgetting policy.
        let promoted = engine.restructure();
        tracing::info!(
            promoted_count = promoted,
            "Memory promotion cycle completed"
        );

        // Apply confidence decay.
        engine.decay_confidence(0.01);

        // Apply additional forgetting policy.
        let forgotten = engine.apply_forgetting();
        tracing::info!(
            forgotten_count = forgotten.len(),
            "Memory forgetting cycle completed"
        );

        tracing::info!("Autonomous memory promotion and consolidation cycle finished");
    }
}

/// Run autonomous memory promotion and consolidation cycle.
/// Per Architecture §17: memory promotion and consolidation SHALL run autonomously.
/// Standalone entry point for callers that do not have a BackgroundWorkerSystem.
pub fn run_memory_maintenance_cycle(engine: &crate::memory::MemoryEngine) {
    let promoted = engine.restructure();
    tracing::info!(
        promoted_count = promoted,
        "Memory promotion cycle completed"
    );

    engine.decay_confidence(0.01);

    let forgotten = engine.apply_forgetting();
    tracing::info!(
        forgotten_count = forgotten.len(),
        "Memory forgetting cycle completed"
    );

    tracing::info!("Autonomous memory promotion and consolidation cycle finished");
}
