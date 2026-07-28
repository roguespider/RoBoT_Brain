// src/workflows/engine/types.rs
#![allow(dead_code)]
//! Type definitions for workflow engine

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::experience::metrics::MetricsCollector;

/// A workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub variables: HashMap<String, String>,
    pub status: WorkflowStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// A single step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub action: String,
    pub parameters: HashMap<String, String>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub timeout_seconds: u64,
    pub on_success: Option<String>,
    pub on_failure: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowStatus {
    Draft,
    Ready,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Workflow execution engine
pub struct WorkflowEngine {
    pub(crate) metrics: Arc<MetricsCollector>,
    pub(crate) workflows: Arc<RwLock<HashMap<String, Workflow>>>,
    pub(crate) executing: Arc<RwLock<HashSet<String>>>,
    pub(crate) database: Option<Arc<crate::database::sqlite::SqliteDatabase>>,
    pub(crate) coordinator: Option<Arc<crate::experience::coordinator::ExperienceCoordinator>>,
}

/// Experience record for learning from workflow execution
#[derive(Debug, Clone)]
pub struct ExperienceRecord {
    pub action: String,
    pub observation: String,
    pub outcome: String,
    pub outcome_kind: String,
    pub search_query: String,
    pub title: String,
    pub interpretation: Option<String>,
    pub reflection_questions: Vec<String>,
}

impl Clone for WorkflowEngine {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            workflows: self.workflows.clone(),
            executing: self.executing.clone(),
            database: self.database.clone(),
            coordinator: self.coordinator.clone(),
        }
    }
}
