//! Bootstrap — Per Architecture Appendix A.
//!
//! Bootstrap subsystem: system startup, shutdown, dependency validation,
//! and health checking for the RoBoT cognitive architecture.

pub mod bootstrap_manager;
pub mod dependency_graph;
pub mod health_check;
pub mod shutdown;
pub mod startup;

// Re-export key types from submodules
pub use bootstrap_manager::BootstrapManager;
pub use dependency_graph::validate_dependencies;
pub use health_check::{HealthStatus, health_check};
pub use shutdown::shutdown;
pub use startup::startup;
