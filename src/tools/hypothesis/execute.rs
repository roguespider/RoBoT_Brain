// src/tools/hypothesis/execute.rs

// Tool execution functions for the Hypothesis Engine

use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;

use crate::database::sqlite::SqliteDatabase;
use crate::tools::ToolOutput;
use crate::database::models::{Hypothesis, HypothesisStatus, Knowledge};

use super::db::{
    record_observation, create_hypothesis, get_hypothesis_by_id, update_hypothesis,
    add_evidence, get_evidence_for_hypothesis, create_knowledge, get_knowledge,
    get_observation_by_id,
};
use crate::database::models::{Observation, Evidence};
use crate::tools::hypothesis::{
    RecordObservationInput, CreateHypothesisInput, AddEvidenceInput,
    GetHypothesisInput, GetObservationInput, ListHypothesesInput, ListObservationsInput,
    EvaluateHypothesisInput, GetKnowledgeInput, ExtractKnowledgeInput,
};

// ============================================================================
// TOOL EXECUTIONS
// ============================================================================

pub async fn execute_record_observation(
    input: RecordObservationInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let observation = Observation::new(
        input.content,
        input.context,
        input.observation_type,
    );
    
    record_observation(db, &observation).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "status": "observation_recorded",
        "observation": {
            "id": observation.id.to_string(),
            "content": observation.content,
            "context": observation.context,
            "observation_type": observation.observation_type,
            "created_at": observation.created_at.to_rfc3339()
        },
        "learning_workflow": "Observation recorded. Use create_hypothesis to form a testable hypothesis from this observation."
    })))
}

pub async fn execute_create_hypothesis(
    input: CreateHypothesisInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let mut hypothesis = Hypothesis::new(input.statement, input.domain);
    hypothesis.source_observations = input.source_observations;
    
    create_hypothesis(db, &hypothesis).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "status": "hypothesis_created",
        "hypothesis": {
            "id": hypothesis.id.to_string(),
            "statement": hypothesis.statement,
            "domain": hypothesis.domain,
            "status": hypothesis.status.to_string(),
            "confidence": hypothesis.confidence,
            "created_at": hypothesis.created_at.to_rfc3339()
        },
        "learning_workflow": "Hypothesis created. Use add_evidence to test this hypothesis with supporting or contradicting evidence."
    })))
}

pub async fn execute_add_evidence(
    input: AddEvidenceInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let hypothesis_id = Uuid::parse_str(&input.hypothesis_id)
        .map_err(|e| anyhow::anyhow!("Invalid hypothesis ID: {}", e))?;
    
    // Get hypothesis to update counts
    let mut hypothesis = get_hypothesis_by_id(db, &hypothesis_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Hypothesis not found"))?;
    
    // Create evidence
    let evidence = Evidence::new(
        hypothesis_id,
        input.content,
        input.evidence_type,
        input.direction.clone(),
        input.strength,
    );
    
    add_evidence(db, &evidence).await?;
    
    // Update hypothesis counts
    if input.direction == "support" {
        hypothesis.supporting_count += 1;
    } else if input.direction == "contradict" {
        hypothesis.contradicting_count += 1;
    }
    hypothesis.updated_at = chrono::Utc::now();
    
    // Recalculate confidence
    let total = hypothesis.supporting_count + hypothesis.contradicting_count;
    if total > 0 {
        hypothesis.confidence = hypothesis.supporting_count as f32 / total as f32;
    }
    
    update_hypothesis(db, &hypothesis).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "status": "evidence_added",
        "evidence": {
            "id": evidence.id.to_string(),
            "content": evidence.content,
            "direction": evidence.direction,
            "strength": evidence.strength,
            "created_at": evidence.created_at.to_rfc3339()
        },
        "hypothesis_updated": {
            "supporting_count": hypothesis.supporting_count,
            "contradicting_count": hypothesis.contradicting_count,
            "confidence": hypothesis.confidence
        },
        "suggestion": "Use evaluate_hypothesis to determine if there's enough evidence to conclude."
    })))
}

pub async fn execute_get_hypothesis(
    input: GetHypothesisInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let hypothesis_id = Uuid::parse_str(&input.hypothesis_id)
        .map_err(|e| anyhow::anyhow!("Invalid hypothesis ID: {}", e))?;
    
    let hypothesis = get_hypothesis_by_id(db, &hypothesis_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Hypothesis not found"))?;
    
    let evidence = get_evidence_for_hypothesis(db, &hypothesis_id).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "hypothesis": {
            "id": hypothesis.id.to_string(),
            "statement": hypothesis.statement,
            "domain": hypothesis.domain,
            "status": hypothesis.status.to_string(),
            "confidence": hypothesis.confidence,
            "supporting_count": hypothesis.supporting_count,
            "contradicting_count": hypothesis.contradicting_count,
            "source_observations": hypothesis.source_observations,
            "created_at": hypothesis.created_at.to_rfc3339(),
            "updated_at": hypothesis.updated_at.to_rfc3339()
        },
        "evidence": evidence.into_iter().map(|e| serde_json::json!({
            "id": e.id.to_string(),
            "content": e.content,
            "evidence_type": e.evidence_type,
            "direction": e.direction,
            "strength": e.strength,
            "created_at": e.created_at.to_rfc3339()
        })).collect::<Vec<_>>()
    })))
}

pub async fn execute_get_observation(
    input: GetObservationInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let observation_id = Uuid::parse_str(&input.observation_id)
        .map_err(|e| anyhow::anyhow!("Invalid observation ID: {}", e))?;
    
    let observation = get_observation_by_id(db, &observation_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Observation not found"))?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "observation": {
            "id": observation.id.to_string(),
            "content": observation.content,
            "context": observation.context,
            "observation_type": observation.observation_type,
            "created_at": observation.created_at.to_rfc3339()
        }
    })))
}

pub async fn execute_list_hypotheses(
    input: ListHypothesesInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let conn = db.connection()?;
    let limit = input.limit.unwrap_or(10) as i64;
    
    let mut results = Vec::new();
    
    // Build query based on filters
    match (&input.domain, &input.status) {
        (Some(domain), Some(status)) => {
            let mut stmt = conn.prepare(
                "SELECT id, statement, domain, status, confidence, supporting_count, contradicting_count, created_at, updated_at 
                 FROM hypotheses WHERE domain = ?1 AND status = ?2 ORDER BY updated_at DESC LIMIT ?3"
            )?;
            let iter = stmt.query_map((domain, status, limit), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "statement": row.get::<_, String>(1)?,
                    "domain": row.get::<_, String>(2)?,
                    "status": row.get::<_, String>(3)?,
                    "confidence": row.get::<_, f32>(4)?,
                    "supporting_count": row.get::<_, u32>(5)?,
                    "contradicting_count": row.get::<_, u32>(6)?,
                    "created_at": row.get::<_, String>(7)?,
                    "updated_at": row.get::<_, String>(8)?
                }))
            })?;
            for h in iter {
                results.push(h?);
            }
        }
        (Some(domain), None) => {
            let mut stmt = conn.prepare(
                "SELECT id, statement, domain, status, confidence, supporting_count, contradicting_count, created_at, updated_at 
                 FROM hypotheses WHERE domain = ?1 ORDER BY updated_at DESC LIMIT ?2"
            )?;
            let iter = stmt.query_map((domain, limit), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "statement": row.get::<_, String>(1)?,
                    "domain": row.get::<_, String>(2)?,
                    "status": row.get::<_, String>(3)?,
                    "confidence": row.get::<_, f32>(4)?,
                    "supporting_count": row.get::<_, u32>(5)?,
                    "contradicting_count": row.get::<_, u32>(6)?,
                    "created_at": row.get::<_, String>(7)?,
                    "updated_at": row.get::<_, String>(8)?
                }))
            })?;
            for h in iter {
                results.push(h?);
            }
        }
        (None, Some(status)) => {
            let mut stmt = conn.prepare(
                "SELECT id, statement, domain, status, confidence, supporting_count, contradicting_count, created_at, updated_at 
                 FROM hypotheses WHERE status = ?1 ORDER BY updated_at DESC LIMIT ?2"
            )?;
            let iter = stmt.query_map((status, limit), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "statement": row.get::<_, String>(1)?,
                    "domain": row.get::<_, String>(2)?,
                    "status": row.get::<_, String>(3)?,
                    "confidence": row.get::<_, f32>(4)?,
                    "supporting_count": row.get::<_, u32>(5)?,
                    "contradicting_count": row.get::<_, u32>(6)?,
                    "created_at": row.get::<_, String>(7)?,
                    "updated_at": row.get::<_, String>(8)?
                }))
            })?;
            for h in iter {
                results.push(h?);
            }
        }
        (None, None) => {
            let mut stmt = conn.prepare(
                "SELECT id, statement, domain, status, confidence, supporting_count, contradicting_count, created_at, updated_at 
                 FROM hypotheses ORDER BY updated_at DESC LIMIT ?1"
            )?;
            let iter = stmt.query_map([limit], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "statement": row.get::<_, String>(1)?,
                    "domain": row.get::<_, String>(2)?,
                    "status": row.get::<_, String>(3)?,
                    "confidence": row.get::<_, f32>(4)?,
                    "supporting_count": row.get::<_, u32>(5)?,
                    "contradicting_count": row.get::<_, u32>(6)?,
                    "created_at": row.get::<_, String>(7)?,
                    "updated_at": row.get::<_, String>(8)?
                }))
            })?;
            for h in iter {
                results.push(h?);
            }
        }
    }
    
    Ok(ToolOutput::success(serde_json::json!({
        "hypotheses": results,
        "count": results.len()
    })))
}

pub async fn execute_list_observations(
    input: ListObservationsInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let conn = db.connection()?;
    let limit = input.limit.unwrap_or(10) as i64;
    
    let query = if input.observation_type.is_some() {
        "SELECT id, content, context, observation_type, created_at FROM observations WHERE observation_type = ?1 ORDER BY created_at DESC LIMIT ?2"
    } else {
        "SELECT id, content, context, observation_type, created_at FROM observations ORDER BY created_at DESC LIMIT ?1"
    };
    
    let mut results = Vec::new();
    
    if let Some(obs_type) = input.observation_type {
        let mut stmt = conn.prepare(query)?;
        let iter = stmt.query_map((obs_type.as_str(), limit), |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let context: String = row.get(2)?;
            let observation_type: String = row.get(3)?;
            let created_at_str: String = row.get(4)?;
            
            Ok(serde_json::json!({
                "id": id_str,
                "content": content,
                "context": context,
                "observation_type": observation_type,
                "created_at": created_at_str
            }))
        })?;
        for o in iter {
            results.push(o?);
        }
    } else {
        let mut stmt = conn.prepare(query)?;
        let iter = stmt.query_map([limit], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let context: String = row.get(2)?;
            let observation_type: String = row.get(3)?;
            let created_at_str: String = row.get(4)?;
            
            Ok(serde_json::json!({
                "id": id_str,
                "content": content,
                "context": context,
                "observation_type": observation_type,
                "created_at": created_at_str
            }))
        })?;
        for o in iter {
            results.push(o?);
        }
    }
    
    Ok(ToolOutput::success(serde_json::json!({
        "observations": results,
        "count": results.len()
    })))
}

pub async fn execute_evaluate_hypothesis(
    input: EvaluateHypothesisInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let hypothesis_id = Uuid::parse_str(&input.hypothesis_id)
        .map_err(|e| anyhow::anyhow!("Invalid hypothesis ID: {}", e))?;
    
    let mut hypothesis = get_hypothesis_by_id(db, &hypothesis_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Hypothesis not found"))?;
    
    let evidence = get_evidence_for_hypothesis(db, &hypothesis_id).await?;
    
    // Calculate new status based on evidence
    let supporting_count = evidence.iter().filter(|e| e.direction == "support").count() as u32;
    let contradicting_count = evidence.iter().filter(|e| e.direction == "contradict").count() as u32;
    let total = supporting_count + contradicting_count;
    
    // Update counts
    hypothesis.supporting_count = supporting_count;
    hypothesis.contradicting_count = contradicting_count;
    hypothesis.updated_at = chrono::Utc::now();
    
    // Determine status
    if total >= 3 {
        // Enough evidence to evaluate
        if supporting_count > contradicting_count * 2 {
            hypothesis.status = HypothesisStatus::Supported;
            hypothesis.confidence = supporting_count as f32 / total as f32;
        } else if contradicting_count > supporting_count * 2 {
            hypothesis.status = HypothesisStatus::Refuted;
            hypothesis.confidence = contradicting_count as f32 / total as f32;
        } else {
            hypothesis.status = HypothesisStatus::Inconclusive;
            hypothesis.confidence = 0.5;
        }
    } else {
        hypothesis.status = HypothesisStatus::Testing;
        hypothesis.confidence = if total > 0 {
            supporting_count as f32 / total as f32
        } else {
            0.5
        };
    }
    
    update_hypothesis(db, &hypothesis).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "hypothesis_id": hypothesis_id.to_string(),
        "evaluation": {
            "total_evidence": total,
            "supporting_count": supporting_count,
            "contradicting_count": contradicting_count
        },
        "result": {
            "status": hypothesis.status.to_string(),
            "confidence": hypothesis.confidence,
            "updated_at": hypothesis.updated_at.to_rfc3339()
        },
        "workflow": if hypothesis.status == HypothesisStatus::Supported {
            "Hypothesis is supported! Use extract_knowledge to convert this into reusable knowledge."
        } else if hypothesis.status == HypothesisStatus::Refuted {
            "Hypothesis is refuted. This learning is still valuable - it prevents future mistakes."
        } else {
            "Not enough evidence yet. Continue gathering evidence with add_evidence."
        }
    })))
}

pub async fn execute_get_knowledge(
    input: GetKnowledgeInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(10);
    let knowledge = get_knowledge(db, input.domain.as_deref(), limit).await?;
    
    let count = knowledge.len();
    let knowledge_json: Vec<_> = knowledge.into_iter().map(|k| serde_json::json!({
        "id": k.id.to_string(),
        "content": k.content,
        "domain": k.domain,
        "confidence": k.confidence,
        "derivation": k.derivation,
        "created_at": k.created_at.to_rfc3339()
    })).collect();
    
    Ok(ToolOutput::success(serde_json::json!({
        "knowledge": knowledge_json,
        "count": count
    })))
}

pub async fn execute_extract_knowledge(
    input: ExtractKnowledgeInput,
    db: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let hypothesis_id = Uuid::parse_str(&input.hypothesis_id)
        .map_err(|e| anyhow::anyhow!("Invalid hypothesis ID: {}", e))?;
    
    let hypothesis = get_hypothesis_by_id(db, &hypothesis_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Hypothesis not found"))?;
    
    // Only allow extracting from supported hypotheses
    if hypothesis.status != HypothesisStatus::Supported {
        return Ok(ToolOutput::error(format!(
            "Can only extract knowledge from supported hypotheses. Current status: {}",
            hypothesis.status.to_string()
        )));
    }
    
    // Create knowledge
    let mut knowledge = Knowledge::new(
        input.knowledge_content,
        hypothesis.domain.clone(),
        format!("Extracted from hypothesis: {}", hypothesis.statement),
    );
    knowledge.source_hypothesis = Some(hypothesis_id);
    knowledge.confidence = hypothesis.confidence;
    
    create_knowledge(db, &knowledge).await?;
    
    // Mark hypothesis as superseded (knowledge extracted)
    let mut updated_hypothesis = hypothesis;
    updated_hypothesis.status = HypothesisStatus::Superseded;
    updated_hypothesis.updated_at = chrono::Utc::now();
    update_hypothesis(db, &updated_hypothesis).await?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "status": "knowledge_extracted",
        "knowledge": {
            "id": knowledge.id.to_string(),
            "content": knowledge.content,
            "domain": knowledge.domain,
            "confidence": knowledge.confidence,
            "derivation": knowledge.derivation,
            "created_at": knowledge.created_at.to_rfc3339()
        },
        "hypothesis_status": "superseded",
        "learning_complete": "This knowledge is now available for future decisions. The hypothesis has been superseded by the extracted knowledge."
    })))
}
