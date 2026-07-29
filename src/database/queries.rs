// src/database/queries.rs

#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::database::models::{MemoryCard, MemoryType};

// ==========================================================
// MEMORY OPERATIONS
// ==========================================================

fn parse_hierarchy_level(s: &str) -> crate::database::models::HierarchyLevel {
    match s {
        "section" => crate::database::models::HierarchyLevel::Section,
        "subsection" => crate::database::models::HierarchyLevel::Subsection,
        "paragraph" => crate::database::models::HierarchyLevel::Paragraph,
        "sentence" => crate::database::models::HierarchyLevel::Sentence,
        _ => crate::database::models::HierarchyLevel::Document,
    }
}

pub fn insert_memory(conn: &Connection, memory: &MemoryCard) -> Result<()> {
    conn.execute(
        "
        INSERT OR REPLACE INTO memories
        (
            id,
            content,
            memory_type,
            layer,
            parent_id,
            hierarchy_level,
            order_index,
            path,
            file_source,
            access_count,
            last_accessed,
            confidence,
            importance,
            created_at,
            updated_at
        )
        VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)
        ",
        params![
            memory.id.to_string(),
            memory.content,
            memory.memory_type.to_string(),
            memory.layer.to_string(),
            memory.parent_id.map(|u| u.to_string()),
            memory.hierarchy_level.to_string(),
            memory.order_index,
            memory.path,
            memory.file_source,
            memory.access_count,
            memory.last_accessed.map(|t| t.to_rfc3339()),
            memory.confidence,
            memory.importance,
            memory.created_at.to_rfc3339(),
            memory.updated_at.to_rfc3339()
        ],
    )?;

    Ok(())
}

/// Delete memories by their IDs (internal helper)
pub(crate) fn delete_memories(conn: &Connection, ids: &[Uuid]) -> Result<usize> {
    if ids.is_empty() {
        return Ok(0);
    }
    
    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let query = format!(
        "DELETE FROM memories WHERE id IN ({})",
        placeholders.join(",")
    );
    
    let params: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter()
        .map(|s| s as &dyn rusqlite::ToSql)
        .collect();
    
    let deleted = conn.execute(&query, params_refs.as_slice())?;
    Ok(deleted)
}

/// Delete memories by their string IDs (convenience function)
pub fn delete_memories_by_string_ids(conn: &Connection, ids: &[String]) -> Result<usize> {
    if ids.is_empty() {
        return Ok(0);
    }
    
    let uuids: Result<Vec<Uuid>, _> = ids
        .iter()
        .map(|s| Uuid::parse_str(s))
        .collect();
    
    match uuids {
        Ok(uuids) => delete_memories(conn, &uuids),
        Err(e) => anyhow::bail!("Invalid UUID: {}", e),
    }
}

pub fn get_memory(conn: &Connection, id: Uuid) -> Result<Option<MemoryCard>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            content,
            memory_type,
            COALESCE(layer, 'working') as layer,
            COALESCE(parent_id, '') as parent_id,
            COALESCE(hierarchy_level, 'document') as hierarchy_level,
            COALESCE(order_index, 0) as order_index,
            COALESCE(path, '') as path,
            file_source,
            COALESCE(access_count, 0) as access_count,
            last_accessed,
            confidence,
            importance,
            created_at,
            updated_at

        FROM memories

        WHERE id=?1
        ",
    )?;

    let result = stmt.query_row([id.to_string()], |row| {
        let uuid_str: String = row.get(0)?;
        let parent_id_str: String = row.get(4)?;
        let last_accessed_str: Option<String> = row.get(10)?;
        Ok(MemoryCard {
            id: Uuid::parse_str(&uuid_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            content: row.get(1)?,
            memory_type: parse_memory_type(&row.get::<_, String>(2)?),
            layer: parse_memory_layer(&row.get::<_, String>(3)?),
            parent_id: if parent_id_str.is_empty() { None } else { Uuid::parse_str(&parent_id_str).ok() },
            hierarchy_level: parse_hierarchy_level(&row.get::<_, String>(5)?),
            order_index: row.get(6)?,
            path: row.get(7)?,
            file_source: row.get(8)?,
            access_count: row.get(9)?,
            last_accessed: last_accessed_str.as_ref().map(|s| parse_time(s)),
            confidence: row.get(11)?,
            importance: row.get(12)?,
            created_at: parse_time(&row.get::<_, String>(13)?),
            updated_at: parse_time(&row.get::<_, String>(14)?),
        })
    });

    match result {
        Ok(memory) => Ok(Some(memory)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn search_memory(conn: &Connection, text: &str, limit: usize) -> Result<Vec<MemoryCard>> {
    let pattern = format!("%{}%", text);

    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            content,
            memory_type,
            COALESCE(layer, 'working') as layer,
            COALESCE(parent_id, '') as parent_id,
            COALESCE(hierarchy_level, 'document') as hierarchy_level,
            COALESCE(order_index, 0) as order_index,
            COALESCE(path, '') as path,
            file_source,
            COALESCE(access_count, 0) as access_count,
            last_accessed,
            confidence,
            importance,
            created_at,
            updated_at

        FROM memories

        WHERE content LIKE ?1

        ORDER BY updated_at DESC

        LIMIT ?2
        ",
    )?;

    let rows = stmt.query_map(params![pattern, limit as i64], |row| {
        let uuid_str: String = row.get(0)?;
        let parent_id_str: String = row.get(4)?;
        let last_accessed_str: Option<String> = row.get(10)?;
        Ok(MemoryCard {
            id: Uuid::parse_str(&uuid_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            content: row.get(1)?,
            memory_type: parse_memory_type(&row.get::<_, String>(2)?),
            layer: parse_memory_layer(&row.get::<_, String>(3)?),
            parent_id: if parent_id_str.is_empty() { None } else { Uuid::parse_str(&parent_id_str).ok() },
            hierarchy_level: parse_hierarchy_level(&row.get::<_, String>(5)?),
            order_index: row.get(6)?,
            path: row.get(7)?,
            file_source: row.get(8)?,
            access_count: row.get(9)?,
            last_accessed: last_accessed_str.as_ref().map(|s| parse_time(s)),
            confidence: row.get(11)?,
            importance: row.get(12)?,
            created_at: parse_time(&row.get::<_, String>(13)?),
            updated_at: parse_time(&row.get::<_, String>(14)?),
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Helper function to map a row to MemoryCard
fn map_row_to_memory_card(row: &rusqlite::Row) -> rusqlite::Result<MemoryCard> {
    let uuid_str: String = row.get(0)?;
    let parent_id_str: String = row.get(4)?;
    let last_accessed_str: Option<String> = row.get(10)?;
    Ok(MemoryCard {
        id: Uuid::parse_str(&uuid_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
        content: row.get(1)?,
        memory_type: parse_memory_type(&row.get::<_, String>(2)?),
        layer: parse_memory_layer(&row.get::<_, String>(3)?),
        parent_id: if parent_id_str.is_empty() { None } else { Uuid::parse_str(&parent_id_str).ok() },
        hierarchy_level: parse_hierarchy_level(&row.get::<_, String>(5)?),
        order_index: row.get(6)?,
        path: row.get(7)?,
        file_source: row.get(8)?,
        access_count: row.get(9)?,
        last_accessed: last_accessed_str.as_ref().map(|s| parse_time(s)),
        confidence: row.get(11)?,
        importance: row.get(12)?,
        created_at: parse_time(&row.get::<_, String>(13)?),
        updated_at: parse_time(&row.get::<_, String>(14)?),
    })
}

pub fn list_memories(conn: &Connection, memory_type: Option<&str>, limit: usize) -> Result<Vec<MemoryCard>> {
    let mut rows = Vec::new();
    
    if let Some(mem_type) = memory_type {
        let query = "SELECT id, content, memory_type, COALESCE(layer, 'working') as layer,
            COALESCE(parent_id, '') as parent_id, COALESCE(hierarchy_level, 'document') as hierarchy_level,
            COALESCE(order_index, 0) as order_index, COALESCE(path, '') as path, file_source,
            COALESCE(access_count, 0) as access_count, last_accessed, confidence, importance, created_at, updated_at
            FROM memories WHERE memory_type = ?1 ORDER BY updated_at DESC LIMIT ?2";
        let mut stmt = conn.prepare(query)?;
        let param = mem_type.to_string();
        let params = rusqlite::params![param, limit as i64];
        let mut rows_iter = stmt.query(params)?;
        while let Some(row) = rows_iter.next()? {
            rows.push(map_row_to_memory_card(row)?);
        }
    } else {
        let query = "SELECT id, content, memory_type, COALESCE(layer, 'working') as layer,
            COALESCE(parent_id, '') as parent_id, COALESCE(hierarchy_level, 'document') as hierarchy_level,
            COALESCE(order_index, 0) as order_index, COALESCE(path, '') as path, file_source,
            COALESCE(access_count, 0) as access_count, last_accessed, confidence, importance, created_at, updated_at
            FROM memories ORDER BY updated_at DESC LIMIT ?1";
        let mut stmt = conn.prepare(query)?;
        let mut rows_iter = stmt.query([limit as i64])?;
        while let Some(row) = rows_iter.next()? {
            rows.push(map_row_to_memory_card(row)?);
        }
    }

    Ok(rows)
}

/// List memories by layer (Working or Permanent)
pub fn list_memories_by_layer(conn: &Connection, layer: &str, limit: usize) -> Result<Vec<MemoryCard>> {
    let query = "SELECT id, content, memory_type, COALESCE(layer, 'working') as layer,
        COALESCE(parent_id, '') as parent_id, COALESCE(hierarchy_level, 'document') as hierarchy_level,
        COALESCE(order_index, 0) as order_index, COALESCE(path, '') as path, file_source,
        COALESCE(access_count, 0) as access_count, last_accessed, confidence, importance, created_at, updated_at
        FROM memories WHERE layer = ?1 ORDER BY updated_at DESC LIMIT ?2";
    
    let mut rows = Vec::new();
    let mut stmt = conn.prepare(query)?;
    let param = layer.to_string();
    let mut rows_iter = stmt.query(rusqlite::params![param, limit as i64])?;
    while let Some(row) = rows_iter.next()? {
        rows.push(map_row_to_memory_card(row)?);
    }

    Ok(rows)
}

// ==========================================================
// HELPERS
// ==========================================================

fn parse_memory_type(value: &str) -> MemoryType {
    match value {
        "fact" => MemoryType::Fact,
        "task" => MemoryType::Task,
        "file" => MemoryType::File,
        "conversation" => MemoryType::Conversation,
        "code" => MemoryType::Code,
        "decision" => MemoryType::Decision,
        "event" => MemoryType::Event,
        "encounter" => MemoryType::Encounter,
        "experience" => MemoryType::Experience,
        _ => MemoryType::Note,
    }
}

fn parse_memory_layer(value: &str) -> crate::database::models::MemoryLayer {
    match value {
        "permanent" => crate::database::models::MemoryLayer::Permanent,
        _ => crate::database::models::MemoryLayer::Working,
    }
}

fn parse_time(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

// ==========================================================
// SCHEDULED TASKS
// ==========================================================

use crate::experience::scheduler::{ScheduledTask, TaskSchedule, TaskStatus, TaskType};

pub fn insert_scheduled_task(conn: &Connection, task: &ScheduledTask) -> Result<()> {
    conn.execute(
        "
        INSERT OR REPLACE INTO scheduled_tasks
        (
            id,
            name,
            task_type,
            schedule,
            status,
            last_run,
            next_run,
            failure_count,
            created_at
        )
        VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
        ",
        params![
            task.id,
            task.name,
            serde_json::to_string(&task.task_type)?,
            serde_json::to_string(&task.schedule)?,
            serde_json::to_string(&task.status)?,
            task.last_run.map(|t| t.to_rfc3339()),
            task.next_run.map(|t| t.to_rfc3339()),
            task.failure_count,
            task.created_at.to_rfc3339()
        ],
    )?;

    Ok(())
}

pub fn get_scheduled_task(conn: &Connection, id: &str) -> Result<Option<ScheduledTask>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            name,
            task_type,
            schedule,
            status,
            last_run,
            next_run,
            failure_count,
            created_at
        FROM scheduled_tasks
        WHERE id = ?1
        ",
    )?;

    let mut rows = stmt.query(params![id])?;
    
    if let Some(row) = rows.next()? {
        Ok(Some(ScheduledTask {
            id: row.get(0)?,
            name: row.get(1)?,
            task_type: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or(TaskType::Custom),
            schedule: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(TaskSchedule::Manual),
            status: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or(TaskStatus::Scheduled),
            last_run: row.get::<_, Option<String>>(5)?.as_deref().map(parse_time),
            next_run: row.get::<_, Option<String>>(6)?.as_deref().map(parse_time),
            failure_count: row.get(7)?,
            created_at: parse_time(&row.get::<_, String>(8)?),
        }))
    } else {
        Ok(None)
    }
}

pub fn list_scheduled_tasks(conn: &Connection) -> Result<Vec<ScheduledTask>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            name,
            task_type,
            schedule,
            status,
            last_run,
            next_run,
            failure_count,
            created_at
        FROM scheduled_tasks
        ORDER BY created_at DESC
        ",
    )?;

    let mut tasks = Vec::new();
    let mut rows = stmt.query([])?;
    
    while let Some(row) = rows.next()? {
        tasks.push(ScheduledTask {
            id: row.get(0)?,
            name: row.get(1)?,
            task_type: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or(TaskType::Custom),
            schedule: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(TaskSchedule::Manual),
            status: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or(TaskStatus::Scheduled),
            last_run: row.get::<_, Option<String>>(5)?.as_deref().map(parse_time),
            next_run: row.get::<_, Option<String>>(6)?.as_deref().map(parse_time),
            failure_count: row.get(7)?,
            created_at: parse_time(&row.get::<_, String>(8)?),
        });
    }

    Ok(tasks)
}

/// Delete a scheduled task by ID
pub fn delete_scheduled_task(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM scheduled_tasks WHERE id = ?1", params![id])?;
    Ok(())
}

// ==========================================================
// OBSERVATION OPERATIONS (Per Architecture §07)
// ==========================================================

use crate::database::models::Observation;

/// Insert an observation (Per Architecture §07: Every experience originates from observations)

pub fn insert_observation(conn: &Connection, observation: &Observation) -> Result<()> {
    conn.execute(
        "
        INSERT OR REPLACE INTO observations
        (
            id,
            content,
            context,
            observation_type,
            related_experiences,
            triggered_hypothesis,
            created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ",
        params![
            observation.id.to_string(),
            observation.content,
            observation.context,
            observation.observation_type,
            serde_json::to_string(&observation.related_experiences)?,
            observation.triggered_hypothesis.map(|u| u.to_string()),
            observation.created_at.to_rfc3339()
        ],
    )?;
    Ok(())
}

/// Get an observation by ID
pub fn get_observation(conn: &Connection, id: Uuid) -> Result<Option<Observation>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, context, observation_type, related_experiences, triggered_hypothesis, created_at
         FROM observations WHERE id = ?1"
    )?;

    let result = stmt.query_row([id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let related_json: String = row.get(4)?;
        let triggered_str: Option<String> = row.get(5)?;
        Ok(Observation {
            id: Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            content: row.get(1)?,
            context: row.get(2)?,
            observation_type: row.get(3)?,
            related_experiences: serde_json::from_str(&related_json).unwrap_or_default(),
            triggered_hypothesis: triggered_str.and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: parse_time(&row.get::<_, String>(6)?),
        })
    });

    match result {
        Ok(obs) => Ok(Some(obs)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// List recent observations

pub fn list_observations(conn: &Connection, limit: usize) -> Result<Vec<Observation>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, context, observation_type, related_experiences, triggered_hypothesis, created_at
         FROM observations ORDER BY created_at DESC LIMIT ?1"
    )?;

    let rows = stmt.query_map([limit as i64], |row| {
        let id_str: String = row.get(0)?;
        let related_json: String = row.get(4)?;
        let triggered_str: Option<String> = row.get(5)?;
        Ok(Observation {
            id: Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            content: row.get(1)?,
            context: row.get(2)?,
            observation_type: row.get(3)?,
            related_experiences: serde_json::from_str(&related_json).unwrap_or_default(),
            triggered_hypothesis: triggered_str.and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: parse_time(&row.get::<_, String>(6)?),
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Link an observation to an experience
pub fn link_observation_to_experience(conn: &Connection, observation_id: Uuid, experience_id: Uuid) -> Result<()> {
    if let Some(mut obs) = get_observation(conn, observation_id)? {
        obs.related_experiences.push(experience_id);
        insert_observation(conn, &obs)?;
    }
    Ok(())
}

// ==========================================================
// EXPERIENCE OPERATIONS (for scheduler use)
// ==========================================================

use crate::experience::types::{Experience, ExperienceContext, ExperienceOutcome, ExperienceScore, ExperienceType};
use crate::experience::types::maturity::KnowledgeMaturity;

/// List recent experiences from the database
pub fn list_experiences(conn: &Connection, limit: usize) -> Result<Vec<Experience>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, experience_type, context, outcome, score, timestamp, observation_ids, encounter_ids, maturity, confidence, lessons, evidence_count, tags, committed, archived, archived_at, metadata
         FROM experiences
         ORDER BY timestamp DESC
         LIMIT ?1"
    )?;
    
    let experiences = stmt.query_map(params![limit], |row| {
        let id_str: String = row.get(0)?;
        let title: String = row.get(1)?;
        let description: String = row.get(2)?;
        let experience_type_json: String = row.get(3)?;
        let context_json: String = row.get(4)?;
        let outcome_json: String = row.get(5)?;
        let score_json: String = row.get(6)?;
        let timestamp_str: String = row.get(7)?;
        let observation_ids_json: String = row.get(8)?;
        let encounter_ids_json: String = row.get(9)?;
        let maturity_json: String = row.get(10)?;
        let confidence: f32 = row.get(11)?;
        let lessons_json: String = row.get(12)?;
        let evidence_count: usize = row.get(13)?;
        let tags_json: String = row.get(14)?;
        let committed: bool = row.get(15)?;
        let archived: bool = row.get(16)?;
        let archived_at_str: Option<String> = row.get(17)?;
        let metadata_json: String = row.get(18)?;
        
        let context: ExperienceContext = serde_json::from_str(&context_json)
            .unwrap_or_default();
        let outcome: ExperienceOutcome = serde_json::from_str(&outcome_json)
            .unwrap_or_else(|_| ExperienceOutcome::failure("Failed to parse outcome"));
        let score: Option<ExperienceScore> = serde_json::from_str(&score_json).ok();
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let observation_ids: Vec<Uuid> = serde_json::from_str(&observation_ids_json).unwrap_or_default();
        let encounter_ids: Vec<Uuid> = serde_json::from_str(&encounter_ids_json).unwrap_or_default();
        let maturity: KnowledgeMaturity = serde_json::from_str(&maturity_json)
            .unwrap_or(KnowledgeMaturity::Emerging);
        let lessons: Vec<String> = serde_json::from_str(&lessons_json).unwrap_or_default();
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        let metadata: std::collections::HashMap<String, String> = serde_json::from_str(&metadata_json).unwrap_or_default();
        let archived_at = archived_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));
        
        Ok(Experience {
            id: Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4()),
            timestamp,
            observation_ids,
            experience_type: serde_json::from_str(&experience_type_json)
                .unwrap_or(ExperienceType::ToolExecution),
            title,
            description,
            context,
            outcome,
            score,
            encounter_ids,
            maturity,
            confidence,
            lessons,
            evidence_count,
            tags,
            committed,
            archived,
            archived_at,
            metadata,
        })
    })?.filter_map(|r| r.ok()).collect();
    
    Ok(experiences)
}

// ==========================================================
// REPUTATION OPERATIONS (for scheduler use)
// ==========================================================

use crate::experience::reputation::reputation::Reputation;

/// List all reputations from the database
pub fn list_reputations(conn: &Connection) -> Result<Vec<Reputation>> {
    let mut stmt = conn.prepare(
        "SELECT id, score, observations, successes, failures, updated_at
         FROM reputations"
    )?;
    
    let reputations = stmt.query_map(params![], |row| {
        let id: String = row.get(0)?;
        let score: f64 = row.get(1)?;
        let observations: u64 = row.get(2)?;
        let successes: u64 = row.get(3)?;
        let failures: u64 = row.get(4)?;
        let updated_at_str: String = row.get(5)?;
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        Ok(Reputation {
            id,
            score,
            factors: Vec::new(), // Factors not stored in this table structure
            observations,
            successes,
            failures,
            updated_at,
            history: Vec::new(),
        })
    })?.filter_map(|r| r.ok()).collect();
    
    Ok(reputations)
}

/// Insert or update a reputation
pub fn insert_reputation(conn: &Connection, reputation: &Reputation) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO reputations
         (id, score, observations, successes, failures, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            reputation.id,
            reputation.score,
            reputation.observations,
            reputation.successes,
            reputation.failures,
            reputation.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

// ==========================================================
// MEMORY RELATIONSHIP OPERATIONS
// ==========================================================

use crate::database::models::MemoryRelationship;

/// Insert a new memory relationship
pub fn insert_memory_relationship(conn: &Connection, relationship: &MemoryRelationship) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO memory_relationships
         (id, memory_id, related_id, relationship_type)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            relationship.id.to_string(),
            relationship.memory_id.to_string(),
            relationship.related_id.to_string(),
            relationship.relationship_type.to_string(),
        ],
    )?;
    Ok(())
}

