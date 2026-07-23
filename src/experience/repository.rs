// src/experience/repository.rs
// Repository functions for experience persistence

#![allow(dead_code)]

use crate::database::models::MemoryCard;
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::experience::types::{Encounter, EncounterResult, Experience};

/// Save an encounter to the repository
pub async fn save_encounter(db: Arc<SqliteDatabase>, encounter: &Encounter) -> Result<()> {
    let conn = db.connection()?;
    queries::insert_memory(
        &conn,
        &MemoryCard::from_encounter(encounter),
    )?;
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
            "Experience: {} - {} (outcome: {:?}, committed: {}, archived: {})",
            experience.title, 
            experience.description, 
            experience.outcome.kind,
            experience.committed,
            experience.archived
        );

        Self {
            id: experience.id,
            content,
            memory_type: crate::database::models::MemoryType::Experience,
            confidence: experience.confidence,
            importance: experience.score.as_ref().map(|s| s.importance).unwrap_or(0.5),
            created_at: experience.timestamp,
            updated_at: experience.timestamp,
        }
    }
}
