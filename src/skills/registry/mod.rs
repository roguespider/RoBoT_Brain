// src/skills/registry/mod.rs
//! Skill registry for managing available skills
//!
//! Per Architecture §2.9, §12, §15:
//! Skills represent reusable capabilities discovered through experience.
//! A skill is not simply stored code.
//! Skills allow RoBoT to improve through repetition.
//!
//! Skills are different from knowledge:
//! - Knowledge: "I know SQL." (information)
//! - Skill: "I can optimize a query." (capability)

pub mod context;
pub mod executor;
pub mod metrics;
pub mod result;
pub mod skill;
pub mod store;
pub mod types;

// Re-export types for convenience
pub use context::ExecutionContext;
pub use executor::SkillExecutor;
pub use skill::Skill;
pub use store::SkillRegistry;
pub use types::{SkillCategory, SkillMetadata, SkillSource};
