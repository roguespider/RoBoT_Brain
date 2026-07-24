// src/workflows/mod.rs
//! Workflow execution engine

//! Scaffolding module - workflow execution for multi-step tasks
#![allow(dead_code)]

pub mod engine;

pub use engine::{Workflow, WorkflowStep, WorkflowEngine, WorkflowStatus};
