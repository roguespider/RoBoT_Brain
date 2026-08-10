// src/agent/mod.rs
//! Goal-driven agent loop (Architecture §5.7 Decision Flow, §5.8 Feedback Loop)
//!
//! Closes the cognitive loop: given a goal, the agent
//!   Goal → Plan → Memory retrieval → Knowledge retrieval → Experience
//!   retrieval → Confidence evaluation → Action selection → Execution →
//!   Outcome recording
//! and records the outcome as a new experience, which (via the §4.04 event
//! spine wired in P0) drives the full learning pipeline.
//!
//! Per TASK-V2-04 this is the single biggest missing piece toward the
//! architecture's vision of a continuously self-improving cognitive loop.
//! The agent owns no business logic of its own; it composes the existing
//! Planner, MemoryRetrieval, KnowledgeStore, ExperienceCoordinator and
//! ExperienceRecorder through their public APIs.

pub mod context;
pub mod decision;
pub mod loop_runner;
pub mod safety_gate;
// V2-09: self_check removed (code now exercised by run_agent_goal MCP tool)
pub mod types;

// Re-export the types that are consumed via the `crate::agent::` path by the
// app initialization and self-check. Other types are accessed via their module
// path to avoid dead re-export warnings (binary crate, no external consumers).
pub use context::AgentDeps;
pub use loop_runner::AgentLoop;
pub use safety_gate::SafetyGate;
