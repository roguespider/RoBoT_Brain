/// Data contracts for the cognitive architecture (v0.0.2 / v0.0.2.1).
/// Defines the exact signatures and constraints for all subsystems.
pub use version::CONTRACT_VERSION;

pub mod context_packet;
pub mod decision;
pub mod execution_result;
pub mod experience_record;
pub mod learning_update;
pub mod memory_record;
pub mod metadata;
pub mod observation;
pub mod plan_contract;
pub mod reflection;
pub mod version;
