

// src/experience/event_handler.rs
// Event handler that processes events from the bus

use crate::experience::bus::ExperienceBus;
use crate::experience::events::types::ExperienceEvent;
use std::sync::Arc;

/// Event handler that subscribes to the event bus and processes events.
pub struct EventHandler {
    bus: Arc<ExperienceBus>,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(bus: Arc<ExperienceBus>) -> Self {
        Self { bus }
    }

    /// Start the event handler - subscribes to events and logs them.
    /// This runs in the background processing events.
    pub fn start(&self) {
        let bus = self.bus.clone();
        let mut receiver = self.bus.subscribe();

        tokio::spawn(async move {
            tracing::info!("Event handler started, listening for events");
            let mut tick_count: u64 = 0;
            loop {
                tokio::select! {
                    Ok(event) = receiver.recv() => {
                        Self::handle_event(&event);
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                        // Periodically log subscriber count for monitoring
                        tick_count += 1;
                        let count = bus.subscriber_count();
                        tracing::debug!(
                            "Event bus health: {} subscribers, {} ticks",
                            count,
                            tick_count
                        );
                    }
                }
            }
        });
    }

    /// Handle a single event
    fn handle_event(event: &ExperienceEvent) {
        tracing::debug!(
            "Event: {} for experience {}",
            event.event_type.name(),
            event.experience_id
        );
    }

    /// Get subscriber count for monitoring
    pub fn subscriber_count(&self) -> usize {
        self.bus.subscriber_count()
    }
}
