// src/workflows/engine/mod.rs
//! Workflow execution engine
//! 
//! This module provides the workflow engine for executing multi-step workflows.

#![allow(dead_code)]

mod engine;
mod executor;
mod experience;
mod types;

pub use engine::SKIP_MEMORY_READ;
pub use executor::{parse_outcome_kind, build_experience_title};
pub use experience::{build_experience_description, build_search_query, extract_result_summary, map_action_to_experience_type};
pub use types::{ExperienceRecord, Workflow, WorkflowEngine, WorkflowStatus, WorkflowStep};
