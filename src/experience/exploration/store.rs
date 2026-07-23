// /src/experience/exploration/store.rs
// Repository for persisting and retrieving explorations

#![allow(dead_code)]

use anyhow::Result;
use std::collections::HashMap;
use std::sync::RwLock;

use super::Exploration;
use super::exploration::ExplorationStatus;

/// Trait for exploration storage
pub trait ExplorationRepository: Send + Sync {
    /// Create a new exploration
    fn create(&self, exploration: &Exploration) -> Result<()>;

    /// Get an exploration by ID
    fn get(&self, id: &str) -> Result<Option<Exploration>>;

    /// Update an existing exploration
    fn update(&self, exploration: &Exploration) -> Result<()>;

    /// List all active explorations
    fn list_active(&self) -> Result<Vec<Exploration>>;
}

/// Thread-safe in-memory implementation of ExplorationRepository
pub struct InMemoryExplorationRepository {
    explorations: RwLock<HashMap<String, Exploration>>,
}

impl InMemoryExplorationRepository {
    /// Create a new empty repository
    pub fn new() -> Self {
        Self {
            explorations: RwLock::new(HashMap::new()),
        }
    }

    /// Get the count of explorations
    pub fn count(&self) -> Result<usize> {
        let explorations = self.explorations.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(explorations.len())
    }

    /// List all explorations
    pub fn list_all(&self) -> Result<Vec<Exploration>> {
        let explorations = self.explorations.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(explorations.values().cloned().collect())
    }

    /// List explorations by status
    pub fn list_by_status(&self, status: ExplorationStatus) -> Result<Vec<Exploration>> {
        let explorations = self.explorations.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(explorations.values()
            .filter(|e| e.status == status)
            .cloned()
            .collect())
    }

    /// Delete an exploration by ID
    pub fn delete(&self, id: &str) -> Result<Option<Exploration>> {
        let mut explorations = self.explorations.write()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(explorations.remove(id))
    }

    /// Search explorations by title
    pub fn search_by_title(&self, query: &str) -> Result<Vec<Exploration>> {
        let explorations = self.explorations.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        let query_lower = query.to_lowercase();
        Ok(explorations.values()
            .filter(|e| e.title.to_lowercase().contains(&query_lower))
            .cloned()
            .collect())
    }
}

impl ExplorationRepository for InMemoryExplorationRepository {
    fn create(&self, exploration: &Exploration) -> Result<()> {
        let mut explorations = self.explorations.write()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        explorations.insert(exploration.id.clone(), exploration.clone());
        Ok(())
    }

    fn get(&self, id: &str) -> Result<Option<Exploration>> {
        let explorations = self.explorations.read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(explorations.get(id).cloned())
    }

    fn update(&self, exploration: &Exploration) -> Result<()> {
        let mut explorations = self.explorations.write()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        if !explorations.contains_key(&exploration.id) {
            return Err(anyhow::anyhow!("Exploration not found: {}", exploration.id));
        }
        explorations.insert(exploration.id.clone(), exploration.clone());
        Ok(())
    }

    fn list_active(&self) -> Result<Vec<Exploration>> {
        self.list_by_status(ExplorationStatus::Active)
    }
}

impl Default for InMemoryExplorationRepository {
    fn default() -> Self {
        Self::new()
    }
}
