// src/database/queries/helpers.rs
//! Helper functions for parsing database values

use chrono::{DateTime, Utc};
use rusqlite::Row;
use uuid::Uuid;

use crate::database::models::{HierarchyLevel, MemoryCard, MemoryLayer, MemoryType};

/// Parse hierarchy level string to enum
pub fn parse_hierarchy_level(s: &str) -> HierarchyLevel {
    match s {
        "section" => HierarchyLevel::Section,
        "subsection" => HierarchyLevel::Subsection,
        "paragraph" => HierarchyLevel::Paragraph,
        "sentence" => HierarchyLevel::Sentence,
        _ => HierarchyLevel::Document,
    }
}

/// Parse memory type string to enum
pub fn parse_memory_type(value: &str) -> MemoryType {
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

/// Parse memory layer string to enum
pub fn parse_memory_layer(value: &str) -> MemoryLayer {
    match value {
        "permanent" => MemoryLayer::Permanent,
        _ => MemoryLayer::Working,
    }
}

/// Parse RFC3339 time string to DateTime<Utc>
pub fn parse_time(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// Convert bytes to f32 vector (for embeddings)
pub fn bytes_to_embedding(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

/// Convert f32 vector to bytes (for embeddings)
pub fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// Map a database row to MemoryCard
pub fn map_row_to_memory_card(row: &Row) -> rusqlite::Result<MemoryCard> {
    let uuid_str: String = row.get(0)?;
    let parent_id_str: String = row.get(4)?;
    let last_accessed_str: Option<String> = row.get(10)?;
    
    Ok(MemoryCard {
        id: Uuid::parse_str(&uuid_str)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
        content: row.get(1)?,
        memory_type: parse_memory_type(&row.get::<_, String>(2)?),
        layer: parse_memory_layer(&row.get::<_, String>(3)?),
        parent_id: if parent_id_str.is_empty() { 
            None 
        } else { 
            Uuid::parse_str(&parent_id_str).ok() 
        },
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
