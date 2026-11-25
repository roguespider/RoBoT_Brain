//! Contribution acceptance gate (Architecture Chapter 24 — AI Contributor Operating
//! Agreement).
//!
//! Per Architecture §24.7 (Function Creation Rules) an AI-generated function must
//! have a defined purpose, a caller, expected inputs, expected outputs, error
//! behaviour and a validation path — "a function without integration is
//! incomplete". Per §24.10 a contribution is only accepted when it
//!
//!   - *does it belong* — which architecture subsystem owns it,
//!   - *does it connect* — who calls it,
//!   - *does it improve capability* — what problem is solved,
//!   - *does it increase complexity* — and is that complexity justified.
//!
//! The gate answers those four questions for a proposed contribution and combines
//! them with the twelve design principles of Architecture §2.1-2.15, so a
//! contribution that violates the architecture is rejected *before* it executes
//! instead of being discovered afterwards.

use super::alignment::{
    architecture_alignment_check, enforce_alignment_check, enforce_trace_protocol,
    subsystem_for_action,
};
use super::autonomous::{ContributorType, OperatingAgreement};
use super::enforcer::{EnforcementResult, PrinciplesEnforcer, ValidationParams};

/// The architectural complexity a contribution introduces (Architecture §24.10).
///
/// §24.10 asks "Does it increase complexity? Is the complexity justified?".
/// A contribution that opens a new subsystem boundary or wires a new
/// cross-subsystem dependency has to say why the added surface is worth its
/// cost; a contribution that stays inside the existing architecture does not.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComplexityImpact {
    /// New architecture subsystem boundaries the contribution introduces.
    pub new_subsystems: usize,
    /// New cross-subsystem dependencies the contribution introduces.
    pub new_dependencies: usize,
    /// Why the added complexity is justified (Architecture §24.10).
    pub justification: String,
}

impl ComplexityImpact {
    /// Whether the contribution adds architectural surface at all.
    pub fn adds_complexity(&self) -> bool {
        self.new_subsystems > 0 || self.new_dependencies > 0
    }
}

/// A contribution presented for acceptance.
#[derive(Debug, Clone)]
pub struct ContributionRequest {
    /// Name of the artifact being accepted (tool, action or function).
    pub artifact: String,
    /// Architecture subsystem that owns the artifact.
    pub owner: String,
    /// Who proposes the artifact — the caller that will use it.
    pub caller: String,
    /// What problem the artifact solves (Architecture §24.10).
    pub purpose: String,
    /// What happens when the artifact fails (Architecture §24.7).
    pub error_behavior: String,
    /// How the artifact is validated (Architecture §24.7).
    pub validation_path: String,
    /// The architectural complexity the artifact introduces (Architecture §24.10).
    pub complexity: ComplexityImpact,
    /// Parameters for the twelve design principle checks.
    pub params: ValidationParams,
    /// The contributor type proposing this artifact (Architecture §24.20).
    pub contributor_kind: Option<ContributorType>,
}

/// Outcome of an acceptance check.
#[derive(Debug, Clone)]
pub enum AcceptanceDecision {
    /// The contribution satisfies the architecture rules.
    Accept {
        /// Number of principle checks that were evaluated.
        checks: usize,
        /// The §24.8 trace produced for the accepted contribution.
        trace: String,
    },
    /// The contribution violates one or more architecture rules.
    Reject {
        /// One reason per violated rule.
        reasons: Vec<String>,
        /// The §24.8 trace produced for the rejected contribution.
        trace: String,
    },
}

impl AcceptanceDecision {
    /// Whether the contribution was accepted.
    pub fn is_accepted(&self) -> bool {
        matches!(self, Self::Accept { .. })
    }

    /// The reasons the contribution was rejected (empty when accepted).
    pub fn reasons(&self) -> &[String] {
        match self {
            Self::Accept { .. } => &[],
            Self::Reject { reasons, .. } => reasons,
        }
    }

    /// The §24.8 trace recorded for the contribution.
    pub fn trace(&self) -> &str {
        match self {
            Self::Accept { trace, .. } => trace,
            Self::Reject { trace, .. } => trace,
        }
    }

    /// Number of principle checks that were evaluated.
    pub fn checks(&self) -> usize {
        match self {
            Self::Accept { checks, .. } => *checks,
            Self::Reject { .. } => 0,
        }
    }
}

/// The gate that accepts or rejects contributions (Architecture Chapter 24).
#[derive(Debug, Clone)]
pub struct ContributionGate {
    enforcer: PrinciplesEnforcer,
    /// The AI Contributor Operating Agreement (Architecture §24.20).
    agreement: OperatingAgreement,
}

impl Default for ContributionGate {
    fn default() -> Self {
        Self::new()
    }
}

impl ContributionGate {
    /// Create a new acceptance gate.
    pub fn new() -> Self {
        Self {
            enforcer: PrinciplesEnforcer::new(),
            agreement: OperatingAgreement::new(),
        }
    }

    /// Get a reference to the operating agreement for inspection.
    pub fn agreement(&self) -> &OperatingAgreement {
        &self.agreement
    }

    /// Evaluate a contribution against the architecture rules.
    ///
    /// The contribution is accepted only when it belongs to an architecture
    /// subsystem, connects to a caller, states its purpose, justifies any
    /// complexity it adds, verifies the AI Contributor Operating Agreement
    /// (Architecture §24.20), and violates none of the twelve design principles.
    pub fn evaluate(&self, request: &ContributionRequest) -> AcceptanceDecision {
        let mut reasons: Vec<String> = Vec::new();

        // §24.10 "Does it belong?" — the artifact must be owned by an
        // architecture subsystem, and the architecture must recognise the
        // artifact itself: either as a registered action or as a name that
        // references a subsystem.
        if request.owner.trim().is_empty() {
            reasons.push(format!(
                "'{}' is not owned by any architecture subsystem (Architecture §24.10)",
                request.artifact
            ));
        } else if let Err(reason) = enforce_alignment_check(&request.owner) {
            reasons.push(reason);
        }
        if subsystem_for_action(&request.artifact).is_none()
            && !architecture_alignment_check(&request.artifact)
        {
            reasons.push(format!(
                "'{}' is neither a registered architecture action nor a name that \
                 references an architecture subsystem (Architecture §24.10)",
                request.artifact
            ));
        }

        // §24.10 "Does it connect?" — a contribution without a caller is
        // incomplete (§24.7).
        if request.caller.trim().is_empty() {
            reasons.push(format!(
                "'{}' has no caller (Architecture §24.10)",
                request.artifact
            ));
        }

        // §24.10 "Does it improve capability?" — the contribution must state
        // the problem it solves.
        if request.purpose.trim().is_empty() {
            reasons.push(format!(
                "'{}' states no purpose (Architecture §24.10)",
                request.artifact
            ));
        }

        // §24.7 — expected error behaviour and a validation path.
        if request.error_behavior.trim().is_empty() {
            reasons.push(format!(
                "'{}' states no error behavior (Architecture §24.7)",
                request.artifact
            ));
        }
        if request.validation_path.trim().is_empty() {
            reasons.push(format!(
                "'{}' states no validation path (Architecture §24.7)",
                request.artifact
            ));
        }

        // §24.10 "Does it increase complexity? Is the complexity justified?"
        if request.complexity.adds_complexity()
            && request.complexity.justification.trim().is_empty()
        {
            reasons.push(format!(
                "'{}' adds architectural complexity ({} new subsystem(s), {} new \
                 dependency/ies) without a justification (Architecture §24.10)",
                request.artifact,
                request.complexity.new_subsystems,
                request.complexity.new_dependencies,
            ));
        }

        // §24.20 — AI Contributor Operating Agreement: the contribution must
        // be subject to the agreement, and the five principles must be verified
        // before implementation (Architecture §24.19 Future Autonomous Development).
        let contributor = match request.contributor_kind {
            Some(kind) => kind,
            None => ContributorType::AIAssistant,
        };
        let check_result = self
            .agreement
            .check_contributor(&contributor, &request.artifact);
        if let (true, Err(reason)) = (
            self.agreement.enforcement_enabled && self.agreement.applies_to(&contributor),
            check_result,
        ) {
            reasons.push(reason);
        }

        // §2.1-2.15 — the twelve design principles.
        let results = self.enforcer.validate_action(&request.params);
        let checks = results.len();
        let principles_satisfied = PrinciplesEnforcer::all_satisfied(&results);
        for result in &results {
            if let EnforcementResult::Violated { principle, reason } = result {
                reasons.push(format!("{}: {}", principle.label(), reason));
            }
        }

        // §24.8 — trace before changing.
        let trace = enforce_trace_protocol(&request.artifact);

        if reasons.is_empty() && principles_satisfied {
            AcceptanceDecision::Accept { checks, trace }
        } else {
            AcceptanceDecision::Reject { reasons, trace }
        }
    }
}
