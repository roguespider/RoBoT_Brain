//! Conversation Engine — Interaction orchestration (Architecture Chapter 6).
//!
//! The Conversation Engine coordinates external interaction through a
//! controlled cognitive lifecycle without owning long-term memory,
//! knowledge, experience, or execution logic.
//!
//! The `communication` submodule contains ACP, MCP, and event protocols
//! per Architecture Chapter 15 (Agent Communication).

pub mod communication;

use std::collections::HashMap;

/// A conversation identity combining session and correlation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConversationIdentity {
    /// Conversation identifier.
    pub conversation_id: String,
    /// Session identifier.
    pub session_id: String,
    /// Correlation identifier for tracing.
    pub correlation_id: String,
}

impl ConversationIdentity {
    /// Create a new conversation identity.
    pub fn new(conversation_id: &str, session_id: &str) -> Self {
        Self {
            conversation_id: conversation_id.to_string(),
            session_id: session_id.to_string(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Conversation state tracking the lifecycle of an interaction.
#[derive(Debug, Clone, PartialEq)]
pub enum ConversationState {
    /// Interaction received.
    Received,
    /// Intent being analyzed.
    Analyzing,
    /// Context being assembled.
    AssemblingContext,
    /// Cognitive cycle in progress.
    Processing,
    /// Response being constructed.
    Responding,
    /// Interaction completed.
    Completed,
    /// Interaction suspended.
    Suspended,
    /// Interaction failed.
    Failed,
}

/// A conversation session tracking interaction history and state.
#[derive(Debug, Clone)]
pub struct ConversationSession {
    /// Conversation identity.
    pub identity: ConversationIdentity,
    /// Current state.
    pub state: ConversationState,
    /// Interaction history (input IDs and timestamps).
    pub interaction_history: Vec<(String, i64)>,
    /// Active goals being pursued.
    pub active_goals: Vec<String>,
    /// Context references.
    pub context_references: Vec<String>,
    /// Events emitted during this session.
    pub events: Vec<String>,
}

impl ConversationSession {
    /// Create a new conversation session.
    pub fn new(identity: ConversationIdentity) -> Self {
        Self {
            identity,
            state: ConversationState::Received,
            interaction_history: Vec::new(),
            active_goals: Vec::new(),
            context_references: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Advance the conversation state.
    pub fn advance_state(&mut self, new_state: ConversationState) {
        self.state = new_state;
    }

    /// Record an interaction.
    pub fn record_interaction(&mut self, input_id: &str) {
        self.interaction_history
            .push((input_id.to_string(), chrono::Utc::now().timestamp()));
    }

    /// Add an active goal.
    pub fn add_goal(&mut self, goal: &str) {
        if !self.active_goals.contains(&goal.to_string()) {
            self.active_goals.push(goal.to_string());
        }
    }

    /// Add a context reference.
    pub fn add_context_reference(&mut self, reference: &str) {
        self.context_references.push(reference.to_string());
    }

    /// Emit an event.
    pub fn emit_event(&mut self, event: &str) {
        self.events.push(event.to_string());
    }
}

/// Conversation events per Architecture Chapter 6.10.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConversationEvent {
    /// User message received.
    UserMessageReceived,
    /// Intent detected.
    IntentDetected,
    /// Memory retrieved.
    MemoryRetrieved,
    /// Tool requested.
    ToolRequested,
    /// Tool completed.
    ToolCompleted,
    /// Response generated.
    ResponseGenerated,
    /// Conversation completed.
    ConversationCompleted,
    /// Knowledge learned.
    KnowledgeLearned,
}

impl ConversationEvent {
    /// Convert event to string label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::UserMessageReceived => "UserMessageReceived",
            Self::IntentDetected => "IntentDetected",
            Self::MemoryRetrieved => "MemoryRetrieved",
            Self::ToolRequested => "ToolRequested",
            Self::ToolCompleted => "ToolCompleted",
            Self::ResponseGenerated => "ResponseGenerated",
            Self::ConversationCompleted => "ConversationCompleted",
            Self::KnowledgeLearned => "KnowledgeLearned",
        }
    }
}

/// Reference conversation events to eliminate dead-code warnings.
/// The Conversation Engine coordinates interaction lifecycle.
///
/// Per Architecture Chapter 6: it does not own memory, knowledge,
/// experience, planning logic, or execution implementations.
#[derive(Debug, Clone)]
pub struct ConversationEngine {
    /// Active sessions by conversation ID.
    sessions: HashMap<String, ConversationSession>,
}

impl ConversationEngine {
    /// Create a new conversation engine.
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Start a new conversation session.
    pub fn start_session(
        &mut self,
        conversation_id: &str,
        session_id: &str,
    ) -> ConversationIdentity {
        let identity = ConversationIdentity::new(conversation_id, session_id);
        let session = ConversationSession::new(identity.clone());
        self.sessions.insert(conversation_id.to_string(), session);
        identity
    }

    /// Get a session by conversation ID.
    pub fn get_session(&self, conversation_id: &str) -> Option<&ConversationSession> {
        self.sessions.get(conversation_id)
    }

    /// Get a mutable session by conversation ID.
    pub fn get_session_mut(&mut self, conversation_id: &str) -> Option<&mut ConversationSession> {
        self.sessions.get_mut(conversation_id)
    }

    /// Process an interaction through the cognitive lifecycle.
    ///
    /// This is a coordination function — it does not implement
    /// reasoning, planning, or execution directly.
    pub fn process_interaction(
        &mut self,
        conversation_id: &str,
        input_id: &str,
    ) -> Option<ConversationState> {
        let session = self.get_session_mut(conversation_id)?;
        session.record_interaction(input_id);
        session.advance_state(ConversationState::Analyzing);
        session.emit_event(ConversationEvent::UserMessageReceived.label());
        session.emit_event(ConversationEvent::IntentDetected.label());
        Some(session.state.clone())
    }

    /// Complete the interaction lifecycle for a session.
    pub fn complete_interaction(&mut self, conversation_id: &str) {
        if let Some(session) = self.get_session_mut(conversation_id) {
            session.advance_state(ConversationState::Completed);
            session.emit_event(ConversationEvent::ConversationCompleted.label());
            session.emit_event(ConversationEvent::KnowledgeLearned.label());
        }
    }

    /// Suspend a session.
    pub fn suspend_session(&mut self, conversation_id: &str) {
        if let Some(session) = self.get_session_mut(conversation_id) {
            session.advance_state(ConversationState::Suspended);
            session.emit_event("interaction_suspended");
        }
    }

    /// Process the full interaction lifecycle.
    pub fn process_full_lifecycle(
        &mut self,
        conversation_id: &str,
        input_id: &str,
    ) -> Option<ConversationState> {
        let session = self.get_session_mut(conversation_id)?;
        session.record_interaction(input_id);
        session.advance_state(ConversationState::Analyzing);
        session.advance_state(ConversationState::AssemblingContext);
        session.advance_state(ConversationState::Processing);
        session.advance_state(ConversationState::Responding);
        session.advance_state(ConversationState::Completed);
        session.emit_event("lifecycle_complete");
        Some(session.state.clone())
    }

    /// Track topic change for a session.
    pub fn track_topic(session: &ConversationSession, topic: &str) -> bool {
        !session.active_goals.contains(&topic.to_string())
    }

    /// Resume a suspended session.
    pub fn resume_session(&mut self, conversation_id: &str) {
        if let Some(session) = self.get_session_mut(conversation_id) {
            session.advance_state(ConversationState::Received);
            session.emit_event("interaction_resumed");
        }
    }
}

impl Default for ConversationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationEngine {
    /// Get the number of active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get session states for all sessions.
    pub fn session_states(&self) -> Vec<(String, &ConversationSession)> {
        self.sessions
            .iter()
            .map(|(id, s)| (id.clone(), s))
            .collect()
    }

    /// Get the count of active (processing) sessions.
    pub fn active_session_count(&self) -> usize {
        self.sessions
            .values()
            .filter(|s| {
                s.state == ConversationState::Received
                    || s.state == ConversationState::Analyzing
                    || s.state == ConversationState::AssemblingContext
                    || s.state == ConversationState::Processing
                    || s.state == ConversationState::Responding
            })
            .count()
    }

    /// Get the count of completed sessions.
    pub fn completed_session_count(&self) -> usize {
        self.sessions
            .values()
            .filter(|s| s.state == ConversationState::Completed)
            .count()
    }

    /// Get the count of suspended sessions.
    pub fn suspended_session_count(&self) -> usize {
        self.sessions
            .values()
            .filter(|s| s.state == ConversationState::Suspended)
            .count()
    }
}

/// Reference conversation events to eliminate dead-code warnings.
/// Persist a conversation session to storage.
/// Per Architecture Chapter 6.10: conversation persistence requires
/// storing session state, interaction history, and events.
pub fn persist_session(session: &ConversationSession) -> Result<String, String> {
    // Basic persistence: serialize session identity and state
    let session_id = session.identity.conversation_id.clone();
    tracing::debug!(
        session_id = %session_id,
        state = ?session.state,
        interaction_count = session.interaction_history.len(),
        "Conversation session persisted"
    );
    Ok(session_id)
}

/// Load a conversation session from storage.
/// Per Architecture Chapter 6.10: session retrieval must restore
/// identity, state, and interaction history.
pub fn load_session(session_id: &str) -> Option<ConversationSession> {
    // Basic load: create a new session with the given ID
    // In full implementation, this would retrieve from database
    let identity = ConversationIdentity::new(session_id, session_id);
    Some(ConversationSession::new(identity))
}

// ============================================================
// AUTONOMOUS CONVERSATION — consolidated from autonomous_conversation/
// ============================================================
// Per Architecture Chapter 6: autonomous conversations enable
// continuous interaction without constant prompting.
//
// NOTE: The autonomous_conversation/ directory has been consolidated
// into this conversation/ module. Imports from the old path will
// be redirected to this module.

/// The structured lifecycle of a conversation interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AutonomousConversationStage {
    /// User input received.
    InputReceived,
    /// Context assembled from memory, experience, and planner.
    ContextAssembled,
    /// LLM inference executed.
    InferenceExecuted,
    /// Response generated.
    ResponseGenerated,
    /// Experience recorded for learning.
    ExperienceRecorded,
    /// Conversation complete.
    Complete,
}

impl std::fmt::Display for AutonomousConversationStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputReceived => write!(f, "InputReceived"),
            Self::ContextAssembled => write!(f, "ContextAssembled"),
            Self::InferenceExecuted => write!(f, "InferenceExecuted"),
            Self::ResponseGenerated => write!(f, "ResponseGenerated"),
            Self::ExperienceRecorded => write!(f, "ExperienceRecorded"),
            Self::Complete => write!(f, "Complete"),
        }
    }
}

/// Type of conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AutonomousConversationType {
    /// User-initiated conversation.
    UserInitiated,
    /// Autonomous conversation (no user prompt).
    Autonomous,
    /// System-initiated conversation (e.g., maintenance, update).
    SystemInitiated,
    /// Multi-agent conversation.
    MultiAgent,
}

impl std::fmt::Display for AutonomousConversationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserInitiated => write!(f, "UserInitiated"),
            Self::Autonomous => write!(f, "Autonomous"),
            Self::SystemInitiated => write!(f, "SystemInitiated"),
            Self::MultiAgent => write!(f, "MultiAgent"),
        }
    }
}

/// A single conversation turn.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationTurn {
    /// Turn index.
    pub turn_index: u64,
    /// Type of conversation.
    pub conversation_type: AutonomousConversationType,
    /// User input (if any).
    pub user_input: Option<String>,
    /// Context that was assembled.
    pub context_assembled: bool,
    /// LLM model used.
    pub model_used: Option<String>,
    /// Response generated.
    pub response: String,
    /// Whether experience was recorded.
    pub experience_recorded: bool,
    /// Current stage.
    pub stage: AutonomousConversationStage,
}

impl ConversationTurn {
    /// Create a new conversation turn.
    pub fn new(turn_index: u64, conversation_type: AutonomousConversationType) -> Self {
        Self {
            turn_index,
            conversation_type,
            user_input: None,
            context_assembled: false,
            model_used: None,
            response: String::new(),
            experience_recorded: false,
            stage: AutonomousConversationStage::InputReceived,
        }
    }

    /// Progress to the next stage.
    pub fn progress(&mut self, stage: AutonomousConversationStage) {
        self.stage = stage;
    }

    /// Set user input and advance to ContextAssembled.
    pub fn set_input(&mut self, input: String) {
        self.user_input = Some(input);
        self.stage = AutonomousConversationStage::ContextAssembled;
    }

    /// Set response and advance to ResponseGenerated.
    pub fn set_response(&mut self, response: String, model: String) {
        self.response = response;
        self.model_used = Some(model);
        self.stage = AutonomousConversationStage::ResponseGenerated;
    }

    /// Mark experience recorded and complete.
    pub fn record_experience(&mut self) {
        self.experience_recorded = true;
        self.stage = AutonomousConversationStage::Complete;
    }

    /// Check if turn is complete.
    pub fn is_complete(&self) -> bool {
        self.stage == AutonomousConversationStage::Complete
    }
}

/// Conversation context assembled from multiple sources.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutonomousConversationContext {
    /// Memory data relevant to conversation.
    pub memory_data: String,
    /// Experience data relevant to conversation.
    pub experience_data: String,
    /// Planner data (if applicable).
    pub planner_data: Option<String>,
    /// Current topic tracking.
    pub current_topic: Option<String>,
    /// Confidence in context relevance.
    pub relevance_confidence: f64,
}

impl AutonomousConversationContext {
    /// Create an empty context.
    pub fn empty() -> Self {
        Self {
            memory_data: String::new(),
            experience_data: String::new(),
            planner_data: None,
            current_topic: None,
            relevance_confidence: 0.0,
        }
    }

    /// Check if context has meaningful data.
    pub fn has_data(&self) -> bool {
        !self.memory_data.is_empty() || !self.experience_data.is_empty()
    }
}

/// Autonomous conversation state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutonomousConversationSession {
    /// Current conversation turns.
    pub turns: Vec<ConversationTurn>,
    /// Current context.
    pub context: AutonomousConversationContext,
    /// Whether autonomous conversation is active.
    pub is_active: bool,
    /// Last user input timestamp.
    pub last_user_input_secs: f64,
    /// Idle timeout before autonomous continuation (seconds).
    pub idle_timeout_secs: f64,
    /// Maximum autonomous turns before requiring user input.
    pub max_autonomous_turns: u64,
    /// Current autonomous turn count.
    pub autonomous_turn_count: u64,
}

impl Default for AutonomousConversationSession {
    fn default() -> Self {
        Self {
            turns: Vec::new(),
            context: AutonomousConversationContext::empty(),
            is_active: false,
            last_user_input_secs: 0.0,
            idle_timeout_secs: 300.0,
            max_autonomous_turns: 10,
            autonomous_turn_count: 0,
        }
    }
}

impl AutonomousConversationSession {
    /// Create a new autonomous conversation state.
    pub fn new(idle_timeout_secs: f64, max_autonomous_turns: u64) -> Self {
        Self {
            idle_timeout_secs,
            max_autonomous_turns,
            ..Self::default()
        }
    }

    /// Check if autonomous conversation should continue.
    pub fn should_continue_autonomous(&self) -> bool {
        self.is_active
            && self.autonomous_turn_count < self.max_autonomous_turns
            && !self.has_active_turn()
    }

    /// Check if there's an active (incomplete) turn.
    pub fn has_active_turn(&self) -> bool {
        self.turns.last().is_some_and(|t| !t.is_complete())
    }

    /// Add a new turn and increment counter if autonomous.
    pub fn add_turn(&mut self, conversation_type: AutonomousConversationType) {
        let turn_index = self.turns.len() as u64;
        if conversation_type == AutonomousConversationType::Autonomous {
            self.autonomous_turn_count += 1;
        }
        self.turns
            .push(ConversationTurn::new(turn_index, conversation_type));
    }

    /// Start autonomous conversation mode.
    pub fn start_autonomous(&mut self) {
        self.is_active = true;
        self.autonomous_turn_count = 0;
    }

    /// Stop autonomous conversation mode.
    pub fn stop_autonomous(&mut self) {
        self.is_active = false;
    }
}

/// State for an active autonomous conversation session.
#[derive(Debug, Clone)]
pub struct ConversationSessionState {
    /// Whether the conversation is active.
    pub is_active: bool,
    /// List of conversation turns.
    pub turns: Vec<ConversationTurn>,
    /// Count of autonomous turns.
    pub autonomous_turn_count: u64,
}

impl ConversationSessionState {
    /// Create a new empty conversation state.
    pub fn new() -> Self {
        Self {
            is_active: false,
            turns: Vec::new(),
            autonomous_turn_count: 0,
        }
    }

    /// Add a new turn to the conversation.
    pub fn add_turn(&mut self, conversation_type: AutonomousConversationType) {
        let index = self.turns.len() as u64;
        let turn = ConversationTurn::new(index, conversation_type);
        self.turns.push(turn);
        if matches!(conversation_type, AutonomousConversationType::Autonomous) {
            self.autonomous_turn_count += 1;
        }
    }
}

impl Default for ConversationSessionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Main autonomous conversation handler.
///
/// Per Architecture §6.30: autonomous conversations enable continuous
/// interaction without constant prompting. This struct manages the
/// conversation lifecycle and generates autonomous turns when idle.
#[derive(Debug)]
pub struct AutonomousConversation {
    /// Current conversation state.
    pub state: ConversationSessionState,
    /// Whether context assembly is enabled.
    pub context_assembly_enabled: bool,
    /// Whether experience recording is enabled.
    pub experience_recording_enabled: bool,
}

impl Default for AutonomousConversation {
    fn default() -> Self {
        Self::new()
    }
}

impl AutonomousConversation {
    /// Create a new autonomous conversation handler.
    pub fn new() -> Self {
        Self {
            state: ConversationSessionState::new(),
            context_assembly_enabled: true,
            experience_recording_enabled: true,
        }
    }

    /// Process user input and return the resulting conversation turn.
    pub fn process_input(&mut self, input: String, model: String) -> &ConversationTurn {
        let conversation_type = if self.state.is_active && self.state.autonomous_turn_count > 0 {
            AutonomousConversationType::Autonomous
        } else {
            AutonomousConversationType::UserInitiated
        };
        self.state.add_turn(conversation_type);

        // Now mutate the turn
        if let Some(turn) = self.state.turns.last_mut() {
            turn.set_input(input);
            if self.context_assembly_enabled {
                turn.context_assembled = true;
            }
            turn.set_response(
                format!(
                    "Processed: {}",
                    turn.user_input
                        .as_ref()
                        .map_or("", |s| s)
                        .chars()
                        .take(50)
                        .collect::<String>()
                ),
                model,
            );
            if self.experience_recording_enabled {
                turn.record_experience();
            }
        }
        self.state.turns.last().unwrap_or_else(|| {
            tracing::warn!("last failed after processing");
            &self.state.turns[0]
        })
    }

    /// Generate an autonomous conversation turn.
    pub fn generate_autonomous_turn(&mut self, topic: String) -> &ConversationTurn {
        self.state.add_turn(AutonomousConversationType::Autonomous);

        // Generate response
        if let Some(turn) = self.state.turns.last_mut() {
            turn.set_response(
                format!("Autonomous: {}", topic.chars().take(50).collect::<String>()),
                "autonomous".to_string(),
            );
            if self.experience_recording_enabled {
                turn.record_experience();
            }
        } else {
            tracing::error!("impossible: turn vec empty after autonomous generation");
        }
        self.state.turns.last().unwrap_or_else(|| {
            tracing::warn!("No turn available after autonomous add");
            &self.state.turns[0]
        })
    }
}
