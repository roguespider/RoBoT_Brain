// src/world_model/mod.rs

//! World Model
//!
//! Per Architecture: Represents the system's understanding of the world,
//! including entities, relationships, and causal connections.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// An entity in the world model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier
    pub id: Uuid,
    
    /// Entity type (person, object, concept, etc.)
    pub entity_type: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Properties of this entity
    pub properties: HashMap<String, PropertyValue>,
    
    /// Relationships this entity participates in
    pub relationship_ids: Vec<Uuid>,
    
    /// Confidence in this entity's existence/accuracy
    pub confidence: f32,
    
    /// When this entity was last observed/updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Property value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<String>),
    Nested(HashMap<String, PropertyValue>),
}

impl PropertyValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_number(&self) -> Option<f64> {
        match self {
            PropertyValue::Number(n) => Some(*n),
            _ => None,
        }
    }
}

/// A relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Unique identifier
    pub id: Uuid,
    
    /// Source entity ID
    pub source_id: Uuid,
    
    /// Target entity ID
    pub target_id: Uuid,
    
    /// Relationship type
    pub relation_type: String,
    
    /// Relationship strength (0.0 - 1.0)
    pub strength: f32,
    
    /// Bidirectional flag
    pub bidirectional: bool,
    
    /// Additional context
    pub context: String,
}

/// Causal link between entities or events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    /// Unique identifier
    pub id: Uuid,
    
    /// Cause entity/event ID
    pub cause_id: Uuid,
    
    /// Effect entity/event ID
    pub effect_id: Uuid,
    
    /// Probability of causation
    pub probability: f32,
    
    /// Time lag (in seconds)
    pub time_lag_secs: i64,
}

/// World model - the system's understanding of the world
pub struct WorldModel {
    /// All entities by ID
    entities: HashMap<Uuid, Entity>,
    
    /// All relationships by ID
    relationships: HashMap<Uuid, Relationship>,
    
    /// All causal links
    causal_links: HashMap<Uuid, CausalLink>,
    
    /// Index by entity type
    by_type: HashMap<String, Vec<Uuid>>,
    
    /// Index by relationship type
    by_relation_type: HashMap<String, Vec<Uuid>>,
}

impl WorldModel {
    /// Create a new world model
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            relationships: HashMap::new(),
            causal_links: HashMap::new(),
            by_type: HashMap::new(),
            by_relation_type: HashMap::new(),
        }
    }
    
    // ========================================================================
    // Entity Operations
    // ========================================================================
    
    /// Add or update an entity
    pub fn add_entity(&mut self, entity: Entity) {
        let id = entity.id;
        
        // Update type index
        self.by_type
            .entry(entity.entity_type.clone())
            .or_default()
            .push(id);
        
        self.entities.insert(id, entity);
    }
    
    /// Get an entity by ID
    pub fn get_entity(&self, id: &Uuid) -> Option<&Entity> {
        self.entities.get(id)
    }
    
    /// Get entities by type
    pub fn get_entities_by_type(&self, entity_type: &str) -> Vec<&Entity> {
        self.by_type
            .get(entity_type)
            .map(|ids| ids.iter().filter_map(|id| self.entities.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Update entity properties
    pub fn update_entity(&mut self, id: &Uuid, property: String, value: PropertyValue) -> bool {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.properties.insert(property, value);
            entity.last_updated = chrono::Utc::now();
            true
        } else {
            false
        }
    }
    
    /// Delete an entity and its relationships
    pub fn delete_entity(&mut self, id: &Uuid) -> bool {
        if self.entities.remove(id).is_some() {
            // Remove from type index
            for ids in self.by_type.values_mut() {
                ids.retain(|i| i != id);
            }
            
            // Remove related relationships
            self.relationships.retain(|_, r| r.source_id != *id && r.target_id != *id);
            
            // Remove related causal links
            self.causal_links.retain(|_, c| c.cause_id != *id && c.effect_id != *id);
            
            true
        } else {
            false
        }
    }
    
    // ========================================================================
    // Relationship Operations
    // ========================================================================
    
    /// Add a relationship between entities
    pub fn add_relationship(&mut self, relationship: Relationship) {
        let id = relationship.id;
        
        // Update type index
        self.by_relation_type
            .entry(relationship.relation_type.clone())
            .or_default()
            .push(id);
        
        // Update entity relationship lists
        if let Some(entity) = self.entities.get_mut(&relationship.source_id) {
            entity.relationship_ids.push(id);
        }
        if relationship.bidirectional {
            if let Some(entity) = self.entities.get_mut(&relationship.target_id) {
                entity.relationship_ids.push(id);
            }
        }
        
        self.relationships.insert(id, relationship);
    }
    
    /// Get relationships between two entities
    pub fn get_relationships_between(&self, source: &Uuid, target: &Uuid) -> Vec<&Relationship> {
        self.relationships
            .values()
            .filter(|r| (r.source_id == *source && r.target_id == *target) 
                || (r.bidirectional && r.source_id == *target && r.target_id == *source))
            .collect()
    }
    
    /// Get all relationships for an entity
    pub fn get_entity_relationships(&self, entity_id: &Uuid) -> Vec<&Relationship> {
        self.relationships
            .values()
            .filter(|r| r.source_id == *entity_id || r.target_id == *entity_id)
            .collect()
    }
    
    // ========================================================================
    // Causal Operations
    // ========================================================================
    
    /// Add a causal link
    pub fn add_causal_link(&mut self, link: CausalLink) {
        self.causal_links.insert(link.id, link);
    }
    
    /// Get causes of an entity/event
    pub fn get_causes(&self, effect_id: &Uuid) -> Vec<&CausalLink> {
        self.causal_links
            .values()
            .filter(|c| c.effect_id == *effect_id)
            .collect()
    }
    
    /// Get effects of an entity/event
    pub fn get_effects(&self, cause_id: &Uuid) -> Vec<&CausalLink> {
        self.causal_links
            .values()
            .filter(|c| c.cause_id == *cause_id)
            .collect()
    }
    
    // ========================================================================
    // Query Operations
    // ========================================================================
    
    /// Find entities matching a property pattern
    pub fn find_entities(&self, property: &str, value: &PropertyValue) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.properties.get(property) == Some(value))
            .collect()
    }
    
    /// Traverse relationships to find connected entities
    pub fn find_connected(&self, entity_id: &Uuid, max_depth: usize) -> Vec<Uuid> {
        let mut visited = HashSet::new();
        let mut queue = vec![(*entity_id, 0)];
        
        while let Some((current, depth)) = queue.pop() {
            if depth > max_depth || visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            
            for rel in self.get_entity_relationships(&current) {
                let next = if rel.source_id == current { rel.target_id } else { rel.source_id };
                if !visited.contains(&next) {
                    queue.push((next, depth + 1));
                }
            }
        }
        
        visited.into_iter().collect()
    }
    
    // ========================================================================
    // Statistics
    // ========================================================================
    
    /// Get world model statistics
    pub fn stats(&self) -> WorldModelStats {
        WorldModelStats {
            entity_count: self.entities.len(),
            relationship_count: self.relationships.len(),
            causal_link_count: self.causal_links.len(),
            entity_types: self.by_type.len(),
            relation_types: self.by_relation_type.len(),
        }
    }
}

/// Statistics about the world model
#[derive(Debug, Clone)]
pub struct WorldModelStats {
    pub entity_count: usize,
    pub relationship_count: usize,
    pub causal_link_count: usize,
    pub entity_types: usize,
    pub relation_types: usize,
}

impl Default for WorldModel {
    fn default() -> Self {
        Self::new()
    }
}
