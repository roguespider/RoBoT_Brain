//! Memory summarization - Per Architecture §7.4 "Context compression"
//!
//! Summarizes aging low-importance memories by combining them.

use crate::data_contracts::memory_record::MemoryRecord;

/// Summarize a collection of memory records into a single summary record.
///
/// Creates a new record with combined content and combined provenance.
pub fn summarize(records: Vec<MemoryRecord>) -> MemoryRecord {
    if records.is_empty() {
        return MemoryRecord::default();
    }

    // Combine content with separator
    let content = records
        .iter()
        .map(|r| r.content.as_str())
        .collect::<Vec<&str>>()
        .join(" | ");

    // Find the maximum importance
    let max_importance = records.iter().map(|r| r.importance).fold(0.0, f32::max);

    // Combine provenance
    let mut provenance: Vec<String> = Vec::new();
    for record in &records {
        for prov in &record.metadata.provenance {
            if !provenance.contains(prov) {
                provenance.push(prov.clone());
            }
        }
    }

    // Create summary record
    let mut summary = MemoryRecord::new(content, records[0].kind);
    summary.importance = max_importance;
    summary.metadata.provenance = provenance;
    summary.access_count = records.iter().map(|r| r.access_count).sum();
    summary.consolidated_from = records.iter().map(|r| r.id.to_string()).collect();

    summary
}
