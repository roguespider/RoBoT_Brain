//! Observability Tracing — Per Architecture Chapter 27.6 (Cognitive Tracing).
//!
//! Cognitive tracing follows decisions through the system, recording
//! inputs, outputs, decisions, confidence, timing, and errors.

/// A trace span representing a stage in a cognitive workflow.
#[derive(Debug, Clone)]
pub struct TraceSpan {
    /// Unique span identifier.
    pub id: String,
    /// Name of the stage (e.g., "Memory Retrieval", "Planning").
    pub name: String,
    /// Start timestamp.
    pub start_time: i64,
    /// End timestamp (if completed).
    pub end_time: Option<i64>,
    /// Duration in milliseconds (if completed).
    pub duration_ms: Option<u64>,
    /// Confidence associated with this stage.
    pub confidence: Option<f32>,
    /// Child spans.
    pub children: Vec<TraceSpan>,
}

impl TraceSpan {
    /// Create a new trace span.
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            start_time: 0,
            end_time: None,
            duration_ms: None,
            confidence: None,
            children: Vec::new(),
        }
    }

    /// Complete the span with an end timestamp.
    pub fn complete(&mut self, end_time: i64, confidence: Option<f32>) {
        self.end_time = Some(end_time);
        self.confidence = confidence;
        if self.start_time > 0 {
            self.duration_ms = Some((end_time - self.start_time) as u64);
        }
    }

    /// Add a child span.
    pub fn add_child(&mut self, child: TraceSpan) {
        self.children.push(child);
    }
}

impl Default for TraceSpan {
    fn default() -> Self {
        Self::new("default".to_string(), "root".to_string())
    }
}

/// A cognitive trace representing a complete workflow.
#[derive(Debug, Clone)]
pub struct CognitiveTrace {
    /// Unique trace identifier.
    pub id: String,
    /// Root span of the trace.
    pub root: TraceSpan,
    /// All spans in the trace.
    pub spans: Vec<TraceSpan>,
}

impl CognitiveTrace {
    /// Create a new cognitive trace.
    pub fn new(id: String, root: TraceSpan) -> Self {
        Self {
            id,
            spans: vec![root.clone()],
            root,
        }
    }

    /// Add a span to the trace.
    pub fn add_span(&mut self, span: TraceSpan) {
        self.spans.push(span);
    }
}
