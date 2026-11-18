// src/workflows/engine/mod.rs
//! Workflow execution engine
//!
//! This module provides the workflow engine for executing multi-step workflows.

mod core;
mod executor;
mod types;

pub use core::SKIP_MEMORY_READ;
pub use types::{
    StepResult, StepStatus, Workflow, WorkflowEngine, WorkflowFailurePolicy, WorkflowStatus,
};

/// Get a summary string for a workflow failure policy
pub fn failure_policy_description(policy: WorkflowFailurePolicy) -> &'static str {
    match policy {
        WorkflowFailurePolicy::StopOnFirstFailure => "stop_on_first_failure",
        WorkflowFailurePolicy::ContinueOnError => "continue_on_error",
        WorkflowFailurePolicy::RetryUnblocked => "retry_unblocked",
    }
}

/// Get the status string for a workflow step result
pub fn step_status_description(result: &StepResult) -> &str {
    match result.status {
        StepStatus::Pending => "pending",
        StepStatus::Running => "running",
        StepStatus::Completed => "completed",
        StepStatus::Failed => "failed",
    }
}
