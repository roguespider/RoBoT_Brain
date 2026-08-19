//! Memory tool definitions: tool-name constants and the JSON-schema
//! [`crate::bridge::mcp::McpTool`] descriptors advertised to MCP clients.

/// Tool name: store a new memory.
pub const STORE_MEMORY: &str = "store_memory";
/// Tool name: search memories by content.
pub const SEARCH_MEMORY: &str = "search_memory";
/// Tool name: get a specific memory by ID.
pub const GET_MEMORY: &str = "get_memory";
/// Tool name: list recent memories.
pub const LIST_MEMORIES: &str = "list_memories";
/// Tool name: store a vector embedding.
pub const STORE_EMBEDDING: &str = "store_embedding";
/// Tool name: get an embedding by memory ID.
pub const GET_EMBEDDING: &str = "get_embedding";
/// Tool name: search for similar memories via vector similarity.
pub const SEARCH_SIMILAR: &str = "search_similar";
/// Tool name: list all memory embeddings.
pub const LIST_EMBEDDINGS: &str = "list_embeddings";
/// Tool name: delete an embedding by memory ID.
pub const DELETE_EMBEDDING: &str = "delete_embedding";
/// Tool name: get vector-index statistics.
pub const GET_EMBEDDING_STATS: &str = "get_embedding_stats";

/// All memory-tool descriptors advertised to MCP clients via `tools/list`.
pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
    macro_rules! desc {
        ($s:expr) => {
            format!("[WORKFLOW: get_workflow + search_memory first] {}", $s)
        };
    }
    vec![
        crate::bridge::mcp::McpTool {
            name: STORE_MEMORY.to_string(),
            description: desc!("Store a new memory in the knowledge base"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "The content to store"
                    },
                    "memory_type": {
                        "type": "string",
                        "description": "Type of memory: note, fact, task, file, conversation, code, decision, event, encounter, experience",
                        "enum": ["note", "fact", "task", "file", "conversation", "code", "decision", "event", "encounter", "experience"]
                    },
                    "confidence": {
                        "type": "number",
                        "description": "Confidence level (0.0 - 1.0)",
                        "minimum": 0.0,
                        "maximum": 1.0
                    },
                    "importance": {
                        "type": "number",
                        "description": "Importance level (0.0 - 1.0)",
                        "minimum": 0.0,
                        "maximum": 1.0
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional tags for categorization"
                    }
                },
                "required": ["content", "memory_type"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: SEARCH_MEMORY.to_string(),
            description: desc!("Search memories by content"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of results",
                        "default": 10
                    }
                },
                "required": ["query"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: GET_MEMORY.to_string(),
            description: desc!("Get a specific memory by ID"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Memory UUID"
                    }
                },
                "required": ["id"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: LIST_MEMORIES.to_string(),
            description: desc!("List recent memories"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "memory_type": {
                        "type": "string",
                        "description": "Filter by memory type"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of results",
                        "default": 20
                    }
                }
            }),
        },
        crate::bridge::mcp::McpTool {
            name: STORE_EMBEDDING.to_string(),
            description: desc!("Store a vector embedding for semantic memory search"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "memory_id": {
                        "type": "string",
                        "description": "The memory UUID to associate with this embedding"
                    },
                    "embedding": {
                        "type": "array",
                        "items": { "type": "number" },
                        "description": "The vector embedding as an array of floats"
                    },
                    "model": {
                        "type": "string",
                        "description": "The model used to generate the embedding",
                        "default": "default"
                    }
                },
                "required": ["memory_id", "embedding"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: GET_EMBEDDING.to_string(),
            description: desc!("Get an embedding by memory ID"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "memory_id": {
                        "type": "string",
                        "description": "The memory UUID"
                    }
                },
                "required": ["memory_id"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: SEARCH_SIMILAR.to_string(),
            description: desc!("Search for similar memories using vector similarity"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query_embedding": {
                        "type": "array",
                        "items": { "type": "number" },
                        "description": "The query vector as an array of floats"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of results",
                        "default": 5
                    },
                    "min_similarity": {
                        "type": "number",
                        "description": "Minimum cosine similarity threshold (0.0 - 1.0)",
                        "minimum": 0.0,
                        "maximum": 1.0,
                        "default": 0.5
                    }
                },
                "required": ["query_embedding"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: LIST_EMBEDDINGS.to_string(),
            description: desc!("List all memory embeddings"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of results",
                        "default": 100
                    }
                }
            }),
        },
        crate::bridge::mcp::McpTool {
            name: DELETE_EMBEDDING.to_string(),
            description: desc!("Delete an embedding by memory ID"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "memory_id": {
                        "type": "string",
                        "description": "The memory UUID"
                    }
                },
                "required": ["memory_id"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: GET_EMBEDDING_STATS.to_string(),
            description: desc!("Get vector index statistics"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}
