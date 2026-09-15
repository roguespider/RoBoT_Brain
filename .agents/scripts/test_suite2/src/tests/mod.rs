//! Test modules for the test suite.
//!
//! Each submodule contains integration tests for a specific subsystem.

pub mod agent;
pub mod audio_transcriber;
pub mod cli_tools;
pub mod concurrent_store;
pub mod context_pressure;
pub mod cooboploop_queue;
pub mod cooboploop_t10_learning;
pub mod cooboploop_t13_strategic;
pub mod cooboploop_t14_cycle;
pub mod cooboploop_t15_autonomy;
pub mod cooboploop_t3_eval;
pub mod cooboploop_t5_cycle;
pub mod data_contracts_adapters;
pub mod data_contracts_context_packet;
pub mod data_contracts_decision;
pub mod data_contracts_execution_result;
pub mod data_contracts_experience_record;
pub mod data_contracts_learning_update;
pub mod data_contracts_memory_record;
pub mod data_contracts_observation;
pub mod data_contracts_plan;
pub mod data_contracts_reflection;
pub mod embeddings;
pub mod error_handling;
pub mod experience;
pub mod exploration_attempt;
pub mod exploration_finding;
pub mod exploration_hypothesis;
pub mod flow_auto_memory_retrieval;
pub mod flow_basic_cognition;
pub mod flow_cross_session_memory;
pub mod flow_experience_capture;
pub mod flow_recovery;
pub mod flow_restart_recovery;
pub mod fresh_start;
pub mod hypothesis;
pub mod knowledge;
pub mod knowledge_query;
pub mod knowledge_store;
pub mod memory;
pub mod memory_failure_isolation;
// Note: memory_permanent_tests.rs references crate::memory which is robot_brain's source.
// test_suite tests robot_brain via MCP, not by importing source. Skipping this module.
// pub mod memory_permanent_tests;
pub mod memory_retrieval;
pub mod memory_types;
pub mod observations;
pub mod personality;
pub mod planner;
pub mod execution_recovery;
pub mod experience_reputation;
pub mod planner_dag;
pub mod memory_promotion;
// T2-130+: conversation, pipeline, and context_engine modules not yet implemented
// pub mod conversation_lifecycle;
// pub mod data_contract_chain;
// pub mod context_assembly_pipeline;
// pub mod pipeline_lifecycle;
pub mod queue_durability;
pub mod reflection;
pub mod research_engine_flow;
pub mod search;
pub mod semantic_chunker;
pub mod session_smoke;
pub mod startup_durability;
pub mod workflow;
pub mod workflow_enforcement_tests;

// Submodule directories (each contains a mod.rs)
pub mod acp;
pub mod agent_simulation;
pub mod ingestor;
pub mod mcp_workflow;
pub mod rmcp;
