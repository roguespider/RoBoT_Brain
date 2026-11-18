/// ContextPacket data contract - Per Architecture Chapter 5.5 and Chapter 7.
///
/// The ContextPacket represents the temporary reasoning environment assembled for a task.
/// Per Architecture §5.5: observations, conversation history, memories, experiences,
/// goals, planner state, active tasks.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A packet of contextual information for reasoning.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ContextPacket {
    /// Shared metadata (version, source, timestamp, correlation, confidence, provenance).
    pub metadata: Metadata,
    /// Session identifier for correlating related packets.
    pub session_id: String,
    /// Observations relevant to the current context.
    pub observations: Vec<String>,
    /// Conversation history for continuity.
    pub conversation_history: Vec<String>,
    /// Retrieved memories for this context.
    pub memories: Vec<String>,
    /// Relevant experiences for this context.
    pub experiences: Vec<String>,
    /// Active goals being pursued.
    pub goals: Vec<String>,
    /// Planner state for this reasoning cycle.
    pub planner_state: String,
    /// Active tasks being executed.
    pub active_tasks: Vec<String>,
    /// A summary of the current conversation state.
    pub summary: String,
}

impl ContextPacket {
    /// Create a new context packet.
    pub fn new(session_id: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            metadata: Metadata::new("context_packet"),
            session_id: session_id.into(),
            observations: Vec::new(),
            conversation_history: Vec::new(),
            memories: Vec::new(),
            experiences: Vec::new(),
            goals: Vec::new(),
            planner_state: String::new(),
            active_tasks: Vec::new(),
            summary: summary.into(),
        }
    }

    /// Add an observation.
    pub fn with_observation(mut self, observation: impl Into<String>) -> Self {
        self.observations.push(observation.into());
        self
    }

    /// Add conversation history entry.
    pub fn with_history(mut self, entry: impl Into<String>) -> Self {
        self.conversation_history.push(entry.into());
        self
    }

    /// Add a memory reference.
    pub fn with_memory(mut self, memory: impl Into<String>) -> Self {
        self.memories.push(memory.into());
        self
    }

    /// Add an experience reference.
    pub fn with_experience(mut self, experience: impl Into<String>) -> Self {
        self.experiences.push(experience.into());
        self
    }

    /// Add a goal.
    pub fn with_goal(mut self, goal: impl Into<String>) -> Self {
        self.goals.push(goal.into());
        self
    }

    /// Set planner state.
    pub fn with_planner_state(mut self, state: impl Into<String>) -> Self {
        self.planner_state = state.into();
        self
    }

    /// Add an active task.
    pub fn with_active_task(mut self, task: impl Into<String>) -> Self {
        self.active_tasks.push(task.into());
        self
    }
}

