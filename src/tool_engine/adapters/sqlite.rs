//! SQLite adapter — per Architecture §13 (Tool Engine).
//! Provides SQLite database operations using the rusqlite crate.

use rusqlite::Connection;
use serde_json;

/// SQLite adapter for database operations.
pub struct SqliteAdapter;

impl SqliteAdapter {
    /// Create a new SQLite adapter.
    pub fn new() -> Self {
        Self
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "sqlite",
            "adapted": true
        })
    }

    /// Open a SQLite database at `path`.
    pub fn open(&self, path: &str) -> Result<Connection, String> {
        Connection::open(path).map_err(|e| format!("Failed to open database {path}: {e}"))
    }

    /// Execute a SQL statement on the given connection.
    pub fn execute(&self, conn: &Connection, sql: &str) -> Result<usize, String> {
        conn.execute(sql, [])
            .map_err(|e| format!("Failed to execute SQL: {e}"))
    }

    /// Query a SQLite database and return results as JSON values.
    pub fn query_json(&self, path: &str, sql: &str) -> Result<Vec<serde_json::Value>, String> {
        let conn = self.open(path)?;
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| format!("Failed to prepare SQL: {e}"))?;
        let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        let rows = stmt
            .query_map([], |row| {
                let mut values = serde_json::Map::new();
                for (i, col) in columns.iter().enumerate() {
                    let value: Option<String> = row.get(i).unwrap_or(None);
                    values.insert(
                        col.clone(),
                        serde_json::Value::String(value.unwrap_or_default()),
                    );
                }
                Ok(serde_json::Value::Object(values))
            })
            .map_err(|e| format!("Failed to query: {e}"))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Failed to read row: {e}"))?);
        }
        Ok(results)
    }
}

impl Default for SqliteAdapter {
    fn default() -> Self {
        Self::new()
    }
}
