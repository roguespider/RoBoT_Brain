//! Rollback journal for the safety gate (Architecture §16 — rollback).
//!
//! The rollback journal records every mutation the autonomous loop performs so
//! that a failed action can be reversed. For append-only systems (memory,
//! knowledge, experiences), "rollback" means marking the mutation as
//! inactive/rolled-back rather than physically deleting it — preserving
//! audit history while undoing the effect.
//!
//! The journal is in-memory per loop iteration. On failure, the loop calls
//! `rollback_all()` to mark all mutations in the current journal as reversed,
//! then records a "rollback" experience so the learning spine learns from
//! the reversal.

use super::types::RollbackEntry;
/// Tracks mutations for potential reversal within a single agent loop
/// iteration.
pub struct RollbackJournal {
    entries: Vec<RollbackEntry>,
}

impl RollbackJournal {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Record a mutation in the journal. Called before executing a mutating
    /// action so the journal has the entry even if execution crashes.
    pub fn record(&mut self, action: &str, target_id: String) -> &RollbackEntry {
        let entry = RollbackEntry::new(action, target_id);
        self.entries.push(entry);
        self.entries.last().expect("just pushed")
    }

    /// Roll back all recorded mutations by marking them as reversed.
    ///
    /// Returns the list of rolled-back entries so the caller can record a
    /// "rollback" experience citing what was reversed.
    pub fn rollback_all(&mut self) -> Vec<RollbackEntry> {
        for entry in &mut self.entries {
            entry.rolled_back = true;
        }
        let result = self.entries.clone();
        tracing::info!(
            "Rollback journal: reversed {} mutation(s)",
            result.len()
        );
        result
    }

    /// Roll back a specific mutation by its target ID.
    pub fn rollback_target(&mut self, target_id: &str) -> Option<&RollbackEntry> {
        let entry = self.entries.iter_mut().find(|e| e.target_id == target_id)?;
        entry.rolled_back = true;
        Some(entry)
    }

    /// Number of active (non-rolled-back) mutations.
    pub fn active_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.rolled_back).count()
    }

    /// All entries (for audit / reporting).
    pub fn entries(&self) -> &[RollbackEntry] {
        &self.entries
    }

    /// Clear the journal (called at the start of a new iteration after the
    /// previous iteration's entries have been recorded to durable storage).
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for RollbackJournal {
    fn default() -> Self {
        Self::new()
    }
}
