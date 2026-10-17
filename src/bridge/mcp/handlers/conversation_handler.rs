// src/bridge/mcp/handlers/conversation_handler.rs
//! Conversation Engine MCP tools — session management and interaction lifecycle.

use std::sync::{Arc, Mutex};

use crate::bridge::mcp::handlers::{HandlerError, ToolHandler, json_to_schema};
use crate::bridge::tools::ToolOutput;
use crate::conversation::ConversationEngine;

/// MCP tool handler for the Conversation Engine (Architecture Chapter 6).
#[derive(Clone)]
pub struct ConversationToolsHandler {
    engine: Arc<Mutex<ConversationEngine>>,
}

impl ConversationToolsHandler {
    /// Create a new conversation handler.
    pub fn new(
        loop_runner: Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
    ) -> Result<Self, crate::bridge::mcp::handlers::HandlerInitError> {
        match loop_runner.lock() {
            Ok(runner_guard) => {
                tracing::debug!("Acquired loop_runner guard for verification");
                drop(runner_guard);
            }
            Err(lock_err) => {
                tracing::warn!(error = %lock_err, "loop_runner lock poisoned during init");
            }
        }
        let engine = Arc::new(Mutex::new(ConversationEngine::new()));
        Ok(Self { engine })
    }
}

impl Default for ConversationToolsHandler {
    fn default() -> Self {
        Self {
            engine: Arc::new(Mutex::new(ConversationEngine::new())),
        }
    }
}

impl ToolHandler for ConversationToolsHandler {
    fn category(&self) -> &str {
        "conversation"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "conversation_start_session".to_string(),
            "conversation_get_session".to_string(),
            "conversation_process_interaction".to_string(),
            "conversation_complete_interaction".to_string(),
            "conversation_suspend_session".to_string(),
            "conversation_resume_session".to_string(),
            "conversation_session_states".to_string(),
            "conversation_active_session_count".to_string(),
            "conversation_completed_session_count".to_string(),
            "conversation_suspended_session_count".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        self.engine.lock().is_ok()
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        vec![
            rmcp::model::Tool::new(
                "conversation_start_session",
                "[WORKFLOW: get_workflow + search_memory first] Start a new conversation session with a given conversation ID and session ID.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "conversation_id": { "type": "string" },
                        "session_id": { "type": "string" }
                    },
                    "required": ["conversation_id", "session_id"]
                })),
            ).with_title("Start Conversation Session"),
            rmcp::model::Tool::new(
                "conversation_get_session",
                "[WORKFLOW: get_workflow + search_memory first] Get the current state of a conversation session by conversation ID.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "conversation_id": { "type": "string" }
                    },
                    "required": ["conversation_id"]
                })),
            ).with_title("Get Conversation Session"),
            rmcp::model::Tool::new(
                "conversation_process_interaction",
                "[WORKFLOW: get_workflow + search_memory first] Process an interaction through the conversation lifecycle.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "conversation_id": { "type": "string" },
                        "input_id": { "type": "string" }
                    },
                    "required": ["conversation_id", "input_id"]
                })),
            ).with_title("Process Conversation Interaction"),
            rmcp::model::Tool::new(
                "conversation_complete_interaction",
                "[WORKFLOW: get_workflow + search_memory first] Complete the interaction lifecycle for a session.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "conversation_id": { "type": "string" }
                    },
                    "required": ["conversation_id"]
                })),
            ).with_title("Complete Conversation Interaction"),
            rmcp::model::Tool::new(
                "conversation_suspend_session",
                "[WORKFLOW: get_workflow + search_memory first] Suspend a conversation session.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "conversation_id": { "type": "string" }
                    },
                    "required": ["conversation_id"]
                })),
            ).with_title("Suspend Conversation Session"),
            rmcp::model::Tool::new(
                "conversation_resume_session",
                "[WORKFLOW: get_workflow + search_memory first] Resume a suspended conversation session.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "conversation_id": { "type": "string" }
                    },
                    "required": ["conversation_id"]
                })),
            ).with_title("Resume Conversation Session"),
            rmcp::model::Tool::new(
                "conversation_session_states",
                "[WORKFLOW: get_workflow + search_memory first] Get all conversation sessions with their current states.",
                json_to_schema(serde_json::json!({ "type": "object" })),
            ).with_title("Get Conversation Session States"),
            rmcp::model::Tool::new(
                "conversation_active_session_count",
                "[WORKFLOW: get_workflow + search_memory first] Get the count of active (processing) conversation sessions.",
                json_to_schema(serde_json::json!({ "type": "object" })),
            ).with_title("Get Active Session Count"),
            rmcp::model::Tool::new(
                "conversation_completed_session_count",
                "[WORKFLOW: get_workflow + search_memory first] Get the count of completed conversation sessions.",
                json_to_schema(serde_json::json!({ "type": "object" })),
            ).with_title("Get Completed Session Count"),
            rmcp::model::Tool::new(
                "conversation_suspended_session_count",
                "[WORKFLOW: get_workflow + search_memory first] Get the count of suspended conversation sessions.",
                json_to_schema(serde_json::json!({ "type": "object" })),
            ).with_title("Get Suspended Session Count"),
        ]
    }

    async fn execute_tool(
        &self,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<ToolOutput, HandlerError> {
        match tool_name {
            "conversation_start_session" => {
                let obj = args
                    .as_object()
                    .ok_or_else(|| HandlerError::InvalidParams("args must be an object".into()))?;
                let conversation_id = obj
                    .get("conversation_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing conversation_id".into()))?;
                let session_id = obj
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing session_id".into()))?;
                let mut engine = self
                    .engine
                    .lock()
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))?;
                let identity = engine.start_session(conversation_id, session_id);
                Ok(ToolOutput::success(serde_json::json!({
                    "status": "ok",
                    "conversation_id": identity.conversation_id,
                    "session_id": identity.session_id,
                    "correlation_id": identity.correlation_id,
                })))
            }
            "conversation_get_session" => {
                let obj = args
                    .as_object()
                    .ok_or_else(|| HandlerError::InvalidParams("args must be an object".into()))?;
                let conversation_id = obj
                    .get("conversation_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing conversation_id".into()))?;
                let engine = self
                    .engine
                    .lock()
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))?;
                match engine.get_session(conversation_id) {
                    Some(session) => Ok(ToolOutput::success(serde_json::json!({
                        "status": "ok",
                        "state": format!("{:?}", session.state),
                        "interaction_count": session.interaction_history.len(),
                        "active_goals": session.active_goals,
                    }))),
                    None => Ok(ToolOutput::error(format!(
                        "Session not found: {}",
                        conversation_id
                    ))),
                }
            }
            "conversation_process_interaction" => {
                let obj = args
                    .as_object()
                    .ok_or_else(|| HandlerError::InvalidParams("args must be an object".into()))?;
                let conversation_id = obj
                    .get("conversation_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing conversation_id".into()))?;
                let input_id = obj
                    .get("input_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing input_id".into()))?;
                let mut engine = self
                    .engine
                    .lock()
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))?;
                match engine.process_interaction(conversation_id, input_id) {
                    Some(state) => Ok(ToolOutput::success(serde_json::json!({
                        "status": "ok",
                        "state": format!("{:?}", state),
                    }))),
                    None => Ok(ToolOutput::error(format!(
                        "Failed to process interaction: session '{}' not found",
                        conversation_id
                    ))),
                }
            }
            "conversation_complete_interaction" => {
                let obj = args
                    .as_object()
                    .ok_or_else(|| HandlerError::InvalidParams("args must be an object".into()))?;
                let conversation_id = obj
                    .get("conversation_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing conversation_id".into()))?;
                let mut engine = self
                    .engine
                    .lock()
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))?;
                engine.complete_interaction(conversation_id);
                Ok(ToolOutput::success(serde_json::json!({
                    "status": "ok",
                    "conversation_id": conversation_id,
                })))
            }
            "conversation_suspend_session" => {
                let obj = args
                    .as_object()
                    .ok_or_else(|| HandlerError::InvalidParams("args must be an object".into()))?;
                let conversation_id = obj
                    .get("conversation_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing conversation_id".into()))?;
                let mut engine = self
                    .engine
                    .lock()
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))?;
                engine.suspend_session(conversation_id);
                Ok(ToolOutput::success(serde_json::json!({
                    "status": "ok",
                    "conversation_id": conversation_id,
                })))
            }
            "conversation_resume_session" => {
                let obj = args
                    .as_object()
                    .ok_or_else(|| HandlerError::InvalidParams("args must be an object".into()))?;
                let conversation_id = obj
                    .get("conversation_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing conversation_id".into()))?;
                let mut engine = self
                    .engine
                    .lock()
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))?;
                engine.resume_session(conversation_id);
                Ok(ToolOutput::success(serde_json::json!({
                    "status": "ok",
                    "conversation_id": conversation_id,
                })))
            }
            "conversation_session_states" => {
                let engine = self
                    .engine
                    .lock()
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))?;
                let states: Vec<serde_json::Value> = engine
                    .session_states()
                    .iter()
                    .map(|(id, s)| {
                        serde_json::json!({
                            "conversation_id": id,
                            "state": format!("{:?}", s.state),
                            "interaction_count": s.interaction_history.len(),
                            "active_goals": s.active_goals,
                            "event_count": s.events.len(),
                        })
                    })
                    .collect();
                Ok(ToolOutput::success(serde_json::json!({
                    "status": "ok",
                    "session_count": states.len(),
                    "sessions": states,
                })))
            }
            "conversation_active_session_count" => {
                let engine = self
                    .engine
                    .lock()
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))?;
                let count = engine.active_session_count();
                Ok(ToolOutput::success(serde_json::json!({
                    "status": "ok",
                    "active_count": count,
                })))
            }
            "conversation_completed_session_count" => {
                let engine = self
                    .engine
                    .lock()
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))?;
                let count = engine.completed_session_count();
                Ok(ToolOutput::success(serde_json::json!({
                    "status": "ok",
                    "completed_count": count,
                })))
            }
            "conversation_suspended_session_count" => {
                let engine = self
                    .engine
                    .lock()
                    .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))?;
                let count = engine.suspended_session_count();
                Ok(ToolOutput::success(serde_json::json!({
                    "status": "ok",
                    "suspended_count": count,
                })))
            }
            _ => Err(HandlerError::ToolNotFound(tool_name.to_string())),
        }
    }
}
