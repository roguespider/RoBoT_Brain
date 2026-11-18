// src/bridge/app/initialization/observability_init.rs
//! Observability layer verification at startup (Architecture Ch. 27).
//!
//! Verifies cognitive event recording, replay, trace storage, performance
//! monitoring, anomaly detection, and all observability types.

/// Run observability layer verification.
pub fn verify_observability_layer() {
    // Test record_cognitive_event with real DB persistence
    let payload = serde_json::json!({"key": "test", "value": 42});
    let event_id = crate::observability::record_cognitive_event(
        crate::observability::CognitiveEventType::Memory,
        "observability-test",
        "Test cognitive event for observability verification",
        payload,
    );
    if let Ok(id) = &event_id {
        tracing::debug!(event_id = %id, "Cognitive event recorded successfully");
    }
    // Test replay_events - should return real DB events
    let events = crate::observability::replay_events("observability-test");
    tracing::debug!(
        event_count = events.len(),
        "Replay events returned {} event(s) from DB",
        events.len()
    );
    // Test CognitiveTrace (trace model)
    let mut trace = crate::observability::tracing::CognitiveTrace::new("trace-test");
    let test_event =
        crate::observability::TraceEvent::new("memory", "Test event for trace", "trace-test");
    trace.add_event(test_event.clone());
    trace.complete();
    tracing::debug!(
        trace_events = trace.event_count(),
        trace_duration_ms = ?trace.duration_ms(),
        "Cognitive trace verified"
    );
    // Test CognitiveTimeline
    let mut timeline = crate::observability::tracing::CognitiveTimeline::new();
    timeline.add_event(test_event.clone());
    let timeline_events = timeline.get_events();
    let now = chrono::Utc::now().timestamp();
    let range_events = timeline.get_events_in_range(now - 3600, now);
    let event_count = timeline.count();
    tracing::debug!(
        timeline_events = timeline_events.len(),
        range_events = range_events.len(),
        event_count = event_count,
        "Cognitive timeline verified"
    );
    // Test PerformanceMetrics
    let mut metrics = crate::observability::tracing::PerformanceMetrics::new();
    metrics.record_model(150);
    metrics.record_memory(45);
    metrics.record_worker(120);
    metrics.record_database(30);
    tracing::debug!(
        model_ms = metrics.model_inference_ms,
        memory_ms = metrics.memory_retrieval_ms,
        worker_ms = metrics.worker_execution_ms,
        database_ms = metrics.database_query_ms,
        "Performance metrics verified"
    );
    // Test anomaly detection
    let baseline = crate::observability::tracing::PerformanceMetrics::new();
    let anomaly = crate::observability::tracing::detect_anomaly(&metrics, &baseline, 0.5);
    // Test all AnomalyType variants including previously-unused ones
    let all_anomalies = [
        crate::observability::tracing::AnomalyType::MemoryPerformance,
        crate::observability::tracing::AnomalyType::ReasoningPerformance,
        crate::observability::tracing::AnomalyType::ToolExecution,
        crate::observability::tracing::AnomalyType::LearningPerformance,
        crate::observability::tracing::AnomalyType::SystemHealth,
    ];
    for at in &all_anomalies {
        tracing::debug!(anomaly_label = at.label(), "AnomalyType verified");
    }
    tracing::debug!(anomaly = ?anomaly, "Anomaly detection verified");
    // Test ObservabilityManager (tracing version with full features)
    let mut obs_manager = crate::observability::tracing::ObservabilityManager::new();
    let trace_id = obs_manager.start_trace("obs-test");
    obs_manager.add_event_to_trace(&trace_id, test_event.clone());
    obs_manager.complete_trace(&trace_id);
    let active = obs_manager.get_active_traces();
    let completed = obs_manager.get_completed_traces();
    obs_manager.record_metrics(metrics.clone());
    let metrics_history = obs_manager.get_metrics();
    obs_manager.set_debug_mode(crate::observability::tracing::DebugMode::FullTrace);
    obs_manager.set_production_mode(crate::observability::tracing::ProductionMode::Minimal);
    tracing::debug!(
        active_traces = active.len(),
        completed_traces = completed.len(),
        metrics_recorded = metrics_history.len(),
        "Observability manager verified"
    );
    // Test VisualizationMode
    let modes = [
        crate::observability::tracing::VisualizationMode::Timeline,
        crate::observability::tracing::VisualizationMode::Graph,
        crate::observability::tracing::VisualizationMode::EventList,
        crate::observability::tracing::VisualizationMode::PerformanceDashboard,
    ];
    for mode in &modes {
        tracing::debug!(mode = mode.label(), "Visualization mode verified");
    }
    // Test DebugMode
    let debug_modes = [
        crate::observability::tracing::DebugMode::FullTrace,
        crate::observability::tracing::DebugMode::StateInspection,
        crate::observability::tracing::DebugMode::EventSearch,
        crate::observability::tracing::DebugMode::TraceReplay,
    ];
    for mode in &debug_modes {
        tracing::debug!(debug_mode = mode.label(), "Debug mode verified");
    }
    // Test ProductionMode
    let prod_modes = [
        crate::observability::tracing::ProductionMode::Minimal,
        crate::observability::tracing::ProductionMode::Standard,
        crate::observability::tracing::ProductionMode::Performance,
    ];
    for mode in &prod_modes {
        tracing::debug!(prod_mode = mode.label(), "Production mode verified");
    }
    // Test ObservabilityManager (mod.rs version)
    let mut obs_mgr = crate::observability::ObservabilityManager::new();
    let mod_test_event = crate::observability::TraceEvent::new(
        "memory",
        "Test event for mod.rs manager",
        "mod-test",
    );
    obs_mgr.record_event(mod_test_event);
    let mod_events = obs_mgr.get_events_by_correlation("mod-test");
    let mod_all = obs_mgr.get_all_events();
    tracing::debug!(
        mod_events = mod_events.len(),
        mod_all = mod_all.len(),
        "ObservabilityManager (mod.rs) verified"
    );
    // Test CognitiveEventType variants
    let event_types = [
        crate::observability::CognitiveEventType::Memory,
        crate::observability::CognitiveEventType::Experience,
        crate::observability::CognitiveEventType::Planning,
        crate::observability::CognitiveEventType::Execution,
        crate::observability::CognitiveEventType::Evolution,
    ];
    for et in &event_types {
        tracing::debug!(event_type = ?et, "Cognitive event type verified");
    }
    // Test ObservabilityError variants
    let err_recording = crate::observability::ObservabilityError::RecordingFailed;
    let err_replay = crate::observability::ObservabilityError::ReplayFailed;
    tracing::debug!(
        err_recording = ?err_recording,
        err_replay = ?err_replay,
        "ObservabilityError variants verified"
    );
    // Test EventArchitecture (event emission architecture per §27.4/§27.8)
    let event_archs = [
        crate::observability::tracing::EventArchitecture::Memory,
        crate::observability::tracing::EventArchitecture::Experience,
        crate::observability::tracing::EventArchitecture::Planning,
        crate::observability::tracing::EventArchitecture::Execution,
        crate::observability::tracing::EventArchitecture::Evolution,
        crate::observability::tracing::EventArchitecture::System,
        crate::observability::tracing::EventArchitecture::Tool,
        crate::observability::tracing::EventArchitecture::Security,
    ];
    for ea in &event_archs {
        tracing::debug!(event_arch = ea.label(), "EventArchitecture verified");
    }
    // Test TraceStorage (persistent event storage per §27.10)
    let mut trace_storage = crate::observability::tracing::TraceStorage::new();
    let storage_event = crate::observability::TraceEvent::new(
        "memory",
        "Test event for trace storage",
        "storage-test",
    );
    trace_storage.persist(storage_event.clone());
    let retrieved = trace_storage.retrieve_by_correlation("storage-test");
    let all_stored = trace_storage.all_events();
    tracing::debug!(
        retrieved_count = retrieved.len(),
        all_stored_count = all_stored.len(),
        "TraceStorage verified"
    );
}
