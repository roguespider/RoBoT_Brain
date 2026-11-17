//! Typed cognitive event types per Architecture Appendix C (Event Definitions).
//!
//! Defines all 60 event types from Appendix C, organized by category:
//! System, Runtime, Memory, Knowledge, Experience, Learning, Context,
//! Conversation, Planning, Execution, Tool, Audio, Vision, Model, Security,
//! and Diagnostic events.

use serde_json::Value;

/// Severity level for a cognitive event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSeverity {
    /// Informational event.
    Info,
    /// Warning condition.
    Warning,
    /// Error condition.
    Error,
    /// Critical condition requiring immediate attention.
    Critical,
}

impl EventSeverity {
    /// Return the severity label as a string.
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

/// All cognitive event types from Architecture Appendix C.
///
/// Organized into 16 categories matching the Appendix C event hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveEventType {
    // System Events (4)
    /// Generated when RoBoT begins execution.
    SystemStarted,
    /// Generated during controlled shutdown.
    SystemShutdown,
    /// Generated after configuration initialization.
    ConfigurationLoaded,
    /// Generated after system validation.
    HealthCheckCompleted,
    // Runtime Events (5)
    /// Generated when an AI model becomes available.
    ModelLoaded,
    /// Generated when a model leaves memory.
    ModelUnloaded,
    /// Generated when model execution begins.
    InferenceStarted,
    /// Generated when execution finishes.
    InferenceCompleted,
    /// Generated after unsuccessful inference.
    InferenceFailed,
    // Memory Events (5)
    /// Generated when new memory is created.
    MemoryCreated,
    /// Generated when memory is accessed.
    MemoryRetrieved,
    /// Generated when memory changes.
    MemoryUpdated,
    /// Generated when temporary information becomes permanent knowledge.
    MemoryConsolidated,
    /// Generated when memory is moved out of active storage.
    MemoryArchived,
    // Knowledge Events (4)
    /// Generated when a knowledge node is created.
    KnowledgeNodeCreated,
    /// Generated when a knowledge relationship is created.
    KnowledgeRelationshipCreated,
    /// Generated when conflicting knowledge is detected.
    KnowledgeConflictDetected,
    /// Generated when knowledge is validated.
    KnowledgeValidated,
    // Experience Events (5)
    /// Generated when an activity begins.
    ExperienceStarted,
    /// Generated when activity finishes.
    ExperienceCompleted,
    /// Generated after analysis.
    ExperienceEvaluated,
    /// Generated when reusable knowledge is extracted.
    LessonCreated,
    /// Generated when capability improves.
    SkillImproved,
    // Learning Events (4)
    /// Generated when a hypothesis is formed.
    HypothesisCreated,
    /// Generated when a hypothesis is tested.
    HypothesisTested,
    /// Generated when information becomes trusted knowledge.
    KnowledgePromoted,
    /// Generated when a learning cycle completes.
    LearningCycleCompleted,
    // Context Events (3)
    /// Generated when context is created.
    ContextCreated,
    /// Generated when context is compressed.
    ContextCompressed,
    /// Generated when context is updated.
    ContextUpdated,
    // Conversation Events (3)
    /// Generated when a user message is received.
    UserMessageReceived,
    /// Generated when a response is generated.
    ResponseGenerated,
    /// Generated when a conversation completes.
    ConversationCompleted,
    // Planning Events (4)
    /// Generated when a goal is created.
    GoalCreated,
    /// Generated when a plan is generated.
    PlanGenerated,
    /// Generated when a plan is validated.
    PlanValidated,
    /// Generated when a plan is modified.
    PlanModified,
    // Execution Events (4)
    /// Generated when a task starts.
    TaskStarted,
    /// Generated when a task completes.
    TaskCompleted,
    /// Generated when a task fails.
    TaskFailed,
    /// Generated when a workflow completes.
    WorkflowCompleted,
    // Tool Events (4)
    /// Generated when a tool is discovered.
    ToolDiscovered,
    /// Generated when a tool is called.
    ToolCalled,
    /// Generated when a tool completes.
    ToolCompleted,
    /// Generated when a tool fails.
    ToolFailed,
    // Audio Events (3)
    /// Generated when audio is received.
    AudioReceived,
    /// Generated when speech transcription completes.
    SpeechTranscriptionCompleted,
    /// Generated when speech is generated.
    SpeechGenerated,
    // Vision Events (3)
    /// Generated when an image is received.
    ImageReceived,
    /// Generated when OCR completes.
    OCRCompleted,
    /// Generated when vision analysis completes.
    VisionAnalysisCompleted,
    // Model Events (3)
    /// Generated when a model is downloaded.
    ModelDownloaded,
    /// Generated when a model is validated.
    ModelValidated,
    /// Generated when a model is updated.
    ModelUpdated,
    // Security Events (3)
    /// Generated when a permission is checked.
    PermissionChecked,
    /// Generated when access is denied.
    AccessDenied,
    /// Generated when a security violation is detected.
    SecurityViolationDetected,
    // Diagnostic Events (3)
    /// Generated when performance is recorded.
    PerformanceRecorded,
    /// Generated when an error is recorded.
    ErrorRecorded,
    /// Generated when a trace completes.
    TraceCompleted,
}

impl CognitiveEventType {
    /// Return the event category as a string.
    pub fn category(self) -> &'static str {
        match self {
            Self::SystemStarted
            | Self::SystemShutdown
            | Self::ConfigurationLoaded
            | Self::HealthCheckCompleted => "system",
            Self::ModelLoaded
            | Self::ModelUnloaded
            | Self::InferenceStarted
            | Self::InferenceCompleted
            | Self::InferenceFailed => "runtime",
            Self::MemoryCreated
            | Self::MemoryRetrieved
            | Self::MemoryUpdated
            | Self::MemoryConsolidated
            | Self::MemoryArchived => "memory",
            Self::KnowledgeNodeCreated
            | Self::KnowledgeRelationshipCreated
            | Self::KnowledgeConflictDetected
            | Self::KnowledgeValidated => "knowledge",
            Self::ExperienceStarted
            | Self::ExperienceCompleted
            | Self::ExperienceEvaluated
            | Self::LessonCreated
            | Self::SkillImproved => "experience",
            Self::HypothesisCreated
            | Self::HypothesisTested
            | Self::KnowledgePromoted
            | Self::LearningCycleCompleted => "learning",
            Self::ContextCreated | Self::ContextCompressed | Self::ContextUpdated => "context",
            Self::UserMessageReceived | Self::ResponseGenerated | Self::ConversationCompleted => {
                "conversation"
            }
            Self::GoalCreated | Self::PlanGenerated | Self::PlanValidated | Self::PlanModified => {
                "planning"
            }
            Self::TaskStarted
            | Self::TaskCompleted
            | Self::TaskFailed
            | Self::WorkflowCompleted => "execution",
            Self::ToolDiscovered | Self::ToolCalled | Self::ToolCompleted | Self::ToolFailed => {
                "tool"
            }
            Self::AudioReceived | Self::SpeechTranscriptionCompleted | Self::SpeechGenerated => {
                "audio"
            }
            Self::ImageReceived | Self::OCRCompleted | Self::VisionAnalysisCompleted => "vision",
            Self::ModelDownloaded | Self::ModelValidated | Self::ModelUpdated => "model",
            Self::PermissionChecked | Self::AccessDenied | Self::SecurityViolationDetected => {
                "security"
            }
            Self::PerformanceRecorded | Self::ErrorRecorded | Self::TraceCompleted => "diagnostic",
        }
    }

    /// Return the event type name as a dot-separated string (e.g. "memory.created").
    pub fn name(self) -> &'static str {
        match self {
            Self::SystemStarted => "system.started",
            Self::SystemShutdown => "system.shutdown",
            Self::ConfigurationLoaded => "system.configuration_loaded",
            Self::HealthCheckCompleted => "system.health_check_completed",
            Self::ModelLoaded => "runtime.model_loaded",
            Self::ModelUnloaded => "runtime.model_unloaded",
            Self::InferenceStarted => "runtime.inference_started",
            Self::InferenceCompleted => "runtime.inference_completed",
            Self::InferenceFailed => "runtime.inference_failed",
            Self::MemoryCreated => "memory.created",
            Self::MemoryRetrieved => "memory.retrieved",
            Self::MemoryUpdated => "memory.updated",
            Self::MemoryConsolidated => "memory.consolidated",
            Self::MemoryArchived => "memory.archived",
            Self::KnowledgeNodeCreated => "knowledge.node_created",
            Self::KnowledgeRelationshipCreated => "knowledge.relationship_created",
            Self::KnowledgeConflictDetected => "knowledge.conflict_detected",
            Self::KnowledgeValidated => "knowledge.validated",
            Self::ExperienceStarted => "experience.started",
            Self::ExperienceCompleted => "experience.completed",
            Self::ExperienceEvaluated => "experience.evaluated",
            Self::LessonCreated => "experience.lesson_created",
            Self::SkillImproved => "experience.skill_improved",
            Self::HypothesisCreated => "learning.hypothesis_created",
            Self::HypothesisTested => "learning.hypothesis_tested",
            Self::KnowledgePromoted => "learning.knowledge_promoted",
            Self::LearningCycleCompleted => "learning.cycle_completed",
            Self::ContextCreated => "context.created",
            Self::ContextCompressed => "context.compressed",
            Self::ContextUpdated => "context.updated",
            Self::UserMessageReceived => "conversation.user_message_received",
            Self::ResponseGenerated => "conversation.response_generated",
            Self::ConversationCompleted => "conversation.completed",
            Self::GoalCreated => "planning.goal_created",
            Self::PlanGenerated => "planning.plan_generated",
            Self::PlanValidated => "planning.plan_validated",
            Self::PlanModified => "planning.plan_modified",
            Self::TaskStarted => "execution.task_started",
            Self::TaskCompleted => "execution.task_completed",
            Self::TaskFailed => "execution.task_failed",
            Self::WorkflowCompleted => "execution.workflow_completed",
            Self::ToolDiscovered => "tool.discovered",
            Self::ToolCalled => "tool.called",
            Self::ToolCompleted => "tool.completed",
            Self::ToolFailed => "tool.failed",
            Self::AudioReceived => "audio.received",
            Self::SpeechTranscriptionCompleted => "audio.speech_transcription_completed",
            Self::SpeechGenerated => "audio.speech_generated",
            Self::ImageReceived => "vision.image_received",
            Self::OCRCompleted => "vision.ocr_completed",
            Self::VisionAnalysisCompleted => "vision.analysis_completed",
            Self::ModelDownloaded => "model.downloaded",
            Self::ModelValidated => "model.validated",
            Self::ModelUpdated => "model.updated",
            Self::PermissionChecked => "security.permission_checked",
            Self::AccessDenied => "security.access_denied",
            Self::SecurityViolationDetected => "security.violation_detected",
            Self::PerformanceRecorded => "diagnostic.performance_recorded",
            Self::ErrorRecorded => "diagnostic.error_recorded",
            Self::TraceCompleted => "diagnostic.trace_completed",
        }
    }
}

/// A typed cognitive event with base fields per Architecture Appendix C.
///
/// All events share a common base structure: id, type, version, timestamp,
/// source, session id, correlation id, optional parent event id, severity,
/// payload, metadata, and confidence.
#[derive(Debug, Clone)]
pub struct CognitiveEvent {
    /// Event identifier.
    pub id: String,
    /// Event type.
    pub event_type: CognitiveEventType,
    /// Event version (e.g. "1.0").
    pub version: String,
    /// Timestamp (Unix epoch seconds).
    pub timestamp: i64,
    /// Source subsystem that emitted the event.
    pub source: String,
    /// Session identifier.
    pub session_id: String,
    /// Correlation identifier for trace linkage.
    pub correlation_id: String,
    /// Optional parent event identifier for event chains.
    pub parent_event_id: Option<String>,
    /// Severity level.
    pub severity: EventSeverity,
    /// Event payload (structured data specific to the event type).
    pub payload: Value,
    /// Additional metadata (version info, device info, etc.).
    pub metadata: Value,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
}

impl CognitiveEvent {
    /// Create a new cognitive event with required fields.
    pub fn new(
        event_type: CognitiveEventType,
        source: &str,
        session_id: &str,
        correlation_id: &str,
        payload: Value,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            source: source.to_string(),
            session_id: session_id.to_string(),
            correlation_id: correlation_id.to_string(),
            parent_event_id: None,
            severity: EventSeverity::Info,
            payload,
            metadata: Value::Object(serde_json::Map::new()),
            confidence: 0.5,
        }
    }

    /// Set the event version.
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    /// Set the parent event identifier.
    pub fn with_parent(mut self, parent_event_id: &str) -> Self {
        self.parent_event_id = Some(parent_event_id.to_string());
        self
    }

    /// Set the severity level.
    pub fn with_severity(mut self, severity: EventSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the confidence score.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Add metadata key-value pair.
    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        if let Value::Object(ref mut map) = self.metadata {
            map.insert(key.to_string(), value);
        }
        self
    }
}

/// Event tracer that emits typed cognitive events to persistent storage.
///
/// Per Architecture Appendix C: events flow through an event bus to storage.
/// The EventTracer records CognitiveEvent instances to the cognitive_events
/// database table for trace replay and debugging.
pub struct EventTracer {
    /// Source subsystem identifier.
    source: String,
}

impl EventTracer {
    /// Create a new event tracer for a source subsystem.
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
        }
    }

    /// Record a cognitive event to persistent storage.
    pub fn trace(&self, event: CognitiveEvent) -> Result<(), crate::tracing::ObservabilityError> {
        crate::tracing::record_cognitive_event(
            event.event_type,
            &event.correlation_id,
            event.payload,
        )
    }

    /// Emit a system event.
    pub fn system_event(
        &self,
        event_type: CognitiveEventType,
        session_id: &str,
        correlation_id: &str,
        payload: Value,
    ) -> Result<(), crate::tracing::ObservabilityError> {
        let event = CognitiveEvent::new(
            event_type,
            &self.source,
            session_id,
            correlation_id,
            payload,
        );
        self.trace(event)
    }

    /// Emit a memory event.
    pub fn memory_event(
        &self,
        event_type: CognitiveEventType,
        session_id: &str,
        correlation_id: &str,
        payload: Value,
    ) -> Result<(), crate::tracing::ObservabilityError> {
        let event = CognitiveEvent::new(
            event_type,
            &self.source,
            session_id,
            correlation_id,
            payload,
        );
        self.trace(event)
    }

    /// Emit an experience event.
    pub fn experience_event(
        &self,
        event_type: CognitiveEventType,
        session_id: &str,
        correlation_id: &str,
        payload: Value,
    ) -> Result<(), crate::tracing::ObservabilityError> {
        let event = CognitiveEvent::new(
            event_type,
            &self.source,
            session_id,
            correlation_id,
            payload,
        );
        self.trace(event)
    }

    /// Emit a planning event.
    pub fn planning_event(
        &self,
        event_type: CognitiveEventType,
        session_id: &str,
        correlation_id: &str,
        payload: Value,
    ) -> Result<(), crate::tracing::ObservabilityError> {
        let event = CognitiveEvent::new(
            event_type,
            &self.source,
            session_id,
            correlation_id,
            payload,
        );
        self.trace(event)
    }

    /// Emit an execution event.
    pub fn execution_event(
        &self,
        event_type: CognitiveEventType,
        session_id: &str,
        correlation_id: &str,
        payload: Value,
    ) -> Result<(), crate::tracing::ObservabilityError> {
        let event = CognitiveEvent::new(
            event_type,
            &self.source,
            session_id,
            correlation_id,
            payload,
        );
        self.trace(event)
    }

    /// Emit a learning event.
    pub fn learning_event(
        &self,
        event_type: CognitiveEventType,
        session_id: &str,
        correlation_id: &str,
        payload: Value,
    ) -> Result<(), crate::tracing::ObservabilityError> {
        let event = CognitiveEvent::new(
            event_type,
            &self.source,
            session_id,
            correlation_id,
            payload,
        );
        self.trace(event)
    }

    /// Emit a security event.
    pub fn security_event(
        &self,
        event_type: CognitiveEventType,
        session_id: &str,
        correlation_id: &str,
        payload: Value,
    ) -> Result<(), crate::tracing::ObservabilityError> {
        let event = CognitiveEvent::new(
            event_type,
            &self.source,
            session_id,
            correlation_id,
            payload,
        );
        self.trace(event)
    }

    /// Emit a diagnostic event.
    pub fn diagnostic_event(
        &self,
        event_type: CognitiveEventType,
        session_id: &str,
        correlation_id: &str,
        payload: Value,
    ) -> Result<(), crate::tracing::ObservabilityError> {
        let event = CognitiveEvent::new(
            event_type,
            &self.source,
            session_id,
            correlation_id,
            payload,
        );
        self.trace(event)
    }
}

impl Default for EventTracer {
    fn default() -> Self {
        Self::new("system")
    }
}
