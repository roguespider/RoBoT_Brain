// src/bridge/app/initialization/engines.rs
//! Learning engines: reflection, hypothesis, evolution, metrics

use std::sync::{Arc, Mutex};

use crate::experience::evolution::EvolutionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::metrics::Metrics;
use crate::experience::reflection::ReflectionEngine;

/// Learning engines built by `build_engines`.
pub(crate) struct LearningEngines {
    pub(crate) reflection_engine: Arc<ReflectionEngine>,
    pub(crate) hypothesis_engine_for_subscriber: Arc<HypothesisEngine>,
    pub(crate) hypothesis_engine: Arc<Mutex<HypothesisEngine>>,
    pub(crate) evolution_engine: Arc<EvolutionEngine>,
    pub(crate) metrics: Arc<Metrics>,
}

/// Build learning engines. The shared hypothesis graph is created internally.
pub(crate) fn build_engines() -> LearningEngines {
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
    // Use the custom-config constructor so the tuning surface stays live and
    // the engine is built with explicit, documented thresholds.
    let evolution_config = crate::experience::evolution::engine::EvolutionConfig {
        min_applications_for_promotion: 5,
        min_confidence_for_promotion: 0.7,
        failure_threshold: 0.5,
        unused_threshold_days: 30,
        applications_before_practice: 10,
        applications_before_integration: 20,
    };
    let evolution_engine = Arc::new(EvolutionEngine::with_config(evolution_config));
    let metrics = Arc::new(Metrics::new());

    LearningEngines {
        reflection_engine,
        hypothesis_engine_for_subscriber,
        hypothesis_engine,
        evolution_engine,
        metrics,
    }
}
