// /src/experience/worker_manager/tests.rs
//! Integration tests for the worker manager

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::experience::events::ExperienceEvent;
use crate::experience::observer::ExperienceObserver;

/// Test observer that tracks observed events
struct TestObserver {
    name: &'static str,
    observed_count: Arc<AtomicU32>,
    accepted_events: Arc<AtomicU32>,
}

impl TestObserver {
    fn new(name: &'static str) -> (Self, Arc<AtomicU32>, Arc<AtomicU32>) {
        let observed_count = Arc::new(AtomicU32::new(0));
        let accepted_events = Arc::new(AtomicU32::new(0));
        let observer = Self {
            name,
            observed_count: observed_count.clone(),
            accepted_events: accepted_events.clone(),
        };
        (observer, observed_count, accepted_events)
    }
}

impl ExperienceObserver for TestObserver {
    fn name(&self) -> &'static str {
        self.name
    }

    fn accepts(&self, event: &ExperienceEvent) -> bool {
        // Track that accept was called with the event
        self.accepted_events.fetch_add(1, Ordering::SeqCst);
        // Accept all events for testing purposes
        let _ = event;
        true
    }

    fn observe(&self, event: &ExperienceEvent) -> anyhow::Result<()> {
        // Track that observe was called with the event
        self.observed_count.fetch_add(1, Ordering::SeqCst);
        // Process the event (for testing, we just track it)
        let _ = event;
        Ok(())
    }
}

#[tokio::test]
async fn test_worker_manager_creation() {
    let bus = Arc::new(ExperienceBus::new());
    let manager = WorkerManager::new(bus);
    
    assert_eq!(manager.worker_count().await, 0);
}

#[tokio::test]
async fn test_worker_manager_end_to_end() {
    let bus = Arc::new(ExperienceBus::new());
    let manager = Arc::new(WorkerManager::new(bus.clone()));

    let (observer, observed_count, accepted_count) = TestObserver::new("TestObserver");
    manager.register_observer(Arc::new(observer)).await.unwrap();

    let manager_clone = manager.clone();
    let bus_clone = bus.clone();
    tokio::spawn(async move {
        let mut receiver = bus_clone.subscribe();
        while let Ok(event) = receiver.recv().await {
            let _ = manager_clone.broadcast_event(event).await;
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let experience = Experience::new(
        "Test Experience".to_string(),
        "A test experience for integration testing".to_string(),
        ExperienceType::ToolExecution,
        vec![],
    );
    let event = ExperienceEvent::experience_recorded(experience);
    bus.publish(event).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    assert!(accepted_count.load(Ordering::SeqCst) >= 1);
    assert!(observed_count.load(Ordering::SeqCst) >= 1);
}

#[tokio::test]
async fn test_worker_manager_multiple_observers() {
    let bus = Arc::new(ExperienceBus::new());
    let manager = Arc::new(WorkerManager::new(bus.clone()));

    let (obs1, count1, _) = TestObserver::new("Observer1");
    let (obs2, count2, _) = TestObserver::new("Observer2");
    manager.register_observer(Arc::new(obs1)).await.unwrap();
    manager.register_observer(Arc::new(obs2)).await.unwrap();

    let manager_clone = manager.clone();
    let bus_clone = bus.clone();
    tokio::spawn(async move {
        let mut receiver = bus_clone.subscribe();
        while let Ok(event) = receiver.recv().await {
            let _ = manager_clone.broadcast_event(event).await;
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let experience = Experience::new(
        "Test Experience".to_string(),
        "A test experience for multi-observer testing".to_string(),
        ExperienceType::ToolExecution,
        vec![],
    );
    let event = ExperienceEvent::experience_recorded(experience);
    bus.publish(event).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    assert!(count1.load(Ordering::SeqCst) >= 1);
    assert!(count2.load(Ordering::SeqCst) >= 1);
}

#[tokio::test]
async fn test_worker_manager_get_stats() {
    let bus = Arc::new(ExperienceBus::new());
    let manager = Arc::new(WorkerManager::new(bus));

    let (observer, _, _) = TestObserver::new("StatsObserver");
    manager.register_observer(Arc::new(observer)).await.unwrap();

    let stats = manager.get_stats().await;
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].observer_name, "StatsObserver");
}
