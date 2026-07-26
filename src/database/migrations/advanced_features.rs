// src/database/migrations/advanced_features.rs
// Migrations 007-009: Lineage, hypothesis engine, and memory graph

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    migration_007_add_lineage(conn)?;
    migration_008_add_hypothesis_engine(conn)?;
    migration_009_add_memory_graph(conn)?;
    Ok(())
}

fn migration_007_add_lineage(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS memory_lineage (
            id TEXT PRIMARY KEY,
            memory_id TEXT NOT NULL UNIQUE,
            superseded_by TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_lineage_memory ON memory_lineage(memory_id);
        CREATE INDEX IF NOT EXISTS idx_lineage_superseded ON memory_lineage(superseded_by);
        CREATE TABLE IF NOT EXISTS lineage_evidence (
            id TEXT PRIMARY KEY,
            lineage_id TEXT NOT NULL,
            evidence_type TEXT NOT NULL,
            confidence REAL DEFAULT 0.5,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_evidence_lineage ON lineage_evidence(lineage_id);
        CREATE TABLE IF NOT EXISTS lineage_observations (
            id TEXT PRIMARY KEY,
            lineage_id TEXT NOT NULL,
            observation_type TEXT NOT NULL,
            outcome TEXT NOT NULL,
            timestamp TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_observation_lineage ON lineage_observations(lineage_id);
        CREATE TABLE IF NOT EXISTS lineage_refinements (
            id TEXT PRIMARY KEY,
            lineage_id TEXT NOT NULL,
            previous_content TEXT NOT NULL,
            new_content TEXT NOT NULL,
            refinement_type TEXT NOT NULL,
            reason TEXT NOT NULL,
            confidence_change REAL DEFAULT 0.0,
            timestamp TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_refinement_lineage ON lineage_refinements(lineage_id);
        CREATE TABLE IF NOT EXISTS lineage_contradictions (
            id TEXT PRIMARY KEY,
            lineage_id TEXT NOT NULL,
            contradicting_memory_id TEXT NOT NULL,
            description TEXT NOT NULL,
            strength REAL DEFAULT 0.5,
            resolved INTEGER DEFAULT 0,
            resolution_type TEXT,
            resolution_data TEXT,
            timestamp TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_contradiction_lineage ON lineage_contradictions(lineage_id);
        CREATE TABLE IF NOT EXISTS lineage_confirmations (
            id TEXT PRIMARY KEY,
            lineage_id TEXT NOT NULL,
            source TEXT NOT NULL,
            source_type TEXT NOT NULL,
            description TEXT NOT NULL,
            confidence_boost REAL DEFAULT 0.1,
            timestamp TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_confirmation_lineage ON lineage_confirmations(lineage_id);
        ",
    )?;
    Ok(())
}

fn migration_008_add_hypothesis_engine(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS hypotheses (
            id TEXT PRIMARY KEY,
            statement TEXT NOT NULL,
            domain TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'testing',
            confidence REAL DEFAULT 0.5,
            supporting_count INTEGER DEFAULT 0,
            contradicting_count INTEGER DEFAULT 0,
            source_observations TEXT NOT NULL,
            related_memories TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_hypothesis_domain ON hypotheses(domain);
        CREATE INDEX IF NOT EXISTS idx_hypothesis_status ON hypotheses(status);
        CREATE TABLE IF NOT EXISTS observations (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            context TEXT NOT NULL,
            observation_type TEXT NOT NULL,
            related_experiences TEXT NOT NULL,
            triggered_hypothesis TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_observation_type ON observations(observation_type);
        CREATE TABLE IF NOT EXISTS evidence (
            id TEXT PRIMARY KEY,
            hypothesis_id TEXT NOT NULL,
            content TEXT NOT NULL,
            evidence_type TEXT NOT NULL,
            direction TEXT NOT NULL,
            strength REAL DEFAULT 0.5,
            experience_id TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_evidence_hypothesis ON evidence(hypothesis_id);
        CREATE TABLE IF NOT EXISTS learned_knowledge (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            source_hypothesis TEXT,
            confidence REAL DEFAULT 0.5,
            domain TEXT NOT NULL,
            derivation TEXT NOT NULL,
            active INTEGER DEFAULT 1
        );
        CREATE INDEX IF NOT EXISTS idx_knowledge_domain ON learned_knowledge(domain);
        ",
    )?;
    Ok(())
}

fn migration_009_add_memory_graph(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS memory_tags (
            id TEXT PRIMARY KEY,
            memory_id TEXT NOT NULL,
            tag TEXT NOT NULL,
            UNIQUE(memory_id, tag)
        );
        CREATE INDEX IF NOT EXISTS idx_tag_memory ON memory_tags(memory_id);
        CREATE INDEX IF NOT EXISTS idx_tag ON memory_tags(tag);
        CREATE TABLE IF NOT EXISTS memory_relationships (
            id TEXT PRIMARY KEY,
            memory_id TEXT NOT NULL,
            related_id TEXT NOT NULL,
            relationship_type TEXT DEFAULT 'related',
            UNIQUE(memory_id, related_id)
        );
        CREATE INDEX IF NOT EXISTS idx_rel_memory ON memory_relationships(memory_id);
        CREATE TABLE IF NOT EXISTS memory_embeddings (
            id TEXT PRIMARY KEY,
            memory_id TEXT NOT NULL UNIQUE,
            embedding BLOB NOT NULL,
            model TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_embed_memory ON memory_embeddings(memory_id);
        ",
    )?;
    Ok(())
}
