// src/bridge/tools/world_model/mod.rs
//! World Model MCP tools (Architecture §14: World Model)
//!
//! Exposes the world-model entity-relationship graph via MCP so the
//! self_check probe is no longer the only thing keeping these code paths
//! live (TASK-V2-09). Tools cover entity CRUD, relationship linking, and
//! the graph-reasoning queries (blockers, dependencies, resources).

use crate::bridge::mcp::McpContext;
use crate::bridge::tools::ToolOutput;
use crate::world_model::types::{Entity, EntityKind, RelationKind, Relationship};
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// INPUT TYPES
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpsertEntityInput {
    pub name: String,
    pub kind: String,
    pub confidence: Option<f32>,
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddRelationshipInput {
    pub source_id: String,
    pub target_id: String,
    pub kind: String,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetEntityInput {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindEntityInput {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntitiesOfKindInput {
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipsForInput {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReasoningInput {
    pub id: String,
}

// =============================================================================
// HELPERS
// =============================================================================

fn parse_entity_kind(s: &str) -> EntityKind {
    match s.to_lowercase().as_str() {
        "object" => EntityKind::Object,
        "place" => EntityKind::Place,
        "person" => EntityKind::Person,
        "event" => EntityKind::Event,
        "time" => EntityKind::Time,
        "goal" => EntityKind::Goal,
        "resource" => EntityKind::Resource,
        _ => EntityKind::Object,
    }
}

fn parse_relation_kind(s: &str) -> RelationKind {
    match s.to_lowercase().as_str() {
        "located_at" | "locatedat" => RelationKind::LocatedAt,
        "owns" => RelationKind::Owns,
        "participates_in" | "participatesin" => RelationKind::ParticipatesIn,
        "causes" => RelationKind::Causes,
        "depends_on" | "dependson" => RelationKind::DependsOn,
        "blocks" => RelationKind::Blocks,
        "part_of" | "partof" => RelationKind::PartOf,
        "alternative_to" | "alternativeto" => RelationKind::AlternativeTo,
        "consumes" => RelationKind::Consumes,
        "produces" => RelationKind::Produces,
        "before" => RelationKind::Before,
        _ => RelationKind::RelatedTo,
    }
}

fn parse_uuid(s: &str) -> Option<Uuid> {
    Uuid::parse_str(s).ok()
}

fn entity_to_json(entity: &Entity) -> serde_json::Value {
    serde_json::json!({
        "id": entity.id.to_string(),
        "name": entity.name,
        "kind": entity.kind.as_str(),
        "confidence": entity.confidence,
        "properties": entity.properties,
    })
}

fn relationship_to_json(rel: &Relationship) -> serde_json::Value {
    serde_json::json!({
        "source": rel.source.to_string(),
        "target": rel.target.to_string(),
        "kind": rel.kind.as_str(),
        "confidence": rel.confidence,
    })
}

// =============================================================================
// TOOL EXECUTORS
// =============================================================================

/// upsert_world_entity: creates or updates an entity in the world model.
pub async fn execute_upsert_entity(
    input: UpsertEntityInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let mut entity = Entity::new(input.name, parse_entity_kind(&input.kind));
    if let Some(c) = input.confidence {
        entity = entity.with_confidence(c);
    }
    if let Some(props) = input.properties.and_then(|p| p.as_object().cloned()) {
        for (key, value) in props {
            let val_str = match value {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            };
            entity = entity.with_property(key, val_str);
        }
    }
    let id = context.world_model.upsert_entity(entity).await;
    Ok(ToolOutput::success(serde_json::json!({
        "id": id.to_string(),
        "created_or_updated": true,
    })))
}

/// add_world_relationship: links two entities with a typed relationship.
pub async fn execute_add_relationship(
    input: AddRelationshipInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let source_id = match parse_uuid(&input.source_id) {
        Some(id) => id,
        None => {
            return Ok(ToolOutput::error(format!("Invalid source_id UUID: {}", input.source_id)));
        }
    };
    let target_id = match parse_uuid(&input.target_id) {
        Some(id) => id,
        None => {
            return Ok(ToolOutput::error(format!("Invalid target_id UUID: {}", input.target_id)));
        }
    };
    let mut rel = Relationship::new(source_id, target_id, parse_relation_kind(&input.kind));
    if let Some(c) = input.confidence {
        rel = rel.with_confidence(c);
    }
    match context.world_model.add_relationship(rel).await {
        Ok(()) => Ok(ToolOutput::success(serde_json::json!({
            "added": true,
        }))),
        Err(e) => Ok(ToolOutput::error(format!("Failed to add relationship: {}", e),
        )),
    }
}

/// get_world_entity: retrieves an entity by ID.
pub async fn execute_get_entity(
    input: GetEntityInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let id = match parse_uuid(&input.id) {
        Some(id) => id,
        None => {
            return Ok(ToolOutput::error(format!("Invalid UUID: {}", input.id)));
        }
    };
    match context.world_model.get_entity(id).await {
        Some(entity) => Ok(ToolOutput::success(entity_to_json(&entity))),
        None => Ok(ToolOutput::success(serde_json::json!({
            "found": false,
        }))),
    }
}

/// find_world_entity: finds an entity by name.
pub async fn execute_find_entity(
    input: FindEntityInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    match context.world_model.find_by_name(&input.name).await {
        Some(entity) => Ok(ToolOutput::success(entity_to_json(&entity))),
        None => Ok(ToolOutput::success(serde_json::json!({
            "found": false,
        }))),
    }
}

/// list_world_entities: lists all entities of a given kind.
pub async fn execute_entities_of_kind(
    input: EntitiesOfKindInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let kind = parse_entity_kind(&input.kind);
    let entities = context.world_model.entities_of_kind(kind).await;
    let list: Vec<serde_json::Value> = entities.iter().map(entity_to_json).collect();
    Ok(ToolOutput::success(serde_json::json!({
        "entities": list,
        "count": list.len(),
    })))
}

/// get_world_relationships: lists all relationships involving an entity.
pub async fn execute_relationships_for(
    input: RelationshipsForInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let id = match parse_uuid(&input.id) {
        Some(id) => id,
        None => {
            return Ok(ToolOutput::error(format!("Invalid UUID: {}", input.id)));
        }
    };
    let rels = context.world_model.relationships_for(id).await;
    let list: Vec<serde_json::Value> = rels.iter().map(relationship_to_json).collect();
    Ok(ToolOutput::success(serde_json::json!({
        "relationships": list,
        "count": list.len(),
    })))
}

/// get_world_blockers: reasoning query - what blocks a goal/entity?
pub async fn execute_get_blockers(
    input: ReasoningInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let id = match parse_uuid(&input.id) {
        Some(id) => id,
        None => {
            return Ok(ToolOutput::error(format!("Invalid UUID: {}", input.id)));
        }
    };
    let blockers = context.world_model.blockers_of(id).await;
    let list: Vec<serde_json::Value> = blockers.iter().map(entity_to_json).collect();
    Ok(ToolOutput::success(serde_json::json!({
        "blockers": list,
        "count": list.len(),
    })))
}

/// get_world_dependencies: reasoning query - what does an entity depend on?
pub async fn execute_get_dependencies(
    input: ReasoningInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let id = match parse_uuid(&input.id) {
        Some(id) => id,
        None => {
            return Ok(ToolOutput::error(format!("Invalid UUID: {}", input.id)));
        }
    };
    let deps = context.world_model.dependencies_of(id).await;
    let list: Vec<serde_json::Value> = deps.iter().map(entity_to_json).collect();
    Ok(ToolOutput::success(serde_json::json!({
        "dependencies": list,
        "count": list.len(),
    })))
}

/// get_consumed_resources: reasoning query - what resources does an entity consume?
pub async fn execute_get_consumed_resources(
    input: ReasoningInput,
    context: &McpContext,
) -> Result<ToolOutput> {
    let id = match parse_uuid(&input.id) {
        Some(id) => id,
        None => {
            return Ok(ToolOutput::error(format!("Invalid UUID: {}", input.id)));
        }
    };
    let resources = context.world_model.resources_consumed_by(id).await;
    let list: Vec<serde_json::Value> = resources.iter().map(entity_to_json).collect();
    Ok(ToolOutput::success(serde_json::json!({
        "resources": list,
        "count": list.len(),
    })))
}

/// get_world_model_stats: returns entity and relationship counts.
pub async fn execute_world_model_stats(
    context: &McpContext,
) -> Result<ToolOutput> {
    let entity_count = context.world_model.entity_count().await;
    let relationship_count = context.world_model.relationship_count().await;
    Ok(ToolOutput::success(serde_json::json!({
        "entity_count": entity_count,
        "relationship_count": relationship_count,
    })))
}
