// src/experience/repository.rs
// Repository functions for experience persistence

use crate::database::models::MemoryCard;
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::experience::types::maturity::KnowledgeMaturity;
use crate::experience::types::{Encounter, EncounterResult, Experience};

/// Save an encounter to the repository
pub async fn save_encounter(db: Arc<SqliteDatabase>, encounter: &Encounter) -> Result<()> {
    let conn = db.connection()?;
    queries::insert_memory(&conn, &MemoryCard::from_encounter(encounter))?;
    Ok(())
}

/// Get an encounter from the repository
pub async fn get_encounter(db: Arc<SqliteDatabase>, id: &Uuid) -> Result<Option<Encounter>> {
    let conn = db.connection()?;
    let memory = queries::get_memory(&conn, *id)?;
    Ok(memory.map(|m| m.into_encounter()))
}

/// Find similar encounters in the repository
pub async fn find_similar_encounters(
    db: Arc<SqliteDatabase>,
    query: &str,
) -> Result<Vec<Encounter>> {
    let conn = db.connection()?;
    let memories = queries::search_memory(&conn, query, 100)?;
    Ok(memories.into_iter().map(|m| m.into_encounter()).collect())
}

/// Save an experience to the repository
pub async fn save_experience(db: Arc<SqliteDatabase>, experience: &Experience) -> Result<()> {
    let conn = db.connection()?;
    let memory = MemoryCard::from_experience(experience);
    queries::insert_memory(&conn, &memory)?;
    Ok(())
}

impl MemoryCard {
    /// Convert an Encounter into a MemoryCard for storage
    pub fn from_encounter(encounter: &Encounter) -> Self {
        let result_str = match &encounter.result {
            EncounterResult::Success => "success".to_string(),
            EncounterResult::Failure => "failure".to_string(),
            EncounterResult::Partial(msg) => format!("partial:{}", msg),
            EncounterResult::Error(msg) => format!("error:{}", msg),
            EncounterResult::Timeout => "timeout".to_string(),
        };

        let content = format!(
            "Encounter: {} | Action: {} | Result: {}",
            encounter.input, encounter.action, result_str
        );

        Self {
            id: encounter.id,
            content,
            memory_type: crate::database::models::MemoryType::Encounter,
            layer: crate::database::models::MemoryLayer::Working,
            parent_id: None,
            hierarchy_level: crate::database::models::HierarchyLevel::Document,
            order_index: 0,
            path: String::new(),
            file_source: None,
            access_count: 0,
            last_accessed: None,
            confidence: 1.0,
            importance: 0.7,
            created_at: encounter.timestamp,
            updated_at: encounter.timestamp,
        }
    }

    /// Convert a MemoryCard back into an Encounter
    pub fn into_encounter(self) -> Encounter {
        let parts: Vec<&str> = self.content.split(" | ").collect();
        let input = parts.get(1).map(|s| s.trim()).unwrap_or("").to_string();
        let action = parts.get(2).map(|s| s.trim()).unwrap_or("").to_string();

        let result = if self.content.contains("success") {
            EncounterResult::Success
        } else if self.content.contains("failure") {
            EncounterResult::Failure
        } else if self.content.contains("partial:") {
            EncounterResult::Partial(self.content.clone())
        } else if self.content.contains("error:") {
            EncounterResult::Error(self.content.clone())
        } else if self.content.contains("timeout") {
            EncounterResult::Timeout
        } else {
            EncounterResult::Success
        };

        Encounter {
            id: self.id,
            timestamp: self.created_at,
            experience_id: None,
            context: crate::experience::types::ExperienceContext::default(),
            input,
            action,
            result,
            metadata: Default::default(),
        }
    }

    /// Convert an Experience into a MemoryCard for storage
    /// Per Architecture §07: Experiences are stored with their metadata
    pub fn from_experience(experience: &Experience) -> Self {
        // Include key experience metadata in content
        let content = format!(
            "Experience: {} - {} (outcome: {:?}, committed: {}, archived: {}, evidence_count: {})",
            experience.title,
            experience.description,
            experience.outcome.kind,
            experience.committed,
            experience.archived,
            experience.evidence_count
        );

        Self {
            id: experience.id,
            content,
            memory_type: crate::database::models::MemoryType::Experience,
            layer: crate::database::models::MemoryLayer::Working,
            parent_id: None,
            hierarchy_level: crate::database::models::HierarchyLevel::Document,
            order_index: 0,
            path: String::new(),
            file_source: None,
            access_count: 0,
            last_accessed: None,
            confidence: experience.confidence,
            importance: experience
                .score
                .as_ref()
                .map(|s| s.importance)
                .unwrap_or(0.5),
            created_at: experience.timestamp,
            updated_at: experience.timestamp,
        }
    }

    /// Convert a MemoryCard back into an Experience
    /// Parses the content string to reconstruct experience metadata
    pub fn into_experience(self) -> Experience {
        use crate::experience::types::{
            ExperienceContext, ExperienceOutcome, ExperienceType, OutcomeKind,
        };

        // Parse content: "Experience: title - description (outcome: OutcomeKind, committed: bool, archived: bool, evidence_count: N)"
        let parsed_title = self
            .content
            .strip_prefix("Experience: ")
            .and_then(|s| s.split(" - ").next())
            .unwrap_or("")
            .to_string();

        let parsed_description = self
            .content
            .strip_prefix("Experience: ")
            .and_then(|s| s.split(" - ").nth(1))
            .map(|s| s.trim_start().to_string())
            .unwrap_or_default();

        let parsed_outcome = match &parsed_description {
            desc if desc.contains("Success") => OutcomeKind::Success,
            desc if desc.contains("Failure") => OutcomeKind::Failure,
            desc if desc.contains("Partial") => OutcomeKind::Partial,
            desc if desc.contains("Timeout") => OutcomeKind::Timeout,
            desc if desc.contains("Interrupted") => OutcomeKind::Interrupted,
            _ => OutcomeKind::Success,
        };

        // Extract committed, archived, and evidence_count from the parentheses section
        let parsed_committed = self.content.contains("committed: true");
        let parsed_archived = self.content.contains("archived: true");
        let parsed_evidence_count = self
            .content
            .strip_prefix("Experience: ")
            .and_then(|s| s.rsplit('(').next())
            .and_then(|s| s.strip_prefix("evidence_count: "))
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        let outcome = match parsed_outcome {
            OutcomeKind::Success => ExperienceOutcome::success(),
            OutcomeKind::Failure => ExperienceOutcome::failure("Recovered from storage"),
            OutcomeKind::Partial => ExperienceOutcome::partial("Recovered from storage"),
            OutcomeKind::Timeout => ExperienceOutcome::timeout(),
            OutcomeKind::Interrupted => ExperienceOutcome::interrupted(),
        };

        Experience {
            id: self.id,
            timestamp: self.created_at,
            observation_ids: Vec::new(),
            experience_type: ExperienceType::System,
            title: parsed_title,
            description: parsed_description,
            context: ExperienceContext::default(),
            outcome,
            score: None,
            encounter_ids: Vec::new(),
            maturity: KnowledgeMaturity::Emerging,
            confidence: self.confidence,
            lessons: Vec::new(),
            evidence_count: parsed_evidence_count,
            evidence_ids: Vec::new(),
            tags: Vec::new(),
            committed: parsed_committed,
            archived: parsed_archived,
            archived_at: None,
            metadata: Default::default(),
        }
    }
}
