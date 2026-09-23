//! Learning patterns - Per Architecture §10.2 "Pattern recognition"
//!
//! Placeholder module for future pattern recognition features.

use serde::{Deserialize, Serialize};

/// Group experiences by their context signature.
pub fn group_by_context_signature(
    experiences: &[crate::data_contracts::experience_record::ExperienceRecord],
) -> std::collections::HashMap<String, Vec<String>> {
    let mut groups: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for exp in experiences {
        let sig = exp.context_signature.clone();
        groups.entry(sig).or_default().push(exp.outcome.clone());
    }
    groups
}

/// Insert a pattern into the database.
pub fn insert_pattern(conn: &rusqlite::Connection, p: &Pattern) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO learning_patterns (id, signature, frequency, success_rate) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![p.id, p.context_signature, p.frequency as i32, p.success_rate],
    )?;
    Ok(())
}

/// Get patterns with a minimum success rate.
pub fn get_patterns(
    conn: &rusqlite::Connection,
    min_success_rate: f32,
) -> Result<Vec<Pattern>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, signature, frequency, success_rate FROM learning_patterns WHERE success_rate >= ?1")?;
    let rows = stmt.query_map(rusqlite::params![min_success_rate], |row| {
        Ok(Pattern {
            id: row.get(0)?,
            context_signature: row.get(1)?,
            frequency: row.get(2)?,
            success_rate: row.get(3)?,
            actions: Vec::new(),
        })
    })?;
    let mut patterns = Vec::new();
    for row in rows {
        patterns.push(row?);
    }
    Ok(patterns)
}

/// Detect patterns from grouped experiences.
pub fn detect_patterns(
    experiences: &[crate::data_contracts::experience_record::ExperienceRecord],
    min_frequency: u32,
) -> Vec<Pattern> {
    let groups = group_by_context_signature(experiences);
    let mut patterns = Vec::new();
    for (sig, actions) in groups {
        if actions.len() as u32 >= min_frequency {
            patterns.push(Pattern {
                id: format!("pattern-{}", sig),
                frequency: actions.len() as u32,
                success_rate: 0.75,
                context_signature: sig,
                actions,
            });
        }
    }
    patterns
}

/// A discovered learning pattern.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pattern {
    /// Pattern identifier.
    pub id: String,
    /// How frequently this pattern occurs.
    pub frequency: u32,
    /// Success rate of this pattern.
    pub success_rate: f32,
    /// Signature describing the context.
    pub context_signature: String,
    /// Actions associated with this pattern.
    pub actions: Vec<String>,
}
