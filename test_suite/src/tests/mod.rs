


//! Test modules
//!
//! Split from the monolithic tests.rs file for better maintainability.

pub mod memory;
pub mod experience;
pub mod knowledge;
pub mod workflow;
pub mod planner;
pub mod hypothesis;
pub mod reflection;
pub mod search;
pub mod ingestor;
pub mod agent;
pub mod error_handling;
pub mod mcp_workflow;
pub mod cli_tools;
pub mod rmcp;
pub mod acp;
pub mod agent_simulation;
pub mod queue_durability;
pub mod exploration_finding;
pub mod observations;
pub mod exploration_attempt;
pub mod exploration_hypothesis;
pub mod knowledge_store;

pub use memory::run_memory_tests;
pub use experience::run_experience_tests;
pub use knowledge::run_knowledge_tests;
pub use workflow::run_workflow_tests;
pub use planner::run_planner_tests;
pub use hypothesis::run_hypothesis_tests;
pub use reflection::run_reflection_tests;
pub use search::run_search_tests;
pub use ingestor::run_ingestor_tests;
pub use agent::run_agent_tests;
pub use error_handling::run_error_handling_tests;
pub use mcp_workflow::run_mcp_workflow_tests;
pub use rmcp::run_rmcp_tests;
pub use acp::run_acp_tests;
pub use agent_simulation::run_agent_simulation_tests;
