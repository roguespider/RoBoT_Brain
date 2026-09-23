//! Event Router — Routes events through the cognitive pipeline (Architecture Chapter 04 — Data Flow).
//!
//! Per Architecture §4.14-4.16 (Data Ownership, Data Lifetime, Architectural Benefits):
//! - Events flow from observation -> context -> memory -> experience -> learning
//! - Each event carries provenance, correlation, and version metadata
//! - Event routing ensures no data loss and full traceability
//! - Wiring: event_router/ -> data_contracts/ (event_contract) -> experience/ + memory/ + learning/

/// Event routing direction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingDirection {
    /// Route to experience engine.
    ToExperience,
    /// Route to memory engine.
    ToMemory,
    /// Route to learning engine.
    ToLearning,
    /// Route to knowledge graph.
    ToKnowledge,
    /// Route to planner.
    ToPlanner,
    /// Route to execution engine.
    ToExecution,
    /// Route to observability.
    ToObservability,
}

impl RoutingDirection {
    /// Return direction label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ToExperience => "ToExperience",
            Self::ToMemory => "ToMemory",
            Self::ToLearning => "ToLearning",
            Self::ToKnowledge => "ToKnowledge",
            Self::ToPlanner => "ToPlanner",
            Self::ToExecution => "ToExecution",
            Self::ToObservability => "ToObservability",
        }
    }
}

/// A routed event with full provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct RoutedEvent {
    /// Event identifier.
    pub event_id: String,
    /// Source subsystem.
    pub source: String,
    /// Target subsystem.
    pub target: String,
    /// Event payload.
    pub payload: serde_json::Value,
    /// Correlation ID.
    pub correlation_id: String,
    /// Routing direction.
    pub direction: RoutingDirection,
    /// Timestamp.
    pub timestamp: i64,
    /// Schema version.
    pub schema_version: String,
    /// Provenance chain.
    pub provenance: Vec<String>,
}

impl RoutedEvent {
    /// Create a new routed event.
    pub fn new(
        source: &str,
        target: &str,
        direction: RoutingDirection,
        payload: serde_json::Value,
        correlation_id: &str,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            source: source.to_string(),
            target: target.to_string(),
            payload,
            correlation_id: correlation_id.to_string(),
            direction,
            timestamp: chrono::Utc::now().timestamp(),
            schema_version: "v0.0.2.1".to_string(),
            provenance: vec![source.to_string()],
        }
    }

    /// Add provenance entry.
    pub fn add_provenance(&mut self, source: &str) {
        self.provenance.push(source.to_string());
    }
}

/// Event router managing event flow through the architecture.
#[derive(Debug, Clone, Default)]
pub struct EventRouter {
    /// Routed events.
    routed_events: Vec<RoutedEvent>,
    /// Routing rules.
    rules: std::collections::HashMap<String, Vec<RoutingDirection>>,
}

impl EventRouter {
    /// Create a new router.
    pub fn new() -> Self {
        Self {
            routed_events: Vec::new(),
            rules: std::collections::HashMap::new(),
        }
    }

    /// Add a routing rule for a source.
    pub fn add_rule(&mut self, source: &str, directions: Vec<RoutingDirection>) {
        self.rules.insert(source.to_string(), directions);
    }

    /// Route an event.
    pub fn route(&mut self, event: RoutedEvent) -> Vec<RoutedEvent> {
        let source = event.source.clone();
        let directions = self.rules.get(&source).cloned().unwrap_or_default();
        let mut routed = Vec::new();
        for direction in directions {
            let mut routed_event = event.clone();
            routed_event.direction = direction.clone();
            routed_event.target = direction.label().to_string();
            routed.push(routed_event);
        }
        if routed.is_empty() {
            // Default routing: route to experience and memory
            let mut to_exp = event.clone();
            to_exp.direction = RoutingDirection::ToExperience;
            to_exp.target = RoutingDirection::ToExperience.label().to_string();
            routed.push(to_exp);

            let mut to_mem = event.clone();
            to_mem.direction = RoutingDirection::ToMemory;
            to_mem.target = RoutingDirection::ToMemory.label().to_string();
            routed.push(to_mem);
        }
        self.routed_events.extend(routed.clone());
        routed
    }

    /// Get routed events.
    pub fn get_routed(&self) -> Vec<RoutedEvent> {
        self.routed_events.clone()
    }

    /// Get routed events by correlation ID.
    pub fn get_by_correlation(&self, correlation_id: &str) -> Vec<RoutedEvent> {
        self.routed_events
            .iter()
            .filter(|e| e.correlation_id == correlation_id)
            .cloned()
            .collect()
    }

    /// Clear routed events.
    pub fn clear(&mut self) {
        self.routed_events.clear();
    }
}

/// Active reference to event router contracts.
pub fn reference_event_router() {
    let mut router = EventRouter::new();
    router.add_rule(
        "observation",
        vec![
            RoutingDirection::ToExperience,
            RoutingDirection::ToMemory,
            RoutingDirection::ToObservability,
        ],
    );
    router.add_rule(
        "execution",
        vec![
            RoutingDirection::ToExperience,
            RoutingDirection::ToLearning,
            RoutingDirection::ToObservability,
        ],
    );
    router.add_rule(
        "reflection",
        vec![RoutingDirection::ToLearning, RoutingDirection::ToMemory],
    );

    let event = RoutedEvent::new(
        "observation",
        "context_engine",
        RoutingDirection::ToExperience,
        serde_json::json!({"content": "test"}),
        "corr-1",
    );
    let routed = router.route(event);
    tracing::debug!(
        routed_count = routed.len(),
        correlation = routed
            .first()
            .map(|e| e.correlation_id.clone())
            .unwrap_or_default(),
        "Event router referenced"
    );
}
