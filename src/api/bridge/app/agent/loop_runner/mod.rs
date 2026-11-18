// src/api/bridge/app/agent/loop_runner/mod.rs
//! Goal-driven cognitive loop (Architecture §5.7)

pub mod runner;
pub mod types;

pub use runner::AgentLoop;
pub use types::AgentLoopOutcome;
