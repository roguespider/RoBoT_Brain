//! SQLite storage — per Architecture §14 (Memory Hierarchy).
//! Provides SQLite-based persistent storage for memory records.

use rusqlite::Connection;
use serde_json;

/// SQLite-backed storage for memory records.
pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    /// Create a new SQLite store connected to the database at `path`.
    pub fn new(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("Failed to open database: {e}"))?;
        Ok(Self { conn })
    }

    /// Store a JSON-serializable record under the given key.
    pub fn store(&self, key: &str, value: &serde_json::Value) -> Result<(), String> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS records (key TEXT PRIMARY KEY, value TEXT)",
                [],
            )
            .map_err(|e| format!("Failed to create table: {e}"))?;
        let json_value =
            serde_json::to_string(value).map_err(|e| format!("Failed to serialize value: {e}"))?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO records (key, value) VALUES (?1, ?2)",
                [key, &json_value],
            )
            .map_err(|e| format!("Failed to store record: {e}"))?;
        Ok(())
    }

    /// Retrieve a record by key.
    pub fn load(&self, key: &str) -> Result<Option<serde_json::Value>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM records WHERE key = ?1")
            .map_err(|e| format!("Failed to prepare query: {e}"))?;
        let mut rows = stmt
            .query_map([key], |row| row.get(0))
            .map_err(|e| format!("Failed to query: {e}"))?;
        let row: Option<String> = rows
            .next()
            .transpose()
            .map_err(|e| format!("Failed to read row: {e}"))?;
        Ok(row.map(|v| serde_json::from_str(&v).unwrap_or(serde_json::Value::Null)))
    }

    /// Delete a record by key.
    pub fn delete(&self, key: &str) -> Result<bool, String> {
        let changes = self
            .conn
            .execute("DELETE FROM records WHERE key = ?1", [key])
            .map_err(|e| format!("Failed to delete record: {e}"))?;
        Ok(changes > 0)
    }
}
