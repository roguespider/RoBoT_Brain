// src/database/queries/experiences.rs
//! Experience and reputation database operations

use anyhow::Result;
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::experience::types::Experience;
use crate::experience::types::maturity::KnowledgeMaturity;
use crate::experience::reputation::score::Reputation;

use super::helpers::parse_time;

/// List recent experiences from the database
pub fn list_experiences(conn: &Connection, limit: usize) -> Result<Vec<Experience>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, experience_type, context, outcome, score, timestamp, observation_ids, encounter_ids, maturity, confidence, lessons, evidence_count, evidence_ids, tags, committed, archived, archived_at, metadata
         FROM experiences
         ORDER BY timestamp DESC
         LIMIT ?1"
    )?;

    let mut experiences = Vec::new();
    let mut rows = stmt.query(params![limit as i64])?;
    
    while let Some(row) = rows.next()? {
        let id_str: String = row.get(0)?;
        let context_json: String = row.get(4)?;
        let outcome_json: String = row.get(5)?;
        let score_json: String = row.get(6)?;
        let obs_json: String = row.get(8)?;
        let lessons_json: String = row.get(12)?;
        let evidence_ids_json: String = row.get(14)?;
        let tags_json: String = row.get(15)?;
        let archived_at_str: Option<String> = row.get(18)?;
        let metadata_json: String = row.get(19)?;

        experiences.push(Experience {
            id: Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            title: row.get(1)?,
            description: row.get(2)?,
            experience_type: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(crate::experience::types::ExperienceType::ToolExecution),
            context: serde_json::from_str(&context_json).unwrap_or_default(),
            outcome: serde_json::from_str(&outcome_json).unwrap_or_else(|_| crate::experience::types::ExperienceOutcome::success()),
            score: serde_json::from_str(&score_json).ok(),
            timestamp: parse_time(&row.get::<_, String>(7)?),
            observation_ids: serde_json::from_str(&obs_json).unwrap_or_default(),
            encounter_ids: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
            maturity: serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or(KnowledgeMaturity::Emerging),
            confidence: row.get(11)?,
            lessons: serde_json::from_str(&lessons_json).unwrap_or_default(),
            evidence_count: row.get::<_, i64>(12)? as usize,
            evidence_ids: serde_json::from_str(&evidence_ids_json).unwrap_or_default(),
            tags: serde_json::from_str(&tags_json).unwrap_or_default(),
            committed: row.get(16)?,
            archived: row.get(17)?,
            archived_at: archived_at_str.as_deref().map(parse_time),
            metadata: serde_json::from_str(&metadata_json).unwrap_or_default(),
        });
    }

    Ok(experiences)
}

/// List all reputations from the database
pub fn list_reputations(conn: &Connection) -> Result<Vec<Reputation>> {
    let mut stmt = conn.prepare(
        "SELECT id, score, factors, observations, successes, failures, updated_at, history
         FROM reputations"
    )?;

    let mut reputations = Vec::new();
    let mut rows = stmt.query([])?;
    
    while let Some(row) = rows.next()? {
        reputations.push(Reputation {
            id: row.get(0)?,
            score: row.get(1)?,
            factors: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
            observations: row.get(3)?,
            successes: row.get(4)?,
            failures: row.get(5)?,
            updated_at: parse_time(&row.get::<_, String>(6)?),
            history: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
        });
    }

    Ok(reputations)
}

/// Insert or replace a reputation
pub fn insert_reputation(conn: &Connection, reputation: &Reputation) -> Result<()> {
    conn.execute(
        "
        INSERT OR REPLACE INTO reputations
        (
            id,
            score,
            factors,
            observations,
            successes,
            failures,
            updated_at,
            history
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ",
        params![
            reputation.id,
            reputation.score,
            serde_json::to_string(&reputation.factors)?,
            reputation.observations,
            reputation.successes,
            reputation.failures,
            reputation.updated_at.to_rfc3339(),
            serde_json::to_string(&reputation.history)?
        ],
    )?;

    Ok(())
}
