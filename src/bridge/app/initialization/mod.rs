// src/bridge/app/initialization/mod.rs
//! Application initialization logic

pub mod candidates;
pub mod core;
pub mod db;
pub mod engines;
pub mod experience_repo;
pub mod exploration_repo;
pub mod hypothesis_manager;
pub mod job_queue;
pub mod learning;
pub mod learning_pipeline;
pub mod lineage_tracker;
pub mod memory_scheduler;
pub mod workers;
pub mod working_memory;

use std::sync::Arc;

use anyhow::Result;

use crate::bridge::acp::{AcpRegistry, AcpRouter};
use crate::bridge::mcp::{McpClient, McpContext};
use crate::bridge::rmcp::run_stdio_server;

use crate::experience::integration::event_subscriber::start_event_subscriber;

use super::{
    acp_agent_count, acp_registry, acp_router, adapt_personality, apply_personality_preset,
    get_communication_style, get_personality_preset, get_personality_success_rate,
    get_personality_timeout, get_personality_traits, list_acp_agents, list_personality_presets,
    personality, route_acp_message, set_personality_traits, should_explore, should_take_risk,
    should_use_creativity,
};
use crate::planner::{Planner, PolicyEngine};
use crate::workflows::engine::WorkflowEngine;

use super::state::App;

impl App {
    /// Build the application.
    pub async fn new() -> Result<Self> {
        // Initialize database
        let database = crate::bridge::app::initialization::db::init_database()?;

        // Build core infrastructure (personality, bus, coordinator, recorder, queue, workers)
        let (core, event_handler) =
            crate::bridge::app::initialization::core::build_core(database.clone());
        tracing::info!(
            "Event handler running: {} bus subscriber(s)",
            event_handler.subscriber_count()
        );
        let shared_personality = core.shared_personality.clone();
        let bus = core.bus.clone();
        // Rebind the core MetricsCollector so the CoreInfra field stays live;
        // it is the same collector the ExperienceCoordinator records into.
        let core_metrics = core.metrics.clone();
        let experience_recorder = core.experience_recorder.clone();
        let coordinator = core.coordinator;
        let job_queue = core.job_queue.clone();
        let worker_manager = core.worker_manager.clone();
        // Rebind to the Arc held by CoreInfra so the field stays live and both
        // halves of initialization share the exact same database handle.
        let database = core.database;
        tracing::info!("Core infrastructure ready: coordinator shares core metrics collector");
        core_metrics.set_gauge_sync("system.startup.core_infra_ready", 1.0);

        // Build learning engines (reflection, hypothesis, evolution, metrics)
        let engines = crate::bridge::app::initialization::engines::build_engines();
        let reflection_engine = engines.reflection_engine;
        let hypothesis_engine_for_subscriber = engines.hypothesis_engine_for_subscriber;
        let hypothesis_engine = engines.hypothesis_engine;
        let evolution_engine = engines.evolution_engine;
        let metrics = engines.metrics;

        // Register all observers and dispatch restored jobs
        crate::bridge::app::initialization::workers::setup_workers(
            &worker_manager,
            &bus,
            &metrics,
            &hypothesis_engine,
        )
        .await?;

        // Verify the durable JobQueue at startup
        crate::bridge::app::initialization::job_queue::verify_job_queue(&job_queue, &database);

        // Verify CandidateGenerator lifecycle at startup
        crate::bridge::app::initialization::candidates::verify_candidates().await;

        // Verify WorkingMemory lifecycle at startup
        crate::bridge::app::initialization::working_memory::verify_working_memory().await;

        // Verify learning subsystems (LineageTracker, HypothesisManager, LearningPipeline)
        lineage_tracker::verify_lineage_tracker().await;

        // Verify HypothesisManager lifecycle
        hypothesis_manager::verify_hypothesis_manager().await;

        // Verify LearningPipeline coordinator
        learning_pipeline::verify_learning_pipeline().await;

        // Build knowledge/skills/coordinator/event-subscriber/reflection pipeline
        let learning = crate::bridge::app::initialization::learning::build_learning_pipeline(
            &database,
            &reflection_engine,
            &hypothesis_engine_for_subscriber,
            &evolution_engine,
            &bus,
            &metrics,
            &experience_recorder,
        )
        .await;
        let knowledge_store = learning.knowledge_store;
        let skills_registry = learning.skills_registry;
        let learning_coordinator = learning.learning_coordinator;
        let event_subscriber = learning.event_subscriber;
        let reflection_pipeline = learning.reflection_pipeline;
        start_event_subscriber(bus.clone(), event_subscriber);
        tracing::info!("Event subscriber started for learning pipeline");

        // Build memory system and scheduler
        let mem_sched =
            crate::bridge::app::initialization::memory_scheduler::build_memory_scheduler(
                &database,
                &knowledge_store,
                &learning_coordinator,
                &reflection_engine,
                &hypothesis_engine,
                &evolution_engine,
                &metrics,
            )
            .await?;
        let working_memory_core = mem_sched.working_memory_core;
        let permanent_memory = mem_sched.permanent_memory;
        let memory_retrieval = mem_sched.memory_retrieval;
        let memory_pipeline = mem_sched.memory_pipeline;
        let scheduler = mem_sched.scheduler;

        // Verify experience repository persistence (Architecture §07/§09)
        experience_repo::verify_experience_repository(&database).await;

        // Verify ExplorationRepository (Architecture §4.06)
        exploration_repo::verify_exploration_repository().await;

        // Create planning system (Architecture §4.03.5, §10)
        let mut planner = Planner::new(metrics.collector());
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

            // Exercise the current-policy package accessors so they stay live:
            // read via the RwLock accessor and the async snapshot getter.
            let package_snapshot = policy_engine.current_policy().read().await.clone();
            let package_get = policy_engine.get_policy().await;
            tracing::info!(
                "Policy package verified: id={} version={} rules_snapshot={} rules_get={} \
                 consistent={}",
                package_get.id,
                package_get.version,
                package_snapshot.rules.len(),
                package_get.rules.len(),
                package_snapshot.id == package_get.id
                    && package_snapshot.rules.len() == package_get.rules.len()
            );
        }

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
        // Wire policy engine into planner for policy-based action gating
        planner.set_policy_engine(policy_engine.clone());
        let planner = Arc::new(planner);

        // Create workflow engine with database access and coordinator for event integration
        // This ensures workflow experiences flow to WorkerManager and EventSubscriber
        let workflow_engine = Arc::new(WorkflowEngine::with_database_and_coordinator(
            metrics.collector(),
            database.clone(),
            coordinator.clone(),
        ));
        tracing::info!("Workflow engine initialized with coordinator");

        // Create ACP router and registry
        let acp_registry = Arc::new(AcpRegistry::new());
        let acp_router = Arc::new(AcpRouter::new(acp_registry.clone()));

        // Register a default Inform broadcast handler so broadcast-style ACP
        // messages are observed even when no agent-specific handler exists.
        acp_router
            .register_handler(crate::bridge::acp::message::AcpMessageType::Inform, |msg| {
                tracing::info!(
                    "ACP Inform broadcast received from {}: {}",
                    msg.sender,
                    msg.payload
                );
                Ok(None)
            })
            .map_err(|e| anyhow::anyhow!("Failed to register ACP Inform handler: {}", e))?;

        // Register system agents
        let system_agent = crate::bridge::acp::system_agent::create_system_agent();
        let worker_agent = crate::bridge::acp::system_agent::create_worker_agent();
        acp_registry
            .register(system_agent)
            .map_err(|e| anyhow::anyhow!("Failed to register system agent: {}", e))?;
        acp_registry
            .register(worker_agent)
            .map_err(|e| anyhow::anyhow!("Failed to register worker agent: {}", e))?;
        tracing::info!("ACP system agents registered (system:main, worker:1)");

        // Create MCP context with all systems

        // World Model (Architecture §14, TASK-V2-06): typed entity-relationship
        // graph representing how the world works. Empty at startup; populated
        // as the system observes entities and relationships.
        let world_model = Arc::new(crate::world_model::WorldModel::new());

        // Workflow enforcement engine (Architecture §22 workflow gate).
        // Shared between McpContext (for admin tools) and McpServerHandler
        // (for per-request enforcement checks).
        let enforcer = Arc::new(crate::workflows::enforcement::WorkflowEnforcer::new());

        let mcp_context = Arc::new(McpContext::new(
            database.clone(),
            job_queue.clone(),
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
            enforcer.clone(),
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
            let disconnected = mcp_client
                .disconnect("startup-probe-server")
                .await
                .unwrap_or(false);
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
            mcp_context.metrics.clone(),
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
            !router
                .registry()
                .list_agents()
                .unwrap_or_default()
                .is_empty()
                || agent_count == 0,
            registry.count(),
            agent_count
        );
        let agents = list_acp_agents(&self)
            .map_err(|e| anyhow::anyhow!("Failed to list ACP agents: {}", e))?;
        for agent_id in &agents {
            tracing::info!("Registered ACP agent: {}", agent_id);
        }

        // Diagnostic: count agents by type so the registry's type-indexed
        // lookup is exercised on startup.
        let worker_agents = router
            .registry()
            .get_by_type("worker")
            .map_err(|e| anyhow::anyhow!("Failed to query ACP agents by type: {}", e))?;
        tracing::info!("ACP worker agents by type: {}", worker_agents.len());

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
            preset,
            traits.curiosity,
            traits.creativity,
            traits.caution,
            success_rate
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
            explore,
            risk,
            creativity,
            timeout
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
        tracing::info!("Personality self-check complete: traits re-set and adaptation exercised");

        // Learning subsystem self-check (Architecture §9 - Learning Pipeline)

        // Metrics subsystem self-check
        let metrics_summary = crate::experience::metrics::run_metrics_self_check().await;
        tracing::info!("{}", metrics_summary);

        // Log subsystem health for engines held by App that are otherwise
        // only accessed during construction (Architecture: observability).
        let graph_stats = self
            .hypothesis_engine
            .lock()
            .map(|g| g.get_graph_stats())
            .unwrap_or_else(
                |_| crate::experience::hypothesis::support::graph::GraphStats {
                    node_count: 0,
                    edge_count: 0,
                    support_edges: 0,
                    contradict_edges: 0,
                    depends_edges: 0,
                    related_edges: 0,
                    cycles: 0,
                },
            );
        tracing::info!(
            "Hypothesis engine ready: {} nodes / {} edges",
            graph_stats.node_count,
            graph_stats.edge_count
        );
        let patterns = self
            .reflection_pipeline
            .analyze_patterns(&[])
            .await
            .unwrap_or_default();
        tracing::info!(
            "Reflection pipeline ready: {} baseline patterns",
            patterns.len()
        );
        let wm_entities = self
            .world_model
            .entities_of_kind(crate::world_model::types::EntityKind::Goal)
            .await;
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
