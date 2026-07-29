// src/tools/hypothesis/db.rs
// Database operations for the Hypothesis Engine


use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;

use crate::database::sqlite::SqliteDatabase;

use crate::database::models::{
    Evidence, Hypothesis, HypothesisStatus, Knowledge, Observation,
};

// ============================================================================
// DATABASE OPERATIONS
// ============================================================================

/// Record an observation in the database
pub async fn record_observation(db: &Arc<SqliteDatabase>, obs: &Observation) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "INSERT INTO observations (id, content, context, observation_type, related_experiences, triggered_hypothesis, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            obs.id.to_string(),
            &obs.content,
            &obs.context,
            &obs.observation_type,
            serde_json::to_string(&obs.related_experiences)?,
            obs.triggered_hypothesis.map(|u| u.to_string()),
            obs.created_at.to_rfc3339(),
        ),
    )?;
    Ok(())
}

/// Create a hypothesis in the database
pub async fn create_hypothesis(db: &Arc<SqliteDatabase>, hyp: &Hypothesis) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "INSERT INTO hypotheses (id, statement, domain, status, confidence, supporting_count, contradicting_count, source_observations, related_memories, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        (
            hyp.id.to_string(),
            &hyp.statement,
            &hyp.domain,
            hyp.status.to_string(),
            hyp.confidence,
            hyp.supporting_count,
            hyp.contradicting_count,
            serde_json::to_string(&hyp.source_observations)?,
            serde_json::to_string(&hyp.related_memories)?,
            hyp.created_at.to_rfc3339(),
            hyp.updated_at.to_rfc3339(),
        ),
    )?;
    Ok(())
}

/// Get hypothesis by ID
pub async fn get_hypothesis_by_id(db: &Arc<SqliteDatabase>, id: &Uuid) -> Result<Option<Hypothesis>> {
    let conn = db.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, statement, domain, status, confidence, supporting_count, contradicting_count, source_observations, related_memories, created_at, updated_at
         FROM hypotheses WHERE id = ?1"
    )?;
    
    let result = stmt.query_row([id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let statement: String = row.get(1)?;
        let domain: String = row.get(2)?;
        let status_str: String = row.get(3)?;
        let confidence: f32 = row.get(4)?;
        let supporting_count: u32 = row.get(5)?;
        let contradicting_count: u32 = row.get(6)?;
        let source_observations_str: String = row.get(7)?;
        let related_memories_str: String = row.get(8)?;
        let created_at_str: String = row.get(9)?;
        let updated_at_str: String = row.get(10)?;
        
        let status = match status_str.as_str() {
            "supported" => HypothesisStatus::Supported,
            "refuted" => HypothesisStatus::Refuted,
            "inconclusive" => HypothesisStatus::Inconclusive,
            "superseded" => HypothesisStatus::Superseded,
            _ => HypothesisStatus::Testing,
        };
        
        Ok(Hypothesis {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            statement,
            domain,
            status,
            confidence,
            supporting_count,
            contradicting_count,
            source_observations: serde_json::from_str(&source_observations_str).unwrap_or_default(),
            related_memories: serde_json::from_str(&related_memories_str).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        })
    });
    
    match result {
        Ok(hyp) => Ok(Some(hyp)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Update hypothesis
pub async fn update_hypothesis(db: &Arc<SqliteDatabase>, hyp: &Hypothesis) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "UPDATE hypotheses SET statement = ?2, domain = ?3, status = ?4, confidence = ?5,
         supporting_count = ?6, contradicting_count = ?7, source_observations = ?8,
         related_memories = ?9, updated_at = ?10 WHERE id = ?1",
        (
            hyp.id.to_string(),
            &hyp.statement,
            &hyp.domain,
            hyp.status.to_string(),
            hyp.confidence,
            hyp.supporting_count,
            hyp.contradicting_count,
            serde_json::to_string(&hyp.source_observations)?,
            serde_json::to_string(&hyp.related_memories)?,
            hyp.updated_at.to_rfc3339(),
        ),
    )?;
    Ok(())
}

/// Add evidence to a hypothesis
pub async fn add_evidence(db: &Arc<SqliteDatabase>, evidence: &Evidence) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "INSERT INTO evidence (id, hypothesis_id, content, evidence_type, direction, strength, experience_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (
            evidence.id.to_string(),
            evidence.hypothesis_id.to_string(),
            &evidence.content,
            &evidence.evidence_type,
            &evidence.direction,
            evidence.strength,
            evidence.experience_id.map(|u| u.to_string()),
            evidence.created_at.to_rfc3339(),
        ),
    )?;
    Ok(())
}

/// Get evidence for a hypothesis
pub async fn get_evidence_for_hypothesis(db: &Arc<SqliteDatabase>, hypothesis_id: &Uuid) -> Result<Vec<Evidence>> {
    let conn = db.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, hypothesis_id, content, evidence_type, direction, strength, experience_id, created_at
         FROM evidence WHERE hypothesis_id = ?1 ORDER BY created_at DESC"
    )?;
    
    let evidence_iter = stmt.query_map([hypothesis_id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let hypothesis_id_str: String = row.get(1)?;
        let content: String = row.get(2)?;
        let evidence_type: String = row.get(3)?;
        let direction: String = row.get(4)?;
        let strength: f32 = row.get(5)?;
        let experience_id: Option<String> = row.get(6)?;
        let created_at_str: String = row.get(7)?;
        
        Ok(Evidence {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            hypothesis_id: Uuid::parse_str(&hypothesis_id_str).unwrap_or_default(),
            content,
            evidence_type,
            direction,
            strength,
            experience_id: experience_id.and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        })
    })?;
    
    let mut results = Vec::new();
    for evidence in evidence_iter {
        results.push(evidence?);
    }
    Ok(results)
}

/// Create learned knowledge
pub async fn create_knowledge(db: &Arc<SqliteDatabase>, knowledge: &Knowledge) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "INSERT INTO learned_knowledge (id, content, source_hypothesis, confidence, domain, derivation, active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (
            knowledge.id.to_string(),
            &knowledge.content,
            knowledge.source_hypothesis.map(|u| u.to_string()),
            knowledge.confidence,
            &knowledge.domain,
            &knowledge.derivation,
            knowledge.active as i32,
            knowledge.created_at.to_rfc3339(),
        ),
    )?;
    Ok(())
}

/// Get knowledge
pub async fn get_knowledge(db: &Arc<SqliteDatabase>, domain: Option<&str>, limit: usize) -> Result<Vec<Knowledge>> {
    let conn = db.connection()?;
    let query = if domain.is_some() {
        "SELECT id, content, source_hypothesis, confidence, domain, derivation, active, created_at
         FROM learned_knowledge WHERE active = 1 AND domain = ?1 ORDER BY confidence DESC LIMIT ?2"
    } else {
        "SELECT id, content, source_hypothesis, confidence, domain, derivation, active, created_at
         FROM learned_knowledge WHERE active = 1 ORDER BY confidence DESC LIMIT ?1"
    };
    
    let mut results = Vec::new();
    
    if let Some(d) = domain {
        let mut stmt = conn.prepare(query)?;
        let iter = stmt.query_map((d, limit as i64), |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let source_hypothesis: Option<String> = row.get(2)?;
            let confidence: f32 = row.get(3)?;
            let domain: String = row.get(4)?;
            let derivation: String = row.get(5)?;
            let active: i32 = row.get(6)?;
            let created_at_str: String = row.get(7)?;
            
            Ok(Knowledge {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                content,
                source_hypothesis: source_hypothesis.and_then(|s| Uuid::parse_str(&s).ok()),
                confidence,
                domain,
                derivation,
                active: active != 0,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            })
        })?;
        for k in iter {
            results.push(k?);
        }
    } else {
        let mut stmt = conn.prepare(query)?;
        let iter = stmt.query_map([limit as i64], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let source_hypothesis: Option<String> = row.get(2)?;
            let confidence: f32 = row.get(3)?;
            let domain: String = row.get(4)?;
            let derivation: String = row.get(5)?;
            let active: i32 = row.get(6)?;
            let created_at_str: String = row.get(7)?;
            
            Ok(Knowledge {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                content,
                source_hypothesis: source_hypothesis.and_then(|s| Uuid::parse_str(&s).ok()),
                confidence,
                domain,
                derivation,
                active: active != 0,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            })
        })?;
        for k in iter {
            results.push(k?);
        }
    }
    
    Ok(results)
}
