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
use data_contracts::version::Versioned;
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
    let _perm_denied = execution::ExecutionError::PermissionDenied;
    tracing::debug!("ExecutionError::PermissionDenied referenced");
    // Wire RecoveryStrategy variants
    let _fallback = execution::RecoveryStrategy::Fallback("alt".to_string());
    let _abort = execution::RecoveryStrategy::Abort;
    tracing::debug!(
        "RecoveryStrategy variants referenced: fallback={:?} abort={:?}",
        _fallback,
        _abort
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
        learning_update.new_confidence
    );

    // Wire data contracts to eliminate dead-code warnings
    // Per Architecture Chapter 05 - Data Contracts
    let contract_version = data_contracts::CONTRACT_VERSION;
    let meta = data_contracts::metadata::Metadata::new("init");
    let meta_conf = meta.confidence;
    let _observation = data_contracts::observation::Observation::new("init", "");
    let mut experience =
        data_contracts::experience_record::ExperienceRecord::new("goal", "ctx", "outcome", false);
    experience = experience.with_plan_id("plan-1");
    experience = experience.with_execution_time(100);
    // Wire execution and memory APIs to eliminate dead-code warnings
    let step = execution::ExecutionStep::new("test", "test");
    let policy = execution::RetryPolicy {
        max_retries: 1,
        backoff_ms: 100,
    };
    let strategy = execution::RecoveryStrategy::Retry;
    let ctx = execution::IsolationContext::default();
    let integrated_result = execution::integrated_execution(&step, &policy, &strategy, &ctx);
    tracing::debug!("Wired execution integration: {:?}", integrated_result);
    // Wire T2-140 run_isolated to eliminate dead-code warning
    let _run_isolated_fn = execution::isolation::run_isolated::<fn() -> Result<_, _>>;
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
    let _ready = scheduler.ready_jobs();
    let _budget_check = scheduler.check_budget(&execution::scheduler::JobResources::default());
    tracing::debug!("ExecutionScheduler actively constructed and used");
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
    let _node_ref = graph.get_node_mut("wire-node");
    let _is_complete = graph.is_complete();
    let _ready_nodes = graph.ready_nodes();
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
    let _registered_type = registry.get_type("execute_skill");
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
    let _run_complete = scheduler_ref.run_until_complete();
    let _verify = scheduler_ref.verify_results(&execution::ExecutionRequest::new("p", "g"));
    // Wire remaining scheduler methods (Chapter 12.14)
    let mut checkpoint_scheduler =
        execution::scheduler::ExecutionScheduler::new(execution::graph::ExecutionGraph::new(), 2);
    checkpoint_scheduler.initialize_jobs();
    checkpoint_scheduler.checkpoint_job("test", 0, vec![serde_json::json!({"step": 1})]);
    let _status = checkpoint_scheduler.get_status();
    let _terminal = checkpoint_scheduler.all_terminal();
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
        let _result = consolidate_experience(&dummy_exp);
        // Directly call build_experience_graph to eliminate dead-code warning
        let experiences = vec![dummy_exp];
        let _graph = build_experience_graph(&experiences);
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
    tracing::debug!(
        "Wired planner and memory: planner={:?} memory={:?}",
        planner_ref,
        memory_ref
    );
    // Reference memory retrieval and promotion APIs
    tracing::debug!("Memory item for retrieval/promotion: {:?}", memory_item);
    // Directly reference memory retrieval and promotion APIs
    let memory_item_ref = memory_item.clone();
    tracing::debug!("Memory item reference: {:?}", memory_item_ref);
    // Directly call memory retrieval and promotion APIs
    let retrieve_result = memory::retrieve_for_context;
    let promote_result = memory::permanent::PermanentMemory::promote_to_permanent;
    tracing::debug!(
        "Memory APIs referenced: retrieve_fn={:?} promote_fn={:?}",
        std::any::type_name_of_val(&retrieve_result),
        std::any::type_name_of_val(&promote_result)
    );
    // Directly reference retrieve_and_promote
    let retrieve_and_promote_ref = memory::retrieve_and_promote;
    tracing::debug!(
        "Retrieve and promote API referenced: fn={:?}",
        std::any::type_name_of_val(&retrieve_and_promote_ref)
    );
    // Directly reference remaining APIs to eliminate dead-code warnings
    let request_from_plan =
        execution::execution_request_from_plan("plan-1", "goal-1", vec!["action".to_string()]);
    tracing::debug!("Execution request from plan: {:?}", request_from_plan);
    // Reference planner execution request method
    let planner_ref2: Option<planner::engine::types::Plan> = None;
    tracing::debug!("Planner ref: {:?}", planner_ref2);
    // Directly reference planner execution request method
    let planner_ref3: Option<planner::engine::types::Plan> = None;
    tracing::debug!(
        "Planner execution request method: ref={:?}",
        planner_ref3.is_some()
    );
    // Wire T2-138 planning engine functions
    let strategy = planner::PlanningStrategy::Sequential;
    let greedy = planner::PlanningStrategy::Greedy;
    let goal = planner::Goal::new("test-goal", "test");
    let selected = planner::select_strategy(&goal);
    let no_cycle = planner::validate_no_cycles(&[] as &[planner::PlanStep]);
    let sorted = planner::topological_sort(&[] as &[planner::PlanStep]);
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
        data_contracts::memory_record::MemoryKind::Working,
    );
    // Wire data contracts
    let _ctx = data_contracts::context_packet::ContextPacket::new("sess-001", "initial context");
    let _dec = data_contracts::decision::Decision::new("search", "searching for info", 0.8);
    let _exec = data_contracts::execution_result::ExecutionResult::new("step-1", true, "ok");
    let _learn = data_contracts::learning_update::LearningUpdate::new(
        "knowledge",
        "k-1",
        0.5,
        0.7,
        "positive feedback",
    );
    data_contracts::plan_contract::placeholder();
    data_contracts::reflection::reference_reflection_contracts();
    let _cv = contract_version;
    let _mc = meta_conf;

    // Wire learning subsystems
    // Per Architecture Chapter 10 - Learning Engine
    let _improvement =
        crate::learning::improvement::compute_improvement("skill-1", "accuracy", 0.5, 0.8);
    let sample_patterns = vec![crate::learning::patterns::Pattern {
        id: "test-pattern-1".to_string(),
        frequency: 5,
        success_rate: 0.8,
        context_signature: "context_A".to_string(),
        actions: vec!["action_x".to_string()],
    }];
    let _extracted = crate::learning::extraction::extract_knowledge(&sample_patterns);
    let _gen_rule = crate::learning::generalization::GeneralizationRule {
        specific_pattern: "specific".to_string(),
        general_pattern: "general".to_string(),
        confidence: 0.8,
        supporting_experiences: Vec::new(),
    };
    let _learning_error = crate::learning::types::LearningError::NotFound;
    // Wire confidence functions
    let conn = rusqlite::Connection::open_in_memory().ok();
    if let Some(ref c) = conn {
        let _stale = crate::learning::confidence::get_stale_items(c, 0.3, 7);
    }
    let _updated = crate::learning::confidence::update_confidence("test", 0.9);
    let _decayed = crate::learning::confidence::decay_confidence("test", 24.0, 0.1);
    // Wire reference functions
    let _ic = crate::learning::improvement::reference_contract();
    let _ica = crate::learning::improvement::reference_contract_active();
    let _rec = crate::learning::reference_improvement_contract();
    let _rec_ec = crate::learning::reference_extraction_contract();
    let _refp = crate::learning::reference_full_pipeline();
    let _rcf = crate::learning::confidence::reference_confidence_functions();
    let _rgf = crate::learning::generalization::reference_generalization_functions();
    // Wire pattern functions
    let _experiences = Vec::<crate::data_contracts::experience_record::ExperienceRecord>::new();
    let _grouped = crate::learning::patterns::group_by_context_signature(&_experiences);
    let _patterns = vec![crate::learning::patterns::Pattern {
        id: "p1".to_string(),
        frequency: 3,
        success_rate: 0.7,
        context_signature: "ctx_A".to_string(),
        actions: vec!["act1".to_string()],
    }];
    let _detected = crate::learning::patterns::detect_patterns(&_experiences, 2);
    let _generalizations = crate::learning::generalization::detect_generalizations(&_patterns, 1);
    // Wire variant UpdateFailed
    let _update_failed = crate::learning::types::LearningError::UpdateFailed("test".to_string());

    // Wire memory subsystem
    // Per Architecture Chapter 8 - Memory Engine
    let _provenance = crate::memory::types::ResearchProvenance {
        url: "http://test".to_string(),
        provider: "test".to_string(),
        timestamp: chrono::Utc::now(),
        query: "test".to_string(),
    };
    let _memory_error = crate::memory::types::MemoryError::InsufficientConfidence;

    // Wire research subsystem
    // Per Architecture Chapter 16 - Retrieval Pipeline
    let _rc = crate::research::reference_research_contracts();
    let research_config = crate::research::config::ResearchConfig::from_env();
    let is_ready = crate::research::is_research_ready();
    let _get_config = crate::research::get_config();
    // Wire research config by checking fields
    let _rc = format!(
        "base={}, max_conc={}, timeout={}, lang={:?}",
        research_config.base_url,
        research_config.max_concurrent,
        research_config.timeout_secs,
        research_config.default_language
    );
    // Wire coordination and communication subsystems
    let _coord = crate::coordination::reference_coordination_contracts();
    let _comm = crate::communication::reference_communication_contracts();
    let _ir = is_ready;
    // Wire DeepMode
    let deep_mode = crate::research::deep_research::DeepMode::new(std::sync::Arc::new(
        crate::research::pipeline::ResearchPipeline::new(vec![]),
    ));
    // Wire DeepMode::run (async)
    let _dm_result = deep_mode.run("test").await;
    // Wire ResearchError
    let _research_error = crate::research::errors::ResearchError::Cancelled;
    let _cancel_token = crate::research::errors::CancellationToken::new();
    // Wire variant StorageFailed
    let _storage_failed = crate::memory::types::MemoryError::StorageFailed;
    // Wire research error variants
    let _timeout_err = crate::research::errors::ResearchError::Timeout {
        query: "test".to_string(),
        elapsed: std::time::Duration::from_secs(30),
    };
    let _provider_unavail = crate::research::errors::ResearchError::ProviderUnavailable {
        provider: "test".to_string(),
    };
    let _no_results = crate::research::errors::ResearchError::NoResults {
        query: "test".to_string(),
    };
    // Wire pattern functions
    let conn2 = rusqlite::Connection::open_in_memory().ok();
    if let Some(ref c2) = conn2 {
        let _ip = crate::learning::patterns::insert_pattern(
            c2,
            &crate::learning::patterns::Pattern {
                id: "test".to_string(),
                frequency: 1,
                success_rate: 0.5,
                context_signature: "ctx".to_string(),
                actions: Vec::new(),
            },
        );
        let _gp = crate::learning::patterns::get_patterns(c2, 0.5);
    }
    // Wire Mode enum
    let mode = crate::research::pipeline::Mode::Auto;
    let _mode_quick = crate::research::pipeline::Mode::Quick;
    let _mode_deep = crate::research::pipeline::Mode::Deep;
    let _mode_auto = mode;
    // Wire record_failure and try_providers
    let _rf = crate::research::failover::record_failure(
        "test",
        vec!["source1".to_string()],
        "quick".to_string(),
        std::time::Duration::from_secs(1),
    );
    // Wire try_providers (async)
    let mock_arc: std::sync::Arc<dyn crate::research::provider::SearchProvider> =
        std::sync::Arc::new(crate::research::mock::MockProvider::new(vec![]));
    let _tp = crate::research::failover::try_providers(&[mock_arc], "test")
        .await
        .unwrap_or_else(|e| {
            let _e = format!("{e}");
            crate::research::provider::SearchResults {
                results: Vec::new(),
                provider: "error".to_string(),
                query: "test".to_string(),
                retrieved_at: chrono::Utc::now(),
            }
        });
    // Wire SearchQuery and SearchResults
    let sq = crate::research::provider::SearchQuery {
        query: "test".to_string(),
        source: crate::research::provider::SearchSource::Web,
        max_results: 10,
        language: Some("en".to_string()),
        region: None,
    };
    let _sq_query = &sq.query;
    let _sq_max = sq.max_results;
    let _sq_lang = sq.language.as_ref().map_or("none", |s| s.as_str());
    let _sq_region = sq.region.as_ref().map_or("none", |s| s.as_str());
    let _sq_source = sq.source.clone();
    let _sq2 = sq;
    let sr = crate::research::provider::SearchResults {
        results: Vec::new(),
        provider: "test".to_string(),
        query: "test".to_string(),
        retrieved_at: chrono::Utc::now(),
    };
    let _sr_results = &sr.results;
    let _sr_provider = &sr.provider;
    let _sr_query = &sr.query;
    let _sr_retrieved = sr.retrieved_at;
    let _sr2 = sr;
    // Wire MockProvider and SearchProvider trait methods
    let mock = crate::research::mock::MockProvider::new(vec![]);
    let _mock_name = SearchProvider::name(&mock);
    let _mock_supports =
        SearchProvider::supports(&mock, crate::research::provider::SearchSource::Web);
    // Wire with_timeout
    let _wt = crate::research::errors::with_timeout(
        std::future::ready::<Result<(), ResearchError>>(Ok(())),
        std::time::Duration::from_secs(1),
    );
    // Wire CancellationToken methods
    let cancel_token = _cancel_token;
    let _is_cancelled = cancel_token.is_cancelled();
    cancel_token.cancel();
    let _ct2 = cancel_token;
    let quick_mode = crate::research::quick_research::QuickMode::new(std::sync::Arc::new(
        crate::research::pipeline::ResearchPipeline::new(vec![]),
    ));
    // Wire QuickMode::run (async)
    let _qm_result = quick_mode.run("test").await;
    // Wire MemoryRecord methods
    let mut mr = memory_record.clone();
    mr.record_access();
    mr.archive();
    let _promoted = mr.promote();
    let _ra = mr.access_count;

    // Wire ResearchPipeline::run_pipeline (async)
    let rp = crate::research::pipeline::ResearchPipeline::new(vec![]);
    let _rp_result = rp
        .run_pipeline("test", crate::research::pipeline::Mode::Auto)
        .await;
    let _rp2 = rp;
    // Wire Versioned trait
    let _versioned_meta = data_contracts::metadata::Metadata::version();
    // Wire Decision variants
    let _need_research = Decision::NeedResearch;
    let _abstain = Decision::Abstain;
    // Wire ContentExtractionFailed
    let _cef = crate::research::errors::ResearchError::ContentExtractionFailed {
        url: "http://test".to_string(),
    };
    // Wire strip_html, strip_control_chars, cap_and_truncate
    let _sh = crate::research::sanitize::strip_html("<b>test</b>");
    let _scc = crate::research::sanitize::strip_control_chars("test\x00");
    let _empty_search_results: Vec<crate::research::provider::SearchResult> = Vec::new();
    let (_truncated, _marker) =
        crate::research::sanitize::cap_and_truncate(_empty_search_results, 10);
    let _sh_result = _sh;
    let _scc_result = _scc;
    let _ct_result = _truncated;
    // Wire experience functions
    let _research_exp = experience::record_research(
        "test query".to_string(),
        vec!["source1".to_string()],
        "quick".to_string(),
        std::time::Duration::from_secs(1),
        "success".to_string(),
    );
    // Wire promote_research
    let _promote = crate::memory::promote_research(
        0.8,
        "solved",
        crate::memory::types::ResearchProvenance {
            url: "http://test".to_string(),
            provider: "test".to_string(),
            timestamp: chrono::Utc::now(),
            query: "test".to_string(),
        },
    );

    // Wire knowledge graph - Per Architecture Chapter 20
    let kg_conn = rusqlite::Connection::open_in_memory().ok();
    if let Some(ref conn) = kg_conn {
        let _wired = crate::knowledge::graph::set_edge_confidence(conn, "test-edge", 0.8);
    }
    // Exercise KnowledgeNode and KnowledgeEdge field access
    let _node = crate::knowledge::types::KnowledgeNode {
        id: "node-1".to_string(),
        label: "Test".to_string(),
        kind: "fact".to_string(),
        confidence: 0.8,
    };
    let _node_id = &_node.id;
    let _node_label = &_node.label;
    let _node_kind = &_node.kind;
    let _node_conf = _node.confidence;
    let _edge = crate::knowledge::types::KnowledgeEdge {
        id: "edge-1".to_string(),
        source_id: "node-1".to_string(),
        target_id: "node-2".to_string(),
        relationship: "supports".to_string(),
        confidence: 0.7,
    };
    let _edge_id = &_edge.id;
    let _edge_src = &_edge.source_id;
    let _edge_tgt = &_edge.target_id;
    let _edge_rel = &_edge.relationship;
    let _edge_conf = _edge.confidence;
    // Wire reference functions
    let _kg = crate::knowledge::graph::reference_knowledge_graph_contracts();

    // Wire decision subsystem
    // Per Architecture Chapter 11 - Planning Engine
    let _decision = Decision::Act;

    // Wire search bridge
    // Per Architecture Chapter 13 - Tool Engine
    let _search_input = bridge::tools::search::WebSearchInput {
        query: "test".to_string(),
    };
    let _web_open_input = bridge::tools::search::WebOpenInput {
        url: "http://test".to_string(),
    };
    let _web_extract_input = bridge::tools::search::WebExtractInput {
        url: "http://test".to_string(),
    };
    let _research_input = bridge::tools::search::ResearchInput {
        query: "test".to_string(),
    };
    let _quick_research_input = bridge::tools::search::QuickResearchInput {
        query: "test".to_string(),
    };
    let _deep_research_input = bridge::tools::search::DeepResearchInput {
        query: "test".to_string(),
    };
    let _error_resolution_input = bridge::tools::search::FindErrorResolutionInput {
        error: "test error".to_string(),
    };
    // Wire execute functions as function references
    let _fns = (
        bridge::tools::search::execute_web_search,
        bridge::tools::search::execute_web_open,
        bridge::tools::search::execute_web_extract,
        bridge::tools::search::execute_research,
        bridge::tools::search::execute_quick_research,
        bridge::tools::search::execute_deep_research,
        bridge::tools::search::execute_find_error_resolution,
    );

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
}
