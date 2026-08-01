// src/experience/integration/event_subscriber/runner.rs

//! Event subscriber runner

use std::sync::Arc;
use tokio::sync::broadcast;

use crate::experience::bus::ExperienceBus;
use super::EventSubscriber;

/// Start the event subscriber as a background task
pub fn start_event_subscriber(
    bus: Arc<ExperienceBus>,
    subscriber: Arc<EventSubscriber>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut receiver = bus.subscribe();
        tracing::info!("Event subscriber started, listening for events");

        loop {
            match receiver.recv().await {
                Ok(event) => {
                    if let Err(e) = subscriber.process_event(&event).await {
                        tracing::error!("Error processing event {}: {}", event.id, e);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Event subscriber lagged {} events", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("Event bus closed, subscriber shutting down");
                    break;
                }
            }
        }
    })
}
