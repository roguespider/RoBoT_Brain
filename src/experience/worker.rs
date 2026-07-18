// /src/experience/worker.rs

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::experience::{
    events::ExperienceEvent, observer::ExperienceObserver, queue::ObserverJob,
};

Encounter



pub struct ExperienceWorker {
    observer: Arc<dyn ExperienceObserver>,
    receiver: mpsc::Receiver<ObserverJob>,
}

impl ExperienceWorker {
    pub fn new(
        observer: Arc<dyn ExperienceObserver>,
        receiver: mpsc::Receiver<ObserverJob>,
    ) -> Self {
        Self { observer, receiver }
    }

    pub async fn start(mut self) -> Result<()> {
        self.observer.start()?;

        while let Some(job) = self.receiver.recv().await {
            let event = job.event;

            if !self.observer.accepts(&event) {
                continue;
            }

            match self.observer.observe(&event) {
                Ok(_) => {
                    // later:
                    // mark job complete in persistence table
                }

                Err(err) => {
                    // later:
                    // retry policy
                    // failure event
                    // persistence update

                    eprintln!("Observer {} failed: {}", self.observer.name(), err);
                }
            }
        }

        self.observer.shutdown()?;

        Ok(())
    }
}
