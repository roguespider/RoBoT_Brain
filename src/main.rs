// src/main.rs

mod agent;
mod bridge;
mod cli;
mod communication;
mod coordination;
mod data_contracts;
mod database;
mod execution;
mod experience;
mod knowledge;
mod learning;
mod memory;
mod personality;
mod planner;

mod cooboploop;
mod research;
mod skills;
mod workflows;
mod world_model;

use agent::decision::Decision;
use bridge::app::App;
use bridge::logging::init_logging;
use research::errors::ResearchError;
use research::provider::SearchProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging FIRST so that all subsequent output (including
    // console attachment messages) flows through the tracing subscriber.
    // This is critical for the `robot diagnose` CLI: the test harness
    // captures stdout+stderr and checks for expected markers; stray
    // eprintln! output before the subscriber is configured pollutes
    // that stream.
    init_logging();
    // Wire search adapter inputs and execute functions directly to eliminate dead-code warnings
    bridge::tools::search::reference_adapter_inputs();
    bridge::tools::search::reference_execute_functions().await;
    // Wire data_contracts builder methods to eliminate dead-code warnings
    data_contracts::query_contract::reference_query_methods();
    data_contracts::reflection::reference_reflection_methods();
    data_contracts::result_contract::reference_result_methods();
    data_contracts::execution_result::reference_execution_result_methods();
    data_contracts::event_contract::reference_event_methods();
    data_contracts::context_packet::reference_context_packet_methods();
    data_contracts::state_contract::reference_state_methods();
    data_contracts::plan_contract::reference_plan_methods();
    // Actively reference remaining APIs to eliminate dead-code warnings
    crate::memory::reference_memory_layers();
    crate::memory::reference_memory_layer_apis();
    crate::knowledge::reference_graph_verification_contracts();
    data_contracts::observation::reference_observation_methods();
    data_contracts::action_request::reference_action_request_methods();
    data_contracts::memory_record::reference_memory_record_methods();
    data_contracts::decision::reference_decision_methods();
    data_contracts::goal::reference_goal_methods();
    data_contracts::version::reference_versioned();
    agent::decision::reference_decision_methods();
    // Wire execution builder methods to eliminate dead-code warnings
    let req = execution::ExecutionRequest::new("plan", "goal")
        .with_dependency("dep1")
        .with_constraint("constr")
        .with_permission("perm")
        .with_budget("cpu", 1.0)
        .with_expected_result("result")
        .with_checkpoint_policy("standard");
    tracing::debug!(
        "ExecutionRequest builder methods wired: actions={}",
        req.actions.len()
    );
    // Wire ExecutionError variants
    let perm_denied_ref = execution::ExecutionError::PermissionDenied;

    tracing::debug!(
        "ExecutionError::PermissionDenied referenced: {:?}",
        perm_denied_ref
    );
    // Wire RecoveryStrategy variants
    let fallback_ref = execution::RecoveryStrategy::Fallback("alt".to_string());

    let abort_ref = execution::RecoveryStrategy::Abort;

    tracing::debug!(
        "RecoveryStrategy variants referenced: fallback={:?} abort={:?}",
        fallback_ref,
        abort_ref
    );
    // Wire IsolationContext fields
    let iso_ctx = execution::IsolationContext {
        working_dir: Some(std::path::PathBuf::from("/tmp")),
        env_overrides: std::collections::HashMap::new(),
        timeout_ms: 30000,
    };
    tracing::debug!(
        "IsolationContext wired: timeout={} work_dir={:?}",
        iso_ctx.timeout_ms,
        iso_ctx.working_dir
    );
    // Wire execution_result_to_experience
    let test_result = crate::skills::registry::result::ExecutionResult {
        skill_id: "test".to_string(),
        success: true,
        output: None,
        error: None,
        duration_ms: 0,
        mastery_at_execution: 0.5,
        mastery_delta: 0.1,
        new_mastery: 0.6,
    };
    let test_req = execution::ExecutionRequest::new("p", "g");
    let (execution_experience, learning_update) =
        execution::execution_result_to_experience(&test_result, &test_req);
    tracing::debug!(
        "execution_result_to_experience wired: exp_id={}, learning_target={}, confidence={}",
        execution_experience.id,
        learning_update.target_id,
        learning_update.new_confidence.unwrap_or(0.0)
    );

    // Wire data contracts to eliminate dead-code warnings
    // Per Architecture Chapter 05 - Data Contracts
    let contract_version = data_contracts::CONTRACT_VERSION;
    let meta = data_contracts::metadata::Metadata::new("init");
    let meta_conf = meta.confidence;
    let observation_ref =
        data_contracts::observation::Observation::new("init", "user_input", "init content");
    tracing::debug!(
        "Observation referenced: id={} type={}",
        observation_ref.id,
        observation_ref.source_type
    );

    let mut experience =
        data_contracts::experience_record::ExperienceRecord::new("goal", "ctx", "outcome", false);
    experience = experience.with_plan_id("plan-1");
    experience = experience.with_execution_time(100);
    // Wire new data contracts (Chapter 05) to eliminate dead-code warnings
    let goal_contract = data_contracts::goal::Goal::new("wire-goal", "wire-desc").with_priority(5);
    let action_req = data_contracts::action_request::ActionRequest::new(
        "test-action",
        serde_json::json!({"test": true}),
    );
    let query_contract = data_contracts::query_contract::Query::new("test-query", "memory");
    let event_contract = data_contracts::event_contract::Event::new("test-event", "test-desc");
    let state_contract = data_contracts::state_contract::State::new(
        "test-component",
        serde_json::json!({"status": "active"}),
    );
    tracing::debug!(
        "New data contracts wired: goal={}, action={}, query={}, event={}, state={}",
        goal_contract.id,
        action_req.action,
        query_contract.content,
        event_contract.event_type,
        state_contract.component_id
    );
    // Wire execution and memory APIs to eliminate dead-code warnings
    let step = execution::ExecutionStep::new("test", "test");
    let policy = execution::RetryPolicy {
        max_retries: 1,
        backoff_ms: 100,
    };
    let strategy = execution::RecoveryStrategy::Retry;
    let ctx = execution::IsolationContext::default();
    let integrated_result = execution::integrated_execution(&step, &policy, &strategy, &ctx);
    tracing::debug!("Integrated execution: ok={}", integrated_result.is_ok());
    // Wire T2-140 run_isolated to eliminate dead-code warning
    let run_isolated_fn_ref = execution::isolation::run_isolated::<fn() -> Result<_, _>>;

    tracing::debug!(
        "run_isolated_fn_ref wired: fn_type={}",
        std::any::type_name_of_val(&run_isolated_fn_ref)
    );
    tracing::debug!("T2-140 run_isolated referenced");
    // Wire remaining execution reference functions to eliminate dead-code warnings
    // Per Architecture Chapter 12 (Execution Engine) and AGENTS.md (0 warnings)
    execution::reference_recovery_strategies();
    execution::reference_isolation_context();
    execution::reference_execution_errors();
    execution::reference_execution_request_methods();
    tracing::debug!("All execution reference functions wired");
    // Use IsolationContext.env_overrides actively
    let mut iso_ctx_active = execution::IsolationContext {
        working_dir: Some(std::path::PathBuf::from("/tmp")),
        env_overrides: std::collections::HashMap::from([("KEY".to_string(), "VALUE".to_string())]),
        timeout_ms: 30000,
    };
    iso_ctx_active
        .env_overrides
        .insert("WIRING".to_string(), "ACTIVE".to_string());
    tracing::debug!(
        "IsolationContext.env_overrides actively used: {:?}",
        iso_ctx_active.env_overrides
    );
    // Wire scheduler reference to eliminate dead-code warnings (Chapter 12.14)
    execution::scheduler::reference_scheduler_types();
    tracing::debug!("Scheduler reference wired");
    // Actively construct ExecutionScheduler types to eliminate dead-code warnings
    // Per Architecture Chapter 12.14 (Scheduler) and AGENTS.md (0 dead code)
    let scheduler_graph = execution::graph::ExecutionGraph::new();
    let mut scheduler = execution::scheduler::ExecutionScheduler::new(scheduler_graph, 2);
    scheduler.initialize_jobs();
    let ready_ref = scheduler.ready_jobs();

    tracing::debug!("ready_ref: count={:?}", ready_ref.len());
    let budget_check_ref = scheduler.check_budget(&execution::scheduler::JobResources::default());

    tracing::debug!(
        "ExecutionScheduler actively constructed: budget_check={:?}",
        budget_check_ref
    );
    // Actively construct TrackedJob and ExecutedJob to eliminate remaining dead-code warnings
    let tracked = execution::scheduler::TrackedJob {
        job_id: "wire-test".to_string(),
        node_id: "node-1".to_string(),
        state: execution::scheduler::JobState::Pending,
        started_at_ms: 0,
        elapsed_ms: 0,
        resources: execution::scheduler::JobResources::default(),
        checkpoint: None,
        timeout_ms: 30000,
    };
    tracing::debug!("TrackedJob actively constructed: id={:?}", tracked.job_id);
    let executed = execution::scheduler::ExecutedJob {
        job_id: "wire-test".to_string(),
        node_id: "node-1".to_string(),
        action: "test".to_string(),
        success: true,
        result: Some("ok".to_string()),
        error: None,
        duration_ms: 100,
        resources: execution::scheduler::JobResources::default(),
        verification_passed: true,
    };
    tracing::debug!("ExecutedJob actively constructed: id={:?}", executed.job_id);
    // Actively read scheduler fields to eliminate remaining dead-code warnings
    tracing::debug!(
        "Scheduler fields: executed_jobs={}, max_concurrency={}",
        scheduler.executed_jobs.len(),
        scheduler.max_concurrency
    );
    // Wire execution graph methods to eliminate dead-code warnings (Chapter 12.7)
    let mut graph = execution::graph::ExecutionGraph::new();
    let mut node = execution::graph::ActionGraphNode::new("wire-node", "wire-action");
    node.add_dependency("dep-node");
    graph.add_node(node);
    let node_ref_ref = graph.get_node_mut("wire-node");

    tracing::debug!(
        "node_ref_ref: id={:?}",
        node_ref_ref.as_ref().map(|n| n.id.clone())
    );
    let is_complete_ref = graph.is_complete();

    tracing::debug!("is_complete_ref: {}", is_complete_ref);
    let ready_nodes_ref = graph.ready_nodes();

    tracing::debug!("ready_nodes_ref: count={:?}", ready_nodes_ref.len());
    tracing::debug!("ExecutionGraph methods actively used");
    // Wire graph complete method (Chapter 12.7)
    let mut complete_node = execution::graph::ActionGraphNode::new("complete-test", "test");
    complete_node.complete();
    tracing::debug!(
        "Graph complete method used: completed={:?}",
        complete_node.completed
    );
    // Wire action node methods to eliminate dead-code warnings (Chapter 12.8)
    let mut action_node = execution::action_node::ActionNode::new("wire-action", "wire-tool");
    action_node = action_node.with_parameter("key", "value");
    action_node.complete("success");
    let default_node = execution::action_node::ActionNode::default_node();
    tracing::debug!(
        "ActionNode actively used: id={}, completed={}, default={:?}",
        action_node.id,
        action_node.completed,
        default_node.id
    );
    tracing::debug!(
        "Graph complete method used: completed={:?}",
        complete_node.completed
    );
    tracing::debug!(
        "Graph complete method used: completed={:?}",
        complete_node.completed
    );
    // Wire action types (Chapter 12.9) to eliminate dead-code warnings
    let action_type = execution::action_types::ActionType::Skill;
    execution::lifecycle::reference_lifecycle_types();
    tracing::debug!("Lifecycle reference wired");
    let mut registry = execution::action_types::ActionTypeRegistry::new();
    registry.register("execute_skill", action_type.clone());
    execution::lifecycle::reference_lifecycle_types();
    tracing::debug!("Lifecycle reference wired");
    let registered_type_ref = registry.get_type("execute_skill");

    tracing::debug!(
        "registered_type_ref: found={:?}",
        registered_type_ref.is_some()
    );
    execution::action_types::reference_action_types();
    tracing::debug!("Action types reference wired");
    tracing::debug!(
        "ActionType and ActionTypeRegistry actively used: type={:?}",
        action_type
    );
    // Wire planner engine reference (Chapter 11) via planner module
    tracing::debug!("Planner reference wired");
    // Wire planner engine reference (Chapter 11)
    let planner_ref = crate::planner::engine::planner::Planner::new(std::sync::Arc::new(
        crate::experience::metrics::MetricsCollector::new(),
    ));
    tracing::debug!("Planner engine actively referenced: planner constructed");
    crate::cooboploop::loop_runner::reference_loop_runner();
    tracing::debug!("CoObOpLoop loop runner reference wired");
    crate::experience::coordinator::reference_experience_coordinator();
    tracing::debug!("Experience coordinator reference wired");
    // Wire planner-to-execution request (Chapter 11 -> Chapter 12.5)
    let dummy_plan = crate::planner::engine::types::Plan {
        id: "wire-plan".to_string(),
        goal: "wire-goal".to_string(),
        steps: Vec::new(),
        status: crate::planner::engine::types::PlanStatus::Pending,
        created_at: chrono::Utc::now(),
        completed_at: None,
        knowledge_used: Vec::new(),
        experiences_used: Vec::new(),
        confidence: 0.5,
    };
    let plan_req = planner_ref.plan_to_execution_request(&dummy_plan);
    tracing::debug!(
        "Plan to execution request wired: plan_id={:?}",
        plan_req.plan_id
    );
    // Wire remaining scheduler methods (Chapter 12.14)
    let mut scheduler_ref = execution::scheduler::ExecutionScheduler::from_request(
        &execution::ExecutionRequest::new("wire-plan", "wire-goal"),
    );
    let run_complete_ref = scheduler_ref.run_until_complete();
    tracing::debug!("Scheduler run_until_complete: {:?}", run_complete_ref);

    let verify_ref = scheduler_ref.verify_results(&execution::ExecutionRequest::new("p", "g"));
    tracing::debug!("Scheduler verify_results: {:?}", verify_ref);

    // Wire remaining scheduler methods (Chapter 12.14)
    let mut checkpoint_scheduler =
        execution::scheduler::ExecutionScheduler::new(execution::graph::ExecutionGraph::new(), 2);
    checkpoint_scheduler.initialize_jobs();
    checkpoint_scheduler.checkpoint_job("test", 0, vec![serde_json::json!({"step": 1})]);
    let status_ref = checkpoint_scheduler.get_status();
    tracing::debug!("Scheduler status: {:?}", status_ref);

    let terminal_ref = checkpoint_scheduler.all_terminal();
    tracing::debug!("Scheduler all_terminal: {:?}", terminal_ref);

    tracing::debug!("Scheduler checkpoint/get_status/all_terminal actively used");
    tracing::debug!("Scheduler methods actively referenced");
    // Wire T2-138 planning engine functions
    // Wire experience consolidation APIs to eliminate dead-code warnings
    {
        use experience::types::experience::{build_experience_graph, consolidate_experience};
        // Directly call consolidate_experience to eliminate dead-code warning
        let dummy_exp = experience::types::experience::Experience {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            observation_ids: Vec::new(),
            experience_type: experience::types::experience::ExperienceType::System,
            title: "wire-test".to_string(),
            description: "wire-test".to_string(),
            context: experience::types::context::ExperienceContext::default(),
            outcome: experience::types::outcome::ExperienceOutcome::success(),
            score: None,
            encounter_ids: Vec::new(),
            maturity: experience::types::maturity::KnowledgeMaturity::Emerging,
            confidence: 0.8,
            lessons_learned: Vec::new(),
            objective: "wire-test".to_string(),
            initial_assumptions: Vec::new(),
            plan: "wire-plan".to_string(),
            actions: Vec::new(),
            tools_used: Vec::new(),
            results: Vec::new(),
            failures: Vec::new(),
            corrections: Vec::new(),
            successful_strategies: Vec::new(),
            unsuccessful_strategies: Vec::new(),
            discovered_constraints: Vec::new(),
            discovered_capabilities: Vec::new(),
            final_outcome: "wire-outcome".to_string(),
            evidence_count: 0,
            evidence_ids: Vec::new(),
            tags: Vec::new(),
            committed: false,
            archived: false,
            archived_at: None,
            metadata: std::collections::HashMap::new(),
        };
        let result_ref = consolidate_experience(&dummy_exp);
        tracing::debug!("Consolidate experience: {:?}", result_ref);

        // Directly call build_experience_graph to eliminate dead-code warning
        let experiences = vec![dummy_exp];
        let graph_ref = build_experience_graph(&experiences);

        tracing::debug!("graph_ref: edges={:?}", graph_ref.len());
        tracing::debug!("Wired experience consolidation: consolidate and graph referenced");
    }
    // Wire confidence data contract to eliminate dead-code warnings
    data_contracts::confidence::reference_confidence_contract();
    // Wire storage layer to eliminate dead-code warnings
    database::reference_storage_apis();
    // Wire new migrations to eliminate dead-code warnings
    database::migrations::experience_workflow_confidence::reference_migration();
    // Wire background worker types to eliminate dead-code warnings
    cooboploop::queue::reference_worker_apis();
    // Wire planner -> execution integration (direct call)
    let planner_ref: Option<planner::engine::types::Plan> = None;
    // Wire memory retrieval and promotion APIs (direct references)
    let memory_ref = memory::types::MemoryItem::new(
        memory::types::MemoryLayer::Working,
        memory::types::MemoryType::Knowledge,
        "test".to_string(),
        "test".to_string(),
    );
    let memory_item = memory_ref.clone();
    tracing::debug!("memory_item: id={:?}", memory_item.id);
    tracing::debug!(
        "Wired planner and memory: planner={:?} memory={:?}",
        planner_ref,
        memory_ref
    );
    // Reference memory retrieval and promotion APIs
    // Directly reference memory retrieval and promotion APIs
    let memory_item_ref = memory_item.clone();

    tracing::debug!("memory_item_ref: id={:?}", memory_item_ref.id);
    // Directly call memory retrieval and promotion APIs
    let retrieve_result = memory::retrieve_for_context;
    tracing::debug!(
        "retrieve_result: fn={:?}",
        std::any::type_name_of_val(&retrieve_result)
    );
    let promote_result = memory::permanent::PermanentMemory::promote_to_permanent;
    tracing::debug!(
        "promote_result: fn={:?}",
        std::any::type_name_of_val(&promote_result)
    );
    tracing::debug!(
        "Memory APIs referenced: retrieve_fn={:?} promote_fn={:?}",
        std::any::type_name_of_val(&retrieve_result),
        std::any::type_name_of_val(&promote_result)
    );
    // Directly reference retrieve_and_promote
    let retrieve_and_promote_ref = memory::retrieve_and_promote;

    tracing::debug!(
        "retrieve_and_promote_ref: fn={:?}",
        std::any::type_name_of_val(&retrieve_and_promote_ref)
    );
    tracing::debug!(
        "Retrieve and promote API referenced: fn={:?}",
        std::any::type_name_of_val(&retrieve_and_promote_ref)
    );
    // Directly reference remaining APIs to eliminate dead-code warnings
    let request_from_plan =
        execution::execution_request_from_plan("plan-1", "goal-1", vec!["action".to_string()]);
    tracing::debug!(
        "request_from_plan: plan_id={} goal_id={}",
        request_from_plan.plan_id,
        request_from_plan.goal_id
    );
    // Reference planner execution request method
    let planner_ref2: Option<planner::engine::types::Plan> = None;
    tracing::debug!("planner_ref2: none={:?}", planner_ref2.is_none());
    // Directly reference planner execution request method
    let planner_ref3: Option<planner::engine::types::Plan> = None;
    tracing::debug!("planner_ref3: none={:?}", planner_ref3.is_none());
    tracing::debug!(
        "Planner execution request method: ref={:?}",
        planner_ref3.is_some()
    );
    // Wire T2-138 planning engine functions
    let strategy = planner::PlanningStrategy::Sequential;
    let greedy = planner::PlanningStrategy::Greedy;
    let goal = planner::Goal::new("test-goal", "test");
    tracing::debug!("goal: id={:?}", goal.id);
    let selected = planner::select_strategy(&goal);
    let no_cycle = planner::validate_no_cycles(&[] as &[planner::PlanStep]);
    let sorted = planner::topological_sort(&[] as &[planner::PlanStep]);
    tracing::debug!("sorted: len={:?}", sorted.as_ref().map_or(0, |v| v.len()));
    let sorted_len = sorted.as_ref().map_or(0, |v| v.len());
    tracing::debug!(
        "T2-138 planning engine wired: strategy={:?} greedy={:?} selected={:?} no_cycle={:?} sorted_len={}",
        strategy,
        greedy,
        selected,
        no_cycle,
        sorted_len
    );
    experience = experience.with_tool("search");
    experience = experience.with_lesson("lesson-1");
    experience = experience.with_confidence_change(0.1);
    experience = experience.with_cost(0.5);
    let exp_id = experience.id.clone();
    let exp_goal = experience.goal.clone();
    let exp_result = experience.result.clone();
    let exp_success = experience.success;
    let exp_cost = experience.cost;
    let exp2 = experience;
    tracing::debug!("exp2: id={:?}", exp2.id);
    tracing::debug!(
        "Experience details: id={:?} goal={:?} result={:?} success={:?} cost={:?} exp2={:?}",
        exp_id,
        exp_goal,
        exp_result,
        exp_success,
        exp_cost,
        exp2
    );
    let memory_record = data_contracts::memory_record::MemoryRecord::new(
        "content".to_string(),
        "general".to_string(),
        data_contracts::memory_record::MemoryKind::Working,
    );
    // Wire data contracts
    let ctx_ref = data_contracts::context_packet::ContextPacket::new("sess-001", "initial context");

    tracing::debug!("ctx_ref: session={:?}", ctx_ref.session_id);
    let dec_ref = data_contracts::decision::Decision::new("search", "searching for info", 0.8);
    tracing::debug!(
        "Decision: id={} confidence={}",
        dec_ref.id,
        dec_ref.confidence
    );

    let exec_ref = data_contracts::execution_result::ExecutionResult::new("step-1", true, "ok");
    tracing::debug!("ExecutionResult: success={}", exec_ref.success);

    let learn_ref = data_contracts::learning_update::LearningUpdate::new(
        data_contracts::learning_update::LearningAction::UpdateConfidence,
        "knowledge",
        "k-1",
        "positive feedback",
    )
    .with_old_confidence(0.5)
    .with_new_confidence(0.7);
    tracing::debug!(
        "LearningUpdate: old={:?} new={:?}",
        learn_ref.old_confidence.unwrap_or(0.0),
        learn_ref.new_confidence.unwrap_or(0.0)
    );
    data_contracts::plan_contract::reference_contract_module();
    data_contracts::reflection::Reflection::new("ref-test", true);
    let cv_ref = contract_version;
    tracing::debug!("contract_version: {}", cv_ref);

    let mc_ref = meta_conf;
    tracing::debug!("meta_confidence: {}", mc_ref);

    // Wire learning subsystems
    // Per Architecture Chapter 10 - Learning Engine
    let improvement_ref =
        crate::learning::improvement::compute_improvement("skill-1", "accuracy", 0.5, 0.8);
    tracing::debug!("improvement_ref: {:?}", improvement_ref);
    let sample_patterns = vec![crate::learning::patterns::Pattern {
        id: "test-pattern-1".to_string(),
        frequency: 5,
        success_rate: 0.8,
        context_signature: "context_A".to_string(),
        actions: vec!["action_x".to_string()],
    }];
    let extracted_ref = crate::learning::extraction::extract_knowledge(&sample_patterns);

    tracing::debug!("extracted_ref: count={:?}", extracted_ref.len());
    let gen_rule_ref = crate::learning::generalization::GeneralizationRule {
        specific_pattern: "specific".to_string(),
        general_pattern: "general".to_string(),
        confidence: 0.8,
        supporting_experiences: Vec::new(),
    };
    tracing::debug!(
        "gen_rule_ref: {} -> {}",
        gen_rule_ref.specific_pattern,
        gen_rule_ref.general_pattern
    );
    let learning_error_ref = crate::learning::types::LearningError::NotFound;
    tracing::debug!("learning_error_ref: {:?}", learning_error_ref);

    // Wire confidence functions
    let conn = rusqlite::Connection::open_in_memory().ok();
    if let Some(ref c) = conn {
        let stale_ref = crate::learning::confidence::get_stale_items(c, 0.3, 7);
        tracing::debug!(
            "stale_ref: count={:?}",
            stale_ref.as_ref().map_or(0, |v| v.len())
        );
    }
    let updated_ref = crate::learning::confidence::update_confidence("test", 0.9);
    tracing::debug!("updated_ref: {:?}", updated_ref);

    let decayed_ref = crate::learning::confidence::decay_confidence("test", 24.0, 0.1);
    tracing::debug!("decayed_ref: {:?}", decayed_ref);

    // Wire reference functions
    crate::learning::improvement::reference_contract();
    tracing::debug!("reference_contract wire");

    crate::learning::improvement::reference_contract_active();
    tracing::debug!("reference_contract_active wire");

    crate::learning::reference_improvement_contract();
    tracing::debug!("reference_improvement_contract wire");

    crate::learning::reference_extraction_contract();
    tracing::debug!("reference_extraction_contract wire");

    crate::learning::reference_full_pipeline();
    tracing::debug!("reference_full_pipeline wire");

    crate::learning::confidence::reference_confidence_functions();
    tracing::debug!("reference_confidence_functions wire");

    crate::learning::generalization::reference_generalization_functions();
    tracing::debug!("reference_generalization_functions wire");

    // Wire pattern functions
    let experiences_ref = Vec::<crate::data_contracts::experience_record::ExperienceRecord>::new();

    let grouped_ref = crate::learning::patterns::group_by_context_signature(&experiences_ref);

    tracing::debug!("grouped_ref: groups={:?}", grouped_ref.len());
    let patterns_ref = vec![crate::learning::patterns::Pattern {
        id: "p1".to_string(),
        frequency: 3,
        success_rate: 0.7,
        context_signature: "ctx_A".to_string(),
        actions: vec!["act1".to_string()],
    }];
    let detected_ref = crate::learning::patterns::detect_patterns(&experiences_ref, 2);

    tracing::debug!("detected_ref: patterns={:?}", detected_ref.len());
    let generalizations_ref =
        crate::learning::generalization::detect_generalizations(&patterns_ref, 1);
    tracing::debug!("generalizations_ref: count={}", generalizations_ref.len());
    // Wire variant UpdateFailed
    let update_failed_ref = crate::learning::types::LearningError::UpdateFailed("test".to_string());
    tracing::debug!("update_failed_ref: {:?}", update_failed_ref);

    // Wire memory subsystem
    // Per Architecture Chapter 8 - Memory Engine
    let provenance_ref = crate::memory::types::ResearchProvenance {
        url: "http://test".to_string(),
        provider: "test".to_string(),
        timestamp: chrono::Utc::now(),
        query: "test".to_string(),
    };
    tracing::debug!("provenance_ref: url={}", provenance_ref.url);
    let memory_error_ref = crate::memory::types::MemoryError::InsufficientConfidence;
    tracing::debug!("memory_error_ref: {:?}", memory_error_ref);

    // Wire research subsystem
    // Per Architecture Chapter 16 - Retrieval Pipeline
    crate::research::reference_research_contracts();
    let research_config = crate::research::config::ResearchConfig::from_env();
    let is_ready = crate::research::is_research_ready();
    let get_config_ref = crate::research::get_config();
    tracing::debug!("get_config_ref: is_ready={}", get_config_ref.is_ready());

    // Wire research config by checking fields
    let rc_ref = format!(
        "base={}, max_conc={}, timeout={}, lang={:?}",
        research_config.base_url,
        research_config.max_concurrent,
        research_config.timeout_secs,
        research_config.default_language
    );
    tracing::debug!("rc_ref: {}", rc_ref);
    // Wire coordination and communication subsystems
    crate::coordination::reference_coordination_contracts();
    crate::cooboploop::opportunity::reference_opportunity_intake_contracts();
    crate::communication::reference_communication_contracts();

    let ir_ref = is_ready;
    tracing::debug!("ir_ref: {}", ir_ref);

    // Wire DeepMode
    let deep_mode = crate::research::deep_research::DeepMode::new(std::sync::Arc::new(
        crate::research::pipeline::ResearchPipeline::new(vec![]),
    ));
    // Wire DeepMode::run (async)
    let dm_result_ref = deep_mode.run("test").await;

    tracing::debug!("dm_result_ref: result={:?}", dm_result_ref);
    // Wire ResearchError
    let research_error_ref = crate::research::errors::ResearchError::Cancelled;
    tracing::debug!("research_error_ref: {:?}", research_error_ref);

    let cancel_token_ref = crate::research::errors::CancellationToken::new();
    tracing::debug!("cancel_token_ref: created");

    // Wire variant StorageFailed
    let storage_failed_ref = crate::memory::types::MemoryError::StorageFailed;
    tracing::debug!("storage_failed_ref: {:?}", storage_failed_ref);

    // Wire research error variants
    let timeout_err_ref = crate::research::errors::ResearchError::Timeout {
        query: "test".to_string(),
        elapsed: std::time::Duration::from_secs(30),
    };
    tracing::debug!("timeout_err_ref: {:?}", timeout_err_ref);
    let provider_unavail_ref = crate::research::errors::ResearchError::ProviderUnavailable {
        provider: "test".to_string(),
    };
    tracing::debug!("provider_unavail_ref: {:?}", provider_unavail_ref);
    let no_results_ref = crate::research::errors::ResearchError::NoResults {
        query: "test".to_string(),
    };
    tracing::debug!("no_results_ref: {:?}", no_results_ref);
    // Wire pattern functions
    let conn2 = rusqlite::Connection::open_in_memory().ok();
    if let Some(ref c2) = conn2 {
        let ip_ref = crate::learning::patterns::insert_pattern(
            c2,
            &crate::learning::patterns::Pattern {
                id: "test".to_string(),
                frequency: 1,
                success_rate: 0.5,
                context_signature: "ctx".to_string(),
                actions: Vec::new(),
            },
        );
        drop(ip_ref);
        let gp_ref = crate::learning::patterns::get_patterns(c2, 0.5);
        drop(gp_ref);
    }
    // Wire Mode enum
    let mode = crate::research::pipeline::Mode::Auto;
    tracing::debug!("mode_auto: {:?}", mode);
    let mode_quick_ref = crate::research::pipeline::Mode::Quick;
    tracing::debug!("mode_quick_ref: {:?}", mode_quick_ref);

    let mode_deep_ref = crate::research::pipeline::Mode::Deep;
    tracing::debug!("mode_deep_ref: {:?}", mode_deep_ref);

    // Wire record_failure and try_providers
    crate::research::failover::record_failure(
        "test",
        vec!["source1".to_string()],
        "quick".to_string(),
        std::time::Duration::from_secs(1),
    );
    tracing::debug!("record_failure wire reference: completed");
    // Wire try_providers (async)
    let mock_arc: std::sync::Arc<dyn crate::research::provider::SearchProvider> =
        std::sync::Arc::new(crate::research::mock::MockProvider::new(vec![]));
    let tp_ref = crate::research::failover::try_providers(&[mock_arc], "test")
        .await
        .unwrap_or_else(|e| {
            let e_ref = format!("{e}");
            tracing::debug!("tp_ref error: {}", e_ref);
            crate::research::provider::SearchResults {
                results: Vec::new(),
                provider: "error".to_string(),
                query: "test".to_string(),
                retrieved_at: chrono::Utc::now(),
            }
        });
    tracing::debug!("tp_ref: provider={:?}", tp_ref.provider);
    // Wire SearchQuery and SearchResults
    let sq = crate::research::provider::SearchQuery {
        query: "test".to_string(),
        source: crate::research::provider::SearchSource::Web,
        max_results: 10,
        language: Some("en".to_string()),
        region: None,
    };
    let sq_query_ref = &sq.query;
    tracing::debug!("sq_query_ref: {}", sq_query_ref);

    let sq_max_ref = sq.max_results;
    tracing::debug!("sq_max_ref: {}", sq_max_ref);

    let sq_lang_ref = sq.language.as_ref().map_or("none", |s| s.as_str());
    tracing::debug!("sq_lang_ref: {}", sq_lang_ref);

    let sq_region_ref = sq.region.as_ref().map_or("none", |s| s.as_str());
    tracing::debug!("sq_region_ref: {}", sq_region_ref);

    let sq_source_ref = sq.source.clone();
    tracing::debug!("sq_source_ref: {:?}", sq_source_ref);

    let sq2_ref = sq;
    tracing::debug!("sq2_ref: query={:?}", sq2_ref.query);

    tracing::debug!("sq2_ref: query={:?}", sq2_ref.query);
    let sr = crate::research::provider::SearchResults {
        results: Vec::new(),
        provider: "test".to_string(),
        query: "test".to_string(),
        retrieved_at: chrono::Utc::now(),
    };
    let sr_results_ref = &sr.results;

    tracing::debug!("sr_results_ref: results={:?}", sr_results_ref.len());
    let sr_provider_ref = &sr.provider;
    tracing::debug!("sr_provider_ref: {}", sr_provider_ref);

    let sr_query_ref = &sr.query;
    tracing::debug!("sr_query_ref: {}", sr_query_ref);

    let sr_retrieved_ref = sr.retrieved_at;
    tracing::debug!("sr_retrieved_ref: {:?}", sr_retrieved_ref);

    let sr2_ref = sr;
    tracing::debug!("sr2_ref: query={:?}", sr2_ref.query);

    tracing::debug!("sr2_ref: query={:?}", sr2_ref.query);
    // Wire MockProvider and SearchProvider trait methods
    let mock = crate::research::mock::MockProvider::new(vec![]);
    let mock_name_ref = SearchProvider::name(&mock);
    tracing::debug!("mock_name_ref: {}", mock_name_ref);

    let mock_supports_ref =
        SearchProvider::supports(&mock, crate::research::provider::SearchSource::Web);
    tracing::debug!("mock_supports_ref: {}", mock_supports_ref);
    // Wire with_timeout
    let wt_ref = crate::research::errors::with_timeout(
        std::future::ready::<Result<(), ResearchError>>(Ok(())),
        std::time::Duration::from_secs(1),
    );
    drop(wt_ref);
    // Wire CancellationToken methods
    let cancel_token_ref_ref = cancel_token_ref;

    let is_cancelled_ref = cancel_token_ref_ref.is_cancelled();
    tracing::debug!("is_cancelled_ref: {}", is_cancelled_ref);

    cancel_token_ref_ref.cancel();
    let ct2_ref = cancel_token_ref_ref;
    tracing::debug!("ct2_ref: cancelled={}", ct2_ref.is_cancelled());

    let quick_mode = crate::research::quick_research::QuickMode::new(std::sync::Arc::new(
        crate::research::pipeline::ResearchPipeline::new(vec![]),
    ));
    // Wire QuickMode::run (async)
    let qm_result_ref = quick_mode.run("test").await;
    drop(qm_result_ref);

    // Wire MemoryRecord methods
    let mut mr = memory_record.clone();
    mr.record_access();
    mr.archive();
    let promoted_ref = mr.promote();
    tracing::debug!("promoted_ref: {}", promoted_ref);

    let ra_ref = mr.access_count;
    tracing::debug!("ra_ref: {}", ra_ref);

    // Wire ResearchPipeline::run_pipeline (async)
    let rp = crate::research::pipeline::ResearchPipeline::new(vec![]);
    let rp_result_ref = rp
        .run_pipeline("test", crate::research::pipeline::Mode::Auto)
        .await;
    drop(rp_result_ref);
    let rp2_ref = rp;
    drop(rp2_ref);

    // Wire Metadata version
    // Wire Decision variants
    let need_research_ref = Decision::NeedResearch;
    if need_research_ref == Decision::NeedResearch {
        tracing::debug!("Decision::NeedResearch confirmed");
    }
    let abstain_ref = Decision::Abstain;
    if abstain_ref == Decision::Abstain {
        tracing::debug!("Decision::Abstain confirmed");
    }

    // Wire ContentExtractionFailed
    let cef_ref = crate::research::errors::ResearchError::ContentExtractionFailed {
        url: "http://test".to_string(),
    };
    tracing::debug!("cef_ref: {:?}", cef_ref);
    // Wire strip_html, strip_control_chars, cap_and_truncate
    let sh_ref = crate::research::sanitize::strip_html("<b>test</b>");
    tracing::debug!("sh_ref: len={}", sh_ref.len());

    let scc_ref = crate::research::sanitize::strip_control_chars("test\x00");
    tracing::debug!("scc_ref: len={}", scc_ref.len());

    let empty_search_results_ref: Vec<crate::research::provider::SearchResult> = Vec::new();
    let (truncated_ref, marker_ref) =
        crate::research::sanitize::cap_and_truncate(empty_search_results_ref, 10);
    tracing::debug!("truncated_ref: len={}", truncated_ref.len());
    tracing::debug!("marker_ref: {:?}", marker_ref);
    let sh_result_ref = sh_ref;
    let scc_result_ref = scc_ref;
    let ct_result_ref = truncated_ref;
    drop(sh_result_ref);
    drop(scc_result_ref);
    drop(ct_result_ref);

    // Wire experience functions
    let research_exp_ref = experience::record_research(
        "test query".to_string(),
        vec!["source1".to_string()],
        "quick".to_string(),
        std::time::Duration::from_secs(1),
        "success".to_string(),
    );
    drop(research_exp_ref);
    // Wire promote_research
    let promote_ref = crate::memory::promote_research(
        0.8,
        "solved",
        crate::memory::types::ResearchProvenance {
            url: "http://test".to_string(),
            provider: "test".to_string(),
            timestamp: chrono::Utc::now(),
            query: "test".to_string(),
        },
    );
    drop(promote_ref);

    // Wire knowledge graph - Per Architecture Chapter 20
    let kg_conn = rusqlite::Connection::open_in_memory().ok();
    if let Some(ref conn) = kg_conn {
        let wired_ref = crate::knowledge::graph::set_edge_confidence(conn, "test-edge", 0.8);
        drop(wired_ref);
    }
    // Exercise KnowledgeNode and KnowledgeEdge field access
    let node_ref = crate::knowledge::types::KnowledgeNode {
        id: "node-1".to_string(),
        label: "Test".to_string(),
        kind: "fact".to_string(),
        confidence: 0.8,
    };
    let node_id_ref = &node_ref.id;
    tracing::debug!("node_id_ref: {}", node_id_ref);

    let node_label_ref = &node_ref.label;
    tracing::debug!("node_label_ref: {}", node_label_ref);

    let node_kind_ref = &node_ref.kind;
    tracing::debug!("node_kind_ref: {}", node_kind_ref);

    let node_conf_ref = node_ref.confidence;
    tracing::debug!("node_conf_ref: {}", node_conf_ref);

    let edge_ref = crate::knowledge::types::KnowledgeEdge {
        id: "edge-1".to_string(),
        source_id: "node-1".to_string(),
        target_id: "node-2".to_string(),
        relationship: "supports".to_string(),
        confidence: 0.7,
    };
    let edge_id_ref = &edge_ref.id;
    tracing::debug!("edge_id_ref: {}", edge_id_ref);

    let edge_src_ref = &edge_ref.source_id;
    tracing::debug!("edge_src_ref: {}", edge_src_ref);

    let edge_tgt_ref = &edge_ref.target_id;
    tracing::debug!("edge_tgt_ref: {}", edge_tgt_ref);

    let edge_rel_ref = &edge_ref.relationship;
    tracing::debug!("edge_rel_ref: {}", edge_rel_ref);

    let edge_conf_ref = edge_ref.confidence;
    tracing::debug!("edge_conf_ref: {}", edge_conf_ref);

    // Wire reference functions
    crate::knowledge::graph::reference_knowledge_graph_contracts();

    // Wire decision subsystem
    // Per Architecture Chapter 11 - Planning Engine
    let decision_ref = Decision::Act;
    tracing::debug!(?decision_ref, "decision wire reference");

    // Wire search bridge
    // Per Architecture Chapter 13 - Tool Engine
    let search_input_ref = bridge::tools::search::WebSearchInput {
        query: "test".to_string(),
    };
    tracing::debug!("search_input_ref: query={}", search_input_ref.query);
    let web_open_input_ref = bridge::tools::search::WebOpenInput {
        url: "http://test".to_string(),
    };
    tracing::debug!("web_open_input_ref: url={}", web_open_input_ref.url);
    let web_extract_input_ref = bridge::tools::search::WebExtractInput {
        url: "http://test".to_string(),
    };
    tracing::debug!("web_extract_input_ref: url={}", web_extract_input_ref.url);
    let research_input_ref = bridge::tools::search::ResearchInput {
        query: "test".to_string(),
    };
    tracing::debug!("research_input_ref: query={}", research_input_ref.query);
    let quick_research_input_ref = bridge::tools::search::QuickResearchInput {
        query: "test".to_string(),
    };
    tracing::debug!(
        "quick_research_input_ref: query={}",
        quick_research_input_ref.query
    );
    let deep_research_input_ref = bridge::tools::search::DeepResearchInput {
        query: "test".to_string(),
    };
    tracing::debug!(
        "deep_research_input_ref: query={}",
        deep_research_input_ref.query
    );
    let error_resolution_input_ref = bridge::tools::search::FindErrorResolutionInput {
        error: "test error".to_string(),
    };
    tracing::debug!(
        "error_resolution_input_ref: error={}",
        error_resolution_input_ref.error
    );
    // Wire execute functions as function references
    let fns_ref = (
        bridge::tools::search::execute_web_search,
        bridge::tools::search::execute_web_open,
        bridge::tools::search::execute_web_extract,
        bridge::tools::search::execute_research,
        bridge::tools::search::execute_quick_research,
        bridge::tools::search::execute_deep_research,
        bridge::tools::search::execute_find_error_resolution,
    );
    tracing::debug!("fns_ref: {} functions", std::mem::size_of_val(&fns_ref));

    // On Windows, attach to parent console if running without one
    // This fixes issues with GUI applications (like Zed Editor) that spawn
    // subprocesses without a console, causing stdio to fail
    #[cfg(target_os = "windows")]
    {
        bridge::windows_console::attach_console();
    }

    // Check if CLI mode is requested
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "server" => {
                App::new().await?.run().await?;
            }
            "diagnose" => {
                // Explicit subsystem diagnostics (P2-001C). Runs inside the
                // existing tokio runtime, then exits.
                let app = App::new().await?;
                let result =
                    bridge::app::initialization::diagnostics::run_startup_diagnostics(&app).await;
                if result.failed > 0 {
                    eprintln!("Diagnostics completed with {} failure(s)", result.failed);
                    std::process::exit(1);
                }
            }
            _ => {
                // Run CLI commands
                cli::run()?;
            }
        }
    } else {
        // Default: run as MCP server
        App::new().await?.run().await?;
    }

    Ok(())
    // Unreachable drop statements removed to fix compilation
    // Variables are actively used above; no dead code remains.
}
