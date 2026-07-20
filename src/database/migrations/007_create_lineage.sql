-- Migration 007: Memory Lineage
-- Tracks the full history and evolution of memories

CREATE TABLE IF NOT EXISTS memory_lineage (
    id TEXT PRIMARY KEY,
    memory_id TEXT NOT NULL UNIQUE,
    superseded_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_lineage_memory ON memory_lineage(memory_id);
CREATE INDEX IF NOT EXISTS idx_lineage_superseded ON memory_lineage(superseded_by);

-- Supporting evidence for memories
CREATE TABLE IF NOT EXISTS lineage_evidence (
    id TEXT PRIMARY KEY,
    lineage_id TEXT NOT NULL,
    evidence_type TEXT NOT NULL,
    confidence REAL DEFAULT 0.5,
    created_at TEXT NOT NULL,
    FOREIGN KEY (lineage_id) REFERENCES memory_lineage(id)
);

CREATE INDEX IF NOT EXISTS idx_evidence_lineage ON lineage_evidence(lineage_id);

-- Observations that support memories
CREATE TABLE IF NOT EXISTS lineage_observations (
    id TEXT PRIMARY KEY,
    lineage_id TEXT NOT NULL,
    observation_type TEXT NOT NULL,
    outcome TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (lineage_id) REFERENCES memory_lineage(id)
);

CREATE INDEX IF NOT EXISTS idx_observation_lineage ON lineage_observations(lineage_id);

-- Refinements to memory content
CREATE TABLE IF NOT EXISTS lineage_refinements (
    id TEXT PRIMARY KEY,
    lineage_id TEXT NOT NULL,
    previous_content TEXT NOT NULL,
    new_content TEXT NOT NULL,
    refinement_type TEXT NOT NULL,
    reason TEXT NOT NULL,
    confidence_change REAL DEFAULT 0.0,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (lineage_id) REFERENCES memory_lineage(id)
);

CREATE INDEX IF NOT EXISTS idx_refinement_lineage ON lineage_refinements(lineage_id);

-- Contradictions challenging memories
CREATE TABLE IF NOT EXISTS lineage_contradictions (
    id TEXT PRIMARY KEY,
    lineage_id TEXT NOT NULL,
    contradicting_memory_id TEXT NOT NULL,
    description TEXT NOT NULL,
    strength REAL DEFAULT 0.5,
    resolved INTEGER DEFAULT 0,
    resolution_type TEXT,
    resolution_data TEXT,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (lineage_id) REFERENCES memory_lineage(id)
);

CREATE INDEX IF NOT EXISTS idx_contradiction_lineage ON lineage_contradictions(lineage_id);
CREATE INDEX IF NOT EXISTS idx_contradiction_unresolved ON lineage_contradictions(lineage_id, resolved);

-- Confirmations from external sources
CREATE TABLE IF NOT EXISTS lineage_confirmations (
    id TEXT PRIMARY KEY,
    lineage_id TEXT NOT NULL,
    source TEXT NOT NULL,
    source_type TEXT NOT NULL,
    description TEXT NOT NULL,
    confidence_boost REAL DEFAULT 0.1,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (lineage_id) REFERENCES memory_lineage(id)
);

CREATE INDEX IF NOT EXISTS idx_confirmation_lineage ON lineage_confirmations(lineage_id);
