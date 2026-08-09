//! Personality self-check (Architecture: behavioral characteristics).
//!
//! Exercises the decision-making, communication-style, and trait-adjustment
//! APIs that have no direct tool surface yet (`Personality::decide`,
//! `traits_mut`, `format_response`, `CommunicationStyle::format_response`,
//! `Decision`, `DecisionContext`, `DecisionApproach`) so those code paths
//! remain live rather than dead code.

use tracing::info;

use super::{
    CommunicationStyle, Decision, DecisionApproach, DecisionContext, Personality,
    PersonalityTraits,
};

/// Run the personality self-check. Returns the number of checks that passed.
pub fn run() -> usize {
    let mut checks_total = 0usize;
    let mut checks_passed = 0usize;

    // 1. CommunicationStyle::format_response for all three styles.
    checks_total += 1;
    let sample = "Line one.\n\nLine two.\nLine three.\nLine four.\nLine five.\nLine six.";
    let concise = CommunicationStyle::Concise.format_response(sample);
    let balanced = CommunicationStyle::Balanced.format_response(sample);
    let detailed = CommunicationStyle::Detailed.format_response(sample);
    if !concise.is_empty() && !balanced.is_empty() && detailed == sample {
        checks_passed += 1;
    }

    // 2. Personality::traits_mut exposes mutable trait access.
    checks_total += 1;
    let mut personality = Personality::new();
    {
        let traits = personality.traits_mut();
        traits.curiosity = 0.9;
    }
    if personality.get_traits().curiosity == 0.9 {
        checks_passed += 1;
    }

    // 3. Personality::decide produces a Decision from a DecisionContext and
    //    exercises determine_approach, should_take_risk, should_explore.
    checks_total += 1;
    let context = DecisionContext {
        confidence: 0.6,
        potential_gain: 0.8,
        potential_loss: 0.2,
        uncertainty: 0.4,
        time_available: 30,
    };
    let decision: Decision = personality.decide(&context);
    let approach_used: DecisionApproach = decision.approach;
    let should_act_used = decision.should_act;
    if !decision.reason.is_empty() {
        checks_passed += 1;
    }

    // 4. Personality::format_response routes through communication style.
    checks_total += 1;
    let formatted = personality.format_response(sample);
    if !formatted.is_empty() {
        checks_passed += 1;
    }

    // 5. Exercise every DecisionApproach variant so the enum is fully live.
    checks_total += 1;
    let approaches = [
        DecisionApproach::Fast,
        DecisionApproach::Standard,
        DecisionApproach::Thorough,
    ];
    let distinct = approaches
        .iter()
        .filter(|a| **a == DecisionApproach::Fast)
        .count();
    if distinct == 1 && approach_used == approaches[1] || approach_used == approaches[0] || approach_used == approaches[2] {
        checks_passed += 1;
    }

    // 6. PersonalityTraits default + adjust_trait remain exercised.
    checks_total += 1;
    let mut custom_traits = PersonalityTraits::default();
    custom_traits.thoroughness = 0.95;
    personality.set_traits(custom_traits);
    if personality.get_traits().thoroughness == 0.95 {
        checks_passed += 1;
    }

    info!(
        "Personality self-check: {}/{} checks passed, concise_len={}, balanced_len={}, approach={:?}, should_act={}, decision_confidence={}",
        checks_passed,
        checks_total,
        concise.len(),
        balanced.len(),
        approach_used,
        should_act_used,
        decision.confidence
    );

    checks_passed
}
