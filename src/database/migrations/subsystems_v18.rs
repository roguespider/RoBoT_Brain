// src/database/migrations/subsystems_v18.rs
// Migration 018: Add missing v0.0.2 Ch 21-22 tables.
//
// Per Architecture Chapter 21-22, 15 tables SHALL exist and be actively used:
// memory (memories), experience, knowledge_graph, workflow (workflow_schema),
// skill, lesson, learning, confidence_history, conversation, planning,
// execution, ai_model, tool, architecture_trace, diagnostics.
//
// This migration adds tables that were missing from prior migrations:
// - experience (full lifecycle table)
// - knowledge_graph (graph metadata table)
// - skill (skill registry)
// - lesson (lessons learned)
// - learning (learning records)
// - conversation (conversation records)
// - planning (planning records)
// - execution (execution records)
// - ai_model (AI model registry)
// - tool (tool registry)
// - architecture_trace (architecture trace records)
// - diagnostics (diagnostics records)

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    migration_018_add_experience(conn)?;
    migration_018_add_knowledge_graph(conn)?;
    migration_018_add_skill(conn)?;
    migration_018_add_lesson(conn)?;
    migration_018_add_learning(conn)?;
    migration_018_add_conversation(conn)?;
    migration_018_add_planning(conn)?;
    migration_018_add_execution(conn)?;
    migration_018_add_ai_model(conn)?;
    migration_018_add_tool(conn)?;
    migration_018_add_architecture_trace(conn)?;
    migration_018_add_diagnostics(conn)?;
    Ok(())
}

/// Table: experience — full lifecycle experience storage.
fn migration_018_add_experience(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS experience (
            id TEXT PRIMARY KEY,
            goal TEXT NOT NULL,
            plan_id TEXT,
            result TEXT NOT NULL,
            success INTEGER NOT NULL,
            execution_time INTEGER,
            cost REAL DEFAULT 0.0,
            confidence_change REAL DEFAULT 0.0,
            tool_usage TEXT,
            lessons TEXT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_experience_goal ON experience(goal);
        CREATE INDEX IF NOT EXISTS idx_experience_success ON experience(success);
        ",
    )?;
    Ok(())
}

/// Table: knowledge_graph — graph metadata and versioning.
fn migration_018_add_knowledge_graph(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS knowledge_graph (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT 'concept',
            node_count INTEGER DEFAULT 0,
            edge_count INTEGER DEFAULT 0,
            version INTEGER DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_graph_kind ON knowledge_graph(kind);
        ",
    )?;
    Ok(())
}

/// Table: skill — skill registry per Architecture Ch 13.
fn migration_018_add_skill(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS skill (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            category TEXT NOT NULL,
            description TEXT NOT NULL,
            version TEXT NOT NULL DEFAULT '1.0.0',
            enabled INTEGER NOT NULL DEFAULT 1,
            mastery REAL NOT NULL DEFAULT 0.0,
            last_used TEXT,
            success_count INTEGER NOT NULL DEFAULT 0,
            failure_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_skill_category ON skill(category);
        CREATE INDEX IF NOT EXISTS idx_skill_mastery ON skill(mastery);
        ",
    )?;
    Ok(())
}

/// Table: lesson — lessons learned per Architecture Ch 9.
fn migration_018_add_lesson(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS lesson (
            id TEXT PRIMARY KEY,
            topic TEXT NOT NULL,
            content TEXT NOT NULL,
            source_experience TEXT,
            confidence REAL NOT NULL DEFAULT 0.5,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_lesson_topic ON lesson(topic);
        CREATE INDEX IF NOT EXISTS idx_lesson_confidence ON lesson(confidence);
        ",
    )?;
    Ok(())
}

/// Table: learning — learning records per Architecture Ch 10.
fn migration_018_add_learning(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS learning (
            id TEXT PRIMARY KEY,
            topic TEXT NOT NULL,
            strategy TEXT NOT NULL,
            outcome TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 0.5,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_learning_topic ON learning(topic);
        CREATE INDEX IF NOT EXISTS idx_learning_outcome ON learning(outcome);
        ",
    )?;
    Ok(())
}

/// Table: conversation — conversation records per Architecture Ch 6.
fn migration_018_add_conversation(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS conversation (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            user_message TEXT NOT NULL,
            assistant_message TEXT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_conversation_session ON conversation(session_id);
        CREATE INDEX IF NOT EXISTS idx_conversation_timestamp ON conversation(timestamp);
        ",
    )?;
    Ok(())
}

/// Table: planning — planning records per Architecture Ch 11.
fn migration_018_add_planning(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS planning (
            id TEXT PRIMARY KEY,
            plan_id TEXT NOT NULL,
            goal TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            confidence REAL NOT NULL DEFAULT 0.5,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_planning_plan ON planning(plan_id);
        CREATE INDEX IF NOT EXISTS idx_planning_status ON planning(status);
        ",
    )?;
    Ok(())
}

/// Table: execution — execution records per Architecture Ch 12.
fn migration_018_add_execution(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS execution (
            id TEXT PRIMARY KEY,
            plan_id TEXT NOT NULL,
            action TEXT NOT NULL,
            result TEXT,
            success INTEGER NOT NULL DEFAULT 0,
            execution_time INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_execution_plan ON execution(plan_id);
        CREATE INDEX IF NOT EXISTS idx_execution_success ON execution(success);
        ",
    )?;
    Ok(())
}

/// Table: ai_model — AI model registry per Architecture Ch 21.
fn migration_018_add_ai_model(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS ai_model (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            provider TEXT NOT NULL,
            model_type TEXT NOT NULL,
            version TEXT NOT NULL DEFAULT '1.0.0',
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_ai_model_provider ON ai_model(provider);
        CREATE INDEX IF NOT EXISTS idx_ai_model_default ON ai_model(is_default);
        ",
    )?;
    Ok(())
}

/// Table: tool — tool registry per Architecture Ch 13.
fn migration_018_add_tool(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS tool (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            category TEXT NOT NULL,
            description TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            version TEXT NOT NULL DEFAULT '1.0.0',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_tool_category ON tool(category);
        CREATE INDEX IF NOT EXISTS idx_tool_enabled ON tool(enabled);
        ",
    )?;
    Ok(())
}

/// Table: architecture_trace — architecture trace records per Architecture Ch 24.
fn migration_018_add_architecture_trace(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS architecture_trace (
            id TEXT PRIMARY KEY,
            function_name TEXT NOT NULL,
            callers TEXT,
            dependencies TEXT,
            data_flow TEXT,
            trace_id TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_trace_function ON architecture_trace(function_name);
        CREATE INDEX IF NOT EXISTS idx_trace_trace_id ON architecture_trace(trace_id);
        ",
    )?;
    Ok(())
}

/// Table: diagnostics — diagnostics records per Architecture Ch 27.
fn migration_018_add_diagnostics(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS diagnostics (
            id TEXT PRIMARY KEY,
            severity TEXT NOT NULL,
            category TEXT NOT NULL,
            message TEXT NOT NULL,
            file TEXT,
            line INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_diag_severity ON diagnostics(severity);
        CREATE INDEX IF NOT EXISTS idx_diag_category ON diagnostics(category);
        CREATE INDEX IF NOT EXISTS idx_diag_created ON diagnostics(created_at);
        ",
    )?;
    Ok(())
}
