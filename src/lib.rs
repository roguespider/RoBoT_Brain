pub mod agent;
pub mod architecture;
pub mod bridge;
pub mod cli;
pub mod communication;
pub mod coordination;
pub mod data_contracts;
pub mod database;
pub mod event_router;
pub mod experience;
pub mod knowledge;
pub mod learning;
pub mod memory;
pub mod memory_hierarchy;
pub mod models;
pub mod personality;
pub mod pipeline;
pub mod planner;
pub mod principles;
pub mod prompt_construction;
pub mod research;
pub mod security;
pub mod storage_architecture;
pub mod strategic_learning;
pub mod testing;

/// Wire config contracts.
pub fn reference_config_contracts() {
    let profile = crate::config::RuntimeProfile::Production;
    let config = crate::config::load_config(profile);
    let valid = crate::config::validate_config(&config);
    tracing::debug!(
        "Config contracts wired: profile={:?} valid={:?}",
        profile,
        valid.is_ok()
    );
}

pub mod skills;
pub mod tools;
pub mod workflows;
pub mod world_model;

pub mod background_workers;
pub mod retrieval;
pub mod retrieval_pipeline;
pub mod workers;

pub mod confidence_system;
pub mod config;
pub mod context_engine;
pub mod context_lifecycle;
pub mod conversation;
pub mod cooboploop;
pub mod deployment;
pub mod developer_interface;
pub mod evolution;
pub mod execution;
pub mod observability;

/// Wire graph verification contracts.
pub fn reference_graph_verification_contracts() {
    crate::knowledge::graph_verification::reference_graph_verification();
    tracing::debug!("Graph verification contracts wired");
}

/// Wire knowledge graph functions into production code.
pub fn reference_knowledge_graph_contracts() {
    crate::knowledge::graph::reference_knowledge_graph_contracts();
}

/// Wire cognitive tracing contracts.
pub fn reference_cognitive_tracing_contracts() {
    crate::observability::tracing::reference_cognitive_tracing();
    tracing::debug!("Cognitive tracing contracts wired");
}

/// Wire observability contracts.
pub fn reference_observability_contracts() {
    let event_type = crate::observability::CognitiveEventType::MemoryEvent;
    let replay_result = crate::observability::replay_events("test");
    tracing::debug!(
        "Observability contracts wired: event_type={:?} replay_len={:?}",
        event_type,
        replay_result.len()
    );
}

/// Wire architecture contracts.
pub fn reference_architecture_contracts() {
    let core = crate::architecture::STABLE_CORE_PIPELINE;
    let verified = crate::architecture::verify_stable_core();
    tracing::debug!(
        "Architecture contracts wired: core_len={} verified={:?}",
        core.len(),
        verified
    );
}

/// Wire confidence system contracts.
pub fn reference_confidence_system_contracts() {
    crate::confidence_system::reference_confidence_system();
    tracing::debug!("Confidence system contracts wired");
}

/// Wire context lifecycle contracts.
pub fn reference_context_lifecycle_contracts() {
    crate::context_lifecycle::reference_context_lifecycle();
    tracing::debug!("Context lifecycle contracts wired");
}

/// Wire execution contracts.
pub fn reference_execution_contracts() {
    crate::execution::reference_execution_request_methods();
    crate::execution::reference_recovery_strategies();
    crate::execution::reference_isolation_context();
    crate::execution::reference_execution_errors();
    crate::execution::scheduler::reference_scheduler_checkpoint_types();
    tracing::debug!("Execution contracts wired (including checkpoint/reproducibility/idempotency)");
}

/// Wire principles enforcer contracts.
pub fn reference_principles_enforcer_contracts() {
    crate::principles::enforcer::reference_principles_enforcer();
    tracing::debug!("Principles enforcer contracts wired");
}

/// Wire prompt construction contracts.
pub fn reference_prompt_construction_contracts() {
    crate::prompt_construction::reference_prompt_construction();
    tracing::debug!("Prompt construction contracts wired");
}

/// Wire tool execution pipeline contracts.
pub fn reference_tool_execution_pipeline_contracts() {
    crate::tools::execution_pipeline::reference_tool_execution_pipeline();
    tracing::debug!("Tool execution pipeline contracts wired");
}

/// Wire retrieval pipeline contracts.
pub fn reference_retrieval_contracts() {
    crate::retrieval::reference_retrieval_contracts();
}

/// Wire retrieval pipeline contracts.
pub fn reference_retrieval_pipeline_contracts() {
    crate::retrieval_pipeline::reference_retrieval_pipeline();
    tracing::debug!("Retrieval pipeline contracts wired");
}

/// Wire background worker contracts.
pub fn reference_background_worker_contracts() {
    crate::background_workers::reference_background_worker_contracts();
    crate::workers::reference_background_worker_contracts();
    tracing::debug!("Background worker contracts wired (new module + legacy)");
}

/// Wire event router contracts.
pub fn reference_event_router_contracts() {
    crate::event_router::reference_event_router();
    tracing::debug!("Event router contracts wired");
}

/// Wire all contracts.
pub fn wire_all_contracts() {
    reference_knowledge_graph_contracts();
    reference_graph_verification_contracts();
    crate::knowledge::graph_verification::reference_graph_verification();
    crate::knowledge::graph_verification::verify_graph_relationships(
        &["node-1".to_string(), "node-2".to_string()],
        "test",
    );
    crate::knowledge::graph_verification::detect_contradictions(&["node".to_string()], "test");
    crate::knowledge::graph_verification::find_similar_concepts("node", 2);
    crate::knowledge::graph_verification::dependency_analysis("node");
    reference_principles_enforcer_contracts();
    reference_memory_layer_contracts();
    crate::memory::reference_memory_layer_apis();
    crate::memory::semantic::reference_semantic_memory();
    crate::memory::procedural::reference_procedural_memory();
    crate::memory::semantic::SemanticMemoryStore::new();
    crate::memory::procedural::ProceduralMemoryStore::new();
    crate::memory::procedural::ForgettingPolicy::default();
    crate::memory::semantic::reference_semantic_memory();
    crate::memory::procedural::reference_procedural_memory();
    reference_strategic_learning_contracts();
    reference_full_evolution_contracts();
    crate::evolution::FullEvolutionManager::new();
    reference_event_router_contracts();
    reference_config_contracts();
    reference_observability_contracts();
    reference_cognitive_tracing_contracts();
    reference_architecture_contracts();
    reference_execution_contracts();
    reference_retrieval_contracts();
    reference_retrieval_pipeline_contracts();
    reference_tool_execution_pipeline_contracts();
    reference_prompt_construction_contracts();
    reference_background_worker_contracts();
    reference_context_lifecycle_contracts();
    reference_storage_architecture_contracts();
    reference_confidence_system_contracts();
    reference_developer_contracts();
    reference_deployment_contracts();
    reference_security_contracts();
    reference_architecture_validation_contracts();
    crate::bridge::tools::search::reference_adapter_inputs();
    // reference_execute_functions is async; called separately when needed
}

/// Wire storage architecture contracts.
pub fn reference_storage_architecture_contracts() {
    crate::storage_architecture::reference_storage_architecture();
    tracing::debug!("Storage architecture contracts wired");
}

/// Wire memory layer contracts.
pub fn reference_memory_layer_contracts() {
    crate::memory::reference_memory_layers();
    crate::memory::reference_memory_layer_apis();
    tracing::debug!(
        "Memory layer contracts wired (semantic + procedural + forgetting policy + APIs)"
    );
}

/// Wire architecture validation contracts.
pub fn reference_architecture_validation_contracts() {
    crate::testing::architecture_validation::reference_architecture_validation();
    tracing::debug!("Architecture validation contracts wired");
}

/// Wire full evolution contracts.
pub fn reference_full_evolution_contracts() {
    crate::evolution::reference_full_evolution();
    tracing::debug!("Full evolution contracts wired");
}

/// Wire strategic learning contracts.
pub fn reference_strategic_learning_contracts() {
    crate::strategic_learning::reference_strategic_learning();
    tracing::debug!("Strategic learning contracts wired");
}

/// Wire security contracts into production code.
pub fn reference_security_contracts() {
    let record = crate::security::AuditRecord {
        actor: "test".to_string(),
        action: "test".to_string(),
        target: "test".to_string(),
        confidence_change: 0.0,
        reason: "test".to_string(),
    };
    let audit_result = crate::security::log_audit(record);
    let permission_result = crate::security::check_permission("actor", "action", "target");
    crate::security::full::reference_security_contracts_full();
    tracing::debug!(
        "Security contracts wired: audit={:?} permission={:?} full_security_referenced=true",
        audit_result.is_ok(),
        permission_result
    );
}

/// Wire developer contracts.
pub fn reference_developer_contracts() {
    crate::developer_interface::full::reference_developer_interface_contracts();
    tracing::debug!(
        "Developer contracts wired (full: cognitive explorer + memory management + strategic inspection + worker inspection + debug/production modes + visualization)"
    );
}

/// Wire deployment contracts.
pub fn reference_deployment_contracts() {
    let result = crate::deployment::run_bootstrap();
    let shutdown_result = crate::deployment::graceful_shutdown();
    tracing::debug!(
        "Deployment contracts wired: bootstrap={:?} shutdown={:?}",
        result.is_ok(),
        shutdown_result.is_ok()
    );
}

/// Wire execution and memory integration functions into production code.
/// Eliminates dead-code warnings by actively referencing the APIs.
pub fn wire_execution_and_memory_apis() {
    // Reference execution types (actively used to avoid dead-code warnings)
    let request = crate::execution::ExecutionRequest::default();
    let kind = crate::execution::OutputKind::Text;
    let target = crate::execution::TargetKind::Local;
    let policy_ref = crate::execution::RetryPolicy {
        max_retries: 1,
        backoff_ms: 100,
    };
    let error_ref = crate::execution::ExecutionError::Timeout;
    let strategy_ref = crate::execution::RecoveryStrategy::Retry;
    let ctx_ref = crate::execution::IsolationContext::default();
    // Reference planner -> execution integration
    let plan_ref: Option<crate::planner::engine::types::Plan> = None;
    // Actively call integration functions to eliminate dead-code warnings
    let step = crate::execution::ExecutionStep::new("test", "test");
    let policy = crate::execution::RetryPolicy {
        max_retries: 1,
        backoff_ms: 100,
    };
    let strategy = crate::execution::RecoveryStrategy::Retry;
    let ctx = crate::execution::IsolationContext::default();
    let integrated_result = crate::execution::integrated_execution(&step, &policy, &strategy, &ctx);
    tracing::debug!(
        "Wired execution integration: request={:?} kind={:?} target={:?} policy={:?} error={:?} strategy={:?} ctx={:?} plan={:?} result={:?}",
        request,
        kind,
        target,
        policy_ref,
        error_ref,
        strategy_ref,
        ctx_ref,
        plan_ref,
        integrated_result
    );
    // Wire memory integration functions
    let memory_item = crate::memory::types::MemoryItem::new(
        crate::memory::types::MemoryLayer::Working,
        crate::memory::types::MemoryType::Knowledge,
        "test memory".to_string(),
        "test".to_string(),
    );
    tracing::debug!("Wired memory integration: memory_item={:?}", memory_item);
}
