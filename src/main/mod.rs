// src/main.rs

pub mod api;
pub mod bootstrap;
pub mod common;
pub mod config;
pub mod context;
pub mod conversation;
pub mod database;
pub mod deployment;
pub mod execution;
pub mod experience;
pub mod knowledge;
pub mod learning;
pub mod memory;
pub use common::personality;
pub mod models;
pub mod planning;
pub use common::principles;
pub mod prompt;
pub mod research;
pub mod retrieval;
pub mod scheduler;
pub mod storage;
pub mod workers;

pub mod wire;

use api::bridge::logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging FIRST so that all subsequent output (including
    // console attachment messages) flows through the tracing subscriber.
    // This is critical for the `robot diagnose` CLI: the test harness
    // captures stdout+stderr and checks for expected markers; stray
    // eprintln! output before the subscriber is configured pollutes
    // that stream.
    init_logging();

    // Wire all subsystems to eliminate dead-code warnings.
    // Per AGENTS.md (0 dead code) and Architecture Chapters 05, 08, 10-13, 20.
    wire::execution::wire_execution();

    let wire::data_contracts::DataContractsEarly {
        contract_version,
        meta_conf,
        mut experience,
    } = wire::data_contracts::wire_data_contracts_early();

    wire::planner::wire_planner();

    experience = wire::experience::wire_experience(experience);

    let memory_record = wire::data_contracts::wire_data_contracts_late(contract_version, meta_conf);

    wire::learning::wire_learning();

    wire::memory_research::wire_memory_research(&memory_record).await;

    wire::knowledge::wire_knowledge();

    wire::search_bridge::wire_search_bridge();

    context_engine::wire_context_engine();

    wire::cli::run_cli().await?;

    Ok(())
}
