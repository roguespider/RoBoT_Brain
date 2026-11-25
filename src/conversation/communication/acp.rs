//! ACP integration - Per Architecture §15.2 "ACP concepts"

use serde::{Deserialize, Serialize};

/// ACP message kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AcpMessageKind {
    Request,
    Query,
    Inform,
    Subscribe,
    Response,
    Ack,
    Error,
}

/// ACP message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcpMessage {
    pub sender: String,
    pub receiver: String,
    pub conversation_id: String,
    pub payload: serde_json::Value,
    pub kind: AcpMessageKind,
}

/// ACP router.
#[derive(Debug, Clone, Default)]
pub struct AcpRouter {
    pub routes: std::collections::HashMap<String, Vec<String>>,
}

/// Active reference to ACP contracts.
/// Route an ACP message using the router's registered routes.
pub fn route_acp(router: &AcpRouter, msg: AcpMessage) -> Result<AcpMessage, String> {
    tracing::debug!(
        sender = %msg.sender,
        receiver = %msg.receiver,
        conversation_id = %msg.conversation_id,
        "Routing ACP message"
    );
    // Actually use the router parameter: check registered routes for the receiver.
    if let Some(route_path) = router.routes.get(&msg.receiver) {
        tracing::trace!(
            receiver = %msg.receiver,
            route_path = ?route_path,
            "ACP message routed via registered route"
        );
        Ok(msg)
    } else {
        Err(format!(
            "No registered ACP route for receiver: {}",
            msg.receiver
        ))
    }
}
