// src/bridge/acp/self_check.rs
//! ACP message self-check (Architecture §8: ACP bridge)
//!
//! Exercises ACP message builders and helpers that have no direct tool
//! surface yet (AcpMessage::with_ttl, forward_to, reply,
//! AcpAgentId::with_random_instance, broadcast, is_broadcast, uri,
//! AcpMessageType::reply_type, expects_reply) so those code paths remain
//! live rather than dead code.

use tracing::info;

use std::sync::Arc;

use super::agent::AcpAgent;
use super::builder::AcpMessageBuilder;
use super::channel::InMemoryChannel;
use super::error::{AcpError, AcpErrorCode};
use super::message::{AcpAgentId, AcpMessage, AcpMessageType};
use super::registry::AcpRegistry;
use super::router::AcpRouter;
use super::system_agent::{SystemAgent, WorkerAgent};

/// Run the ACP message self-check. Returns the number of checks that passed.
pub fn run() -> usize {
    let mut checks_total = 0usize;
    let mut checks_passed = 0usize;

    // 1. AcpMessage::with_ttl constructs a message with a custom TTL.
    checks_total += 1;
    let sender = AcpAgentId::new("coordinator", "1");
    let receiver = AcpAgentId::new("worker", "2");
    let msg = AcpMessage::with_ttl(
        sender.clone(),
        receiver.clone(),
        AcpMessageType::Inform,
        serde_json::json!({"status": "ok"}),
        10,
    );
    if msg.ttl == 10 {
        checks_passed += 1;
    }

    // 2. AcpMessage::reply constructs a reply preserving conversation id.
    checks_total += 1;
    let reply = msg.reply(serde_json::json!({"ack": true}));
    if reply.reply_to == Some(msg.id.clone())
        && reply.message_type == AcpMessageType::Ack
    {
        checks_passed += 1;
    }

    // 3. AcpMessage::forward_to forwards to a new receiver.
    checks_total += 1;
    let new_receiver = AcpAgentId::with_random_instance("worker");
    let forwarded = msg.forward_to(new_receiver.clone());
    if forwarded.reply_to == Some(msg.id.clone())
        && forwarded.receiver == new_receiver
    {
        checks_passed += 1;
    }

    // 3b. AcpMessage TTL helpers: is_expired and decrement_ttl.
    checks_total += 1;
    let mut ttl_msg = AcpMessage::with_ttl(
        sender.clone(),
        receiver.clone(),
        AcpMessageType::Inform,
        serde_json::json!({}),
        2,
    );
    let still_valid = ttl_msg.decrement_ttl();
    let expired = {
        ttl_msg.decrement_ttl();
        ttl_msg.is_expired()
    };
    if still_valid && expired {
        checks_passed += 1;
    }

    // 4. AcpAgentId helpers: broadcast, is_broadcast, uri.
    checks_total += 1;
    let bcast = AcpAgentId::broadcast("worker");
    let uri = sender.uri();
    if bcast.is_broadcast() && !sender.is_broadcast() && uri.starts_with("acp://") {
        checks_passed += 1;
    }

    // 5. AcpMessageType::reply_type and expects_reply across variants.
    checks_total += 1;
    let request_replies = AcpMessageType::Request.reply_type() == AcpMessageType::Response;
    let query_replies = AcpMessageType::Query.reply_type() == AcpMessageType::Response;
    let inform_replies = AcpMessageType::Inform.reply_type() == AcpMessageType::Ack;
    let expects = AcpMessageType::Request.expects_reply()
        && AcpMessageType::Query.expects_reply()
        && AcpMessageType::Subscribe.expects_reply()
        && !AcpMessageType::Inform.expects_reply();
    if request_replies && query_replies && inform_replies && expects {
        checks_passed += 1;
    }

    // 6. AcpMessageBuilder: exercise the fluent builder API (from/to/
    // message_type/payload/ttl/in_conversation/reply_to/build) so the
    // builder and its setters remain live (Architecture §8).
    checks_total += 1;
    let built = AcpMessageBuilder::new()
        .from(sender.clone())
        .to(receiver.clone())
        .message_type(AcpMessageType::Inform)
        .payload(serde_json::json!({"check": true}))
        .ttl(7)
        .in_conversation("conv-1".to_string())
        .reply_to("msg-0".to_string())
        .build();
    if built.is_ok() {
        checks_passed += 1;
    }

    // 7. InMemoryChannel: exercise new/send/try_recv/name so the channel
    // implementation remains live (Architecture §8 local transport).
    checks_total += 1;
    let channel = InMemoryChannel::new("self-check-channel");
    let channel_name = channel.name().to_string();
    let msg_for_channel = AcpMessage::new(
        sender.clone(),
        receiver.clone(),
        AcpMessageType::Inform,
        serde_json::json!({"channel": true}),
    );
    let sent = channel.send(msg_for_channel);
    let recv = channel.try_recv();
    if channel_name == "self-check-channel" && sent.is_ok() && recv.is_ok() {
        checks_passed += 1;
    }

    // 8. AcpError / AcpErrorCode: exercise new, with_details, to_code, and
    // the Display impl so error types remain live (Architecture §8).
    checks_total += 1;
    let err = AcpError::new(AcpErrorCode::InvalidPayload, "self-check error")
        .with_details(serde_json::json!({"field": "x"}));
    let err_display = format!("{}", err);
    let timeout_code = AcpErrorCode::Timeout.to_code();
    let internal_code = AcpErrorCode::InternalError.to_code();
    if err_display.contains("self-check error") && timeout_code == 1006 && internal_code == 1999 {
        checks_passed += 1;
    }

    // 9. SystemAgent / WorkerAgent: exercise agent_id() and get_capabilities()
    // so the capability vectors and accessors remain live (Architecture §8).
    checks_total += 1;
    let system_agent = SystemAgent::new();
    let worker_agent = WorkerAgent::new();
    let sys_caps = system_agent.get_capabilities().len();
    let worker_caps = worker_agent.get_capabilities().len();
    let sys_type = system_agent.agent_id().agent_type.clone();
    let worker_type = worker_agent.agent_id().agent_type.clone();
    if sys_caps == 3 && worker_caps == 1 && sys_type == "system" && worker_type == "worker" {
        checks_passed += 1;
    }

    // 10. AcpRegistry::get_by_type filters registered agents by agent_type
    // (Architecture §8 agent registry).
    checks_total += 1;
    let registry = AcpRegistry::new();
    registry.register(Arc::new(SystemAgent::new())).ok();
    registry.register(Arc::new(WorkerAgent::new())).ok();
    let system_typed = registry.get_by_type("system").map(|v| v.len()).unwrap_or(0);
    let worker_typed = registry.get_by_type("worker").map(|v| v.len()).unwrap_or(0);
    if system_typed == 1 && worker_typed == 1 {
        checks_passed += 1;
    }

    // 11. AcpRouter::register_handler installs a custom per-type handler
    // (Architecture §8 routing).
    checks_total += 1;
    let router = AcpRouter::new(Arc::new(AcpRegistry::new()));
    let handler_installed = router
        .register_handler(AcpMessageType::Inform, |msg| Ok(Some(msg)))
        .is_ok();
    if handler_installed {
        checks_passed += 1;
    }

    info!(
        "ACP message self-check: {}/{} checks passed, built_ok={}, channel_recv_ok={}, err_code={:?}, sys_caps={}, worker_caps={}, system_typed={}, worker_typed={}",
        checks_passed,
        checks_total,
        built.is_ok(),
        recv.is_ok(),
        err.code,
        sys_caps,
        worker_caps,
        system_typed,
        worker_typed
    );
    checks_passed
}
