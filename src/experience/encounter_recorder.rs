// \src\experience\encounter_recorder.rs
//! Experience recording with observation tracking per Architecture §07
//!
//! NOTE: This module is implemented but not yet integrated into the coordinator.
//! It provides structured experience recording with observation tracking.

#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::database::models::MemoryCard;
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::types::{Experience, ExperienceContext, ExperienceOutcome, ExperienceType};

/// Records experiences to storage
pub struct ExperienceRecorder {
    database: Arc<SqliteDatabase>,
}

impl ExperienceRecorder {
    pub fn new(database: Arc<SqliteDatabase>) -> Self {
        Self { database }
    }

    /// Record a completed experience with observation origins (Architecture §07)
    pub fn record(
        &self,
        experience_type: ExperienceType,
        title: impl Into<String>,
        description: impl Into<String>,
        context: ExperienceContext,
        outcome: ExperienceOutcome,
        observation_ids: Vec<Uuid>, // Required per Architecture §07 invariant
    ) -> Result<String> {
        // Create experience with observation origins (Architecture §07 invariant)
        let mut experience = Experience::new(
            title.into(),
            description.into(),
            experience_type,
            observation_ids,
        );
        experience.context = context;
        experience.outcome = outcome;

        let id = experience.id;

        // Store in database
        let conn = self.database.connection()?;
        let memory = MemoryCard::from_experience(&experience);
        queries::insert_memory(&conn, &memory)?;
        
        tracing::info!("Recorded experience: {}", id);

        Ok(id.to_string())
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
            vec![], // No observations tracked by default
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
            vec![],
        )
    }
}
