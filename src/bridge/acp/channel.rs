// src/bridge/acp/channel.rs
//! ACP channel implementations

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;

use super::message::AcpMessage;

/// In-memory channel for local agent communication
pub(crate) struct InMemoryChannel {
    name: String,
    messages: Arc<std::sync::Mutex<Vec<AcpMessage>>>,
    waiting: Arc<AtomicBool>,
}

impl InMemoryChannel {
    /// Create a new in-memory channel
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            messages: Arc::new(std::sync::Mutex::new(Vec::new())),
            waiting: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Send a message through the channel
    pub fn send(&self, message: AcpMessage) -> Result<()> {
        let mut messages = self.messages.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
        messages.push(message);
        self.waiting.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Receive a message from the channel (non-blocking)
    pub fn try_recv(&self) -> Result<Option<AcpMessage>> {
        let mut messages = self.messages.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
        self.waiting.store(false, Ordering::SeqCst);
        Ok(messages.pop())
    }

    /// Get channel name for debugging
    pub fn name(&self) -> &str {
        &self.name
    }
}
