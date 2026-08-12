// src/database/queries/embeddings.rs
//! Embedding database operations

use anyhow::Result;
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::database::models::MemoryEmbedding;

use super::helpers::{bytes_to_embedding, embedding_to_bytes};

/// Insert or replace a memory embedding
pub fn insert_embedding(conn: &Connection, embedding: &MemoryEmbedding) -> Result<()> {
    let bytes = embedding_to_bytes(&embedding.embedding);
    conn.execute(
        "INSERT OR REPLACE INTO memory_embeddings
         (id, memory_id, embedding, model)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            embedding.id.to_string(),
            embedding.memory_id.to_string(),
            bytes,
            embedding.model,
        ],
    )?;
    Ok(())
}

    let mut stmt = conn.prepare(
        "SELECT id, memory_id, embedding, model FROM memory_embeddings WHERE memory_id = ?1"
    )?;

    let result = stmt.query_row([memory_id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let memory_id_str: String = row.get(1)?;
        let bytes: Vec<u8> = row.get(2)?;
        let model: String = row.get(3)?;
        
        Ok(MemoryEmbedding {
            id: Uuid::parse_str(&id_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            memory_id: Uuid::parse_str(&memory_id_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            embedding: bytes_to_embedding(&bytes),
            model,
        })
    });

    match result {
        Ok(embedding) => Ok(Some(embedding)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// List all embeddings with their memory IDs
pub fn list_embeddings(conn: &Connection, limit: usize) -> Result<Vec<MemoryEmbedding>> {
    let mut stmt = conn.prepare(
        "SELECT id, memory_id, embedding, model FROM memory_embeddings LIMIT ?1"
    )?;

    let rows = stmt.query_map([limit as i64], |row| {
        let id_str: String = row.get(0)?;
        let memory_id_str: String = row.get(1)?;
        let bytes: Vec<u8> = row.get(2)?;
        let model: String = row.get(3)?;
        
        Ok(MemoryEmbedding {
            id: Uuid::parse_str(&id_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            memory_id: Uuid::parse_str(&memory_id_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            embedding: bytes_to_embedding(&bytes),
            model,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Delete embeddings by memory ID
pub fn delete_embedding_by_memory_id(conn: &Connection, memory_id: Uuid) -> Result<bool> {
    let deleted = conn.execute(
        "DELETE FROM memory_embeddings WHERE memory_id = ?1",
        [memory_id.to_string()],
    )?;
    Ok(deleted > 0)
}

/// Get embedding count
pub fn count_embeddings(conn: &Connection) -> Result<usize> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_embeddings",
        [],
        |row| row.get(0),
    )?;
    Ok(count as usize)
}
