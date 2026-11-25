//! Architecture alignment rules (Architecture Chapter 24 — AI Contributor Operating
//! Agreement).
//!
//! Per Architecture §24.10 (Architecture Alignment Check), a contribution may only
//! be accepted when it answers three questions:
//!
//!   - *Does it belong?* — which architecture subsystem owns it?
//!   - *Does it connect?* — who calls it?
//!   - *Does it improve capability?* — what problem is solved?
//!
//! Per §24.8 (Trace Before Changing) a change is traced before it is accepted:
//! `Function -> Callers -> Dependencies -> Data Flow -> Tests -> Architecture Purpose`.
//!
//! This module also owns the architecture's action registry — the single source of
//! truth mapping an action to the subsystem that owns it and to its risk class.
//! The agent safety gate (Architecture §16) and the contribution acceptance gate
//! (Architecture §24) both read from it, so the two can never disagree about which
//! actions the architecture knows and who owns them.

/// The architecture subsystems that own behaviour (Architecture §2.2 ownership
/// boundaries).
pub const ARCHITECTURE_SUBSYSTEMS: &[&str] = &[
    "memory",
    "experience",
    "learning",
    "knowledge",
    "planning",
    "execution",
    "tool",
    "model",
    "communication",
    "coordination",
    "context",
    "conversation",
    "observation",
    "reflection",
];

/// Risk class of an action within the architecture's action registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchitectureRisk {
    /// Read-only retrieval.
    Read,
    /// Append-only or idempotent mutation.
    Mutate,
    /// Not part of the autonomous action registry; treated as destructive.
    Destructive,
}

/// The architecture's action registry: `(action, owning subsystem, risk)`.
///
/// Only actions the architecture knows are listed. Every other action is
/// classified as `Destructive` by [`action_risk_class`].
pub const ARCHITECTURE_ACTIONS: &[(&str, &str, ArchitectureRisk)] = &[
    // Read-only retrieval.
    ("search_memory", "memory", ArchitectureRisk::Read),
    ("get_memory", "memory", ArchitectureRisk::Read),
    ("list_memories", "memory", ArchitectureRisk::Read),
    ("query_knowledge", "knowledge", ArchitectureRisk::Read),
    ("get_knowledge", "knowledge", ArchitectureRisk::Read),
    ("global_search", "observation", ArchitectureRisk::Read),
    ("list_experiences", "experience", ArchitectureRisk::Read),
    ("get_insights", "learning", ArchitectureRisk::Read),
    ("get_plan", "planning", ArchitectureRisk::Read),
    ("list_plans", "planning", ArchitectureRisk::Read),
    ("list_workflows", "execution", ArchitectureRisk::Read),
    ("get_system_status", "observation", ArchitectureRisk::Read),
    ("discover_skill", "execution", ArchitectureRisk::Read),
    ("list_tools", "tool", ArchitectureRisk::Read),
    // Append-only / idempotent mutations.
    ("store_memory", "memory", ArchitectureRisk::Mutate),
    ("add_knowledge", "knowledge", ArchitectureRisk::Mutate),
    ("record_experience", "experience", ArchitectureRisk::Mutate),
    ("create_plan", "planning", ArchitectureRisk::Mutate),
    ("create_workflow", "execution", ArchitectureRisk::Mutate),
    ("start_workflow", "execution", ArchitectureRisk::Mutate),
    ("create_hypothesis", "learning", ArchitectureRisk::Mutate),
    ("add_evidence", "learning", ArchitectureRisk::Mutate),
    ("register_agent", "communication", ArchitectureRisk::Mutate),
    ("register_skill", "execution", ArchitectureRisk::Mutate),
];

/// Map an action to the architecture subsystem that owns it.
///
/// Returns `None` when the architecture does not know the action.
pub fn subsystem_for_action(action: &str) -> Option<&'static str> {
    ARCHITECTURE_ACTIONS
        .iter()
        .find(|(name, _, _)| *name == action)
        .map(|(_, subsystem, _)| *subsystem)
}

/// Classify an action's risk within the architecture's action registry.
///
/// Actions the architecture does not know are `Destructive`.
pub fn action_risk_class(action: &str) -> ArchitectureRisk {
    ARCHITECTURE_ACTIONS
        .iter()
        .find(|(name, _, _)| *name == action)
        .map(|(_, _, risk)| *risk)
        .unwrap_or(ArchitectureRisk::Destructive)
}

/// Check if a name aligns with the architecture subsystems.
///
/// This is the free-form check used for function, module and artifact names:
/// a name aligns when it references an architecture subsystem, either as the
/// name itself (`memory`) or as a token inside a longer identifier
/// (`retrieve_memory_context`).
pub fn architecture_alignment_check(name: &str) -> bool {
    ARCHITECTURE_SUBSYSTEMS.iter().any(|sub| name.contains(sub))
}

/// Check whether `owner` names an architecture subsystem exactly.
///
/// Ownership is stricter than name alignment: a subsystem owns a contribution
/// only when the owner *is* the subsystem, not when the owner merely mentions
/// one (Architecture §24.10 "Does it belong?").
pub fn is_architecture_subsystem(owner: &str) -> bool {
    ARCHITECTURE_SUBSYSTEMS.contains(&owner)
}

/// Enforce the architecture alignment check.
///
/// Returns an error describing the misalignment when the name does not belong
/// to any architecture subsystem.
pub fn enforce_alignment_check(name: &str) -> Result<(), String> {
    if is_architecture_subsystem(name) {
        Ok(())
    } else {
        Err(format!(
            "'{}' does not belong to any architecture subsystem",
            name
        ))
    }
}

/// Return the required trace format before changing a function (Architecture §24.8).
pub fn trace_before_changing(function: &str) -> String {
    format!(
        "Function → Callers → Dependencies → Data Flow → Tests → Architecture Purpose (function: {})",
        function
    )
}

/// Enforce the trace protocol before changing (Architecture §24.8).
pub fn enforce_trace_protocol(function: &str) -> String {
    let result = trace_before_changing(function);
    tracing::debug!(function, "trace_protocol_result={}", result);
    result
}
