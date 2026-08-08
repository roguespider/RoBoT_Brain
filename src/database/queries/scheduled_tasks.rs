// src/database/queries/scheduled_tasks.rs
//! Scheduled task database operations

use anyhow::Result;
use rusqlite::{Connection, params};

use crate::experience::scheduler::{ScheduledTask, TaskSchedule, TaskStatus, TaskType};

use super::helpers::parse_time;

/// Insert or replace a scheduled task
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

/// Get a scheduled task by ID
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

/// List all scheduled tasks
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
