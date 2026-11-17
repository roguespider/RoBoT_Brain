pub mod api;
// NOTE: architecture/ consolidated into common/architecture.rs per Appendix A
pub mod bootstrap;
pub mod common;
pub use common::data_contracts;
pub mod database;
pub mod experience;
pub mod knowledge;
pub mod learning;

pub mod memory;
// NOTE: memory_hierarchy/ consolidated into memory/hierarchy.rs per Architecture §14
pub mod models;

pub mod planning;

// NOTE: prompt_construction/ consolidated into prompt/ per Architecture §17
pub mod prompt;
pub mod research;
pub mod security;
pub mod storage;
pub mod testing;

pub mod tool_engine;

pub mod agent;
pub mod retrieval;
pub mod scheduler;
pub mod workers;

pub mod confidence;
pub mod config;
pub mod context;
pub mod conversation;
pub use scheduler::cooboploop;
pub mod deployment;

pub mod execution;
pub mod navigation;
pub mod tracing;
