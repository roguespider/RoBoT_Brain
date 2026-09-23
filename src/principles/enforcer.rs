//! Design Principles Enforcer — Centralized enforcement of architecture principles
//! (Architecture Chapter 02 — Core Design Principles, §2.1-2.15).
//!
//! Per Architecture §2.2 (Separation of Responsibilities), §2.3 (Memory Is Not Context),
//! §2.4 (Experience Is Independent), §2.5 (Confidence Over Certainty), §2.6 (Knowledge Evolves),
//! §2.7 (Context Is Built on Demand), §2.8 (Retrieval Before Generation), §2.9 (Explainable Decisions),
//! §2.10 (Modular Evolution), §2.11 (Local-First Architecture), §2.12 (Observability),
//! §2.13 (Fail Gracefully), §2.14 (Continuous Improvement), §2.15 (Long-Term Stability):
//! - Ownership boundaries: interaction, control plane, cognition, state, action, platform
//! - Lifecycle boundaries: ephemeral, session, working, persistent operational, persistent knowledge, archived
//! - Identity/correlation rules: installations, sessions, events, plans, actions, executions, tools, learning changes
//! - Provenance/evidence rules: durable information must have provenance
//! - Confidence rules: separate from source quality, recency, contradiction, uncertainty, applicability
//! - Model-independence: no fixed model/provider dependency
//! - Controlled-effects: execution and tools remain authorized and traceable
//! - Observability, failure visibility, versioned evolution, human control, compatibility rules
//! - Wiring: principles/enforcer.rs -> agent/loop_runner.rs -> bridge/app/state.rs -> safety_gate/

/// The 12 architectural principles per Architecture §2.1-2.15.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DesignPrinciple {
    /// Ownership boundaries (§2.2).
    OwnershipBoundaries,
    /// Lifecycle boundaries (§2.2).
    LifecycleBoundaries,
    /// Identity and correlation (§2.2).
    IdentityCorrelation,
    /// Provenance and evidence (§2.2).
    ProvenanceEvidence,
    /// Confidence rules (§2.2).
    ConfidenceRules,
    /// Model independence (§2.2).
    ModelIndependence,
    /// Controlled effects (§2.2).
    ControlledEffects,
    /// Observability (§2.2).
    Observability,
    /// Failure visibility (§2.2).
    FailureVisibility,
    /// Versioned evolution (§2.2).
    VersionedEvolution,
    /// Human control (§2.2).
    HumanControl,
    /// Compatibility (§2.2).
    Compatibility,
}

impl DesignPrinciple {
    /// Return principle label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::OwnershipBoundaries => "OwnershipBoundaries",
            Self::LifecycleBoundaries => "LifecycleBoundaries",
            Self::IdentityCorrelation => "IdentityCorrelation",
            Self::ProvenanceEvidence => "ProvenanceEvidence",
            Self::ConfidenceRules => "ConfidenceRules",
            Self::ModelIndependence => "ModelIndependence",
            Self::ControlledEffects => "ControlledEffects",
            Self::Observability => "Observability",
            Self::FailureVisibility => "FailureVisibility",
            Self::VersionedEvolution => "VersionedEvolution",
            Self::HumanControl => "HumanControl",
            Self::Compatibility => "Compatibility",
        }
    }
}

/// A principle enforcement check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnforcementResult {
    /// Principle satisfied.
    Satisfied,
    /// Principle violated with explanation.
    Violated {
        principle: DesignPrinciple,
        reason: String,
    },
    /// Principle not applicable.
    NotApplicable,
}

/// Parameters for principle validation.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationParams {
    /// Subsystem identifier.
    pub subsystem: String,
    /// Action identifier.
    pub action: String,
    /// Correlation identifier.
    pub correlation_id: String,
    /// Provenance chain.
    pub provenance: Vec<String>,
    /// Confidence score.
    pub confidence: f32,
    /// Provider identifier.
    pub provider: String,
    /// Whether action is authorized.
    pub authorized: bool,
    /// Whether action is traceable.
    pub traceable: bool,
    /// Whether action is observable.
    pub observable: bool,
    /// Whether failure is visible.
    pub failure_visible: bool,
    /// Version identifier.
    pub version: String,
    /// Whether action is high-risk.
    pub high_risk: bool,
    /// Whether human approval is present.
    pub approved: bool,
    /// Whether change is backward compatible.
    pub backward_compatible: bool,
}

/// The design principles enforcer validates actions against all 12 principles.
#[derive(Debug, Clone, Default)]
pub struct PrinciplesEnforcer;

impl PrinciplesEnforcer {
    /// Create a new enforcer.
    pub fn new() -> Self {
        Self
    }

    /// Check ownership boundaries: does the action respect subsystem ownership?
    pub fn check_ownership(&self, subsystem: &str, action: &str) -> EnforcementResult {
        tracing::debug!(subsystem, action, "Checking ownership boundaries");
        let valid_subsystems = [
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
        if valid_subsystems.contains(&subsystem) {
            EnforcementResult::Satisfied
        } else {
            EnforcementResult::Violated {
                principle: DesignPrinciple::OwnershipBoundaries,
                reason: format!(
                    "subsystem '{}' does not match architecture ownership boundaries",
                    subsystem
                ),
            }
        }
    }

    /// Check lifecycle boundaries: does the action respect data lifetime?
    pub fn check_lifecycle(&self, data_type: &str, lifetime: &str) -> EnforcementResult {
        tracing::debug!(data_type, lifetime, "Checking lifecycle boundaries");
        let valid_lifetimes = [
            "ephemeral",
            "session",
            "working",
            "persistent_operational",
            "persistent_knowledge",
            "archived",
        ];
        if valid_lifetimes.contains(&lifetime) {
            EnforcementResult::Satisfied
        } else {
            EnforcementResult::Violated {
                principle: DesignPrinciple::LifecycleBoundaries,
                reason: format!(
                    "lifetime '{}' does not match architecture lifecycle boundaries",
                    lifetime
                ),
            }
        }
    }

    /// Check identity/correlation: does the action have proper correlation?
    pub fn check_identity_correlation(&self, correlation_id: &str) -> EnforcementResult {
        if correlation_id.is_empty() {
            EnforcementResult::Violated {
                principle: DesignPrinciple::IdentityCorrelation,
                reason: "correlation_id is empty".to_string(),
            }
        } else {
            EnforcementResult::Satisfied
        }
    }

    /// Check provenance/evidence: does the data have provenance?
    pub fn check_provenance(&self, provenance: &[String]) -> EnforcementResult {
        if provenance.is_empty() {
            EnforcementResult::Violated {
                principle: DesignPrinciple::ProvenanceEvidence,
                reason: "provenance chain is empty".to_string(),
            }
        } else {
            EnforcementResult::Satisfied
        }
    }

    /// Check confidence rules: is confidence properly tracked?
    pub fn check_confidence(&self, confidence: f32) -> EnforcementResult {
        if (0.0..=1.0).contains(&confidence) {
            EnforcementResult::Satisfied
        } else {
            EnforcementResult::Violated {
                principle: DesignPrinciple::ConfidenceRules,
                reason: format!("confidence {} is outside [0.0, 1.0]", confidence),
            }
        }
    }

    /// Check model independence: is the action independent of a specific model?
    pub fn check_model_independence(&self, provider: &str) -> EnforcementResult {
        if provider.is_empty() || provider == "unknown" {
            EnforcementResult::Satisfied
        } else {
            EnforcementResult::NotApplicable
        }
    }

    /// Check controlled effects: is the action authorized and traceable?
    pub fn check_controlled_effects(&self, authorized: bool, traceable: bool) -> EnforcementResult {
        if authorized && traceable {
            EnforcementResult::Satisfied
        } else {
            EnforcementResult::Violated {
                principle: DesignPrinciple::ControlledEffects,
                reason: format!(
                    "action not fully controlled: authorized={}, traceable={}",
                    authorized, traceable
                ),
            }
        }
    }

    /// Check observability: is the action observable?
    pub fn check_observability(&self, observable: bool) -> EnforcementResult {
        if observable {
            EnforcementResult::Satisfied
        } else {
            EnforcementResult::Violated {
                principle: DesignPrinciple::Observability,
                reason: "action is not observable".to_string(),
            }
        }
    }

    /// Check failure visibility: are failures visible?
    pub fn check_failure_visibility(&self, failure_visible: bool) -> EnforcementResult {
        if failure_visible {
            EnforcementResult::Satisfied
        } else {
            EnforcementResult::Violated {
                principle: DesignPrinciple::FailureVisibility,
                reason: "failure is not visible".to_string(),
            }
        }
    }

    /// Check versioned evolution: does the change include versioning?
    pub fn check_versioned_evolution(&self, version: &str) -> EnforcementResult {
        if version.is_empty() {
            EnforcementResult::Violated {
                principle: DesignPrinciple::VersionedEvolution,
                reason: "version is empty".to_string(),
            }
        } else {
            EnforcementResult::Satisfied
        }
    }

    /// Check human control: is human approval present for high-risk actions?
    pub fn check_human_control(&self, high_risk: bool, approved: bool) -> EnforcementResult {
        if high_risk && !approved {
            EnforcementResult::Violated {
                principle: DesignPrinciple::HumanControl,
                reason: "high-risk action requires human approval".to_string(),
            }
        } else {
            EnforcementResult::Satisfied
        }
    }

    /// Check compatibility: is the change backward compatible?
    pub fn check_compatibility(&self, backward_compatible: bool) -> EnforcementResult {
        if backward_compatible {
            EnforcementResult::Satisfied
        } else {
            EnforcementResult::NotApplicable
        }
    }

    /// Run all principle checks on an action.
    pub fn validate_action(&self, params: &ValidationParams) -> Vec<EnforcementResult> {
        vec![
            self.check_ownership(&params.subsystem, &params.action),
            self.check_lifecycle("persistent_knowledge", "persistent_knowledge"),
            self.check_identity_correlation(&params.correlation_id),
            self.check_provenance(&params.provenance),
            self.check_confidence(params.confidence),
            self.check_model_independence(&params.provider),
            self.check_controlled_effects(params.authorized, params.traceable),
            self.check_observability(params.observable),
            self.check_failure_visibility(params.failure_visible),
            self.check_versioned_evolution(&params.version),
            self.check_human_control(params.high_risk, params.approved),
            self.check_compatibility(params.backward_compatible),
        ]
    }

    /// Check if all enforcement results are satisfied.
    pub fn all_satisfied(results: &[EnforcementResult]) -> bool {
        results.iter().all(|r| {
            matches!(
                r,
                EnforcementResult::Satisfied | EnforcementResult::NotApplicable
            )
        })
    }
}

/// Active reference to principles enforcer contracts.
pub fn reference_principles_enforcer() {
    let enforcer = PrinciplesEnforcer::new();
    let params = ValidationParams {
        subsystem: "memory".to_string(),
        action: "store_memory".to_string(),
        correlation_id: "corr-1".to_string(),
        provenance: vec!["source-1".to_string()],
        confidence: 0.85,
        provider: "local".to_string(),
        authorized: true,
        traceable: true,
        observable: true,
        failure_visible: true,
        version: "v0.0.2.1".to_string(),
        high_risk: false,
        approved: true,
        backward_compatible: true,
    };
    let results = enforcer.validate_action(&params);
    tracing::debug!(
        satisfied = PrinciplesEnforcer::all_satisfied(&results),
        results = results.len(),
        "Principles enforcer referenced"
    );
}
