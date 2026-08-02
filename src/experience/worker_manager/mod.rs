// /src/experience/worker_manager/mod.rs
//! Worker Manager - manages workers per observer per Architecture §22
//!
//! Design per README.md Pipeline Design:
//! Experience Recorded → Recorder → Bus → Job Queue → Workers → Observers
//!
//! This module connects the event bus to workers, routing events to appropriate
//! observers based on their acceptance criteria.

pub mod background;
pub mod manager;

pub use manager::WorkerManager;
