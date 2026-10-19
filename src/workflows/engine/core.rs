// src/workflows/engine/core.rs

//! Workflow engine core implementation
//!
//! Provides multi-step workflow execution with:
//! - Dependency tracking (check_dependencies)
//! - Step status management (Pending/Running/Completed/Failed)
//! - Step-level execution (execute_step) with retries
//! - Failure recovery policies

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::bridge::tools::ToolOutput;
use crate::experience::metrics::MetricsCollector;
use crate::workflows::engine::types::{
    StepResult, StepStatus, Workflow, WorkflowEngine, WorkflowFailurePolicy, WorkflowStatus,
    WorkflowStep,
};

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
                depends_on: Vec::new(),
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

        // Execute workflow steps asynchronously with dependency-aware execution
        let engine = self.clone();
        let workflow_id_owned = workflow_id.to_string();
        let policy = WorkflowFailurePolicy::default();
        tokio::spawn(async move {
            match engine
                .execute_workflow_with_recovery(&workflow_id_owned, policy)
                .await
            {
                Ok(results) => {
                    let success_count = results
                        .iter()
                        .filter(|r| r.status == StepStatus::Completed)
                        .count();
                    let fail_count = results
                        .iter()
                        .filter(|r| r.status == StepStatus::Failed)
                        .count();
                    tracing::info!(
                        "Workflow {} completed: {} succeeded, {} failed",
                        workflow_id_owned,
                        success_count,
                        fail_count
                    );
                }
                Err(e) => {
                    tracing::error!("Workflow {} execution error: {}", workflow_id_owned, e);
                }
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

    /// Check if all dependencies for a step are satisfied (completed successfully).
    /// Returns Ok(()) if all dependencies are met, or an error listing the blocking steps.
    pub async fn check_dependencies(
        &self,
        workflow_id: &str,
        step: &WorkflowStep,
        completed_steps: &HashSet<String>,
    ) -> Result<()> {
        if step.depends_on.is_empty() {
            return Ok(());
        }

        let mut unsatisfied = Vec::new();
        for dep_id in &step.depends_on {
            if !completed_steps.contains(dep_id) {
                unsatisfied.push(dep_id.clone());
            }
        }

        if !unsatisfied.is_empty() {
            tracing::warn!(
                workflow = workflow_id,
                step = step.name,
                missing = ?unsatisfied,
                "Dependencies not met for workflow step"
            );
            anyhow::bail!(
                "Workflow {}: Step '{}' cannot run: dependencies not met - missing: {}",
                workflow_id,
                step.name,
                unsatisfied.join(", ")
            );
        }

        Ok(())
    }

    /// Execute a single workflow step with retry logic and status tracking.
    /// Returns the StepResult with the outcome of execution.
    pub async fn execute_step(
        &self,
        workflow_id: &str,
        step: &WorkflowStep,
        variables: &HashMap<String, String>,
        step_results: &HashMap<String, ToolOutput>,
    ) -> StepResult {
        let mut result = StepResult::new(step.id.clone(), step.name.clone());
        result.started_at = Some(chrono::Utc::now());

        tracing::info!(
            workflow = workflow_id,
            step = step.name,
            "Executing workflow step"
        );

        // Read relevant memories before action
        let memory_context = self
            .read_memory_before_action(&step.action, &step.parameters)
            .await;
        if let Some(ctx) = memory_context
            && let Some(memories) = ctx
                .data
                .get("memories")
                .and_then(|v| v.as_array())
                .filter(|m| !m.is_empty())
        {
            tracing::info!(
                "Found {} relevant memories before action '{}'",
                memories.len(),
                step.action
            );
        }

        // Try up to max_retries + 1 times (initial + retries)
        let max_attempts = step.max_retries + 1;
        for attempt in 0..max_attempts {
            result.retry_count = attempt;

            // Execute the step action
            match self
                .execute_step_action(&step.action, &step.parameters, variables, step_results)
                .await
            {
                Ok(output) => {
                    tracing::info!("Step '{}' completed on attempt {}", step.name, attempt + 1);
                    result.status = StepStatus::Completed;
                    result.output = Some(output.clone());
                    result.completed_at = Some(chrono::Utc::now());
                    self.metrics.increment("workflows.steps.completed").await;
                    // Record experience for reflection
                    self.record_experience_after_action(&step.action, &step.parameters, &output)
                        .await;
                    return result;
                }
                Err(e) => {
                    tracing::warn!("Step '{}' attempt {} failed: {}", step.name, attempt + 1, e);

                    // If we have retries left, continue to next attempt
                    if attempt < step.max_retries {
                        // Backoff between retries (short delay)
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            100 * (attempt as u64 + 1),
                        ))
                        .await;
                        continue;
                    }

                    // All retries exhausted - mark as failed
                    result.status = StepStatus::Failed;
                    result.error = Some(e.to_string());
                    result.completed_at = Some(chrono::Utc::now());
                    self.metrics.increment("workflows.steps.failed").await;
                    return result;
                }
            }
        }

        // Should not reach here, but handle gracefully
        result.status = StepStatus::Failed;
        result
            .error
            .get_or_insert_with(|| "Unknown execution error".to_string());
        result
    }

    // Execute a workflow with dependency-based ordering and failure recovery.
    /// Steps are executed in topological order respecting depends_on dependencies.
    /// Configurable failure policy determines whether to continue after step failures.
    pub async fn execute_workflow_with_recovery(
        &self,
        workflow_id: &str,
        failure_policy: WorkflowFailurePolicy,
    ) -> Result<Vec<StepResult>> {
        let (steps, variables) = {
            let workflows = self.workflows.read().await;
            let steps = workflows.get(workflow_id).map(|w| w.steps.clone());
            let vars = workflows
                .get(workflow_id)
                .map(|w| w.variables.clone())
                .unwrap_or_default();
            (steps, vars)
        };

        let Some(steps) = steps else {
            anyhow::bail!("Workflow {} not found", workflow_id);
        };

        if steps.is_empty() {
            return Ok(Vec::new());
        }

        // Build a lookup map for steps by ID
        let step_map: HashMap<&str, &WorkflowStep> =
            steps.iter().map(|s| (s.id.as_str(), s)).collect();

        // Validate all dependency references are valid step IDs
        for step in &steps {
            for dep_id in &step.depends_on {
                if !step_map.contains_key(dep_id.as_str()) {
                    anyhow::bail!(
                        "Step '{}' references non-existent dependency: {}",
                        step.name,
                        dep_id
                    );
                }
            }
        }

        // Track completed step IDs for dependency checking
        let mut completed_steps: HashSet<String> = HashSet::new();
        let mut failed_steps: HashSet<String> = HashSet::new();
        let mut step_results: Vec<StepResult> = Vec::new();
        let mut executed: HashSet<String> = HashSet::new();
        let mut step_results_map: HashMap<String, ToolOutput> = HashMap::new();

        // Execute steps in dependency order using a ready-queue approach
        let mut ready_queue: Vec<WorkflowStep> = Vec::new();
        let all_steps: Vec<WorkflowStep> = steps.clone();

        // Initialize ready queue with steps that have no dependencies
        let mut steps_to_process: Vec<WorkflowStep> = Vec::new();
        for step in &all_steps {
            if step.depends_on.is_empty() {
                ready_queue.push(step.clone());
            } else {
                steps_to_process.push(step.clone());
            }
        }

        let max_iterations = all_steps.len() * all_steps.len() + 1;
        let mut iteration = 0;

        while !ready_queue.is_empty() || !steps_to_process.is_empty() {
            iteration += 1;
            if iteration > max_iterations {
                anyhow::bail!(
                    "Workflow execution loop limit exceeded - possible circular dependency"
                );
            }

            let mut newly_ready: Vec<WorkflowStep> = Vec::new();

            // Process all ready steps - take ownership to avoid borrow issues
            let mut steps_to_process: Vec<WorkflowStep> = std::mem::take(&mut ready_queue);
            for step in &steps_to_process {
                // Check dependencies again (defensive)
                if let Err(e) = self
                    .check_dependencies(workflow_id, step, &completed_steps)
                    .await
                {
                    tracing::warn!("Step '{}' dependency check failed: {}", step.name, e);
                    let sr = StepResult::with_error(
                        step.id.clone(),
                        step.name.clone(),
                        format!("Dependency check failed: {}", e),
                    );
                    step_results.push(sr);
                    failed_steps.insert(step.id.clone());
                    continue;
                }

                // Execute the step
                let sr = self
                    .execute_step(workflow_id, step, &variables, &step_results_map)
                    .await;
                step_results.push(sr.clone());

                if sr.status == StepStatus::Completed {
                    completed_steps.insert(step.id.clone());
                    if let Some(output) = &sr.output {
                        step_results_map.insert(step.id.clone(), output.clone());
                    }
                } else {
                    failed_steps.insert(step.id.clone());

                    match failure_policy {
                        WorkflowFailurePolicy::StopOnFirstFailure => {
                            // Mark remaining steps as skipped
                            for remaining in &steps_to_process {
                                if !executed.contains(&remaining.id) {
                                    let skipped = StepResult::with_error(
                                        remaining.id.clone(),
                                        remaining.name.clone(),
                                        "Workflow stopped due to prior failure".to_string(),
                                    );
                                    step_results.push(skipped);
                                }
                            }
                            return Ok(step_results);
                        }
                        WorkflowFailurePolicy::ContinueOnError => {
                            tracing::warn!(
                                "Step '{}' failed, continuing per failure policy",
                                step.name
                            );
                        }
                        WorkflowFailurePolicy::RetryUnblocked => {
                            // Don't add to failed_steps permanently; the step
                            // may be re-executed if its dependencies are later met
                            tracing::warn!("Step '{}' failed, will retry if unblocked", step.name);
                        }
                    }
                }

                executed.insert(step.id.clone());
            }

            // Check if any remaining steps are now ready (outside the loop to avoid borrow issues)
            let mut still_blocked: Vec<WorkflowStep> = Vec::new();
            for remaining in steps_to_process.drain(..) {
                if executed.contains(&remaining.id) {
                    continue;
                }
                // Check if all dependencies are now completed
                let deps_met = remaining
                    .depends_on
                    .iter()
                    .all(|dep_id| completed_steps.contains(dep_id));
                if deps_met {
                    newly_ready.push(remaining);
                } else {
                    still_blocked.push(remaining);
                }
            }
            steps_to_process = still_blocked;
            ready_queue.extend(std::mem::take(&mut newly_ready));

            // If nothing is ready and nothing is blocked, we have a deadlock
            if ready_queue.is_empty() && !steps_to_process.is_empty() {
                // Mark remaining blocked steps as failed
                for remaining in steps_to_process.drain(..) {
                    if !executed.contains(&remaining.id) {
                        let stuck = StepResult::with_error(
                            remaining.id.clone(),
                            remaining.name.clone(),
                            "Step stuck: dependencies never satisfied".to_string(),
                        );
                        step_results.push(stuck);
                        failed_steps.insert(remaining.id.clone());
                    }
                }
            }
        }

        // Update workflow status based on results
        let has_failed = step_results.iter().any(|r| r.status == StepStatus::Failed);
        let workflow_status = if has_failed {
            WorkflowStatus::Failed
        } else {
            WorkflowStatus::Completed
        };

        {
            let mut workflows = self.workflows.write().await;
            if let Some(workflow) = workflows.get_mut(workflow_id) {
                workflow.status = workflow_status;
                workflow.completed_at = Some(chrono::Utc::now());
                workflow.variables = variables;
            }
        }

        {
            let mut executing = self.executing.write().await;
            executing.remove(workflow_id);
        }

        if !has_failed {
            self.metrics.increment("workflows.completed").await;
        }

        Ok(step_results)
    }

    /// Add a step to a workflow with dependency specification
    pub async fn add_step_with_deps(
        &self,
        workflow_id: &str,
        name: impl Into<String>,
        action: impl Into<String>,
        depends_on: Vec<String>,
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
                depends_on,
                on_success: None,
                on_failure: None,
            };

            workflow.steps.push(step.clone());
            self.metrics.increment("workflows.steps.added").await;

            return Ok(Some(step));
        }

        Ok(None)
    }

    /// Get step execution results for a workflow
    pub async fn get_step_results(&self, workflow_id: &str) -> Result<Vec<StepResult>> {
        let workflows = self.workflows.read().await;
        if let Some(workflow) = workflows.get(workflow_id) {
            Ok(workflow
                .steps
                .iter()
                .map(|s| StepResult::new(s.id.clone(), s.name.clone()))
                .collect())
        } else {
            anyhow::bail!("Workflow {} not found", workflow_id);
        }
    }
}
