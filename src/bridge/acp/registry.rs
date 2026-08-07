// src/bridge/acp/registry.rs
//! ACP registry for agent discovery

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};

use super::agent::AcpAgent;
use super::message::AcpAgentId;

/// ACP registry for agent discovery
pub struct AcpRegistry {
    agents: std::sync::RwLock<HashMap<AcpAgentId, Arc<dyn AcpAgent>>>,
}

impl AcpRegistry {
    pub fn new() -> Self {
        Self {
            agents: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Register an agent
    pub fn register(&self, agent: Arc<dyn AcpAgent>) -> Result<()> {
        let id = agent.id().clone();
        let mut agents = self.agents.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
        agents.insert(id, agent);
        Ok(())
    }

    /// Unregister an agent
    pub fn unregister(&self, id: &AcpAgentId) -> Result<Option<Arc<dyn AcpAgent>>> {
        let mut agents = self.agents.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
        Ok(agents.remove(id))
    }

    /// Get an agent by ID
    pub fn get(&self, id: &AcpAgentId) -> Result<Option<Arc<dyn AcpAgent>>> {
        let agents = self.agents.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
        Ok(agents.get(id).cloned())
    }

    /// Get all agents of a specific type
    pub fn get_by_type(&self, agent_type: &str) -> Result<Vec<Arc<dyn AcpAgent>>> {
        let agents = self.agents.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
        Ok(agents
            .values()
            .filter(|a| a.id().agent_type == agent_type)
            .cloned()
            .collect())
    }

    /// List all registered agent IDs
    pub fn list_agents(&self) -> Result<Vec<AcpAgentId>> {
        let agents = self.agents.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
        Ok(agents.keys().cloned().collect())
    }

    /// Count registered agents
    pub fn count(&self) -> usize {
        self.agents.read().map(|g| g.len()).unwrap_or(0)
    }
}

impl Default for AcpRegistry {
    fn default() -> Self {
        Self::new()
    }
}
