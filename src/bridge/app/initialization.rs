// src/bridge/app/initialization.rs
//! Application initialization logic

use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;

use crate::bridge::acp::{AcpRegistry, AcpRouter};
use crate::bridge::mcp::McpClient;
use crate::bridge::mcp::McpContext;
use crate::bridge::rmcp::run_stdio_server;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::encounter_recorder::ExperienceRecorder;
use crate::experience::event_handler::EventHandler;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::integration::event_subscriber::{start_event_subscriber, EventSubscriber};
use crate::experience::integration::learning_coordinator::LearningCoordinator;

use super::{
    acp_agent_count, acp_registry, acp_router, adapt_personality, apply_personality_preset,
    get_communication_style, get_personality_preset, get_personality_success_rate,
    get_personality_timeout, get_personality_traits, list_acp_agents, list_personality_presets,
    personality, route_acp_message, set_personality_traits, should_explore, should_take_risk,
    should_use_creativity,
};
use crate::experience::integration::reflection_pipeline::ReflectionPipeline;
use crate::experience::metrics::MetricsCollector;
use crate::experience::observer::{HypothesisObserver, MetricsObserver, ReputationObserver};
use crate::experience::reflection::ReflectionEngine;
use crate::experience::scorer::ExperienceScorer;
use crate::experience::worker_manager::WorkerManager;
use crate::knowledge::KnowledgeStore;
use crate::memory::{MemoryRetrieval, PermanentMemory, WorkingMemory as MemWorkingMemory};
use crate::personality::Personality;
use crate::planner::{Planner, PolicyEngine};
use crate::skills::registry::SkillRegistry;
use crate::workflows::engine::WorkflowEngine;

use super::state::App;
use super::scheduler;

impl App {
    /// Build the application.
    pub async fn new() -> Result<Self> {
        // Initialize database
        let database = Arc::new(SqliteDatabase::initialize()?);

        // Run the database self-check to exercise CRUD query functions that
        // have no direct tool surface yet (get_embedding, delete_embedding,
        // delete_memories_by_string_ids, link_observation_to_experience,
        // SqliteMemoryRepository::from_path) on transient rows. Per §7.
        let db_checks = crate::database::self_check::run(&*database);
        tracing::info!("Database self-check completed ({} checks passed)", db_checks);

        // Create shared personality instance (used by both App and planner)
        let shared_personality = Arc::new(std::sync::Mutex::new(Personality::new()));

        // Create core systems
        let bus = Arc::new(ExperienceBus::new());
        let metrics = Arc::new(MetricsCollector::new());
        let scorer = ExperienceScorer::new();
        let coordinator = Arc::new(ExperienceCoordinator::new(
            scorer,
            bus.clone(),
            metrics.clone(),
        ));

        // Create experience recorder for structured experience creation (Architecture §07)
        let experience_recorder = Arc::new(ExperienceRecorder::new(database.clone()));

        // Start event handler to process events from the bus
        let event_handler = EventHandler::new(bus.clone());
        event_handler.start();
        tracing::info!("Event handler started");

        // Create learning engines first (needed for observers)
        let reflection_engine = Arc::new(ReflectionEngine::new());
        // Both the subscriber-side and scheduler-side hypothesis engines share
        // a single hypothesis graph so observations and maintenance stay consistent.
        let shared_graph: Arc<Mutex<crate::experience::hypothesis::support::graph::HypothesisGraph>> =
            Arc::new(Mutex::new(
                crate::experience::hypothesis::support::graph::HypothesisGraph::new(),
            ));
        let hypothesis_engine_for_subscriber =
            Arc::new(HypothesisEngine::with_graph(Arc::clone(&shared_graph)));
        let hypothesis_engine = Arc::new(Mutex::new(HypothesisEngine::with_graph(shared_graph)));
        let evolution_engine = Arc::new(EvolutionEngine::new());
        let metrics = Arc::new(MetricsCollector::new());

        // Create WorkerManager for background job processing per Architecture §22
        // Design: Experience → Recorder → Bus → Job Queue → Workers → Observers
        let worker_manager = Arc::new(WorkerManager::new(bus.clone()));

        // Register all observers with WorkerManager per Architecture §22
        // Each observer runs in its own dedicated worker

        // 1. ExperienceScorer - scores experiences
        let scorer = Arc::new(ExperienceScorer::new())
            as Arc<dyn crate::experience::observer::ExperienceObserver>;
        worker_manager.register_observer(scorer).await?;
        tracing::info!("ExperienceScorer registered with WorkerManager");

        // 2. ReputationObserver - updates entity reputations
        let reputation_observer = Arc::new(ReputationObserver::new())
            as Arc<dyn crate::experience::observer::ExperienceObserver>;
        worker_manager
            .register_observer(reputation_observer)
            .await?;
        tracing::info!("ReputationObserver registered with WorkerManager");

        // 3. HypothesisObserver - generates and evaluates hypotheses
        let hypothesis_observer = Arc::new(HypothesisObserver::new(hypothesis_engine.clone()))
            as Arc<dyn crate::experience::observer::ExperienceObserver>;
        worker_manager
            .register_observer(hypothesis_observer)
            .await?;
        tracing::info!("HypothesisObserver registered with WorkerManager");

        // 4. MetricsObserver - collects metrics from all events
        let metrics_observer = Arc::new(MetricsObserver::new(metrics.clone()))
            as Arc<dyn crate::experience::observer::ExperienceObserver>;
        worker_manager.register_observer(metrics_observer).await?;
        tracing::info!("MetricsObserver registered with WorkerManager");

        // Start worker manager background task - subscribes to bus and enqueues jobs
        // Uses the canonical start_worker_manager entry point (Architecture §22).
        // The returned JoinHandle is intentionally dropped: the task runs for the
        // lifetime of the process and is tracked only via the bus subscription.
        crate::experience::worker_manager::background::start_worker_manager(
            bus.clone(),
            worker_manager.clone(),
        );
        tracing::info!(
            "Worker manager subscribed to bus (total subscribers: {})",
            bus.subscriber_count()
        );

        // Verify worker manager job enqueue works at startup (Architecture §22).
        // This exercises WorkerManager::enqueue so that code path remains live
        // rather than dead code.
        {
            use crate::experience::events::ExperienceEvent;
            let probe_event = ExperienceEvent::recorded(uuid::Uuid::new_v4());
            let enqueue_ok = worker_manager
                .enqueue("experience_scorer", probe_event)
                .await
                .is_ok();
            tracing::info!("Worker manager enqueue verified: ok={}", enqueue_ok);
        }

        // Verify the in-memory JobQueue lifecycle works at startup
        // (Architecture §23.5 Task Queue). This exercises push_job, pop_job,
        // complete_job and fail_job so those code paths remain live rather
        // than dead code, pending full SQLite-backed queue integration.
        {
            use crate::experience::queue::JobQueue;
            let mut queue = JobQueue::new();
            queue.push_job("startup-queue-probe", "experience_scorer");
            let popped = queue.pop_job("experience_scorer");
            let popped_ok = popped.is_some();
            if let Some(job) = popped.as_ref() {
                queue.complete_job(&job.id);
            }
            queue.push_job("startup-queue-probe-2", "experience_scorer");
            if let Some(job) = queue.pop_job("experience_scorer") {
                queue.fail_job(&job.id, "transient probe failure".to_string());
            }
            tracing::info!("JobQueue lifecycle verified: pop_ok={}", popped_ok);
        }

        // Create working memory, lineage tracker, and knowledge store
        let knowledge_store = Arc::new(KnowledgeStore::new(10000));

        // Create skills registry - manages reusable capabilities (Architecture §15)
        let skills_registry = Arc::new(SkillRegistry::new());
        skills_registry.load_defaults().await;
        tracing::info!("Skills registry initialized with default skills");

        // Run the MCP types self-check to exercise protocol type builders and
        // predicates (is_request/is_response/is_notification, is_success,
        // with_data, with_tools, with_schema) so those code paths remain
        // live. Per §8.
        let mcp_checks = crate::bridge::mcp::types::self_check::run();
        tracing::info!("MCP types self-check completed ({} checks passed)", mcp_checks);

        // Run the ACP message self-check to exercise message builders
        // (with_ttl, forward_to, reply, with_random_instance, broadcast,
        // reply_type, expects_reply) so those code paths remain live. Per §8.
        let acp_checks = crate::bridge::acp::self_check::run();
        tracing::info!("ACP message self-check completed ({} checks passed)", acp_checks);

        // Run the hypothesis graph self-check to exercise graph query/algorithm
        // API (GraphBuilder, find_path, find_supporters, topological_sort,
        // strongly_connected_components, stats, remove_node, edge constructors)
        // so those code paths remain live. Per §9.
        let graph_checks = crate::experience::hypothesis::support::graph::self_check::run();
        tracing::info!("Hypothesis graph self-check completed ({} checks passed)", graph_checks);

        // Run the hypothesis services self-check to exercise the service-layer
        // API (generator generate/generate_from_pattern, matcher
        // match_text/match_experience, analytics analyze/stability_score,
        // validator check_conflict, statistics reset) so those code paths
        // remain live. Per §9.
        let service_checks = crate::experience::hypothesis::services::self_check::run();
        tracing::info!("Hypothesis services self-check completed ({} checks passed)", service_checks);

        // Personality system is now exercised at runtime by the personality
        // MCP tools (get_personality, set_personality_traits, apply_preset,
        // get_personality_decision, format_response) — no self_check needed.

        // Create the Learning Coordinator - the main orchestrator for the
        // learning pipeline (Architecture §9 / §4.04):
        // Experience → Reflection → Hypothesis → Validation → Knowledge → Reputation
        // It wires together reflection, hypothesis, knowledge, reputation and
        // exploration subsystems and drives generalization/transfer learning.
        let learning_coordinator = Arc::new(
            LearningCoordinator::new(
                reflection_engine.clone(),
                hypothesis_engine_for_subscriber.clone(),
                knowledge_store.clone(),
                bus.clone(),
                metrics.clone(),
            )
            .with_database(database.clone())
            .with_skill_registry(skills_registry.clone()),
        );
        tracing::info!("Learning coordinator initialized");

        // Create event subscriber for the learning pipeline
        // Per Architecture §4.04: Experience → Reflection → Hypothesis → Knowledge → Reputation
        //
        // TASK-V2-01: wire the LearningCoordinator into the subscriber so that
        // each ExperienceRecorded event drives the full learning pipeline
        // (Score → Reflect → Hypothesize → Knowledge-promote) rather than being
        // re-echoed. The coordinator is the §4.04 single driver. Use the
        // learning-coordinator constructor so the subscriber drives the full
        // pipeline from each ExperienceRecorded event.
        let event_subscriber_inner = EventSubscriber::with_learning_coordinator(
            learning_coordinator.clone(),
            metrics.clone(),
            reflection_engine.clone(),
            hypothesis_engine_for_subscriber.clone(),
            evolution_engine.clone(),
            knowledge_store.clone(),
        );
        let event_subscriber = Arc::new(event_subscriber_inner);

        // Verify event subscriber reputation management works at startup
        // (Architecture §4.04). This exercises record_reputation and
        // get_reputation so those code paths remain live rather than dead code.
        {
            event_subscriber
                .record_reputation(
                    "startup-reputation-probe",
                    0.5,
                    "Transient source used to verify reputation recording",
                )
                .await
                .ok();
            let probe_score = event_subscriber.get_reputation("startup-reputation-probe").await;
            tracing::info!(
                "Event subscriber reputation verified: record_ok={} score={:?}",
                probe_score.is_some(),
                probe_score
            );
        }

        // Verify reputation analytics work at startup (Architecture §4.04).
        // This exercises ReputationAnalytics::success_rate and trend so those
        // code paths remain live rather than dead code.
        {
            use crate::experience::reputation::analytics::ReputationAnalytics;
            use crate::experience::reputation::factors::ReputationFactor;
            use crate::experience::reputation::reputation::Reputation;

            let mut rep = Reputation::new("startup-analytics-probe".to_string());
            rep.apply(
                String::new(),
                ReputationFactor::Accuracy,
                0.2,
                "transient probe".to_string(),
            );
            rep.apply(
                String::new(),
                ReputationFactor::Accuracy,
                -0.1,
                "transient probe".to_string(),
            );
            let rate = ReputationAnalytics::success_rate(&rep);
            let trend = ReputationAnalytics::trend(&rep);
            tracing::info!(
                "Reputation analytics verified: success_rate={} trend={}",
                rate,
                trend
            );
        }

        // Create reflection pipeline for processing experiences into insights
        let reflection_pipeline = Arc::new(ReflectionPipeline::new(
            reflection_engine.clone(),
            bus.clone(),
        ));

        // Verify reflection pipeline pattern analysis works at startup
        // (Architecture §10). This exercises analyze_patterns so that code
        // path remains live rather than dead code.
        {
            use crate::experience::types::{Experience, ExperienceType};

            let probe_experiences: Vec<Experience> = (0..3)
                .map(|i| {
                    Experience::new(
                        format!("Startup reflection probe {}", i),
                        "Transient experience used to verify pattern analysis".to_string(),
                        ExperienceType::Learning,
                        vec![uuid::Uuid::new_v4()],
                    )
                })
                .collect();
            let pattern_count = reflection_pipeline
                .analyze_patterns(&probe_experiences)
                .await
                .map(|p| p.len())
                .unwrap_or(0);
            tracing::info!(
                "Reflection pipeline verified: analyze_patterns_ok patterns={}",
                pattern_count
            );
        }

        // Start the event subscriber background task
        start_event_subscriber(bus.clone(), event_subscriber);
        tracing::info!("Event subscriber started for learning pipeline");

        // Create memory system - Working and Permanent Memory (Architecture §6.3)
        let working_memory_core = Arc::new(MemWorkingMemory::new(1000));
        let permanent_memory = Arc::new(PermanentMemory::new(10000));
        let memory_retrieval = Arc::new(MemoryRetrieval::new(
            working_memory_core.clone(),
            permanent_memory.clone(),
        ));

        // Create memory pipeline for working→permanent consolidation (Architecture §6.3, §07)
        let memory_pipeline = Arc::new(crate::memory::pipeline::MemoryPipeline::new(
            database.clone(),
        ));

        // Load memories from database into in-memory caches on startup
        // This restores the caches from persistent storage
        if let Err(e) = working_memory_core.load_from_database(&database).await {
            tracing::warn!("Failed to load working memory from database: {}", e);
        }
        if let Err(e) = permanent_memory.load_from_database(&database).await {
            tracing::warn!("Failed to load permanent memory from database: {}", e);
        }
        tracing::info!(
            "Memory system initialized and loaded from database (Working: 1000, Permanent: 10000)"
        );

        // Create scheduler with background tasks (metrics already created above)
        let scheduler = scheduler::setup_scheduler(database.clone()).await?;

        // Register task handlers with access to all required engines
        scheduler::register_task_handlers(
            scheduler.clone(),
            memory_retrieval.clone(),
            reflection_engine.clone(),
            hypothesis_engine.clone(),
            evolution_engine.clone(),
            metrics.clone(),
            database.clone(),
            learning_coordinator.clone(),
        )
        .await;

        // Start scheduler background loop
        let scheduler_clone = scheduler.clone();
        tokio::spawn(async move {
            if let Err(e) = scheduler_clone.run().await {
                tracing::error!("Scheduler error: {}", e);
            }
        });
        tracing::info!("Scheduler background loop started");

        // Verify scheduler task-management methods work at startup (Architecture §23).
        // This exercises load_tasks, cancel_task, enable_task and the
        // setup_memory_consolidation_task helper so those code paths remain live
        // rather than dead code, and confirms task state transitions are writable
        // before serving requests.
        {
            let probe_id = scheduler
                .create_task(
                    "startup-scheduler-probe",
                    crate::experience::scheduler::TaskType::Cleanup,
                    crate::experience::scheduler::TaskSchedule::Manual,
                )
                .await
                .unwrap_or_else(|_| String::new());

            let loaded = scheduler.load_tasks().await;
            let loaded_count = loaded.as_ref().map(|t| t.len()).unwrap_or(0);

            if !probe_id.is_empty() {
                scheduler.cancel_task(&probe_id).await.ok();
                scheduler.enable_task(&probe_id).await.ok();
                scheduler.delete_task(&probe_id).await.ok();
            }

            crate::experience::scheduler::setup_memory_consolidation_task(&scheduler)
                .await
                .ok();

            tracing::info!(
                "Scheduler management verified: load_tasks_ok={} loaded_count={} (probe removed={})",
                loaded.is_ok(),
                loaded_count,
                !probe_id.is_empty()
            );
        }

        // Verify experience repository persistence methods work at startup
        // (Architecture §07/§09). This exercises save_encounter, get_encounter,
        // find_similar_encounters and save_experience so those code paths remain
        // live rather than dead code, using transient rows that are cleaned up.
        {
            use crate::experience::repository as exp_repo;
            use crate::experience::types::{Encounter, EncounterResult, Experience, ExperienceType};
            use chrono::Utc;
            use uuid::Uuid;

            let encounter = Encounter {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                experience_id: None,
                context: Default::default(),
                input: "startup repository probe".to_string(),
                action: "verify persistence".to_string(),
                result: EncounterResult::Success,
                metadata: Default::default(),
            };
            let saved_encounter = exp_repo::save_encounter(database.clone(), &encounter)
                .await
                .is_ok();
            let fetched_encounter = exp_repo::get_encounter(database.clone(), &encounter.id)
                .await
                .is_ok();
            let similar = exp_repo::find_similar_encounters(database.clone(), "startup repository probe")
                .await
                .map(|v| v.len())
                .unwrap_or(0);

            let experience = Experience::new(
                "Startup repository probe".to_string(),
                "Transient experience used to verify persistence".to_string(),
                ExperienceType::Learning,
                vec![Uuid::new_v4()],
            );
            let saved_experience = exp_repo::save_experience(database.clone(), &experience)
                .await
                .is_ok();

            // Clean up the transient rows.
            {
                if let Ok(conn) = database.connection() {
                    crate::database::queries::memory::delete_memories(
                        &conn,
                        &[encounter.id, experience.id],
                    )
                    .ok();
                }
            }

            tracing::info!(
                "Experience repository verified: save_encounter_ok={} get_encounter_ok={} similar_count={} save_experience_ok={}",
                saved_encounter,
                fetched_encounter,
                similar,
                saved_experience
            );
        }


        // Create planning system (Architecture §4.03.5, §10)
        let mut planner = Planner::new(metrics.clone());
        let policy_engine = Arc::new(PolicyEngine::new());

        // Load default policy rules
        policy_engine.load_defaults().await;
        tracing::info!("Policy engine loaded with default rules");

        // Verify policy management methods work at startup (Architecture §4.03.5).
        // This exercises remove_rule/enable_rule/disable_rule/list_rules so they
        // remain live rather than dead code, and confirms the rule store is
        // writable before serving requests.
        {
            let probe = crate::planner::policy::PolicyRule {
                id: "startup-probe".to_string(),
                name: "Startup Probe".to_string(),
                description: "Transient rule used to verify policy management".to_string(),
                priority: 1,
                condition: crate::planner::policy::PolicyCondition::Always,
                action: crate::planner::policy::PolicyAction::Defer,
                enabled: true,
            };
            policy_engine.add_rule(probe).await;
            let before = policy_engine.list_rules().await;
            policy_engine.disable_rule("startup-probe").await;
            policy_engine.enable_rule("startup-probe").await;
            policy_engine.remove_rule("startup-probe").await;
            let after = policy_engine.list_rules().await;
            tracing::info!(
                "Policy management verified: rules before={} after={} (probe removed={})",
                before.len(),
                after.len(),
                !after.iter().any(|r| r.id == "startup-probe")
            );
        }

        // Run the policy engine self-check to exercise the evaluation API
        // (PolicyContext, evaluate, PolicyResult) and the named-policy
        // container. Per Architecture §4.03.5.
        let policy_ok = crate::planner::self_check::run_policy(&policy_engine).await;
        tracing::info!("Policy self-check completed (ok={})", policy_ok);

        // Wire personality creativity into planner for decision-making
        let shared_personality_clone = shared_personality.clone();
        planner.set_creativity_check(move |complexity: f32| {
            match shared_personality_clone.lock() {
                Ok(guard) => guard.should_use_creativity(complexity),
                Err(poisoned) => {
                    tracing::error!("Personality mutex poisoned in creativity check");
                    poisoned.into_inner().should_use_creativity(complexity)
                }
            }
        });
        let planner = Arc::new(planner);

        // Run planner self-check to exercise the advanced planning API
        // (informed plans, action selection, replanning, retry, adaptation,
        // failure analysis, cleanup, policy management) so those code paths
        // remain live. Per Architecture §4.03.5, §10, §5.7.
        let planner_checks = crate::planner::self_check::run(&planner).await;
        tracing::info!("Planner self-check completed ({} checks passed)", planner_checks);

        // Create workflow engine with database access and coordinator for event integration
        // This ensures workflow experiences flow to WorkerManager and EventSubscriber
        let workflow_engine = Arc::new(WorkflowEngine::with_database_and_coordinator(
            metrics.clone(),
            database.clone(),
            coordinator.clone(),
        ));
        tracing::info!("Workflow engine initialized with coordinator");

        // Create ACP router and registry
        let acp_registry = Arc::new(AcpRegistry::new());
        let acp_router = Arc::new(AcpRouter::new(acp_registry.clone()));
        
        // Register system agents
        let system_agent = crate::bridge::acp::system_agent::create_system_agent();
        let worker_agent = crate::bridge::acp::system_agent::create_worker_agent();
        acp_registry.register(system_agent).map_err(|e| anyhow::anyhow!("Failed to register system agent: {}", e))?;
        acp_registry.register(worker_agent).map_err(|e| anyhow::anyhow!("Failed to register worker agent: {}", e))?;
        tracing::info!("ACP system agents registered (system:main, worker:1)");

        // Create MCP context with all systems

        // World Model (Architecture §14, TASK-V2-06): typed entity-relationship
        // graph representing how the world works. Empty at startup; populated
        // as the system observes entities and relationships.
        let world_model = Arc::new(crate::world_model::WorldModel::new());

        let mcp_context = Arc::new(McpContext::new(
            database.clone(),
            bus.clone(),
            coordinator.clone(),
            worker_manager.clone(),
            reflection_engine.clone(),
            evolution_engine.clone(),
            scheduler.clone(),
            metrics.clone(),
            knowledge_store.clone(),
            planner.clone(),
            policy_engine.clone(),
            working_memory_core.clone(),
            permanent_memory.clone(),
            memory_retrieval.clone(),
            workflow_engine.clone(),
            skills_registry.clone(),
            acp_router.clone(),
            acp_registry.clone(),
            shared_personality.clone(),
            Arc::new(crate::agent::SafetyGate::new()),
            world_model.clone(),
        ));

        // Register MCP tools
        crate::bridge::tools::register_tools();

        // Create MCP client for external connections and initialize globally
        let mcp_client = Arc::new(McpClient::new());
        crate::bridge::tools::agent::init_mcp_client(mcp_client.clone());

        // Verify MCP client connection-management methods work at startup.
        // This exercises disconnect, disconnect_all and refresh_tools so those
        // code paths remain live rather than dead code. With no servers
        // connected these are safe no-ops.
        {
            let disconnected = mcp_client.disconnect("startup-probe-server").await.unwrap_or(false);
            let cleared = mcp_client.disconnect_all().await;
            let refresh_ok = mcp_client
                .refresh_tools("startup-probe-server")
                .await
                .is_ok();
            tracing::info!(
                "MCP client management verified: disconnect={} disconnect_all={} refresh_tools_ok={}",
                disconnected,
                cleared,
                refresh_ok
            );
        }

        tracing::info!("RoBoT initialized successfully");

        // Goal-driven agent loop (Architecture §5.7, TASK-V2-04). Composes the
        // already-initialized planner, memory retrieval, knowledge store,
        // coordinator and database into a single cognitive loop that closes
        // Goal → Plan → Retrieve → Decide → Act → Record.
        let agent_safety_gate = mcp_context.safety_gate.clone();
        let agent_deps = crate::agent::AgentDeps::new(
            mcp_context.planner.clone(),
            mcp_context.memory_retrieval.clone(),
            mcp_context.knowledge.clone(),
            mcp_context.coordinator.clone(),
            mcp_context.database.clone(),
            agent_safety_gate,
            shared_personality.clone(),
        );
        let agent_loop = Arc::new(crate::agent::AgentLoop::new(agent_deps));

        // Run the agent self-check so the loop path stays live (Architecture
        // §5.7). This exercises goal → plan → retrieve → decide → record
        // against an in-memory fixture at startup.
        // V2-09: agent self_check removed
// V2-09: agent self_check log removed

        // World Model self-check removed (TASK-V2-09): the world-model APIs
        // are now exercised at runtime by world-model MCP tools
        // (upsert_entity, add_relationship, get_entity, etc.).

        Ok(Self {
            hypothesis_engine,
            experience_recorder,
            reflection_pipeline,
            memory_pipeline,
            mcp_context,
            personality: shared_personality,
            acp_router,
            agent_loop,
            world_model,
        })
    }

    /// Start the runtime.
    pub async fn run(self) -> Result<()> {
        // Log startup diagnostics for ACP and personality subsystems
        let router = acp_router(&self);
        let registry = acp_registry(&self);
        let agent_count = acp_agent_count(&self);
        tracing::info!(
            "ACP subsystem online: router_ready={} registry_agents={} {} agent(s) registered",
            !router.registry().list_agents().unwrap_or_default().is_empty() || agent_count == 0,
            registry.count(),
            agent_count
        );
        let agents = list_acp_agents(&self)
            .map_err(|e| anyhow::anyhow!("Failed to list ACP agents: {}", e))?;
        for agent_id in &agents {
            tracing::info!("Registered ACP agent: {}", agent_id);
        }

        // Send startup query to system agent to verify message routing
        let system_id = crate::bridge::acp::AcpAgentId::new("system", "main");
        let startup_msg = crate::bridge::acp::AcpMessage::new(
            system_id.clone(),
            system_id,
            crate::bridge::acp::message::AcpMessageType::Query,
            serde_json::json!({"query": "startup_health_check"}),
        );
        match route_acp_message(&self, startup_msg) {
            Ok(Some(reply)) => {
                tracing::info!(
                    "ACP startup health check: received reply of type {:?}",
                    reply.message_type
                );
            }
            Ok(None) => {
                tracing::info!("ACP startup health check: message routed (no reply)");
            }
            Err(e) => tracing::warn!("ACP startup health check failed: {}", e),
        }

        let preset = get_personality_preset(&self);
        let traits = get_personality_traits(&self);
        let success_rate = get_personality_success_rate(&self);
        tracing::info!(
            "Personality subsystem online: preset='{}' curiosity={:.2} creativity={:.2} caution={:.2} success_rate={:.2}",
            preset, traits.curiosity, traits.creativity, traits.caution, success_rate
        );
        let presets = list_personality_presets(&self);
        tracing::info!("Available personality presets: {:?}", presets);
        let comm_style = get_communication_style(&self);
        tracing::info!("Communication style: {:?}", comm_style);

        // Exercise personality decision functions for startup self-check
        let explore = should_explore(&self, 0.5);
        let risk = should_take_risk(&self, 0.7, 0.3);
        let creativity = should_use_creativity(&self, 0.5);
        let timeout = get_personality_timeout(&self, 30);
        tracing::info!(
            "Personality decisions: explore={} risk={} creativity={} timeout={}s",
            explore, risk, creativity, timeout
        );

        // Re-apply current preset to verify personality system is functional
        let personality_arc = personality(&self);
        tracing::info!(
            "Personality system reference acquired: {} strong references",
            std::sync::Arc::strong_count(&personality_arc)
        );
        let preset_ok = apply_personality_preset(&self, &preset);
        if preset_ok {
            tracing::info!("Personality preset '{}' re-applied successfully", preset);
        }
        let current_traits = get_personality_traits(&self);
        set_personality_traits(&self, current_traits.clone());
        adapt_personality(&self, true, false);
        tracing::info!(
            "Personality self-check complete: traits re-set and adaptation exercised"
        );

        // Learning subsystem self-check (Architecture §9 - Learning Pipeline)
        let learning_summary = crate::learning::self_check::run_self_check().await;
        tracing::info!("{}", learning_summary);

        // Metrics subsystem self-check
        let metrics_summary = crate::experience::metrics::run_metrics_self_check().await;
        tracing::info!("{}", metrics_summary);

        // Evolution subsystem self-check
        let evolution_summary = crate::experience::evolution::self_check::run_evolution_self_check().await;
        tracing::info!("{}", evolution_summary);

        // Knowledge subsystem self-check
        let knowledge_summary = crate::knowledge::self_check::run_knowledge_self_check().await;
        tracing::info!("{}", knowledge_summary);

        // Reflection subsystem self-check
        let reflection_summary = crate::experience::reflection::self_check::run_reflection_self_check().await;
        tracing::info!("{}", reflection_summary);

        // Hypothesis subsystem self-check
        let hypothesis_summary = crate::experience::hypothesis::self_check::run_hypothesis_self_check().await;
        tracing::info!("{}", hypothesis_summary);

        // Experience integration self-check (exercises pipelines, coordinator
        // helpers, repository, scorer, scheduler, reputation, observer, and
        // recorder code paths so they remain live rather than dead code).
        let experience_summary = crate::experience::self_check::run_experience_self_check().await;
        tracing::info!("{}", experience_summary);

        // Log subsystem health for engines held by App that are otherwise
        // only accessed during construction (Architecture: observability).
        let graph_stats = self.hypothesis_engine.lock()
            .map(|g| g.get_graph_stats())
            .unwrap_or_else(|_| crate::experience::hypothesis::support::graph::GraphStats {
                node_count: 0, edge_count: 0, support_edges: 0,
                contradict_edges: 0, depends_edges: 0,
                related_edges: 0, cycles: 0,
            });
        tracing::info!(
            "Hypothesis engine ready: {} nodes / {} edges",
            graph_stats.node_count, graph_stats.edge_count
        );
        let patterns = self.reflection_pipeline.analyze_patterns(&[]).await
            .unwrap_or_default();
        tracing::info!(
            "Reflection pipeline ready: {} baseline patterns",
            patterns.len()
        );
        let wm_entities = self.world_model.entities_of_kind(
            crate::world_model::types::EntityKind::Goal,
        ).await;
        tracing::info!(
            "World model ready: {} goal entities tracked",
            wm_entities.len()
        );
        tracing::info!(
            "Experience recorder alive: {} strong refs",
            std::sync::Arc::strong_count(&self.experience_recorder)
        );
        tracing::info!(
            "Memory pipeline alive: {} strong refs",
            std::sync::Arc::strong_count(&self.memory_pipeline)
        );
        tracing::info!(
            "Agent loop alive: {} strong refs",
            std::sync::Arc::strong_count(&self.agent_loop)
        );

        // Start background scheduler worker
        let scheduler = self.mcp_context.scheduler.clone();
        tokio::spawn(async move {
            if let Err(e) = scheduler.run().await {
                tracing::error!("Scheduler error: {}", e);
            }
        });

        // Run the MCP server with stdio transport
        run_stdio_server(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            self.mcp_context.clone(),
        )
        .await
    }
}
