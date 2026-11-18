//! Database repositories — per Architecture §21 (Storage Architecture).
//!
//! Provides repository interfaces for memory, experience, knowledge,
//! workflow, skill, and execution records.

/// Repository trait for database entities.
pub trait Repository<T> {
    fn find_by_id(&self, id: &str) -> Option<T>;
    fn save(&self, entity: T) -> Result<(), crate::database::sqlite::SqliteError>;
    fn delete(&self, id: &str) -> Result<bool, crate::database::sqlite::SqliteError>;
}
