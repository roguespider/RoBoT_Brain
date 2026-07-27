// src/memory/repository.rs
//! Memory Repository - Per Architecture §6.3
//!
//! Provides persistence layer for memory items using SQLite.
//! Bridges in-memory structures with database storage.

use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::sqlite::SqliteDatabase;
use crate::memory::types::{MemoryItem, MemoryLayer, MemoryStatus, MemoryType};

/// Repository trait for memory persistence (scaffolding for future use)

pub trait MemoryRepository: Send + Sync {
    /// Store a memory item
    fn store(&self, item: &MemoryItem) -> Result<()>;

    /// Retrieve a memory item by ID
    fn retrieve(&self, id: &Uuid) -> Result<Option<MemoryItem>>;

    /// Search memories by content
    fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryItem>>;

    /// List memories by type
    fn list_by_type(&self, memory_type: &MemoryType, limit: usize) -> Result<Vec<MemoryItem>>;

    /// List all memories with limit
    fn list_all(&self, limit: usize) -> Result<Vec<MemoryItem>>;

    /// Update a memory item
    fn update(&self, item: &MemoryItem) -> Result<()>;

    /// Delete a memory item
    fn delete(&self, id: &Uuid) -> Result<()>;

    /// Get related memories (graph traversal)
    fn get_related(&self, id: &Uuid, depth: usize) -> Result<Vec<MemoryItem>>;

    /// Add a relationship between memories
    fn add_relationship(&self, from_id: &Uuid, to_id: &Uuid, relationship_type: &str) -> Result<()>;

    /// Get memory statistics
    fn stats(&self) -> Result<MemoryStats>;
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_count: usize,
    pub working_count: usize,
    pub permanent_count: usize,
    pub by_type: Vec<(String, usize)>,
    pub avg_confidence: f32,
    pub avg_importance: f32,
}

/// SQLite implementation of MemoryRepository
pub struct SqliteMemoryRepository {
    db: SqliteDatabase,
}

impl SqliteMemoryRepository {
    /// Create a new SQLite memory repository
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// Create from database path
    pub fn from_path(path: &std::path::Path) -> Result<Self> {
        let db = SqliteDatabase::initialize_at(path)?;
        Ok(Self::new(db))
    }
}

impl MemoryRepository for SqliteMemoryRepository {
    fn store(&self, item: &MemoryItem) -> Result<()> {
        let conn = self.db.connection()?;
        conn.execute(
            "
            INSERT OR REPLACE INTO memories 
            (id, content, memory_type, confidence, importance, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            rusqlite::params![
                item.id.to_string(),
                item.content,
                memory_type_to_string(&item.memory_type),
                item.confidence,
                item.importance,
                item.created_at.to_rfc3339(),
                item.modified_at.to_rfc3339(),
            ],
        )?;

        // Store memory tags if any
        for tag in &item.tags {
            conn.execute(
                "INSERT OR IGNORE INTO memory_tags (memory_id, tag) VALUES (?1, ?2)",
                rusqlite::params![item.id.to_string(), tag],
            )?;
        }

        // Store relationships if any
        for related_id in &item.related_ids {
            conn.execute(
                "INSERT OR IGNORE INTO memory_relationships (memory_id, related_id) VALUES (?1, ?2)",
                rusqlite::params![item.id.to_string(), related_id.to_string()],
            )?;
        }

        Ok(())
    }

    fn retrieve(&self, id: &Uuid) -> Result<Option<MemoryItem>> {
        let conn = self.db.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, content, memory_type, confidence, importance, created_at, updated_at 
             FROM memories WHERE id = ?1"
        )?;

        let result = stmt.query_row([id.to_string()], |row| {
            let uuid_str: String = row.get(0)?;
            let memory_type_str: String = row.get(2)?;
            
            Ok(MemoryItem {
                id: Uuid::parse_str(&uuid_str).unwrap_or_default(),
                layer: MemoryLayer::Permanent,
                memory_type: string_to_memory_type(&memory_type_str),
                status: MemoryStatus::Active,
                content: row.get(1)?,
                confidence: row.get(3)?,
                importance: row.get(4)?,
                created_at: parse_time(&row.get::<_, String>(5)?),
                accessed_at: parse_time(&row.get::<_, String>(6)?),
                modified_at: parse_time(&row.get::<_, String>(6)?),
                last_consolidated: Some(parse_time(&row.get::<_, String>(6)?)),
                access_count: 0,
                tags: Vec::new(),
                source: "database".to_string(),
                related_ids: Vec::new(),
            })
        });

        match result {
            Ok(mut item) => {
                // Load tags
                let mut tag_stmt = conn.prepare(
                    "SELECT tag FROM memory_tags WHERE memory_id = ?1"
                )?;
                let tags: Vec<String> = tag_stmt
                    .query_map([id.to_string()], |row| row.get(0))?
                    .filter_map(|r| r.ok())
                    .collect();
                item.tags = tags;

                // Load relationships
                let mut rel_stmt = conn.prepare(
                    "SELECT related_id FROM memory_relationships WHERE memory_id = ?1"
                )?;
                let related_ids: Vec<Uuid> = rel_stmt
                    .query_map([id.to_string()], |row| {
                        let id_str: String = row.get(0)?;
                        Ok(Uuid::parse_str(&id_str).unwrap_or_default())
                    })?
                    .filter_map(|r| r.ok())
                    .filter(|id| *id != Uuid::nil())
                    .collect();
                item.related_ids = related_ids;

                Ok(Some(item))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryItem>> {
        let conn = self.db.connection()?;
        let pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            "SELECT id, content, memory_type, confidence, importance, created_at, updated_at 
             FROM memories 
             WHERE content LIKE ?1
             ORDER BY confidence DESC, updated_at DESC
             LIMIT ?2"
        )?;

        let items = stmt.query_map(rusqlite::params![pattern, limit as i64], |row| {
            let uuid_str: String = row.get(0)?;
            let memory_type_str: String = row.get(2)?;
            
            Ok(MemoryItem {
                id: Uuid::parse_str(&uuid_str).unwrap_or_default(),
                layer: MemoryLayer::Permanent,
                memory_type: string_to_memory_type(&memory_type_str),
                status: MemoryStatus::Active,
                content: row.get(1)?,
                confidence: row.get(3)?,
                importance: row.get(4)?,
                created_at: parse_time(&row.get::<_, String>(5)?),
                accessed_at: parse_time(&row.get::<_, String>(6)?),
                modified_at: parse_time(&row.get::<_, String>(6)?),
                last_consolidated: Some(parse_time(&row.get::<_, String>(6)?)),
                access_count: 0,
                tags: Vec::new(),
                source: "database".to_string(),
                related_ids: Vec::new(),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(items)
    }

    fn list_by_type(&self, memory_type: &MemoryType, limit: usize) -> Result<Vec<MemoryItem>> {
        let conn = self.db.connection()?;
        let type_str = memory_type_to_string(memory_type);

        let mut stmt = conn.prepare(
            "SELECT id, content, memory_type, confidence, importance, created_at, updated_at 
             FROM memories 
             WHERE memory_type = ?1
             ORDER BY updated_at DESC
             LIMIT ?2"
        )?;

        let items = stmt.query_map(rusqlite::params![type_str, limit as i64], |row| {
            let uuid_str: String = row.get(0)?;
            
            Ok(MemoryItem {
                id: Uuid::parse_str(&uuid_str).unwrap_or_default(),
                layer: MemoryLayer::Permanent,
                memory_type: string_to_memory_type(&row.get::<_, String>(2)?),
                status: MemoryStatus::Active,
                content: row.get(1)?,
                confidence: row.get(3)?,
                importance: row.get(4)?,
                created_at: parse_time(&row.get::<_, String>(5)?),
                accessed_at: parse_time(&row.get::<_, String>(6)?),
                modified_at: parse_time(&row.get::<_, String>(6)?),
                last_consolidated: Some(parse_time(&row.get::<_, String>(6)?)),
                access_count: 0,
                tags: Vec::new(),
                source: "database".to_string(),
                related_ids: Vec::new(),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(items)
    }

    fn list_all(&self, limit: usize) -> Result<Vec<MemoryItem>> {
        let conn = self.db.connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, content, memory_type, confidence, importance, created_at, updated_at 
             FROM memories 
             ORDER BY updated_at DESC
             LIMIT ?1"
        )?;

        let items = stmt.query_map([limit as i64], |row| {
            let uuid_str: String = row.get(0)?;
            
            Ok(MemoryItem {
                id: Uuid::parse_str(&uuid_str).unwrap_or_default(),
                layer: MemoryLayer::Permanent,
                memory_type: string_to_memory_type(&row.get::<_, String>(2)?),
                status: MemoryStatus::Active,
                content: row.get(1)?,
                confidence: row.get(3)?,
                importance: row.get(4)?,
                created_at: parse_time(&row.get::<_, String>(5)?),
                accessed_at: parse_time(&row.get::<_, String>(6)?),
                modified_at: parse_time(&row.get::<_, String>(6)?),
                last_consolidated: Some(parse_time(&row.get::<_, String>(6)?)),
                access_count: 0,
                tags: Vec::new(),
                source: "database".to_string(),
                related_ids: Vec::new(),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(items)
    }

    fn update(&self, item: &MemoryItem) -> Result<()> {
        let conn = self.db.connection()?;
        conn.execute(
            "UPDATE memories SET content = ?2, memory_type = ?3, confidence = ?4, 
             importance = ?5, updated_at = ?6 WHERE id = ?1",
            rusqlite::params![
                item.id.to_string(),
                item.content,
                memory_type_to_string(&item.memory_type),
                item.confidence,
                item.importance,
                item.modified_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn delete(&self, id: &Uuid) -> Result<()> {
        let conn = self.db.connection()?;
        conn.execute("DELETE FROM memories WHERE id = ?1", [id.to_string()])?;
        conn.execute("DELETE FROM memory_tags WHERE memory_id = ?1", [id.to_string()])?;
        conn.execute("DELETE FROM memory_relationships WHERE memory_id = ?1 OR related_id = ?1", [id.to_string()])?;
        Ok(())
    }

    fn get_related(&self, id: &Uuid, depth: usize) -> Result<Vec<MemoryItem>> {
        if depth == 0 {
            return Ok(Vec::new());
        }

        let conn = self.db.connection()?;
        let mut visited = std::collections::HashSet::new();
        let mut result = Vec::new();
        let mut queue = vec![id.to_string()];

        while let Some(current_id) = queue.pop() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());

            // Get direct relationships
            let mut stmt = conn.prepare(
                "SELECT related_id FROM memory_relationships WHERE memory_id = ?1"
            )?;
            let related: Vec<String> = stmt
                .query_map([&current_id], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect();

            for rel_id in related {
                if let Ok(uuid) = Uuid::parse_str(&rel_id) {
                    if !visited.contains(&rel_id) {
                        if let Ok(Some(item)) = self.retrieve(&uuid) {
                            result.push(item);
                            if depth > 1 {
                                queue.push(rel_id);
                            }
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    fn add_relationship(&self, from_id: &Uuid, to_id: &Uuid, _relationship_type: &str) -> Result<()> {
        let conn = self.db.connection()?;
        conn.execute(
            "INSERT OR IGNORE INTO memory_relationships (memory_id, related_id) VALUES (?1, ?2)",
            rusqlite::params![from_id.to_string(), to_id.to_string()],
        )?;
        Ok(())
    }

    fn stats(&self) -> Result<MemoryStats> {
        let conn = self.db.connection()?;

        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memories", 
            [], |row| row.get(0)
        )?;

        let avg_confidence: f64 = conn.query_row(
            "SELECT COALESCE(AVG(confidence), 0) FROM memories",
            [], |row| row.get(0)
        )?;

        let avg_importance: f64 = conn.query_row(
            "SELECT COALESCE(AVG(importance), 0) FROM memories",
            [], |row| row.get(0)
        )?;

        let mut stmt = conn.prepare(
            "SELECT memory_type, COUNT(*) FROM memories GROUP BY memory_type"
        )?;
        let by_type: Vec<(String, usize)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(MemoryStats {
            total_count: total as usize,
            working_count: 0, // Working memory is in-memory only
            permanent_count: total as usize,
            by_type,
            avg_confidence: avg_confidence as f32,
            avg_importance: avg_importance as f32,
        })
    }
}

// ==========================================================
// HELPERS
// ==========================================================

fn memory_type_to_string(mt: &MemoryType) -> String {
    mt.to_string()
}

fn string_to_memory_type(s: &str) -> MemoryType {
    match s.to_lowercase().as_str() {
        "experience" => MemoryType::Experience,
        "knowledge" => MemoryType::Knowledge,
        "skill" => MemoryType::Skill,
        "workflow" => MemoryType::Workflow,
        "context" => MemoryType::Context,
        "observation" => MemoryType::Observation,
        _ => MemoryType::Experience,
    }
}

fn parse_time(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
