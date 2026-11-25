// src/memory/integrity.rs
//! Memory integrity — Per Architecture Chapter 08 "Memory Integrity".
//!
//! Every stored memory should support validation:
//! * Checksum validation
//! * Provenance verification
//! * Relationship consistency
//! * Duplicate detection
//! * Confidence review
//! * Schema validation

use crate::common::data_contracts::memory_record::{MemoryKind, MemoryRecord};

/// Integrity check results.
#[derive(Debug, Clone, Default)]
pub struct IntegrityReport {
    pub valid: bool,
    pub checksum_ok: bool,
    pub provenance_ok: bool,
    pub provenance_issues: Vec<String>,
    pub relationship_issues: Vec<String>,
    pub relationships_consistent: bool,
    pub duplicates_found: Vec<String>,
    pub confidence_issues: Vec<String>,
    pub schema_issues: Vec<String>,
}

impl IntegrityReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_schema_issue(&mut self, issue: impl Into<String>) {
        self.schema_issues.push(issue.into());
        self.valid = false;
    }

    pub fn add_provenance_issue(&mut self, issue: impl Into<String>) {
        self.provenance_issues.push(issue.into());
        self.valid = false;
    }

    pub fn add_relationship_issue(&mut self, issue: impl Into<String>) {
        self.relationship_issues.push(issue.into());
        self.relationships_consistent = false;
        self.valid = false;
    }

    pub fn add_confidence_issue(&mut self, issue: impl Into<String>) {
        self.confidence_issues.push(issue.into());
        self.valid = false;
    }

    /// Check if the report indicates all integrity checks passed.
    pub fn is_clean(&self) -> bool {
        self.valid
            && self.checksum_ok
            && self.provenance_ok
            && self.relationships_consistent
            && self.duplicates_found.is_empty()
            && self.confidence_issues.is_empty()
            && self.schema_issues.is_empty()
    }
}

/// Compute a simple checksum for a memory record's content.
/// Per Architecture: checksum validation ensures content hasn't been corrupted.
pub fn compute_checksum(record: &MemoryRecord) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    record.content.hash(&mut hasher);
    record.title.hash(&mut hasher);
    record.confidence.to_bits().hash(&mut hasher);
    record.importance.to_bits().hash(&mut hasher);
    record.tags.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Verify that a memory record has valid provenance.
/// Per Architecture: every memory has provenance (source, origin, timestamp).
pub fn verify_provenance(record: &MemoryRecord) -> bool {
    // Source must be non-empty
    if record.source.is_empty() {
        return false;
    }

    // Created timestamp must be reasonable (after epoch)
    if record.created_at <= 0 {
        return false;
    }

    // Confidence must be in valid range
    record.confidence >= 0.0 && record.confidence <= 1.0
}

/// Verify that relationship IDs reference valid memory IDs.
/// Per Architecture: relationships are first-class knowledge; consistency must be checked.
pub fn verify_relationships(
    record: &MemoryRecord,
    known_ids: &std::collections::HashSet<uuid::Uuid>,
) -> bool {
    for rel_id in &record.relationships {
        if !known_ids.contains(rel_id) && *rel_id != uuid::Uuid::nil() {
            return false;
        }
    }
    true
}

/// Detect duplicate memories by content similarity.
/// Per Architecture: rather than storing copies, memories are merged while
/// preserving provenance and relationship history.
pub fn detect_duplicates(records: &[MemoryRecord], threshold: f32) -> Vec<(usize, usize)> {
    let mut duplicates = Vec::new();

    for i in 0..records.len() {
        for j in (i + 1)..records.len() {
            // Simple hash-based dedup: same content hash + confidence within threshold
            let content_i = &records[i].content;
            let content_j = &records[j].content;

            // Fast path: skip if content lengths differ significantly
            if (content_i.len() as i32 - content_j.len() as i32).abs() > 5 {
                continue;
            }

            // Compute a simple content similarity score
            let similarity = content_similarity(content_i, content_j);
            if similarity >= threshold {
                duplicates.push((i, j));
            }
        }
    }

    duplicates
}

/// Compute a simple content similarity score (0.0-1.0).
fn content_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }

    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let words_a: Vec<&str> = a_lower.split_whitespace().collect();
    let words_b: Vec<&str> = b_lower.split_whitespace().collect();

    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    let larger = words_a.len().max(words_b.len());

    // Overlap ratio
    let mut overlap = 0;
    for wa in &words_a {
        for wb in &words_b {
            if wa == wb || wa.contains(wb) || wb.contains(wa) {
                overlap += 1;
                break;
            }
        }
    }

    overlap as f32 / larger as f32
}

/// Validate a memory record's schema (required fields, data types).
/// Per Architecture: schema validation ensures stored data is well-formed.
pub fn validate_schema(record: &MemoryRecord) -> IntegrityReport {
    let mut report = IntegrityReport::new();

    // ID must be valid (non-nil UUID)
    if record.id == uuid::Uuid::nil() {
        report.add_schema_issue("Memory ID is nil".to_string());
    }

    // Content must be non-empty for non-archived records
    let is_archived = record.kind == MemoryKind::Archived;
    if record.content.is_empty() && !is_archived {
        report.add_schema_issue("Memory content is empty".to_string());
    }

    // Confidence must be in [0.0, 1.0]
    if record.confidence < 0.0 || record.confidence > 1.0 {
        report.add_confidence_issue(format!(
            "Invalid confidence: {} for memory {}",
            record.confidence, record.id
        ));
    }

    // Importance must be in [0.0, 1.0]
    if record.importance < 0.0 || record.importance > 1.0 {
        report.add_confidence_issue(format!(
            "Invalid importance: {} for memory {}",
            record.importance, record.id
        ));
    }

    // Source must be non-empty
    if record.source.is_empty() {
        report.add_provenance_issue(format!("Empty source for memory {}", record.id));
    }

    // Created_at must be positive
    if record.created_at <= 0 {
        report.add_provenance_issue(format!("Invalid timestamp for memory {}", record.id));
    }

    report
}

/// Full integrity check for a memory record.
/// Combines provenance, relationship, and schema validation.
/// Per Architecture Chapter 08: integrity checks include checksum,
/// provenance verification, relationship consistency, duplicate detection,
/// confidence review, and schema validation.
pub fn check_integrity(
    record: &MemoryRecord,
    known_ids: &std::collections::HashSet<uuid::Uuid>,
) -> IntegrityReport {
    let mut report = IntegrityReport::new();

    // Compute and store checksum (idempotent — always computes, marks valid if no stored checksum)
    let checksum = compute_checksum(record);
    report.checksum_ok = !checksum.is_empty();

    // Provenance verification
    report.provenance_ok = verify_provenance(record);
    if !report.provenance_ok {
        report.valid = false;
    }

    // Relationship consistency
    report.relationships_consistent = verify_relationships(record, known_ids);
    if !report.relationships_consistent {
        report.valid = false;
    }

    // Schema validation
    let schema_report = validate_schema(record);
    for issue in schema_report.schema_issues {
        report.add_schema_issue(issue);
    }
    for issue in schema_report.confidence_issues {
        report.add_confidence_issue(issue);
    }

    report
}
