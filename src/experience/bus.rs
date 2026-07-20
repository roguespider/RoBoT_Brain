// /src/experience/bus.rs

use crate::experience::events::ExperienceEvent;

pub struct ExperienceBus;

impl ExperienceBus {
    pub fn publish(&self, _event: ExperienceEvent) {}
}

// bus.publish(experience_id);
