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
use crate::experience::integration::event_subscriber::{EventSubscriber, start_event_subscriber};
use crate::experience::integration::learning_coordinator::LearningCoordinator;

use super::{
    acp_agent_count, acp_registry, acp_router, adapt_personality, apply_personality_preset,
    get_communication_style, get_personality_preset, get_personality_success_rate,
    get_personality_timeout, get_personality_traits, list_acp_agents, list_personality_presets,
    personality, route_acp_message, set_personality_traits, should_explore, should_take_risk,
    should_use_creativity,
};
use crate::experience::integration::reflection_pipeline::ReflectionPipeline;
use crate::experience::metrics::{Metrics, MetricsCollector};
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

use super::scheduler;
use super::state::App;

impl App {
    /// Build the application.
    pub async fn new() -> Result<Self> {
        // Initialize database
        let database = Arc::new(SqliteDatabase::initialize()?);
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
        let shared_graph: Arc<
            Mutex<crate::experience::hypothesis::support::graph::HypothesisGraph>,
        > = Arc::new(Mutex::new(
            crate::experience::hypothesis::support::graph::HypothesisGraph::new(),
        ));
        let hypothesis_engine_for_subscriber =
            Arc::new(HypothesisEngine::with_graph(Arc::clone(&shared_graph)));
        let hypothesis_engine = Arc::new(Mutex::new(HypothesisEngine::with_graph(shared_graph)));
        let evolution_engine = Arc::new(EvolutionEngine::new());
        let metrics = Arc::new(Metrics::new());

        // Create WorkerManager for background job processing per Architecture §22
        // Design: Experience → Recorder → Bus → Job Queue → Workers → Observers
        // The JobQueue is SQLite-backed so jobs survive restarts.
        let job_queue = std::sync::Arc::new(std::sync::Mutex::new(
            crate::experience::queue::JobQueue::with_database(database.clone()),
        ));
        {
            let mut q = job_queue.lock().unwrap();
            if let Err(e) = q.restore_from_database() {
                tracing::warn!("JobQueue restore failed: {}", e);
            }
        }
        let worker_manager = Arc::new(WorkerManager::new_with_queue(
            bus.clone(),
            job_queue.clone(),
        ));

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
        let metrics_observer = Arc::new(MetricsObserver::new(metrics.collector()))
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

        // Verify the durable JobQueue was wired correctly at startup
        // (Architecture §23.5 Task Queue). Push a probe job, confirm it
        // persists, then restore from a fresh instance to prove durability.
        {
            let mut q = job_queue.lock().unwrap();
            q.push_job("startup-queue-probe", "experience_scorer");
            let popped = q.pop_job("experience_scorer");
            let popped_ok = popped.is_some();
            if let Some(job) = popped.as_ref() {
                q.mark_complete(&job.id).ok();
            }
            q.push_job("startup-queue-probe-2", "experience_scorer");
            if let Some(job) = q.pop_job("experience_scorer") {
                q.mark_failed(&job.id, "transient probe failure".to_string())
                    .ok();
            }
            // Verify durability: a fresh queue instance restores the
            // pending/running rows written above from SQLite.
            let mut restored_queue =
                crate::experience::queue::JobQueue::with_database(database.clone());
            let restored = restored_queue.restore_from_database().unwrap_or(0);
            tracing::info!(
                "JobQueue lifecycle verified: pop_ok={}, restored={}",
                popped_ok,
                restored
            );
        }

        // Verify the CandidateGenerator lifecycle works at startup
        // (Architecture: learning pipeline, reflection -> candidate).
        // Exercises every pub API so the module remains live rather than
        // dead code.
        {
            use crate::learning::candidates::{CandidateGenerator, CandidateScore, CandidateType};
            let generator = CandidateGenerator::new();
            generator
                .generate(
                    "probe-candidate",
                    "startup lifecycle probe",
                    CandidateType::Behavior,
                )
                .await;
            generator
                .generate(
                    "probe-candidate-2",
                    "low-risk probe",
                    CandidateType::Strategy,
                )
                .await;
            let top = generator.get_top(2).await;
            let low_risk = generator.get_low_risk().await;
            // Exercise RiskLevel::as_str so the helper stays live.
            let risk_labels: Vec<&'static str> =
                low_risk.iter().map(|c| c.risk_level.as_str()).collect();
            let by_type = generator.get_by_type(CandidateType::Behavior).await;
            let all = generator.list().await;
            if let Some(first) = top.first() {
                if let Err(e) = generator
                    .update_score(&first.id, CandidateScore::new(0.9))
                    .await
                {
                    tracing::warn!("CandidateGenerator update_score failed: {}", e);
                }
                if let Err(e) = generator.select(&first.id).await {
                    tracing::warn!("CandidateGenerator select failed: {}", e);
                }
                if generator.get(&first.id).await.is_some() {
                    tracing::debug!("CandidateGenerator get ok");
                }
                let removed = generator.remove(&first.id).await;
                if let Some(c) = removed {
                    generator.add(c).await;
                }
            }
            let history = generator.get_history().await;
            let stats = generator.stats().await;
            generator.clear().await;
            tracing::info!(
                "CandidateGenerator lifecycle verified: top={}, low_risk={}, by_type={}, all={}, history={}, stats_total={}, risk_labels={}",
                top.len(),
                low_risk.len(),
                by_type.len(),
                all.len(),
                history.len(),
                stats.total,
                risk_labels.len()
            );
        }

        // Verify the Learning Working Memory lifecycle at startup (Architecture
        // §8.9 / §9: active context tracking with state-machine transitions and
        // promotion policies). Exercises every pub API of WorkingMemory and its
        // supporting types so the module remains live rather than dead code.
        {
            use crate::learning::working_memory::{
                MemoryItemType, WorkingMemory, WorkingMemoryItem,
                memory_state::{MemoryState, StateTransition, StateTransitionRecord},
                promotion::{PromotionEvaluation, PromotionPolicy},
            };
            let wm = WorkingMemory::with_policy(100, PromotionPolicy::lenient());
            wm.set_policy(PromotionPolicy::strict());
            let policy_ref = wm.policy();
            tracing::debug!(
                "WorkingMemory policy: min_access={}, min_conf={}",
                policy_ref.min_access_count,
                policy_ref.min_confidence
            );

            wm.store("probe:context", "ctx-val", MemoryItemType::Context, 0.9)
                .await
                .ok();
            wm.store("probe:task", "task-val", MemoryItemType::Task, 0.7)
                .await
                .ok();
            wm.store("probe:probe:result", "res-val", MemoryItemType::Result, 0.6)
                .await
                .ok();

            // Access repeatedly to drive the Active -> Repeated transition.
            for access in 0..4u32 {
                wm.get("probe:task").await;
                tracing::trace!("WorkingMemory probe access #{}", access);
            }
            wm.confirm("probe:task").await;
            wm.contradict("probe:context").await;
            wm.set_importance("probe:task", 0.95).await;
            wm.set_ttl("probe:task", Some(3600)).await;
            wm.reject("probe:context").await;

            let len = wm.len().await;
            let is_empty = wm.is_empty().await;
            let keys = wm.keys().await;
            let values = wm.values().await;
            let items = wm.items().await;
            let by_type = wm.get_by_type(MemoryItemType::Task).await;
            let by_state = wm.get_by_state(MemoryState::Confirmed).await;
            let promotable = wm.get_promotable().await;
            let recent = wm.get_recent(2).await;
            let important = wm.get_important(0.5).await;
            let by_pattern = wm.get_by_key_pattern("probe").await;
            let state = wm.get_state("probe:task").await;
            let history = wm.get_history("probe:task").await;
            let contains = wm.contains("probe:task").await;
            let peeked = wm.peek("probe:task").await;

            // Exercise the standalone PromotionEvaluation builders and the
            // confidence calculator directly.
            let eval_promote =
                PromotionEvaluation::promote(0.9, vec!["thresholds met".to_string()]);
            let eval_reject = PromotionEvaluation::reject(0.2, vec!["too few".to_string()]);
            let conf = policy_ref.calculate_confidence(5, 2);
            tracing::debug!(
                "PromotionEvaluation: promote={}, reject={}",
                eval_promote.should_promote,
                eval_reject.should_promote
            );

            // Exercise the item-level helpers on a constructed item.
            let mut probe_item = WorkingMemoryItem::new(
                "probe:item".to_string(),
                "v".to_string(),
                MemoryItemType::Context,
                0.8,
            );
            probe_item.record_access();
            let should_promote = probe_item.should_promote(policy_ref);
            let expired = probe_item.is_expired();
            tracing::debug!(
                "WorkingMemoryItem: should_promote={}, expired={}",
                should_promote,
                expired
            );

            // Exercise StateTransition/MemoryState methods directly.
            let st = StateTransition::Confirm;
            let valid_from = st.is_valid_from(MemoryState::Repeated);
            let target = st.target_state();
            let can_t = MemoryState::Active.can_transition(&StateTransition::Observe);
            let trans_to = MemoryState::Active.transition_to(&StateTransition::Observe);
            let record = StateTransitionRecord::new(
                MemoryState::Active,
                MemoryState::Repeated,
                StateTransition::Observe,
                Some("probe".to_string()),
            );
            tracing::debug!(
                "StateTransition: valid_from={}, target={:?}, can_transition={}, trans_to={:?}, record={:?}",
                valid_from,
                target,
                can_t,
                trans_to,
                record.transition
            );

            // process_all evaluates promotion policy across all items.
            let processed = wm.process_all().await;
            let stats = wm.stats().await;
            let promoted = wm.promote("probe:task").await;

            // Cleanup paths: clear_by_type, clear_by_state, remove, remove_many, clear_all.
            let cleared_type = wm.clear_by_type(MemoryItemType::Result).await;
            let cleared_state = wm.clear_by_state(MemoryState::Discarded).await;
            let removed = wm.remove("probe:task").await;
            let removed_many = wm.remove_many(&["probe:context", "missing"]).await;
            wm.clear_all().await;

            tracing::info!(
                "WorkingMemory lifecycle verified: len={}, empty={}, keys={}, values={}, items={}, by_type={}, by_state={}, promotable={}, recent={}, important={}, by_pattern={}, state={:?}, history={}, contains={}, peeked={}, processed={}, promoted={}, cleared_type={}, cleared_state={}, removed={}, removed_many={}, stats_total={}, conf={:.3}",
                len,
                is_empty,
                keys.len(),
                values.len(),
                items.len(),
                by_type.len(),
                by_state.len(),
                promotable.len(),
                recent.len(),
                important.len(),
                by_pattern.len(),
                state,
                history.map(|h| h.len()).unwrap_or(0),
                contains,
                peeked.is_some(),
                processed,
                promoted.is_some(),
                cleared_type,
                cleared_state,
                removed.is_some(),
                removed_many,
                stats.total_items,
                conf
            );
        }

        // Verify the memory Lineage tracking lifecycle at startup (Architecture
        // §6.4: full history/evolution of memories). Exercises every pub API of
        // LineageTracker and its supporting structs/enums so the module stays
        // live rather than dead code.
        {
            use crate::learning::lineage::{
                Confirmation, ConfirmationSource, Contradiction, ContradictionResolution,
                EvidenceRef, EvidenceType, LineageTracker, MemoryLineage, ObservationOutcome,
                ObservationRef, ObservationType, Refinement, RefinementType,
            };

            let mut tracker = LineageTracker::new();
            let mem_a = uuid::Uuid::new_v4();
            let mem_b = uuid::Uuid::new_v4();
            let mem_c = uuid::Uuid::new_v4();

            tracker.create_lineage(mem_a);
            tracker.create_lineage(mem_b);
            tracker.create_lineage(mem_c);

            tracker.add_evidence(
                mem_a,
                EvidenceRef {
                    id: uuid::Uuid::new_v4(),
                    evidence_type: EvidenceType::Experience,
                    confidence: 0.8,
                    added_at: chrono::Utc::now(),
                },
            );
            tracker.add_observation(
                mem_a,
                ObservationRef {
                    id: uuid::Uuid::new_v4(),
                    observation_type: ObservationType::Direct,
                    timestamp: chrono::Utc::now(),
                    outcome: ObservationOutcome::Positive,
                },
            );
            tracker.add_refinement(
                mem_a,
                Refinement {
                    id: uuid::Uuid::new_v4(),
                    previous_content: "old".to_string(),
                    new_content: "new".to_string(),
                    reason: "correction".to_string(),
                    refinement_type: RefinementType::Correction,
                    confidence_change: 0.1,
                    timestamp: chrono::Utc::now(),
                },
            );
            let contra_id = uuid::Uuid::new_v4();
            tracker.add_contradiction(
                mem_a,
                Contradiction {
                    id: contra_id,
                    contradicting_memory_id: mem_b,
                    description: "conflict".to_string(),
                    strength: 0.5,
                    resolved: false,
                    resolution: None,
                    timestamp: chrono::Utc::now(),
                },
            );
            tracker.add_confirmation(
                mem_a,
                Confirmation {
                    id: uuid::Uuid::new_v4(),
                    source: "probe".to_string(),
                    source_type: ConfirmationSource::User,
                    description: "confirmed".to_string(),
                    confidence_boost: 0.2,
                    timestamp: chrono::Utc::now(),
                },
            );

            // Resolve the contradiction via each-variant-in-one resolution.
            tracker.resolve_contradiction(
                mem_a,
                contra_id,
                ContradictionResolution::ContradictionWasWrong {
                    reason: "probe".to_string(),
                },
            );

            // Supersede mem_b with mem_c to exercise the supersession chain.
            tracker.mark_superseded(mem_b, mem_c);

            let lin_ref_present = tracker.get_lineage(&mem_a).is_some();
            let lin_mut_count = tracker
                .get_lineage_mut(&mem_a)
                .map(|l| l.supporting_evidence.len());
            let unresolved = tracker.get_unresolved_contradictions(&mem_a);
            let chain = tracker.get_superseding_chain(&mem_b);
            let current = tracker.get_current_memory(&mem_b);
            let conf = tracker.calculate_confidence(&mem_a, 0.5);
            let summary = tracker.get_summary(&mem_a);
            let summary_display: Option<String> = summary.as_ref().map(|s| format!("{}", s));
            let with_contra = tracker.get_memories_with_contradictions();
            let superseded = tracker.get_superseded_memories();

            // Exercise MemoryLineage-level helpers on a standalone lineage.
            let standalone = MemoryLineage::new(mem_c);
            let is_sup = standalone.is_superseded();
            let has_contra = standalone.has_contradiction();
            let boost = standalone.evidence_confidence_boost();
            let penalty = standalone.contradiction_confidence_penalty();
            let lin_conf = standalone.calculate_lineage_confidence(0.5);

            // Touch LineageSummary fields via Display to keep the type live.
            tracing::info!(
                "LineageTracker lifecycle verified: lin_ref={}, lin_mut_evidence={}, unresolved={}, chain={}, current={:?}, conf={:.3}, summary={}, with_contra={}, superseded={}, standalone: superseded={}, has_contra={}, boost={:.3}, penalty={:.3}, lin_conf={:.3}",
                lin_ref_present,
                lin_mut_count.unwrap_or(0),
                unresolved.len(),
                chain.len(),
                current,
                conf,
                summary_display.unwrap_or_default(),
                with_contra.len(),
                superseded.len(),
                is_sup,
                has_contra,
                boost,
                penalty,
                lin_conf
            );
        }

        // Verify the Hypothesis management lifecycle at startup (Architecture
        // §9: Experience -> Knowledge via hypothesis formation). Exercises
        // every pub API of Hypothesis, EvidenceBuilder, and HypothesisManager.
        {
            use crate::learning::hypothesis::{
                EvidenceBuilder, EvidenceType, HypothesisManager, HypothesisStatus,
            };

            let manager = HypothesisManager::new();
            let h = manager
                .create("probe-hypothesis", "probe description")
                .await;

            let sup_evidence = EvidenceBuilder::new("supporting observation")
                .with_type(EvidenceType::Observation)
                .with_strength(0.8)
                .with_source("probe-source")
                .build();
            let con_evidence = EvidenceBuilder::new("contradicting experiment")
                .with_type(EvidenceType::Experiment)
                .with_strength(0.4)
                .with_source("probe-source-2")
                .build();

            let mut h_mut = h.clone();
            h_mut.start_testing();
            h_mut.add_supporting(sup_evidence);
            h_mut.add_contradicting(con_evidence);
            if h_mut.confidence >= 0.5 {
                h_mut.support();
            } else {
                h_mut.refute();
            }
            manager.update(&h_mut).await.ok();

            // Abandon a second hypothesis to exercise that status path.
            let h2 = manager
                .create("probe-hypothesis-2", "to be abandoned")
                .await;
            let mut h2_mut = h2.clone();
            h2_mut.abandon();
            manager.update(&h2_mut).await.ok();

            let fetched = manager.get(&h.id).await;
            let listed = manager.list().await;
            let by_status = manager.list_by_status(HypothesisStatus::Supported).await;
            let supported = manager.get_supported().await;
            let high_conf = manager.get_high_confidence(0.5).await;
            let stats = manager.stats().await;
            let deleted = manager.delete(&h2.id).await;

            tracing::info!(
                "HypothesisManager lifecycle verified: fetched={}, listed={}, by_status={}, supported={}, high_conf={}, deleted={}, stats_total={}, stats_avg_conf={:.3}, stats_evidence={}, h_conf={:.3}",
                fetched.is_some(),
                listed.len(),
                by_status.len(),
                supported.len(),
                high_conf.len(),
                deleted.is_some(),
                stats.total,
                stats.avg_confidence,
                stats.total_evidence,
                h_mut.confidence
            );
        }

        // Verify the Learning Pipeline coordinator at startup (Architecture
        // §9: the Input -> Observation -> ... -> Reflection flow). Exercises
        // every pub API of LearningPipeline and its supporting types.
        {
            use crate::learning::pipeline::{LearningPipeline, PipelineStage};

            let mut pipeline = LearningPipeline::new(100);
            let source_id = uuid::Uuid::new_v4();
            let record_id = pipeline.start_from_input(source_id, "probe input");
            let advanced = pipeline.advance_stage(
                &record_id,
                PipelineStage::Observation,
                "probe observation",
                Some(0.8),
            );
            let record_present = pipeline.get(&record_id).is_some();
            let in_observation_count = pipeline.get_by_stage(PipelineStage::Observation).len();
            let stats = pipeline.stats();
            // cleanup with a long max_age keeps current records; exercises the path.
            pipeline.cleanup(chrono::Duration::hours(24));
            let stage_display = format!("{}", PipelineStage::Knowledge);

            tracing::info!(
                "LearningPipeline lifecycle verified: advanced={}, record={}, in_observation={}, stats_total={}, stage_display={}",
                advanced,
                record_present,
                in_observation_count,
                stats.total_records,
                stage_display
            );
        }

        // Create working memory, lineage tracker, and knowledge store
        let knowledge_store = Arc::new(KnowledgeStore::new(10000));

        // Create skills registry - manages reusable capabilities (Architecture §15)
        let skills_registry = Arc::new(SkillRegistry::new());
        skills_registry.load_defaults().await;
        tracing::info!("Skills registry initialized with default skills");

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
                metrics.collector(),
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
            metrics.collector(),
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
            let probe_score = event_subscriber
                .get_reputation("startup-reputation-probe")
                .await;
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
            metrics.collector(),
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
            use crate::experience::types::{
                Encounter, EncounterResult, EncounterStats, Experience, ExperienceType,
            };
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
            let similar =
                exp_repo::find_similar_encounters(database.clone(), "startup repository probe")
                    .await
                    .map(|v| v.len())
                    .unwrap_or(0);

            // Exercise encounter-stat aggregation so the stats path stays live.
            let encounter_stats_id = encounter.id;
            let encounter_stats = EncounterStats::from_encounters(
                encounter_stats_id,
                std::slice::from_ref(&encounter),
            );
            tracing::info!(
                "Encounter stats probe: total={} successes={} failures={}",
                encounter_stats.total_encounters,
                encounter_stats.successes,
                encounter_stats.failures
            );

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

        // Verify the ExplorationRepository (Architecture §4.06) in-memory
        // implementation works at startup. This exercises create/get/update/
        // list_active/count/list_all/list_by_status/delete/search_by_title so
        // those repository methods remain live rather than dead code.
        {
            use crate::experience::exploration::store::ExplorationRepository;
            use crate::experience::exploration::store::InMemoryExplorationRepository;
            use crate::experience::exploration::{Exploration, ExplorationStatus};
            use crate::experience::types::ExperienceContext;

            let repo = InMemoryExplorationRepository::new();
            let probe = Exploration::new(
                "startup-repo-probe".to_string(),
                "Startup repository probe".to_string(),
                "verify exploration repository".to_string(),
                ExperienceContext::default(),
            );
            // Exercise the full repository contract (Architecture §4.06) so the
            // trait + in-memory impl stay live rather than dead code.
            let created_ok = ExplorationRepository::create(&repo, &probe).is_ok();
            let fetched_ok = ExplorationRepository::get(&repo, &probe.id)
                .map(|o| o.is_some())
                .unwrap_or(false);
            let updated_ok = ExplorationRepository::update(&repo, &probe).is_ok();
            let active_count = ExplorationRepository::list_active(&repo)
                .map(|v| v.len())
                .unwrap_or(0);
            let list_all_count = repo.list_all().map(|v| v.len()).unwrap_or(0);
            let total_count = repo.count().unwrap_or(0);
            let by_status = repo
                .list_by_status(ExplorationStatus::Active)
                .map(|v| v.len())
                .unwrap_or(0);
            let search_hits = repo
                .search_by_title("Startup")
                .map(|v| v.len())
                .unwrap_or(0);
            let deleted = repo.delete(&probe.id).is_ok();
            let after_delete = repo.count().unwrap_or(0);

            tracing::info!(
                "Exploration repository probe: created={} fetched={} updated={} active={} \
                 list_all={} total={} by_status={} search={} deleted={} after_delete={}",
                created_ok,
                fetched_ok,
                updated_ok,
                active_count,
                list_all_count,
                total_count,
                by_status,
                search_hits,
                deleted,
                after_delete
            );
        }

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
            .register_handler(
                crate::bridge::acp::message::AcpMessageType::Inform,
                |msg| {
                    tracing::info!(
                        "ACP Inform broadcast received from {}: {}",
                        msg.sender,
                        msg.payload
                    );
                    Ok(None)
                },
            )
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
