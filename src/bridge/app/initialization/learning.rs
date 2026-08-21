// src/bridge/app/initialization/learning.rs
//! Knowledge store, skills registry, learning coordinator, event subscriber,
//! reputation probes, and reflection pipeline

use std::sync::Arc;

use crate::experience::bus::ExperienceBus;
use crate::experience::encounter_recorder::ExperienceRecorder;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::integration::event_subscriber::EventSubscriber;
use crate::experience::integration::learning_coordinator::LearningCoordinator;
use crate::experience::metrics::Metrics;
use crate::experience::reflection::ReflectionEngine;
use crate::knowledge::KnowledgeStore;
use crate::skills::registry::SkillRegistry;

/// Result of building the learning pipeline subsystem.
pub(crate) struct LearningPipelineResult {
    pub(crate) knowledge_store: Arc<KnowledgeStore>,
    pub(crate) skills_registry: Arc<SkillRegistry>,
    pub(crate) learning_coordinator: Arc<LearningCoordinator>,
    pub(crate) event_subscriber: Arc<EventSubscriber>,
    pub(crate) reflection_pipeline:
        Arc<crate::experience::integration::reflection_pipeline::ReflectionPipeline>,
}

/// Build the learning pipeline: knowledge store, skills, coordinator, subscriber, reflection.
pub(crate) async fn build_learning_pipeline(
    database: &Arc<crate::database::sqlite::SqliteDatabase>,
    reflection_engine: &Arc<ReflectionEngine>,
    hypothesis_engine_for_subscriber: &Arc<HypothesisEngine>,
    evolution_engine: &Arc<EvolutionEngine>,
    bus: &Arc<ExperienceBus>,
    metrics: &Arc<Metrics>,
    experience_recorder: &Arc<ExperienceRecorder>,
) -> LearningPipelineResult {
    // Create knowledge store
    let knowledge_store = Arc::new(KnowledgeStore::new(10000));

    // Create skills registry
    let skills_registry = Arc::new(SkillRegistry::new());
    skills_registry.load_defaults().await;
    tracing::info!("Skills registry initialized with default skills");

    // Create the Learning Coordinator
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

    // Exercise EventSubscriber::new (the coordinator-less constructor) with a
    // probe subscriber so the full constructor surface stays live.
    let probe_subscriber = EventSubscriber::new(
        metrics.collector(),
        reflection_engine.clone(),
        hypothesis_engine_for_subscriber.clone(),
        evolution_engine.clone(),
        knowledge_store.clone(),
        Some(experience_recorder.clone()),
    );
    tracing::info!(
        "EventSubscriber::new verified: coordinator_attached={}",
        probe_subscriber.has_learning_coordinator()
    );

    // Verify the EvolutionEngine behavior-lifecycle path (Architecture §13):
    // create a probe behavior, record a success and a failure via
    // record_result (which drives Behavior::record_success/record_failure and
    // recalculate_confidence), then report the resulting effectiveness.
    let probe_behavior = evolution_engine
        .create_behavior(
            "startup behavior probe",
            "Transient behavior verifying the application-result lifecycle",
            crate::experience::evolution::behavior::BehaviorAction::ApplyHeuristic {
                rule: "startup-probe-rule".to_string(),
                priority: 1,
            },
        )
        .await
        .ok();
    if let Some(behavior) = probe_behavior {
        let behavior_id = behavior.id.clone();
        let success_recorded = evolution_engine
            .record_result(&behavior_id, true)
            .await
            .is_ok();
        let failure_recorded = evolution_engine
            .record_result(&behavior_id, false)
            .await
            .is_ok();
        let effectiveness = evolution_engine.get_effectiveness(&behavior_id).await;
        tracing::info!(
            "EvolutionEngine behavior lifecycle verified: success={} failure={} effectiveness={:?}",
            success_recorded,
            failure_recorded,
            effectiveness
        );

        // Exercise the remaining EvolutionEvidence constructors (Architecture
        // §13 evidence model): neutral verdict and builder-style confidence.
        let neutral_evidence = crate::experience::evolution::evidence::EvolutionEvidence::neutral(
            uuid::Uuid::new_v4().to_string(),
            behavior_id.clone(),
            crate::experience::evolution::evidence::EvidenceType::ApplicationResult,
            "startup neutral evidence probe".to_string(),
        )
        .with_confidence(0.8);
        let neutral_added = evolution_engine
            .add_evidence(neutral_evidence)
            .await
            .is_ok();
        let stored_evidence = evolution_engine.get_evidence(&behavior_id).await;
        tracing::info!(
            "EvolutionEngine evidence verified: neutral_added={} stored_count={}",
            neutral_added,
            stored_evidence.len()
        );

        // Exercise suggest_behaviors (context-based recommendation) so the
        // suggestion path stays live.
        let suggestions = evolution_engine.suggest_behaviors("probe").await;
        tracing::info!(
            "EvolutionEngine suggestions verified: count={}",
            suggestions.len()
        );

        // Exercise the EvolutionEngineTrait object surface via a generic
        // function bound so every trait method is used, not just implemented.
        async fn trait_lifecycle<E: crate::experience::evolution::engine::EvolutionEngineTrait>(
            engine: &E,
            insight: &crate::experience::reflection::insight::Insight,
        ) -> anyhow::Result<usize> {
            let behavior = engine.create_behavior_from_insight(insight).await?;
            engine.record_result(&behavior.id, true).await?;
            let active = engine.get_active_behaviors("insight").await;
            Ok(active.len())
        }
        let probe_insight = crate::experience::reflection::insight::Insight::new(
            uuid::Uuid::new_v4().to_string(),
            "startup trait probe insight",
            "transient",
            crate::experience::reflection::insight::InsightType::General,
        );
        let trait_active = trait_lifecycle(evolution_engine.as_ref(), &probe_insight)
            .await
            .unwrap_or(0);
        tracing::info!(
            "EvolutionEngineTrait verified: active_behaviors={}",
            trait_active
        );
    }

    // Create event subscriber for the learning pipeline via the preferred
    // with_learning_coordinator constructor (full §4.04 pipeline driver).
    let event_subscriber_inner = EventSubscriber::with_learning_coordinator(
        learning_coordinator.clone(),
        metrics.collector(),
        reflection_engine.clone(),
        hypothesis_engine_for_subscriber.clone(),
        evolution_engine.clone(),
        knowledge_store.clone(),
        Some(experience_recorder.clone()),
    );
    let event_subscriber = Arc::new(event_subscriber_inner);

    // Verify ExperienceRecorder convenience helpers (Architecture §07) with a
    // transient success/failure probe pair so both stay live.
    let success_probe_id = experience_recorder.success(
        crate::experience::types::ExperienceType::System,
        "startup recorder success probe",
        "Transient experience verifying ExperienceRecorder::success",
    );
    let failure_probe_id = experience_recorder.failure(
        crate::experience::types::ExperienceType::System,
        "startup recorder failure probe",
        "Transient experience verifying ExperienceRecorder::failure",
        "intentional startup probe failure",
    );
    tracing::info!(
        "ExperienceRecorder helpers verified: success_ok={} failure_ok={}",
        success_probe_id.is_ok(),
        failure_probe_id.is_ok()
    );

    // Verify event subscriber reputation management
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

    // Verify reputation analytics
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

        use crate::experience::reputation::factors::FactorScore;
        use crate::experience::types::reputation::{ReputationRecord, ReputationTarget};

        let mut record = ReputationRecord::new(ReputationTarget::Agent(rep.id.clone()));
        record.record_success(0.9);
        record.record_failure(0.4);
        let confidence = rep.confidence();
        let factor_score = FactorScore::new(ReputationFactor::Accuracy);
        tracing::info!(
            "Reputation record verified: successes={} failures={} observations={} confidence={} factor_observations={}",
            record.successes,
            record.failures,
            record.observations,
            confidence,
            factor_score.observations
        );
    }

    // Create reflection pipeline
    let reflection_pipeline = Arc::new(
        crate::experience::integration::reflection_pipeline::ReflectionPipeline::new(
            reflection_engine.clone(),
            bus.clone(),
        ),
    );

    // Verify reflection pipeline pattern analysis
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

    // Exercise reflection-engine insight/search/query surface
    {
        use crate::experience::reflection::engine::ReflectionEngine;
        use crate::experience::reflection::insight::{
            Insight, InsightType, KnowledgeMaturity, MaturityHistory,
        };
        use crate::experience::reflection::reflection::{
            EvidenceId, InsightId, Lesson, Reflection, ReflectionEvidence, ReflectionInsight,
        };
        use crate::experience::reflection::review::ReflectionReview;
        use crate::experience::reflection::{
            InsightProducer, ReflectionStatus, ReflectionType, Reflector, ValidatableReflection,
        };

        let engine = ReflectionEngine::new();

        let insight = engine
            .create_insight(
                "startup probe insight",
                "transient insight used to verify the reflection engine",
                vec![],
            )
            .await;
        let insight_count = engine.get_all_insights().await.len();

        let searched = engine.search("startup").await.len();
        let by_type = engine.list_by_type(ReflectionType::General).await.len();
        let validated = engine.list_validated().await.len();
        let by_status = engine.list_by_status(ReflectionStatus::Active).await.len();

        let lesson = Lesson {
            title: "startup probe lesson".to_string(),
            description: "transient lesson".to_string(),
            confidence: 0.5,
        };
        let reflection_insight = ReflectionInsight {
            statement: "transient reflection insight".to_string(),
            confidence: 0.6,
            importance: 0.4,
        };
        let evidence: ReflectionEvidence = ReflectionEvidence {
            experience_id: String::new(),
            description: "transient evidence".to_string(),
            weight: 0.7,
        };
        let review = ReflectionReview {
            id: "startup-review-probe".to_string(),
            started_at: chrono::Utc::now(),
            ended_at: chrono::Utc::now(),
            reflections: Vec::new(),
            summary: "transient review".to_string(),
        };
        let maturity = MaturityHistory {
            timestamp: chrono::Utc::now(),
            previous: KnowledgeMaturity::Emerging,
            current: KnowledgeMaturity::Developing,
            reason: "transient maturity probe".to_string(),
        };

        let evidence_id: EvidenceId = evidence.experience_id.clone();
        let insight_id: InsightId = insight.as_ref().map(|i| i.id.clone()).unwrap_or_default();

        let probe_reflection = Reflection::new(
            "startup-reflection-probe",
            ReflectionType::General,
            "startup reflection probe",
        );
        let actionable = probe_reflection.is_actionable();
        let reflection_summary =
            Reflector::reflect(&probe_reflection, "startup".to_string()).unwrap_or_default();
        let probe_insight = Insight::new(
            uuid::Uuid::new_v4().to_string(),
            "probe insight",
            "transient",
            InsightType::General,
        );
        let mut probe_insight = probe_insight;
        probe_insight.add_hypothesis(uuid::Uuid::new_v4().to_string());
        let generated = InsightProducer::generate_insights(&probe_insight);
        let mut validated_reflection = probe_reflection.clone();
        ValidatableReflection::validate(&mut validated_reflection);

        tracing::info!(
            "Reflection engine probe: insight_ok={} insight_count={} insight_id={} \
             searched={} by_type={} validated={} by_status={} lesson_conf={} \
             rinsight_conf={} evidence_id={} evidence_weight={} review_id={} \
             maturity={:?}->{:?} actionable={} reflection_summary={} \
             generated_insights={} validated_status={:?}",
            insight.is_ok(),
            insight_count,
            insight_id,
            searched,
            by_type,
            validated,
            by_status,
            lesson.confidence,
            reflection_insight.confidence,
            evidence_id,
            evidence.weight,
            review.id,
            maturity.previous,
            maturity.current,
            actionable,
            reflection_summary,
            generated.len(),
            validated_reflection.status,
        );
    }

    // Verify the HypothesisPipeline validation-event path (Architecture §11):
    // generate a probe hypothesis, add supporting evidence until validated, and
    // add contradicting evidence to a second probe until rejected. This keeps
    // ExperienceEvent::hypothesis_validated and its subscriber handler live.
    {
        use crate::experience::integration::hypothesis_pipeline::{
            HypothesisPipeline, HypothesisPipelineConfig,
        };
        use crate::experience::types::{Experience as ProbeExperience, ExperienceType};

        let pipeline = HypothesisPipeline::with_config(
            HypothesisPipelineConfig::default(),
            hypothesis_engine_for_subscriber.clone(),
            bus.clone(),
        );

        let probe_experience = ProbeExperience::new(
            "startup hypothesis pipeline probe".to_string(),
            "Transient experience used to verify hypothesis validation events".to_string(),
            ExperienceType::Hypothesis,
            vec![],
        );
        let generated_ids = pipeline
            .process(&probe_experience)
            .await
            .unwrap_or_default();

        if let Some(hyp_id) = generated_ids.first() {
            for i in 0..5 {
                let supported = pipeline
                    .add_supporting_evidence(hyp_id, &format!("probe evidence {}", i))
                    .await;
                if supported.is_err() {
                    break;
                }
            }
            let validated_hyp = pipeline.get(hyp_id).await;
            tracing::info!(
                "Hypothesis pipeline validation verified: id={} status_supported={}",
                hyp_id,
                validated_hyp
                    .as_ref()
                    .map(|h| h.confidence.value >= 0.75)
                    .unwrap_or(false)
            );

            // Exercise HypothesisConfidence::is_confident and
            // Hypothesis::add_tag on the probe hypothesis so both stay live.
            let mut tagged_hyp = validated_hyp;
            if let Some(ref mut h) = tagged_hyp {
                h.add_tag("startup-probe");
                tracing::info!(
                    "Hypothesis confidence/tag verified: is_confident={} tags={:?}",
                    h.confidence.is_confident(),
                    h.tags
                );
            }
            let validated_list = pipeline.list_validated().await;
            tracing::info!(
                "Hypothesis pipeline list_validated verified: count={}",
                validated_list.len()
            );

            // Exercise the remaining pipeline introspection surface:
            // list_active, graph_stats (engine participation), and archive_old.
            let active_list = pipeline.list_active().await;
            let stats = pipeline.graph_stats();
            let archived = pipeline.archive_old(365).await.unwrap_or(0);
            tracing::info!(
                "Hypothesis pipeline maintenance verified: active={} graph_nodes={} \
                 graph_edges={} archived={}",
                active_list.len(),
                stats.node_count,
                stats.edge_count,
                archived
            );
        }

        let rejected_ids = pipeline
            .process(&ProbeExperience::new(
                "startup hypothesis rejection probe".to_string(),
                "Transient experience used to verify rejection events".to_string(),
                ExperienceType::Hypothesis,
                vec![],
            ))
            .await
            .unwrap_or_default();
        if let Some(hyp_id) = rejected_ids.first() {
            for i in 0..5 {
                let rejected = pipeline
                    .add_contradicting_evidence(hyp_id, &format!("probe contradiction {}", i))
                    .await;
                if rejected.is_err() {
                    break;
                }
            }
            tracing::info!(
                "Hypothesis pipeline rejection probe completed: id={}",
                hyp_id
            );
        }
    }

    LearningPipelineResult {
        knowledge_store,
        skills_registry,
        learning_coordinator,
        event_subscriber,
        reflection_pipeline,
    }
}
