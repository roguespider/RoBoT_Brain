//! Entity resolution - Per Architecture §20.4 "Knowledge discovery"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Entity resolution mapping aliases to canonical IDs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityResolution {
    /// The canonical identifier for the entity.
    pub canonical_id: String,
    /// Known aliases for this entity.
    pub aliases: Vec<String>,
}

/// Resolve an entity name to its canonical ID.
pub fn resolve_entity(name: &str, table: &HashMap<String, String>) -> Option<String> {
    table.get(name).cloned()
}

/// Register an alias mapping to a canonical ID.
pub fn register_alias(table: &mut HashMap<String, String>, canonical_id: &str, alias: &str) {
    table.insert(alias.to_string(), canonical_id.to_string());
}

/// Active reference to knowledge resolution contracts.
pub fn reference_knowledge_resolution_contracts() {
    let mut table = HashMap::new();
    register_alias(&mut table, "canonical-id", "alias-name");
    let resolved = resolve_entity("alias-name", &table);
    tracing::info!(
        resolved = ?resolved,
        "Knowledge resolution actively referenced"
    );
}
