// src/bridge/acp/channel.rs
//! ACP channel implementations

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{anyhow, Result};

use super::message::AcpMessage;

/// ACP channel for sending and receiving messages
pub trait AcpChannel: Send + Sync {
    /// Send a message through the channel
    fn send(&self, message: AcpMessage) -> Result<()>;

    /// Receive a message from the channel (non-blocking)
    fn try_recv(&self) -> Result<Option<AcpMessage>>;

    /// Get channel name for debugging
    fn name(&self) -> &str;
}

/// In-memory channel for local agent communication
pub struct InMemoryChannel {
    name: String,
    messages: Arc<std::sync::Mutex<Vec<AcpMessage>>>,
    waiting: Arc<AtomicBool>,
}

impl InMemoryChannel {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            messages: Arc::new(std::sync::Mutex::new(Vec::new())),
            waiting: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_buffer(name: &str, capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            messages: Arc::new(std::sync::Mutex::new(Vec::with_capacity(capacity))),
            waiting: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl AcpChannel for InMemoryChannel {
    fn send(&self, message: AcpMessage) -> Result<()> {
        let mut messages = self.messages.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
        messages.push(message);
        self.waiting.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn try_recv(&self) -> Result<Option<AcpMessage>> {
        let mut messages = self.messages.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
        self.waiting.store(false, Ordering::SeqCst);
        Ok(messages.pop())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
