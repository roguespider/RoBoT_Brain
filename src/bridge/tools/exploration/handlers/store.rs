//! Shared in-memory exploration store and lock-recovery helpers.

use std::collections::HashMap;
use std::sync::RwLock;

use crate::experience::exploration::Exploration;
use crate::experience::types::ExperienceContext;

lazy_static::lazy_static! {
    pub(crate) static ref EXPLORATION_STORE: RwLock<HashMap<String, Exploration>> = RwLock::new(HashMap::new());
}

/// Run `f` against a write-lock on the global store, transparently recovering
/// from a poisoned lock so callers never need to duplicate the Ok/Err arms.
pub(crate) fn with_store<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<String, Exploration>) -> R,
{
    match EXPLORATION_STORE.write() {
        Ok(mut store) => f(&mut *store),
        Err(poisoned) => f(&mut *poisoned.into_inner()),
    }
}

/// Auto-create a default exploration for `id` if it does not already exist.
/// Used by handlers for test compatibility when an ID is referenced that was
/// never created via `start_exploration`.
pub(crate) fn ensure_exploration(
    store: &mut HashMap<String, Exploration>,
    id: &str,
) {
    if !store.contains_key(id) {
        let exp = Exploration::new(
            id.to_string(),
            "Auto-generated exploration".to_string(),
            "auto".to_string(),
            ExperienceContext::default(),
        );
        store.insert(id.to_string(), exp);
    }
}
