// src/world_model/store.rs
//! The World Model store: an in-memory entity-relationship graph.
//!
//! Per Architecture §14, the World Model "stores understanding" — typed
//! entities and the relationships between them. This store supports:
//!
//!   * Upserting entities and relationships (with confidence merging).
//!   * Direct lookup by id and by name.
//!   * Neighbor traversal (who is related to X, and how?).
//!   * Reasoning queries: what blocks a goal? what depends on a resource?
//!     what did an event produce/consume?
//!
//! These queries let the agent loop (§5.7) reason about the world when
//! evaluating an action's safety and confidence.

use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::types::{Entity, EntityKind, RelationKind, Relationship};

/// The World Model: a typed entity-relationship graph.
pub struct WorldModel {
    entities: RwLock<HashMap<Uuid, Entity>>,
    /// Name → entity id index for fast lookup by human-readable name.
    name_index: RwLock<HashMap<String, Uuid>>,
    relationships: RwLock<Vec<Relationship>>,
}

impl WorldModel {
    pub fn new() -> Self {
        Self {
            entities: RwLock::new(HashMap::new()),
            name_index: RwLock::new(HashMap::new()),
            relationships: RwLock::new(Vec::new()),
        }
    }

    /// Add or update an entity. If an entity with the same name and kind
    /// already exists, its properties are merged and confidence takes the max.
    pub async fn upsert_entity(&self, mut entity: Entity) -> Uuid {
        let mut name_index = self.name_index.write().await;
        if let Some(existing_id) = name_index.get(&entity.name) {
            // Same name: merge into the existing entity.
            let mut entities = self.entities.write().await;
            if let Some(existing) = entities.get_mut(existing_id) {
                for (k, v) in entity.properties.drain() {
                    existing.properties.insert(k, v);
                }
                existing.confidence = existing.confidence.max(entity.confidence);
                existing.salience = existing.salience.max(entity.salience);
                existing.updated_at = chrono::Utc::now();
                return *existing_id;
            }
        }
        let id = entity.id;
        name_index.insert(entity.name.clone(), id);
        self.entities.write().await.insert(id, entity);
        id
    }

    /// Add a typed relationship between two entities.
    pub async fn add_relationship(&self, rel: Relationship) -> Result<()> {
        let entities = self.entities.read().await;
        if !entities.contains_key(&rel.source) {
            anyhow::bail!("source entity {} not found", rel.source);
        }
        if !entities.contains_key(&rel.target) {
            anyhow::bail!("target entity {} not found", rel.target);
        }
        drop(entities);
        self.relationships.write().await.push(rel);
        Ok(())
    }

    /// Look up an entity by id.
    pub async fn get_entity(&self, id: Uuid) -> Option<Entity> {
        self.entities.read().await.get(&id).cloned()
    }

    /// Look up an entity by name.
    pub async fn find_by_name(&self, name: &str) -> Option<Entity> {
        let name_index = self.name_index.read().await;
        let id = name_index.get(name)?;
        self.entities.read().await.get(id).cloned()
    }

    /// All entities of a given kind.
    pub async fn entities_of_kind(&self, kind: EntityKind) -> Vec<Entity> {
        self.entities
            .read()
            .await
            .values()
            .filter(|e| e.kind == kind)
            .cloned()
            .collect()
    }

    /// All relationships involving `id` (as source or target).
    pub async fn relationships_for(&self, id: Uuid) -> Vec<Relationship> {
        self.relationships
            .read()
            .await
            .iter()
            .filter(|r| r.source == id || r.target == id)
            .cloned()
            .collect()
    }

    /// Reasoning query (§14): what entities block the given goal?
    pub async fn blockers_of(&self, goal_id: Uuid) -> Vec<Entity> {
        let rels = self.relationships.read().await;
        let blocking_ids: Vec<Uuid> = rels
            .iter()
            .filter(|r| r.target == goal_id && r.kind == RelationKind::Blocks)
            .map(|r| r.source)
            .collect();
        drop(rels);
        let entities = self.entities.read().await;
        blocking_ids
            .iter()
            .filter_map(|id| entities.get(id).cloned())
            .collect()
    }

    /// Reasoning query (§14): what does `id` depend on?
    pub async fn dependencies_of(&self, id: Uuid) -> Vec<Entity> {
        let rels = self.relationships.read().await;
        let dep_ids: Vec<Uuid> = rels
            .iter()
            .filter(|r| r.source == id && r.kind == RelationKind::DependsOn)
            .map(|r| r.target)
            .collect();
        drop(rels);
        let entities = self.entities.read().await;
        dep_ids
            .iter()
            .filter_map(|id| entities.get(id).cloned())
            .collect()
    }

    /// Reasoning query (§14): what resources does `id` consume?
    pub async fn resources_consumed_by(&self, id: Uuid) -> Vec<Entity> {
        let rels = self.relationships.read().await;
        let res_ids: Vec<Uuid> = rels
            .iter()
            .filter(|r| r.source == id && r.kind == RelationKind::Consumes)
            .map(|r| r.target)
            .collect();
        drop(rels);
        let entities = self.entities.read().await;
        res_ids
            .iter()
            .filter_map(|id| entities.get(id).cloned())
            .collect()
    }

    /// Total entity count (for diagnostics / self-check).
    pub async fn entity_count(&self) -> usize {
        self.entities.read().await.len()
    }

    /// Total relationship count (for diagnostics / self-check).
    pub async fn relationship_count(&self) -> usize {
        self.relationships.read().await.len()
    }
}

impl Default for WorldModel {
    fn default() -> Self {
        Self::new()
    }
}
