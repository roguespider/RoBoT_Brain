// src/experience/mod.rs
//! Experience System - Per Architecture §2.1, §5, §07, §22
//!
//! The Experience System is the foundation of learning.
//! It records events, observations, actions, outcomes, and environmental changes.

use crate::experience::types::context::ExperienceContext;
use crate::experience::types::experience::{Experience, ExperienceType};
use crate::experience::types::maturity::KnowledgeMaturity;
use crate::experience::types::outcome::{ExperienceOutcome, OutcomeKind};
use std::collections::HashMap;
use uuid::Uuid;

pub mod bus;
pub mod coordinator;
pub mod encounter_recorder;
pub mod event_handler;

pub mod events;

pub mod evolution;
pub mod exploration;
pub mod hypothesis;

pub mod integration;

pub mod metrics;
pub mod observer; // Observer trait + concrete implementations
pub mod queue;

pub mod reflection;
pub mod repository;

pub mod reputation;

pub fn record_research(
    query: String,
    sources: Vec<String>,
    mode: String,
    duration: std::time::Duration,
    outcome: String,
) -> Experience {
    let experience_id = Uuid::new_v4();
    let title = format!("Research: {query}");
    let description = format!(
        "Research query='{}' mode='{}' sources={} duration={}s outcome='{}'",
        query,
        mode,
        sources.len(),
        duration.as_secs(),
        outcome
    );
    let experience = Experience {
        id: experience_id,
        timestamp: chrono::Utc::now(),
        observation_ids: Vec::new(),
        experience_type: ExperienceType::Custom("research".to_string()),
        title,
        description,
        context: ExperienceContext::default(),
        outcome: ExperienceOutcome {
            kind: if outcome.to_lowercase().contains("success")
                || outcome.to_lowercase().contains("found")
            {
                OutcomeKind::Success
            } else {
                OutcomeKind::Failure
            },
            message: Some(outcome.clone()),
            error: None,
            duration_ms: Some(duration.as_millis() as u64),
        },
        score: None,
        encounter_ids: Vec::new(),
        maturity: KnowledgeMaturity::Emerging,
        confidence: 0.5,
        lessons_learned: Vec::new(),
        objective: query.clone(),
        initial_assumptions: Vec::new(),
        plan: format!("Research mode: {}", mode),
        actions: vec!["query_sources".to_string()],
        tools_used: vec!["research_pipeline".to_string()],
        results: vec![format!("{} sources found", sources.len())],
        failures: Vec::new(),
        corrections: Vec::new(),
        successful_strategies: Vec::new(),
        unsuccessful_strategies: Vec::new(),
        discovered_constraints: Vec::new(),
        discovered_capabilities: Vec::new(),
        final_outcome: outcome,
        evidence_count: sources.len(),
        evidence_ids: Vec::new(),
        tags: vec!["research".to_string(), mode],
        committed: true,
        archived: false,
        archived_at: None,
        metadata: HashMap::new(),
    };
    tracing::info!(
        experience_id = %experience_id,
        experience_type = ?experience.experience_type,
        "Research experience recorded"
    );
    experience
}

pub mod scheduler;
pub mod scorer;
pub mod types;
pub mod worker;
pub mod worker_manager;
