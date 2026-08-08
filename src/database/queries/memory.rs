// src/database/queries/memory.rs
//! Memory database operations

use anyhow::Result;
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::database::models::MemoryCard;

use super::helpers::map_row_to_memory_card;

/// Insert or replace a memory card in the database
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

/// Delete memories by their UUIDs
pub fn delete_memories(conn: &Connection, ids: &[Uuid]) -> Result<usize> {
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

/// Delete memories by their string IDs
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

/// Get a memory card by ID
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
        map_row_to_memory_card(row)
    });

    match result {
        Ok(memory) => Ok(Some(memory)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Search memories by text content
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
        ORDER BY importance DESC, confidence DESC
        LIMIT ?2
        ",
    )?;

    let rows = stmt.query_map(params![pattern, limit as i64], |row| {
        map_row_to_memory_card(row)
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

/// List memories, optionally filtered by type
pub fn list_memories(conn: &Connection, memory_type: Option<&str>, limit: usize) -> Result<Vec<MemoryCard>> {
    let mut stmt = if let Some(mtype) = memory_type {
        conn.prepare(
            "
            SELECT
                id, content, memory_type, COALESCE(layer, 'working') as layer,
                COALESCE(parent_id, '') as parent_id, COALESCE(hierarchy_level, 'document') as hierarchy_level,
                COALESCE(order_index, 0) as order_index, COALESCE(path, '') as path, file_source,
                COALESCE(access_count, 0) as access_count, last_accessed, confidence, importance,
                created_at, updated_at
            FROM memories
            WHERE memory_type = ?1
            ORDER BY importance DESC, created_at DESC
            LIMIT ?2
            ",
        )?
    } else {
        conn.prepare(
            "
            SELECT
                id, content, memory_type, COALESCE(layer, 'working') as layer,
                COALESCE(parent_id, '') as parent_id, COALESCE(hierarchy_level, 'document') as hierarchy_level,
                COALESCE(order_index, 0) as order_index, COALESCE(path, '') as path, file_source,
                COALESCE(access_count, 0) as access_count, last_accessed, confidence, importance,
                created_at, updated_at
            FROM memories
            ORDER BY importance DESC, created_at DESC
            LIMIT ?1
            ",
        )?
    };

    let mut rows = if let Some(mtype) = memory_type {
        stmt.query(params![mtype, limit as i64])?
    } else {
        stmt.query(params![limit as i64])?
    };

    let mut memories = Vec::new();
    while let Some(row) = rows.next()? {
        memories.push(map_row_to_memory_card(row)?);
    }

    Ok(memories)
}

/// List memories by layer
pub fn list_memories_by_layer(conn: &Connection, layer: &str, limit: usize) -> Result<Vec<MemoryCard>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            id, content, memory_type, COALESCE(layer, 'working') as layer,
            COALESCE(parent_id, '') as parent_id, COALESCE(hierarchy_level, 'document') as hierarchy_level,
            COALESCE(order_index, 0) as order_index, COALESCE(path, '') as path, file_source,
            COALESCE(access_count, 0) as access_count, last_accessed, confidence, importance,
            created_at, updated_at
        FROM memories
        WHERE layer = ?1
        ORDER BY importance DESC, created_at DESC
        LIMIT ?2
        ",
    )?;

    let mut rows = stmt.query(params![layer, limit as i64])?;
    let mut memories = Vec::new();
    
    while let Some(row) = rows.next()? {
        memories.push(map_row_to_memory_card(row)?);
    }

    Ok(memories)
}
