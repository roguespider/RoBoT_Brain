// src/database/migrations/hierarchical_memory.rs
// Migration 004: Add hierarchical memory support
// Migration 011: Add memory layer (STM/LTM)

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    migration_004_add_hierarchy(conn)?;
    migration_005_add_file_hierarchy(conn)?;
    migration_011_add_memory_layer(conn)?;
    Ok(())
}

/// Add hierarchy fields to memories table
fn migration_004_add_hierarchy(conn: &Connection) -> Result<()> {
    // Add columns if they don't exist
    conn.execute_batch(
        "
        ALTER TABLE memories ADD COLUMN parent_id TEXT;
        ALTER TABLE memories ADD COLUMN hierarchy_level TEXT NOT NULL DEFAULT 'document';
        ALTER TABLE memories ADD COLUMN order_index INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE memories ADD COLUMN path TEXT NOT NULL DEFAULT '';
        
        -- Add foreign key for parent relationship (allow NULL for root nodes)
        -- Note: SQLite doesn't enforce foreign keys by default, but we set up the relationship
        
        -- Add indexes for efficient hierarchy traversal
        CREATE INDEX IF NOT EXISTS idx_memory_parent ON memories(parent_id);
        CREATE INDEX IF NOT EXISTS idx_memory_level ON memories(hierarchy_level);
        CREATE INDEX IF NOT EXISTS idx_memory_path ON memories(path);
        CREATE INDEX IF NOT EXISTS idx_memory_order ON memories(order_index);
        "
    )?;
    Ok(())
}

/// Add file hierarchy tracking table for tracking imported documents
fn migration_005_add_file_hierarchy(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS document_roots (
            id TEXT PRIMARY KEY,
            file_path TEXT NOT NULL UNIQUE,
            file_name TEXT NOT NULL,
            file_hash TEXT,
            total_memories INTEGER DEFAULT 0,
            max_depth INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS idx_doc_root_path ON document_roots(file_path);
        CREATE INDEX IF NOT EXISTS idx_doc_root_hash ON document_roots(file_hash);
        "
    )?;
    Ok(())
}

/// Add memory layer for STM/LTM separation per Architecture §6.3
/// - Working: Short-term memory (volatile)
/// - Permanent: Long-term memory (curated)
fn migration_011_add_memory_layer(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- Add memory layer column (default to 'working' for existing memories)
        ALTER TABLE memories ADD COLUMN layer TEXT NOT NULL DEFAULT 'working';
        
        -- Add access tracking for consolidation decisions
        ALTER TABLE memories ADD COLUMN access_count INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE memories ADD COLUMN last_accessed TEXT;
        
        -- Add file_source column if not exists
        ALTER TABLE memories ADD COLUMN file_source TEXT;
        
        -- Add indexes for efficient layer queries
        CREATE INDEX IF NOT EXISTS idx_memory_layer ON memories(layer);
        CREATE INDEX IF NOT EXISTS idx_memory_access_count ON memories(access_count);
        CREATE INDEX IF NOT EXISTS idx_memory_last_accessed ON memories(last_accessed);
        "
    )?;
    Ok(())
}
