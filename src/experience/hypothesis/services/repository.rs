// robot/src/experience/hypothesis/services/repository.rs

// robot/src/experience/hypothesis/services/repository.rs

//! ============================================================================
//! HYPOTHESIS REPOSITORY
//! ============================================================================
//!
//! Provides storage and retrieval operations for hypotheses.
//!
//! The repository abstracts persistence away from the rest of the system.
//!
//! Future implementations may use:
//! - SQLite
//! - graph database
//! - vector storage
//! - distributed storage
//! - file persistence

use std::collections::HashMap;

use anyhow::Result;

use crate::experience::hypothesis::core::hypothesis::{
    Hypothesis,
    HypothesisId,
};


/// ============================================================================
/// REPOSITORY
/// ============================================================================

#[derive(Debug, Default)]
pub struct HypothesisRepository {
    hypotheses: HashMap<String, Hypothesis>,
}


impl HypothesisRepository {

    /// Create a new repository.
    pub fn new() -> Self {
        Self {
            hypotheses: HashMap::new(),
        }
    }


    /// Store or update a hypothesis.
    pub fn save(
        &mut self,
        hypothesis: Hypothesis,
    ) -> Result<()> {

        let id = hypothesis.id.0.clone();

        self.hypotheses.insert(
            id,
            hypothesis,
        );

        Ok(())
    }



    /// Retrieve a hypothesis by ID.
    pub fn get(
        &self,
        id: &HypothesisId,
    ) -> Option<&Hypothesis> {

        self.hypotheses.get(&id.0)
    }



    /// Retrieve a mutable hypothesis.
    pub fn get_mut(
        &mut self,
        id: &HypothesisId,
    ) -> Option<&mut Hypothesis> {

        self.hypotheses.get_mut(&id.0)
    }



    /// Remove a hypothesis.
    pub fn delete(
        &mut self,
        id: &HypothesisId,
    ) -> Option<Hypothesis> {

        self.hypotheses.remove(&id.0)
    }



    /// Return all hypotheses.
    pub fn all(
        &self,
    ) -> Vec<&Hypothesis> {

        self.hypotheses.values().collect()
    }



    /// Count stored hypotheses.
    pub fn count(
        &self,
    ) -> usize {

        self.hypotheses.len()
    }



    /// Check whether a hypothesis exists.
    pub fn exists(
        &self,
        id: &HypothesisId,
    ) -> bool {

        self.hypotheses.contains_key(&id.0)
    }



    /// Remove all hypotheses.
    pub fn clear(
        &mut self,
    ) {

        self.hypotheses.clear();
    }
}
