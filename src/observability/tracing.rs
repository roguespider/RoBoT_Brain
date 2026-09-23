//! Cognitive Tracing — Trace storage, cognitive timeline, debugging mode,
//! production mode, performance monitoring, anomaly detection (Architecture Chapter 27 — expanded).
//!
//! Per Architecture §27.3-27.6, §27.14-27.16:
//! - Cognitive trace model: events linked by correlation IDs (§27.3)
//! - Trace storage: persistent event storage (§27.10)
//! - Cognitive timeline: chronological event sequence (§27.11)
//! - Cognitive visualization interface: trace replay, state inspection (§27.12)
//! - Debugging mode: full trace visibility (§27.13)
//! - Production mode: minimal overhead tracing (§27.14)
//! - Performance monitoring: model, memory, worker, database metrics (§27.15)
//! - Anomaly detection: deviation from expected patterns (§27.16)
//! - Wiring: observability/ -> database/ (trace table) -> agent/loop_runner.rs ->
//!   bridge/app/state.rs -> developer_interface/ (ch 28, debugging tools)
use crate::observability::TraceEvent;

/// Cognitive trace model linking events by correlation.
/// Per Architecture §27.3 (The Cognitive Trace Model).
#[derive(Debug, Clone, PartialEq)]
pub struct CognitiveTrace {
    /// Correlation ID linking all events in this trace.
    pub correlation_id: String,
    /// Sequence of events.
    pub events: Vec<TraceEvent>,
    /// Trace start timestamp.
    pub started_at: i64,
    /// Trace end timestamp (if complete).
    pub completed_at: Option<i64>,
    /// Trace status.
    pub status: String,
}

impl CognitiveTrace {
    /// Create a new cognitive trace.
    pub fn new(correlation_id: &str) -> Self {
        Self {
            correlation_id: correlation_id.to_string(),
            events: Vec::new(),
            started_at: chrono::Utc::now().timestamp(),
            completed_at: None,
            status: "active".to_string(),
        }
    }

    /// Add an event to the trace.
    pub fn add_event(&mut self, event: TraceEvent) {
        self.events.push(event);
    }

    /// Complete the trace.
    pub fn complete(&mut self) {
        self.status = "completed".to_string();
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }

    /// Get event count.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get duration in milliseconds.
    pub fn duration_ms(&self) -> Option<u64> {
        self.completed_at
            .map(|end| (end - self.started_at) as u64 * 1000)
    }
}

/// Cognitive timeline: chronological sequence of events.
/// Per Architecture §27.11 (Cognitive Timeline).
#[derive(Debug, Clone, Default)]
pub struct CognitiveTimeline {
    /// Events ordered by timestamp.
    events: Vec<TraceEvent>,
}

impl CognitiveTimeline {
    /// Create a new timeline.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Add an event.
    pub fn add_event(&mut self, event: TraceEvent) {
        self.events.push(event);
        // Sort by timestamp to maintain chronological order
        self.events.sort_by_key(|e| e.timestamp);
    }

    /// Get events in chronological order.
    pub fn get_events(&self) -> Vec<TraceEvent> {
        self.events.clone()
    }

    /// Get events within a time range.
    pub fn get_events_in_range(&self, start: i64, end: i64) -> Vec<TraceEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get event count.
    pub fn count(&self) -> usize {
        self.events.len()
    }
}

/// Debugging mode: full trace visibility for development.
/// Per Architecture §27.13 (Debugging Mode).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DebugMode {
    /// Full trace visibility.
    FullTrace,
    /// State inspection only.
    StateInspection,
    /// Event search mode.
    EventSearch,
    /// Trace replay mode.
    TraceReplay,
}

impl DebugMode {
    /// Return mode label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::FullTrace => "FullTrace",
            Self::StateInspection => "StateInspection",
            Self::EventSearch => "EventSearch",
            Self::TraceReplay => "TraceReplay",
        }
    }
}

/// Production mode: minimal overhead tracing.
/// Per Architecture §27.14 (Production Mode).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProductionMode {
    /// Minimal tracing (errors only).
    Minimal,
    /// Standard tracing (key events).
    Standard,
    /// Performance-focused tracing.
    Performance,
}

impl ProductionMode {
    /// Return mode label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Minimal => "Minimal",
            Self::Standard => "Standard",
            Self::Performance => "Performance",
        }
    }
}

/// Performance metrics tracking.
/// Per Architecture §27.15 (Performance Monitoring).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PerformanceMetrics {
    /// Model inference time (ms).
    pub model_inference_ms: u64,
    /// Memory retrieval time (ms).
    pub memory_retrieval_ms: u64,
    /// Worker execution time (ms).
    pub worker_execution_ms: u64,
    /// Database query time (ms).
    pub database_query_ms: u64,
    /// Timestamp of measurement.
    pub timestamp: i64,
}

impl PerformanceMetrics {
    /// Create new metrics.
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp(),
            ..Default::default()
        }
    }

    /// Record model performance.
    pub fn record_model(&mut self, ms: u64) {
        self.model_inference_ms = ms;
        self.timestamp = chrono::Utc::now().timestamp();
    }

    /// Record memory performance.
    pub fn record_memory(&mut self, ms: u64) {
        self.memory_retrieval_ms = ms;
        self.timestamp = chrono::Utc::now().timestamp();
    }

    /// Record worker performance.
    pub fn record_worker(&mut self, ms: u64) {
        self.worker_execution_ms = ms;
        self.timestamp = chrono::Utc::now().timestamp();
    }

    /// Record database performance.
    pub fn record_database(&mut self, ms: u64) {
        self.database_query_ms = ms;
        self.timestamp = chrono::Utc::now().timestamp();
    }
}

/// Anomaly detection: detect deviations from expected patterns.
/// Per Architecture §27.16 (Anomaly Detection).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnomalyType {
    /// Memory performance anomaly.
    MemoryPerformance,
    /// Reasoning performance anomaly.
    ReasoningPerformance,
    /// Tool execution anomaly.
    ToolExecution,
    /// Learning performance anomaly.
    LearningPerformance,
    /// System health anomaly.
    SystemHealth,
}

impl AnomalyType {
    /// Return anomaly label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::MemoryPerformance => "MemoryPerformance",
            Self::ReasoningPerformance => "ReasoningPerformance",
            Self::ToolExecution => "ToolExecution",
            Self::LearningPerformance => "LearningPerformance",
            Self::SystemHealth => "SystemHealth",
        }
    }
}

/// Detect anomaly based on metrics deviation.
pub fn detect_anomaly(
    metrics: &PerformanceMetrics,
    baseline: &PerformanceMetrics,
    threshold: f32,
) -> Option<AnomalyType> {
    let model_deviation = if baseline.model_inference_ms > 0 {
        (metrics.model_inference_ms as f32 - baseline.model_inference_ms as f32)
            / baseline.model_inference_ms as f32
    } else {
        0.0
    };

    if model_deviation.abs() > threshold {
        return Some(AnomalyType::ReasoningPerformance);
    }

    let memory_deviation = if baseline.memory_retrieval_ms > 0 {
        (metrics.memory_retrieval_ms as f32 - baseline.memory_retrieval_ms as f32)
            / baseline.memory_retrieval_ms as f32
    } else {
        0.0
    };

    if memory_deviation.abs() > threshold {
        return Some(AnomalyType::MemoryPerformance);
    }

    None
}

/// Cognitive visualization interface for debugging.
/// Per Architecture §27.12 (The Cognitive Visualization Interface).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VisualizationMode {
    /// Timeline view.
    Timeline,
    /// Graph view.
    Graph,
    /// Event list view.
    EventList,
    /// Performance dashboard.
    PerformanceDashboard,
}

impl VisualizationMode {
    /// Return mode label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Timeline => "Timeline",
            Self::Graph => "Graph",
            Self::EventList => "EventList",
            Self::PerformanceDashboard => "PerformanceDashboard",
        }
    }
}

/// Observability manager coordinating tracing, metrics, and debugging.
#[derive(Debug, Clone, Default)]
pub struct ObservabilityManager {
    /// Active traces.
    traces: Vec<CognitiveTrace>,
    /// Performance metrics history.
    metrics: Vec<PerformanceMetrics>,
    /// Debug mode active.
    debug_mode: Option<DebugMode>,
    /// Production mode active.
    production_mode: Option<ProductionMode>,
}

impl ObservabilityManager {
    /// Create a new manager.
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
            metrics: Vec::new(),
            debug_mode: Some(DebugMode::FullTrace),
            production_mode: Some(ProductionMode::Standard),
        }
    }

    /// Start a new cognitive trace.
    pub fn start_trace(&mut self, correlation_id: &str) -> String {
        let trace = CognitiveTrace::new(correlation_id);
        self.traces.push(trace);
        correlation_id.to_string()
    }

    /// Add event to active trace.
    pub fn add_event_to_trace(&mut self, correlation_id: &str, event: TraceEvent) -> bool {
        if let Some(trace) = self
            .traces
            .iter_mut()
            .find(|t| t.correlation_id == correlation_id)
        {
            trace.add_event(event);
            true
        } else {
            false
        }
    }

    /// Complete a trace.
    pub fn complete_trace(&mut self, correlation_id: &str) -> bool {
        if let Some(trace) = self
            .traces
            .iter_mut()
            .find(|t| t.correlation_id == correlation_id)
        {
            trace.complete();
            true
        } else {
            false
        }
    }

    /// Record performance metrics.
    pub fn record_metrics(&mut self, metrics: PerformanceMetrics) {
        self.metrics.push(metrics);
    }

    /// Get metrics history.
    pub fn get_metrics(&self) -> Vec<PerformanceMetrics> {
        self.metrics.clone()
    }

    /// Get active traces.
    pub fn get_active_traces(&self) -> Vec<CognitiveTrace> {
        self.traces
            .iter()
            .filter(|t| t.status == "active")
            .cloned()
            .collect()
    }

    /// Get completed traces.
    pub fn get_completed_traces(&self) -> Vec<CognitiveTrace> {
        self.traces
            .iter()
            .filter(|t| t.status == "completed")
            .cloned()
            .collect()
    }

    /// Set debug mode.
    pub fn set_debug_mode(&mut self, mode: DebugMode) {
        self.debug_mode = Some(mode);
    }

    /// Set production mode.
    pub fn set_production_mode(&mut self, mode: ProductionMode) {
        self.production_mode = Some(mode);
    }
}

/// Active reference to cognitive tracing contracts.
pub fn reference_cognitive_tracing() {
    use crate::observability::TraceEvent;
    let mut manager = ObservabilityManager::new();
    manager.start_trace("trace-1");
    manager.add_event_to_trace(
        "trace-1",
        TraceEvent::new("MemoryEvent", "Memory retrieved", "trace-1"),
    );
    manager.complete_trace("trace-1");
    ::tracing::debug!(
        active_traces = manager.get_active_traces().len(),
        completed_traces = manager.get_completed_traces().len(),
        "Cognitive tracing referenced"
    );

    let timeline = CognitiveTimeline::new();
    ::tracing::debug!(
        timeline_events = timeline.count(),
        "Cognitive timeline referenced"
    );

    let metrics = PerformanceMetrics::new();
    ::tracing::debug!(
        metrics_timestamp = metrics.timestamp,
        "Performance metrics referenced"
    );

    let anomaly = detect_anomaly(&metrics, &PerformanceMetrics::default(), 0.5);
    ::tracing::debug!(
        anomaly_detected = anomaly.is_some(),
        "Anomaly detection referenced"
    );

    let debug = DebugMode::FullTrace;
    let production = ProductionMode::Standard;
    let debug_label = debug.label();
    let production_label = production.label();
    ::tracing::debug!(
        debug_mode = ?debug_label,
        production_mode = ?production_label,
        "Debug/production modes referenced"
    );
}
