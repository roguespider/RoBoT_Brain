// src/database/migrations/cooboploop.rs
// Migration 013: Continuous Objective & Opportunity Loop hardware snapshots.

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS hardware_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_at TEXT DEFAULT (datetime('now')),
            cpu_model TEXT,
            cpu_cores INTEGER,
            memory_total_mb INTEGER,
            memory_available_mb INTEGER,
            storage_total_gb INTEGER,
            storage_available_gb REAL,
            gpu_model TEXT,
            network_interfaces TEXT,
            thermal_state TEXT,
            other TEXT
        );

        CREATE TABLE IF NOT EXISTS strategic_objectives (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT,
            status TEXT DEFAULT 'active',
            priority REAL DEFAULT 0.0,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS inspection_issues (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            target TEXT NOT NULL,
            severity TEXT,
            description TEXT,
            recommended_action TEXT,
            detected_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS opportunity_intake_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            opportunity_id TEXT NOT NULL,
            source_url TEXT,
            source_type TEXT,
            decision TEXT,
            reason TEXT,
            value_ratio REAL,
            risk_score REAL,
            processed_at TEXT DEFAULT (datetime('now'))
        );

        -- §4 / T-COO-15: full objective queue persistence with execution history and completion state
        CREATE TABLE IF NOT EXISTS objective_queue (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT DEFAULT 'Discovered',
            priority REAL DEFAULT 0.5,
            source TEXT,
            expected_value REAL DEFAULT 0.5,
            risk REAL DEFAULT 0.3,
            learning_value REAL DEFAULT 0.5,
            required_capabilities TEXT DEFAULT '[]',
            dependencies TEXT DEFAULT '[]',
            deadline TEXT,
            execution_history TEXT DEFAULT '[]',
            completion_state TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            last_evaluated TEXT DEFAULT (datetime('now'))
        );

        -- §8 / T-COO-15: post-task evaluation records
        CREATE TABLE IF NOT EXISTS post_task_evaluations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            goal_id TEXT,
            did_succeed INTEGER DEFAULT 0,
            verification_confirmed INTEGER DEFAULT 0,
            unexpected_problems TEXT DEFAULT '[]',
            knowledge_gaps TEXT DEFAULT '[]',
            new_bugs TEXT DEFAULT '[]',
            capability_limitation TEXT,
            created_work TEXT DEFAULT '[]',
            efficiency_score REAL DEFAULT 0.0,
            future_planning_adjustment TEXT,
            improvement_opportunity TEXT,
            evaluated_at TEXT DEFAULT (datetime('now'))
        );

        -- §15 / T-COO-15: experience records
        CREATE TABLE IF NOT EXISTS experience_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            objective TEXT NOT NULL,
            experience_type TEXT,
            outcome TEXT,
            confidence REAL DEFAULT 0.5,
            actions TEXT DEFAULT '[]',
            results TEXT DEFAULT '[]',
            final_outcome TEXT,
            lessons_learned TEXT DEFAULT '[]',
            recorded_at TEXT DEFAULT (datetime('now'))
        );

        -- §15 / T-COO-15: learning updates
        CREATE TABLE IF NOT EXISTS learning_updates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            topic TEXT,
            success INTEGER DEFAULT 1,
            confidence REAL DEFAULT 0.5,
            capability_updates TEXT DEFAULT '[]',
            knowledge_additions TEXT DEFAULT '[]',
            strategy_refinements TEXT DEFAULT '[]',
            risk_adjustments TEXT DEFAULT '[]',
            timestamp TEXT DEFAULT (datetime('now'))
        );
    ",
    )?;
    Ok(())
}
