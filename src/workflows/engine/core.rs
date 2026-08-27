// src/workflows/engine/engine.rs

//! Workflow engine core implementation

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::experience::metrics::MetricsCollector;
use crate::workflows::engine::types::{Workflow, WorkflowEngine, WorkflowStatus, WorkflowStep};

/// Actions that should skip memory read (already do their own lookup)
pub const SKIP_MEMORY_READ: &[&str] = &[
    "search_memory",
    "list_memories",
    "get_memory",
    "get_experience",
    "list_experiences",
    "get_experience_stats",
];

impl WorkflowEngine {
    /// Create a new workflow engine with database, coordinator, and optional memory retrieval
    pub fn with_database_and_coordinator(
        metrics: Arc<MetricsCollector>,
        database: Arc<crate::database::sqlite::SqliteDatabase>,
        coordinator: Arc<crate::experience::coordinator::ExperienceCoordinator>,
        memory_retrieval: Option<Arc<crate::memory::retrieval::MemoryRetrieval>>,
    ) -> Self {
        Self {
            metrics,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            executing: Arc::new(RwLock::new(HashSet::new())),
            database: Some(database),
            coordinator: Some(coordinator),
            memory_retrieval,
        }
    }

    /// Create a new workflow definition
    pub async fn create_workflow(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Workflow {
        let workflow = Workflow {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.into(),
            steps: Vec::new(),
            variables: HashMap::new(),
            status: WorkflowStatus::Draft,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };

        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id.clone(), workflow.clone());

        self.metrics.increment("workflows.created").await;

        workflow
    }

    /// Add a step to a workflow
    pub async fn add_step(
        &self,
        workflow_id: &str,
        name: impl Into<String>,
        action: impl Into<String>,
    ) -> Result<Option<WorkflowStep>> {
        let mut workflows = self.workflows.write().await;

        if let Some(workflow) = workflows.get_mut(workflow_id) {
            let step = WorkflowStep {
                id: Uuid::new_v4().to_string(),
                name: name.into(),
                action: action.into(),
                parameters: HashMap::new(),
                retry_count: 0,
                max_retries: 3,
                timeout_seconds: 300,
                on_success: None,
                on_failure: None,
            };

            workflow.steps.push(step.clone());
            self.metrics.increment("workflows.steps.added").await;

            return Ok(Some(step));
        }

        Ok(None)
    }

    /// Set workflow variable
    pub async fn set_variable(
        &self,
        workflow_id: &str,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<()> {
        let mut workflows = self.workflows.write().await;

        if let Some(workflow) = workflows.get_mut(workflow_id) {
            workflow.variables.insert(key.into(), value.into());
        }

        Ok(())
    }

    /// Get a workflow by ID
    pub async fn get_workflow(&self, workflow_id: &str) -> Option<Workflow> {
        let workflows = self.workflows.read().await;
        workflows.get(workflow_id).cloned()
    }

    /// List all workflows
    pub async fn list_workflows(&self) -> Vec<Workflow> {
        let workflows = self.workflows.read().await;
        workflows.values().cloned().collect()
    }

    /// List workflows by status
    pub async fn list_by_status(&self, status: WorkflowStatus) -> Vec<Workflow> {
        let workflows = self.workflows.read().await;
        workflows
            .values()
            .filter(|w| w.status == status)
            .cloned()
            .collect()
    }

    /// Validate workflow readiness
    pub async fn validate_workflow(&self, workflow_id: &str) -> Result<bool> {
        let workflows = self.workflows.read().await;

        if let Some(workflow) = workflows.get(workflow_id) {
            if workflow.steps.is_empty() {
                return Ok(false);
            }

            // Check step references are valid
            for step in &workflow.steps {
                if let Some(ref on_success) = step.on_success
                    && !workflow.steps.iter().any(|s| &s.id == on_success)
                {
                    anyhow::bail!(
                        "Step {} references non-existent success target: {}",
                        step.id,
                        on_success
                    );
                }
                if let Some(ref on_failure) = step.on_failure
                    && !workflow.steps.iter().any(|s| &s.id == on_failure)
                {
                    anyhow::bail!(
                        "Step {} references non-existent failure target: {}",
                        step.id,
                        on_failure
                    );
                }
            }

            return Ok(true);
        }

        Ok(false)
    }

    /// Start workflow execution
    pub async fn start(&self, workflow_id: &str) -> Result<()> {
        // Check if already executing
        {
            let executing = self.executing.read().await;
            if executing.contains(workflow_id) {
                anyhow::bail!("Workflow {} is already executing", workflow_id);
            }
        }

        let is_valid = self.validate_workflow(workflow_id).await?;
        if !is_valid {
            anyhow::bail!("Workflow {} is not valid", workflow_id);
        }

        // Mark as executing
        {
            let mut executing = self.executing.write().await;
            executing.insert(workflow_id.to_string());
        }

        // Update workflow status
        {
            let mut workflows = self.workflows.write().await;
            if let Some(workflow) = workflows.get_mut(workflow_id) {
                workflow.status = WorkflowStatus::Running;
                workflow.started_at = Some(chrono::Utc::now());
            }
        }

        self.metrics.increment("workflows.started").await;

        // Execute workflow steps asynchronously
        let engine = self.clone();
        let workflow_id_owned = workflow_id.to_string();
        tokio::spawn(async move {
            if let Err(e) = engine.execute_workflow(&workflow_id_owned).await {
                tracing::error!("Workflow {} execution error: {}", workflow_id_owned, e);
            }
        });

        Ok(())
    }

    /// Pause workflow execution
    pub async fn pause(&self, workflow_id: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(workflow_id)
            && workflow.status == WorkflowStatus::Running
        {
            workflow.status = WorkflowStatus::Paused;
            self.metrics.increment("workflows.paused").await;
        }
        Ok(())
    }

    /// Resume paused workflow
    pub async fn resume(&self, workflow_id: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(workflow_id)
            && workflow.status == WorkflowStatus::Paused
        {
            workflow.status = WorkflowStatus::Running;
            self.metrics.increment("workflows.resumed").await;
        }
        Ok(())
    }

    /// Cancel workflow execution
    pub async fn cancel(&self, workflow_id: &str) -> Result<()> {
        {
            let mut executing = self.executing.write().await;
            executing.remove(workflow_id);
        }

        {
            let mut workflows = self.workflows.write().await;
            if let Some(workflow) = workflows.get_mut(workflow_id) {
                workflow.status = WorkflowStatus::Cancelled;
            }
        }

        self.metrics.increment("workflows.cancelled").await;

        Ok(())
    }

    /// Delete workflow
    pub async fn delete(&self, workflow_id: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        workflows.remove(workflow_id);
        Ok(())
    }
}
