//! Conversation Engine — Interaction orchestration (Architecture Chapter 6).
//!
//! The Conversation Engine coordinates external interaction through a
//! controlled cognitive lifecycle without owning long-term memory,
//! knowledge, experience, or execution logic.
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
        session.emit_event("interaction_received");
        Some(session.state.clone())
    }

    /// Complete the interaction lifecycle for a session.
    pub fn complete_interaction(&mut self, conversation_id: &str) {
        if let Some(session) = self.get_session_mut(conversation_id) {
            session.advance_state(ConversationState::Completed);
            session.emit_event("interaction_completed");
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
