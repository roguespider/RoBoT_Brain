// src/bridge/mcp/handlers/acp_handler.rs
//! ACP (Agent Communication Protocol) tools handler

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::bridge::acp::message::AcpMessageType;
use crate::bridge::acp::{AcpAgent, AcpAgentId, AcpMessage};
use crate::bridge::mcp::McpContext;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitResult, ToolHandler};

/// ACP tools input types
#[derive(Debug, Deserialize, Serialize)]
pub struct ListAcpAgentsInput {}

#[derive(Debug, Deserialize, Serialize)]
pub struct AcpAgentCountInput {}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetAcpRouterInput {}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetAcpRegistryInput {}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetAgentCapabilitiesInput {
    pub agent_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetSystemStatusInput {}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterAgentInput {
    pub agent_type: String,
    pub instance_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UnregisterAgentInput {
    pub agent_type: String,
    pub instance_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAcpMessageInput {
    pub sender: Option<AcpAgentIdInput>,
    pub receiver: Option<AcpAgentIdInput>,
    pub message_type: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub ttl: Option<u32>,
    pub reply_to: Option<String>,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RouteAcpMessageInput {
    pub sender: Option<AcpAgentIdInput>,
    pub receiver: Option<AcpAgentIdInput>,
    pub message_type: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub ttl: Option<u32>,
    pub reply_to: Option<String>,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AcpAgentIdInput {
    pub agent_type: String,
    pub instance_id: String,
}

/// Dynamic ACP agent for registration
struct DynamicAgent {
    id: AcpAgentId,
    capabilities: Vec<String>,
}

impl DynamicAgent {
    fn new(agent_type: &str, instance_id: &str, capabilities: Vec<String>) -> Self {
        Self {
            id: AcpAgentId::new(agent_type, instance_id),
            capabilities,
        }
    }
}

impl AcpAgent for DynamicAgent {
    fn id(&self) -> &AcpAgentId {
        &self.id
    }

    fn handle(&self, message: AcpMessage) -> anyhow::Result<Option<AcpMessage>> {
        let response_payload = serde_json::json!({
            "status": "handled",
            "agent": self.id.uri(),
            "original_action": message.payload.get("action"),
            "capabilities": self.capabilities,
        });
        Ok(Some(message.reply(response_payload)))
    }
}

/// Handler for ACP (Agent Communication Protocol) tools
#[derive(Clone)]
pub struct AcpToolsHandler {
    context: Arc<McpContext>,
}

impl AcpToolsHandler {
    /// Create a new ACP tools handler
    pub fn new(context: Arc<McpContext>) -> HandlerInitResult<Self> {
        Ok(Self { context })
    }

    /// Create an ACP message without routing it
    pub async fn execute_create_acp_message(
        &self,
        input: CreateAcpMessageInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        // Build sender ID
        let sender = input
            .sender
            .map(|s| AcpAgentId::new(&s.agent_type, &s.instance_id))
            .unwrap_or_else(|| AcpAgentId::new("client", "1"));

        // Build receiver ID
        let receiver = input
            .receiver
            .map(|r| AcpAgentId::new(&r.agent_type, &r.instance_id))
            .unwrap_or_else(|| AcpAgentId::new("worker", "1"));

        // Parse message type - convert string to enum
        let message_type = input
            .message_type
            .as_ref()
            .map(|t| match t.as_str() {
                "Request" => AcpMessageType::Request,
                "Query" => AcpMessageType::Query,
                "Inform" => AcpMessageType::Inform,
                "Subscribe" => AcpMessageType::Subscribe,
                "Response" => AcpMessageType::Response,
                "Ack" => AcpMessageType::Ack,
                "Error" => AcpMessageType::Error,
                _ => AcpMessageType::Request,
            })
            .unwrap_or(AcpMessageType::Request);

        // Create the message
        let mut message = AcpMessage::new(
            sender,
            receiver,
            message_type,
            input.payload.unwrap_or(serde_json::json!({})),
        );

        // Set TTL if provided
        if let Some(ttl) = input.ttl {
            message.ttl = ttl;
        }

        // Set reply_to if provided
        if let Some(reply_to) = input.reply_to {
            message.reply_to = Some(reply_to);
        }

        // Set conversation_id if provided
        if let Some(conv_id) = input.conversation_id {
            message.conversation_id = Some(conv_id);
        }

        Ok(crate::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "message_id": message.id,
                "sender": message.sender.uri(),
                "receiver": message.receiver.uri(),
                "message_type": format!("{:?}", message.message_type),
                "payload": message.payload,
                "ttl": message.ttl,
                "reply_to": message.reply_to,
                "conversation_id": message.conversation_id,
                "timestamp": message.timestamp.to_rfc3339(),
                "status": "created"
            }),
        ))
    }

    /// List all registered ACP agents
    pub async fn execute_list_acp_agents(
        &self,
        _: ListAcpAgentsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        let agents = self
            .context
            .acp_registry
            .list_agents()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let agent_list: Vec<String> = agents.iter().map(|id| id.uri()).collect();

        Ok(crate::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "agents": agent_list,
                "count": agent_list.len()
            }),
        ))
    }

    /// Get count of registered ACP agents
    pub async fn execute_acp_agent_count(
        &self,
        _: AcpAgentCountInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        let count = self.context.acp_registry.count();

        Ok(crate::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "count": count
            }),
        ))
    }

    /// Get ACP router info
    pub async fn execute_acp_router(
        &self,
        _: GetAcpRouterInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        Ok(crate::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "status": "available",
                "message": "ACP router is available"
            }),
        ))
    }

    /// Get ACP registry info
    pub async fn execute_acp_registry(
        &self,
        _: GetAcpRegistryInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        let count = self.context.acp_registry.count();
        Ok(crate::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "status": "available",
                "agent_count": count
            }),
        ))
    }

    /// Route an ACP message
    pub async fn execute_route_acp_message(
        &self,
        input: RouteAcpMessageInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        // Build sender ID
        let sender = input
            .sender
            .map(|s| AcpAgentId::new(&s.agent_type, &s.instance_id))
            .unwrap_or_else(|| AcpAgentId::new("client", "1"));

        // Build receiver ID
        let receiver = input
            .receiver
            .map(|r| AcpAgentId::new(&r.agent_type, &r.instance_id))
            .unwrap_or_else(|| AcpAgentId::new("worker", "1"));

        // Parse message type - convert string to enum
        let message_type = input
            .message_type
            .as_ref()
            .map(|t| match t.as_str() {
                "Request" => AcpMessageType::Request,
                "Query" => AcpMessageType::Query,
                "Inform" => AcpMessageType::Inform,
                "Subscribe" => AcpMessageType::Subscribe,
                "Response" => AcpMessageType::Response,
                "Ack" => AcpMessageType::Ack,
                "Error" => AcpMessageType::Error,
                _ => AcpMessageType::Request,
            })
            .unwrap_or(AcpMessageType::Request);

        // Create the ACP message
        let message = AcpMessage::new(
            sender,
            receiver,
            message_type,
            input.payload.unwrap_or(serde_json::json!({})),
        );

        // Route the message
        match self
            .context
            .acp_router
            .route(message)
            .map_err(|e| anyhow::anyhow!("{}", e))?
        {
            Some(response) => Ok(crate::bridge::tools::ToolOutput::success(
                serde_json::json!({
                    "routed": true,
                    "response_id": response.id,
                    "message": "Message routed successfully"
                }),
            )),
            None => Ok(crate::bridge::tools::ToolOutput::success(
                serde_json::json!({
                    "routed": true,
                    "response_id": null,
                    "message": "Message routed, no response"
                }),
            )),
        }
    }

    /// Get agent capabilities
    pub async fn execute_get_agent_capabilities(
        &self,
        input: GetAgentCapabilitiesInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        // Parse agent_id - try "type:instance" format or just use as type
        let agent_id = if input.agent_id.contains(':') {
            let parts: Vec<&str> = input.agent_id.split(':').collect();
            if parts.len() >= 2 {
                AcpAgentId::new(parts[0], parts[1])
            } else {
                AcpAgentId::new(&input.agent_id, "1")
            }
        } else if input.agent_id == "system" {
            AcpAgentId::new("system", "main")
        } else {
            AcpAgentId::new(&input.agent_id, "1")
        };

        if let Some(agent) = self
            .context
            .acp_registry
            .get(&agent_id)
            .map_err(|e| anyhow::anyhow!("{}", e))?
        {
            let caps: Vec<serde_json::Value> = agent
                .capabilities()
                .into_iter()
                .map(|(name, description)| {
                    serde_json::json!({"name": name, "description": description})
                })
                .collect();
            Ok(crate::bridge::tools::ToolOutput::success(
                serde_json::json!({
                    "agent_id": agent.id().uri(),
                    "agent_type": agent.id().agent_type,
                    "instance_id": agent.id().instance_id,
                    "capabilities": caps
                }),
            ))
        } else {
            Ok(crate::bridge::tools::ToolOutput::success(
                serde_json::json!({
                    "agent_id": agent_id.uri(),
                    "capabilities": []
                }),
            ))
        }
    }

    /// Get system status
    pub async fn execute_get_system_status(
        &self,
        _: GetSystemStatusInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        let agent_count = self.context.acp_registry.count();
        let agents = self
            .context
            .acp_registry
            .list_agents()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Include subsystem diagnostics from McpContext fields that are
        // owned but otherwise never read (Architecture: observability).
        let all_counters = self.context.metrics.collector().get_all_counters().await;
        let all_gauges = self.context.metrics.collector().get_all_gauges().await;
        let behavior_count = self.context.evolution.list_behaviors().await.len();
        let active_behaviors = self.context.evolution.list_active_behaviors().await.len();
        let policy_rules = self.context.policy.list_rules().await.len();
        let bus_subscribers = self.context.bus.subscriber_count();
        // Exercise the bus accessor (Architecture §5 observability): the
        // manager exposes the shared bus so callers can subscribe beyond the
        // worker's own subscription. Report the handle's identity alongside
        // the worker's own subscriber count.
        let worker_bus = self.context.worker_manager.bus();
        let worker_bus_subscribers = self.context.worker_manager.bus_subscriber_count();
        let worker_bus_ptr = Arc::as_ptr(&worker_bus) as usize;
        let pending_jobs = self
            .context
            .job_queue
            .lock()
            .map(|q| q.pending_count())
            .unwrap_or(0);

        // Get evolution metrics (async)
        let metrics = self.context.evolution.get_metrics().await;
        let integrated = self.context.evolution.get_integrated_behaviors().await;
        let deprecated = self.context.evolution.get_deprecated_behaviors().await;
        // For get_effectiveness and should_recommend, use the first integrated
        // behavior ID if available; otherwise report null.
        let first_integrated_id = integrated.first().map(|b| b.id.clone());
        let effectiveness = if let Some(ref fid) = first_integrated_id {
            self.context.evolution.get_effectiveness(fid).await
        } else {
            None
        };
        let should_recommend = if let Some(ref fid) = first_integrated_id {
            self.context.evolution.should_recommend(fid).await
        } else {
            false
        };

        Ok(crate::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "status": "running",
                "server": {
                    "name": self.context.server_info.name,
                    "version": self.context.server_info.version,
                    "capabilities": {
                        "tools": self.context.capabilities.tools.is_some(),
                        "resources": self.context.capabilities.resources.is_some(),
                        "prompts": self.context.capabilities.prompts.is_some(),
                        "logging": self.context.capabilities.logging.is_some(),
                    }
                },
                "agent_count": agent_count,
                "agents": agents.iter().map(|a| a.uri()).collect::<Vec<_>>(),
                "router_status": "active",
                "registry_status": "active",
                "metrics": {
                    "counters": all_counters,
                    "gauges": all_gauges,
                },
                "evolution": {
                    "total_behaviors": behavior_count,
                    "active_behaviors": active_behaviors,
                    "metrics": {
                        "total_behaviors": metrics.total_behaviors,
                        "behaviors_by_status": metrics.behaviors_by_status,
                        "total_evidence": metrics.total_evidence,
                        "supporting_evidence": metrics.supporting_evidence,
                        "average_confidence": metrics.average_confidence,
                    },
                    "integrated_behavior_count": integrated.len(),
                    "integrated_behaviors": integrated
                        .iter()
                        .map(|b| serde_json::json!({
                            "id": b.id,
                            "name": b.name,
                            "description": b.description,
                            "confidence": b.confidence,
                            "application_count": b.application_count,
                            "success_count": b.success_count,
                        }))
                        .collect::<Vec<_>>(),
                    "deprecated_behavior_count": deprecated.len(),
                    "first_integrated_effectiveness": effectiveness,
                    "first_integrated_should_recommend": should_recommend,
                },
                "policy": {
                    "rules": policy_rules,
                },
                "event_bus": {
                    "subscribers": bus_subscribers,
                    "worker_subscribers": worker_bus_subscribers,
                    "worker_bus_addr": worker_bus_ptr,
                    "pending_jobs": pending_jobs,
                },
                // Loop-health metrics (T1-13..T1-16)
                "loop_health": {
                    "loop_latency_ms": self.context.metrics.get_loop_latency_ms().await,
                    "confidence_drift": self.context.metrics.get_confidence_drift().await,
                    "promotion_throughput": self.context.metrics.get_promotion_throughput().await,
                }
            }),
        ))
    }

    /// Register an agent
    pub async fn execute_register_agent(
        &self,
        input: RegisterAgentInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        let agent = Arc::new(DynamicAgent::new(
            &input.agent_type,
            &input.instance_id,
            input.capabilities.clone(),
        ));

        self.context
            .acp_registry
            .register(agent)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(crate::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "registered": true,
                "agent_type": input.agent_type,
                "instance_id": input.instance_id,
                "capabilities": input.capabilities
            }),
        ))
    }

    /// Unregister an agent
    pub async fn execute_unregister_agent(
        &self,
        input: UnregisterAgentInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        let agent_id = AcpAgentId::new(&input.agent_type, &input.instance_id);

        if let Some(..) = self
            .context
            .acp_registry
            .unregister(&agent_id)
            .map_err(|e| anyhow::anyhow!("{}", e))?
        {
            Ok(crate::bridge::tools::ToolOutput::success(
                serde_json::json!({
                    "unregistered": true,
                    "agent_type": input.agent_type,
                    "instance_id": input.instance_id
                }),
            ))
        } else {
            Ok(crate::bridge::tools::ToolOutput::success(
                serde_json::json!({
                    "unregistered": false,
                    "agent_type": input.agent_type,
                    "instance_id": input.instance_id,
                    "message": "Agent was not registered"
                }),
            ))
        }
    }
}

impl ToolHandler for AcpToolsHandler {
    fn category(&self) -> &str {
        "acp"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "list_acp_agents".to_string(),
            "acp_agent_count".to_string(),
            "acp_router".to_string(),
            "acp_registry".to_string(),
            "create_acp_message".to_string(),
            "route_acp_message".to_string(),
            "get_agent_capabilities".to_string(),
            "get_system_status".to_string(),
            "register_agent".to_string(),
            "unregister_agent".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        // ACP is healthy if context has router and registry
        true
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "list_acp_agents",
                "[WORKFLOW: get_workflow + search_memory first] List all registered ACP agents",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("List ACP Agents"),
            rmcp::model::Tool::new(
                "acp_agent_count",
                "[WORKFLOW: get_workflow + search_memory first] Get count of registered ACP agents",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("ACP Agent Count"),
            rmcp::model::Tool::new(
                "acp_router",
                "[WORKFLOW: get_workflow + search_memory first] Get ACP router information",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Get ACP Router"),
            rmcp::model::Tool::new(
                "acp_registry",
                "[WORKFLOW: get_workflow + search_memory first] Get ACP registry information",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Get ACP Registry"),
            rmcp::model::Tool::new(
                "create_acp_message",
                "[WORKFLOW: get_workflow + search_memory first] Create an ACP message without routing it",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "sender": {
                            "type": "object",
                            "properties": {
                                "agent_type": { "type": "string" },
                                "instance_id": { "type": "string" }
                            }
                        },
                        "receiver": {
                            "type": "object",
                            "properties": {
                                "agent_type": { "type": "string" },
                                "instance_id": { "type": "string" }
                            }
                        },
                        "message_type": { "type": "string", "enum": ["Request", "Query", "Inform", "Subscribe", "Response", "Ack", "Error"] },
                        "payload": { "type": "object" },
                        "ttl": { "type": "integer" },
                        "reply_to": { "type": "string" },
                        "conversation_id": { "type": "string" }
                    }
                })),
            ).with_title("Create ACP Message"),
            rmcp::model::Tool::new(
                "route_acp_message",
                "[WORKFLOW: get_workflow + search_memory first] Route an ACP message to an agent",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "sender": {
                            "type": "object",
                            "properties": {
                                "agent_type": { "type": "string" },
                                "instance_id": { "type": "string" }
                            }
                        },
                        "receiver": {
                            "type": "object",
                            "properties": {
                                "agent_type": { "type": "string" },
                                "instance_id": { "type": "string" }
                            }
                        },
                        "message_type": { "type": "string", "enum": ["Request", "Query", "Inform", "Subscribe", "Response", "Ack", "Error"] },
                        "payload": { "type": "object" },
                        "ttl": { "type": "integer" },
                        "reply_to": { "type": "string" },
                        "conversation_id": { "type": "string" }
                    }
                })),
            ).with_title("Route ACP Message"),
            rmcp::model::Tool::new(
                "get_agent_capabilities",
                "[WORKFLOW: get_workflow + search_memory first] Get capabilities of an ACP agent",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" }
                    },
                    "required": ["agent_id"]
                })),
            ).with_title("Get Agent Capabilities"),
            rmcp::model::Tool::new(
                "get_system_status",
                "[WORKFLOW: get_workflow + search_memory first] Get ACP system status",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Get System Status"),
            rmcp::model::Tool::new(
                "register_agent",
                "[WORKFLOW: get_workflow + search_memory first] Register a new ACP agent",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "agent_type": { "type": "string" },
                        "instance_id": { "type": "string" },
                        "capabilities": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["agent_type", "instance_id"]
                })),
            ).with_title("Register Agent"),
            rmcp::model::Tool::new(
                "unregister_agent",
                "[WORKFLOW: get_workflow + search_memory first] Unregister an ACP agent",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "agent_type": { "type": "string" },
                        "instance_id": { "type": "string" }
                    },
                    "required": ["agent_type", "instance_id"]
                })),
            ).with_title("Unregister Agent"),
        ]
    }

    fn execute_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send
    {
        async move {
            match name {
                "list_acp_agents" => {
                    let input: ListAcpAgentsInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_list_acp_agents(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "acp_agent_count" => {
                    let input: AcpAgentCountInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_acp_agent_count(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "acp_router" => {
                    let input: GetAcpRouterInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_acp_router(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "acp_registry" => {
                    let input: GetAcpRegistryInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_acp_registry(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "create_acp_message" => {
                    let input: CreateAcpMessageInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_create_acp_message(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "route_acp_message" => {
                    let input: RouteAcpMessageInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_route_acp_message(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_agent_capabilities" => {
                    let input: GetAgentCapabilitiesInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_agent_capabilities(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_system_status" => {
                    let input: GetSystemStatusInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_system_status(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "register_agent" => {
                    let input: RegisterAgentInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_register_agent(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "unregister_agent" => {
                    let input: UnregisterAgentInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_unregister_agent(input)
                        .await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string())),
            }
        }
    }
}
