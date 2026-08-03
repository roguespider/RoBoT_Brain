// src/experience/mod.rs
//! Experience System - Per Architecture §2.1, §5, §07, §22
//!
//! The Experience System is the foundation of learning.
//! It records events, observations, actions, outcomes, and environmental changes.

pub mod bus;
pub mod coordinator;
pub mod encounter_recorder;
pub mod event_handler;

pub mod events;

pub mod evolution;
pub mod exploration;
pub mod hypothesis;

pub mod integration;

pub mod metrics;
pub mod observer; // Observer trait + concrete implementations
pub mod queue;

pub mod reflection;
pub mod repository;

pub mod reputation;

pub mod scheduler;
pub mod scorer;
pub mod types;
pub mod worker;
pub mod worker_manager;
