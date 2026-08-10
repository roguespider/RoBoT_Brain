// src/bridge/mcp/handlers/world_model_handler.rs
// World Model tools handler - exposes the world-model graph via MCP
// (Architecture §14: World Model)

use std::sync::Arc;

use crate::bridge::mcp::handlers::{HandlerError, HandlerInitResult, ToolHandler};
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::world_model::{
    AddRelationshipInput, EntitiesOfKindInput, FindEntityInput, GetEntityInput,
    ReasoningInput, RelationshipsForInput, UpsertEntityInput,
    execute_add_relationship, execute_entities_of_kind, execute_find_entity,
    execute_get_blockers, execute_get_consumed_resources, execute_get_dependencies,
    execute_get_entity, execute_relationships_for, execute_upsert_entity,
    execute_world_model_stats,
};

/// Handler for world-model tools
#[derive(Clone)]
pub struct WorldModelToolsHandler {
    context: Arc<McpContext>,
}

impl WorldModelToolsHandler {
    pub fn new(context: Arc<McpContext>) -> HandlerInitResult<Self> {
        Ok(Self { context })
    }

    pub async fn execute_upsert_entity(
        &self,
        input: UpsertEntityInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_upsert_entity(input, &self.context).await
    }

    pub async fn execute_add_relationship(
        &self,
        input: AddRelationshipInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_add_relationship(input, &self.context).await
    }

    pub async fn execute_get_entity(
        &self,
        input: GetEntityInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_get_entity(input, &self.context).await
    }

    pub async fn execute_find_entity(
        &self,
        input: FindEntityInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_find_entity(input, &self.context).await
    }

    pub async fn execute_entities_of_kind(
        &self,
        input: EntitiesOfKindInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_entities_of_kind(input, &self.context).await
    }

    pub async fn execute_relationships_for(
        &self,
        input: RelationshipsForInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_relationships_for(input, &self.context).await
    }

    pub async fn execute_get_blockers(
        &self,
        input: ReasoningInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_get_blockers(input, &self.context).await
    }

    pub async fn execute_get_dependencies(
        &self,
        input: ReasoningInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_get_dependencies(input, &self.context).await
    }

    pub async fn execute_get_consumed_resources(
        &self,
        input: ReasoningInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_get_consumed_resources(input, &self.context).await
    }

    pub async fn execute_world_model_stats(
        &self,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_world_model_stats(&self.context).await
    }
}

impl ToolHandler for WorldModelToolsHandler {
    fn category(&self) -> &str {
        "world_model"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "upsert_world_entity".to_string(),
            "add_world_relationship".to_string(),
            "get_world_entity".to_string(),
            "find_world_entity".to_string(),
            "list_world_entities".to_string(),
            "get_world_relationships".to_string(),
            "get_world_blockers".to_string(),
            "get_world_dependencies".to_string(),
            "get_consumed_resources".to_string(),
            "get_world_model_stats".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        true
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        let entity_kinds = "object, place, person, event, time, goal, resource";
        let relation_kinds = "located_at, owns, participates_in, causes, depends_on, blocks, part_of, alternative_to, consumes, produces, before, related_to";
        vec![
            rmcp::model::Tool::new(
                "upsert_world_entity",
                "Create or update an entity in the world model (Architecture §14). Entities are typed: object, place, person, event, time, goal, resource.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Entity name" },
                        "kind": { "type": "string", "description": entity_kinds },
                        "confidence": { "type": "number", "description": "Confidence 0.0-1.0 (default 0.5)" },
                        "properties": { "type": "object", "description": "Free-form key-value properties" }
                    },
                    "required": ["name", "kind"]
                })),
            ).with_title("Upsert World Entity"),
            rmcp::model::Tool::new(
                "add_world_relationship",
                "Add a typed relationship between two entities (Architecture §14).",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "source_id": { "type": "string", "description": "Source entity UUID" },
                        "target_id": { "type": "string", "description": "Target entity UUID" },
                        "kind": { "type": "string", "description": relation_kinds },
                        "confidence": { "type": "number", "description": "Confidence 0.0-1.0 (default 0.5)" }
                    },
                    "required": ["source_id", "target_id", "kind"]
                })),
            ).with_title("Add World Relationship"),
            rmcp::model::Tool::new(
                "get_world_entity",
                "Get an entity by its UUID.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Entity UUID" }
                    },
                    "required": ["id"]
                })),
            ).with_title("Get World Entity"),
            rmcp::model::Tool::new(
                "find_world_entity",
                "Find an entity by name.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Entity name to search for" }
                    },
                    "required": ["name"]
                })),
            ).with_title("Find World Entity"),
            rmcp::model::Tool::new(
                "list_world_entities",
                "List all entities of a given kind.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string", "description": entity_kinds }
                    },
                    "required": ["kind"]
                })),
            ).with_title("List World Entities"),
            rmcp::model::Tool::new(
                "get_world_relationships",
                "Get all relationships involving an entity.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Entity UUID" }
                    },
                    "required": ["id"]
                })),
            ).with_title("Get World Relationships"),
            rmcp::model::Tool::new(
                "get_world_blockers",
                "Reasoning query: what entities block the given goal/entity?",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Goal/entity UUID" }
                    },
                    "required": ["id"]
                })),
            ).with_title("Get World Blockers"),
            rmcp::model::Tool::new(
                "get_world_dependencies",
                "Reasoning query: what does the given entity depend on?",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Entity UUID" }
                    },
                    "required": ["id"]
                })),
            ).with_title("Get World Dependencies"),
            rmcp::model::Tool::new(
                "get_consumed_resources",
                "Reasoning query: what resources does the given entity consume?",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Entity UUID" }
                    },
                    "required": ["id"]
                })),
            ).with_title("Get Consumed Resources"),
            rmcp::model::Tool::new(
                "get_world_model_stats",
                "Get world-model statistics: entity count and relationship count.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Get World Model Stats"),
        ]
    }

    fn execute_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            match name {
                "upsert_world_entity" => {
                    let input: UpsertEntityInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_upsert_entity(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "add_world_relationship" => {
                    let input: AddRelationshipInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_add_relationship(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_world_entity" => {
                    let input: GetEntityInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_entity(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "find_world_entity" => {
                    let input: FindEntityInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_find_entity(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_world_entities" => {
                    let input: EntitiesOfKindInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_entities_of_kind(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_world_relationships" => {
                    let input: RelationshipsForInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_relationships_for(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_world_blockers" => {
                    let input: ReasoningInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_blockers(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_world_dependencies" => {
                    let input: ReasoningInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_dependencies(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_consumed_resources" => {
                    let input: ReasoningInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_consumed_resources(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_world_model_stats" => {
                    self.execute_world_model_stats()
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string())),
            }
        }
    }
}
