// /src/CoObOpLoop/conformance/mod.rs
// Conformance mapping: architecture sections to source modules (§23 / T16).

/// Section-to-module mapping summary (§T16).
pub const CONFORMANCE_MAP: &[(&str, &str)] = &[
    (
        "§2 Core Principle",
        "loop_runner.rs: run_cycle, should_continue (T16.3)",
    ),
    (
        "§3 Sources",
        "sources.rs: ObjectiveSource, ObjectiveSourceRegistry (T16.4)",
    ),
    (
        "§4 Queue",
        "queue.rs: GoalStatus, AgentGoal, ObjectiveQueue (T16.5)",
    ),
    (
        "§5 Evaluation",
        "evaluation.rs: EvaluationCriteria, compute_priority, PriorityPolicy (T16.6)",
    ),
    (
        "§6 Capability",
        "capability.rs: CapabilityId, CapabilityRegistry, CapabilityAssessment (T16.7)",
    ),
    (
        "§7 Loop",
        "loop_runner.rs: LoopStage, LoopRunner, run_cycle (T16.8)",
    ),
    (
        "§8 Post-Task",
        "loop_runner.rs: stage 10 post-task eval, stage 11 generate_objectives (T16.9)",
    ),
    (
        "§9-10 Idle",
        "idle.rs: IdlePhase, IdleState, should_wait (T16.10)",
    ),
    (
        "§11 Research",
        "research.rs: ResearchTrigger, ResearchObjective, PersistenceTarget (T16.11)",
    ),
    (
        "§12 Hardware",
        "hardware.rs: HardwareProfile, HardwareDiscovery (T16.12)",
    ),
    (
        "§13 Inspection",
        "inspection.rs: InspectionTarget, Inspector (T16.13)",
    ),
    (
        "§14 Self-Improvement",
        "self_improvement.rs: ImprovementStage, SelfImprovementPipeline, ModificationBoundary (T16.14)",
    ),
    (
        "§15 Learning",
        "learning_pipeline.rs: LearningPipeline, LearningUpdate (T16.15)",
    ),
    (
        "§16 Human",
        "human.rs: HumanAction, HumanActionHandler (T16.16)",
    ),
    (
        "§17 Opportunity",
        "opportunity/mod.rs: Opportunity, OpportunityAdapter, OpportunityIntake (T16.17)",
    ),
    (
        "§18-19 Strategic/Hierarchy",
        "strategic.rs: StrategicObjective, ObjectiveHierarchy (T16.18)",
    ),
    (
        "§20 Cycle",
        "loop_runner.rs: 9-stage diagram comment, WAIT branch (T16.19)",
    ),
    (
        "§21-22 Principles",
        "loop_runner.rs: metrics fields; llm_provider.rs: LlmProvider trait (T16.20)",
    ),
    ("§23 Definition", "conformance/mod.rs: this file (T16.21)"),
];

/// Verify conformance is registered (§T16.22 — acceptance test step 1).
pub fn verify_registered() -> bool {
    !CONFORMANCE_MAP.is_empty() && full_text().contains("§23")
}

/// Return the full conformance map as a formatted string (§T16.21).
pub fn full_text() -> String {
    let mut out = String::new();
    out.push_str("RoBoT CoObOpLoop Conformance Map (§23)\n");
    out.push_str("=======================================\n\n");
    for (section, mapping) in CONFORMANCE_MAP {
        out.push_str(&format!("{} -> {}\n", section, mapping));
    }
    out
}
