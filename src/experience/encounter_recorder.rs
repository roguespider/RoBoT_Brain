// \src\experience\encounter_recorder.rs

use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::database::queries::ExperienceQueries;
use crate::experience::types::{Experience, ExperienceContext, ExperienceOutcome, ExperienceType};

// record() creates Encounter records.
pub fn record(

pub struct ExperienceRecorder {
    queries: ExperienceQueries,
}

impl ExperienceRecorder {
    pub fn new(queries: ExperienceQueries) -> Self {
        Self { queries }
    }

    /// Record a completed experience.
    pub fn record(
        &self,
        experience_type: ExperienceType,
        title: impl Into<String>,
        description: impl Into<String>,
        context: ExperienceContext,
        outcome: ExperienceOutcome,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();

        let experience = Experience {
            id: id.clone(),
            timestamp: Utc::now(),
            experience_type,
            title: title.into(),
            description: description.into(),
            context,
            outcome,
        };

        self.queries.insert_experience(&experience)?;

        Ok(id)
    }

    /// Convenience helper for successful actions.
    pub fn success(
        &self,
        experience_type: ExperienceType,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<String> {
        self.record(
            experience_type,
            title,
            description,
            ExperienceContext::default(),
            ExperienceOutcome::success(),
        )
    }

    /// Convenience helper for failed actions.
    pub fn failure(
        &self,
        experience_type: ExperienceType,
        title: impl Into<String>,
        description: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<String> {
        self.record(
            experience_type,
            title,
            description,
            ExperienceContext::default(),
            ExperienceOutcome::failure(reason),
        )
    }
}
