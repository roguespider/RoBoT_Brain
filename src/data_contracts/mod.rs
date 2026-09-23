/// Data contracts for the cognitive architecture (v0.0.2 / v0.0.2.1).
/// Defines the exact signatures and constraints for all subsystems.
pub use version::CONTRACT_VERSION;

pub mod action_request;
pub mod confidence;
pub mod context_packet;
pub mod contract_validator;
pub mod decision;
pub mod event_contract;
pub mod execution_result;
pub mod experience_record;
pub mod goal;
pub mod learning_update;
pub mod memory_record;
pub mod metadata;
pub mod observation;
pub mod plan_contract;
pub mod query_contract;
pub mod reflection;
pub mod result_contract;
pub mod state_contract;
pub mod version;
