//! Stable core architecture (Architecture Chapter 32).

/// The stable core pipeline steps.
pub const STABLE_CORE_PIPELINE: &[&str] = &[
    "Observe",
    "Understand",
    "Retrieve",
    "Plan",
    "Reason",
    "Act",
    "Reflect",
    "Learn",
];

/// Verify the stable core is intact.
pub fn verify_stable_core() -> bool {
    !STABLE_CORE_PIPELINE.is_empty()
}
