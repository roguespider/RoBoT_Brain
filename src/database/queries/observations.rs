// src/database/queries/observations.rs
//! Observation database operations (Per Architecture §07)

use anyhow::Result;
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::database::models::Observation;

use super::helpers::parse_time;

/// Insert or replace an observation
/// Per Architecture §07: Every experience originates from observations
pub fn insert_observation(conn: &Connection, observation: &Observation) -> Result<()> {
    conn.execute(
        "
        INSERT OR REPLACE INTO observations
        (
            id,
            content,
            context,
            observation_type,
            related_experiences,
            triggered_hypothesis,
            created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ",
        params![
            observation.id.to_string(),
            observation.content,
            observation.context,
            observation.observation_type,
            serde_json::to_string(&observation.related_experiences)?,
            observation.triggered_hypothesis.map(|u| u.to_string()),
            observation.created_at.to_rfc3339()
        ],
    )?;
    Ok(())
}

/// Get an observation by ID
#[cfg(test)]
pub fn get_observation(conn: &Connection, id: Uuid) -> Result<Option<Observation>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, context, observation_type, related_experiences, triggered_hypothesis, created_at
         FROM observations WHERE id = ?1"
    )?;

    let result = stmt.query_row([id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let related_json: String = row.get(4)?;
        let triggered_str: Option<String> = row.get(5)?;
        Ok(Observation {
            id: Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            content: row.get(1)?,
            context: row.get(2)?,
            observation_type: row.get(3)?,
            related_experiences: serde_json::from_str(&related_json).unwrap_or_default(),
            triggered_hypothesis: triggered_str.and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: parse_time(&row.get::<_, String>(6)?),
        })
    });

    match result {
        Ok(obs) => Ok(Some(obs)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// List recent observations
pub fn list_observations(conn: &Connection, limit: usize) -> Result<Vec<Observation>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, context, observation_type, related_experiences, triggered_hypothesis, created_at
         FROM observations ORDER BY created_at DESC LIMIT ?1"
    )?;
    
    let rows = stmt.query_map([limit as i64], |row| {
        let id_str: String = row.get(0)?;
        let related_json: String = row.get(4)?;
        let triggered_str: Option<String> = row.get(5)?;
        Ok(Observation {
            id: Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            content: row.get(1)?,
            context: row.get(2)?,
            observation_type: row.get(3)?,
            related_experiences: serde_json::from_str(&related_json).unwrap_or_default(),
            triggered_hypothesis: triggered_str.and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: parse_time(&row.get::<_, String>(6)?),
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Link an observation to an experience
#[cfg(test)]
pub fn link_observation_to_experience(conn: &Connection, observation_id: Uuid, experience_id: Uuid) -> Result<()> {
    if let Some(mut obs) = get_observation(conn, observation_id)? {
        obs.related_experiences.push(experience_id);
        insert_observation(conn, &obs)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::Observation;

    #[test]
    fn test_link_observation_to_experience() {
        let conn = Connection::open_in_memory()
            .or_else(|_| Connection::open(":memory:"))
            .expect("open in-memory db");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS observations (
                id TEXT PRIMARY KEY,
                content TEXT,
                context TEXT,
                observation_type TEXT,
                related_experiences TEXT,
                triggered_hypothesis TEXT,
                created_at TEXT
            )",
            [],
        )
        .expect("create table");

        let obs = Observation::new(
            "test observation".to_string(),
            "test context".to_string(),
            "pattern".to_string(),
        );
        let obs_id = obs.id;
        let exp_id = Uuid::new_v4();
        assert!(insert_observation(&conn, &obs).is_ok());
        assert!(link_observation_to_experience(&conn, obs_id, exp_id).is_ok());
        let fetched = get_observation(&conn, obs_id).expect("get observation");
        assert!(fetched.is_some());
        assert!(fetched
            .expect("observation exists")
            .related_experiences
            .contains(&exp_id));
    }
}
