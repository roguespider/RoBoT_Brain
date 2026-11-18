// src/agent/autonomous.rs
//! Autonomous background agents (Architecture §3.8, §5.7).
//!
//! Autonomous background agents are agents that operate independently
//! without requiring continuous human input. They maintain their own
//! objective queue, cycle through the cognitive loop, and record
//! outcomes as experiences.
//!
//! Per v0.0.2 §3.8 "Future Growth":
//! "autonomous background agents" are a future expansion capability
//! that extends the architecture beyond reactive task execution.
//!
//! Per v0.0.2 §5.7: each agent runs the full cognitive loop:
//! Goal -> Plan -> Retrieve -> Evaluate -> Act -> Record -> Learn

use std::sync::Arc;

use chrono::Utc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::types::AgentGoal;

/// An autonomous agent that operates independently.
///
/// Each agent maintains its own objective queue and runs the cognitive
/// loop autonomously. Agents are identified by a unique ID and have
/// a mission statement that guides their autonomous behavior.
#[derive(Clone)]
pub struct AutonomousAgent {
    /// Unique identifier for this agent.
    pub id: String,
    /// Mission statement guiding autonomous behavior.
    pub mission: String,
    /// Current objective the agent is pursuing.
    pub current_goal: Option<AgentGoal>,
    /// Status of the autonomous agent.
    pub status: AgentStatus,
    /// Total autonomous goals completed.
    pub goals_completed: u64,
    /// Total autonomous goals failed.
    pub goals_failed: u64,
    /// Whether the agent is currently running.
    pub running: bool,
    /// Creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Status of an autonomous background agent.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AgentStatus {
    /// Agent is idle, waiting for objectives.
    #[default]
    Idle,
    /// Agent is actively processing an objective.
    Active,
    /// Agent is paused (manually suspended).
    Paused,
    /// Agent has stopped.
    Stopped,
}

impl AutonomousAgent {
    /// Create a new autonomous agent.
    pub fn new(mission: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            mission: mission.into(),
            current_goal: None,
            status: AgentStatus::Idle,
            goals_completed: 0,
            goals_failed: 0,
            running: false,
            created_at: Utc::now(),
        }
    }

    /// Start the autonomous agent.
    pub fn start(&mut self) {
        self.running = true;
        self.status = AgentStatus::Idle;
        tracing::info!(
            agent_id = %self.id,
            mission = %self.mission,
            "Autonomous agent started"
        );
    }

    /// Stop the autonomous agent.
    pub fn stop(&mut self) {
        self.running = false;
        self.status = AgentStatus::Stopped;
        tracing::info!(agent_id = %self.id, "Autonomous agent stopped");
    }

    /// Pause the autonomous agent.
    pub fn pause(&mut self) {
        self.status = AgentStatus::Paused;
        tracing::info!(agent_id = %self.id, "Autonomous agent paused");
    }

    /// Resume a paused agent.
    pub fn resume(&mut self) {
        if self.running {
            self.status = AgentStatus::Idle;
            tracing::info!(agent_id = %self.id, "Autonomous agent resumed");
        }
    }

    /// Set the current goal being pursued.
    pub fn set_goal(&mut self, goal: AgentGoal) {
        self.current_goal = Some(goal);
        self.status = AgentStatus::Active;
    }

    /// Clear the current goal after completion.
    pub fn clear_goal(&mut self, success: bool) {
        if self.current_goal.take().is_some() {
            if success {
                self.goals_completed += 1;
            } else {
                self.goals_failed += 1;
            }
            self.status = if self.running {
                AgentStatus::Idle
            } else {
                AgentStatus::Stopped
            };
        }
    }

    /// Get agent statistics.
    pub fn stats(&self) -> AgentStats {
        AgentStats {
            id: self.id.clone(),
            status: self.status.clone(),
            running: self.running,
            goals_completed: self.goals_completed,
            goals_failed: self.goals_failed,
            mission: self.mission.clone(),
        }
    }
}

/// Statistics about an autonomous agent.
#[derive(Debug, Clone)]
pub struct AgentStats {
    pub id: String,
    pub status: AgentStatus,
    pub running: bool,
    pub goals_completed: u64,
    pub goals_failed: u64,
    pub mission: String,
}

/// Manages a collection of autonomous background agents.
pub struct AutonomousAgentManager {
    agents: Arc<Mutex<Vec<AutonomousAgent>>>,
}

impl Default for AutonomousAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AutonomousAgentManager {
    /// Create a new autonomous agent manager.
    pub fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a new autonomous agent.
    pub async fn register(&self, agent: AutonomousAgent) -> String {
        let id = agent.id.clone();
        let mut agents = self.agents.lock().await;
        agents.push(agent);
        tracing::info!("Registered autonomous agent: {}", id);
        id
    }

    /// Get an agent by ID.
    pub async fn get(&self, id: &str) -> Option<AutonomousAgent> {
        let agents = self.agents.lock().await;
        agents.iter().find(|a| a.id == id).cloned()
    }

    /// List all registered agents.
    pub async fn list(&self) -> Vec<AutonomousAgent> {
        let agents = self.agents.lock().await;
        agents.clone()
    }

    /// Start all registered agents.
    pub async fn start_all(&self) {
        let mut agents = self.agents.lock().await;
        for agent in agents.iter_mut() {
            agent.start();
        }
        tracing::info!("Started {} autonomous agents", agents.len());
    }

    /// Stop all registered agents.
    pub async fn stop_all(&self) {
        let mut agents = self.agents.lock().await;
        for agent in agents.iter_mut() {
            agent.stop();
        }
        tracing::info!("Stopped {} autonomous agents", agents.len());
    }

    /// Get stats for all agents.
    pub async fn stats(&self) -> Vec<AgentStats> {
        let agents = self.agents.lock().await;
        agents.iter().map(|a| a.stats()).collect()
    }
}
