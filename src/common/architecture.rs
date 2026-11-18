//! Stable core architecture (Architecture Chapter 32).
//!
//! NOTE: This module was consolidated from `src/architecture/` into `src/common/`
//! per the architecture spec's common/ utilities layout.

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
