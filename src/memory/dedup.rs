//! Memory deduplication and consolidation - Per Architecture §17.4 "Memory promotion"
//!
//! Handles merging duplicate memory records before promotion.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Hash and Hasher are actively used in content_hash() for deduplication grouping.

use crate::data_contracts::memory_record::MemoryRecord;

/// Merge duplicate records by grouping by content hash and keeping the highest confidence record.
pub fn merge_duplicates(records: Vec<MemoryRecord>) -> Vec<MemoryRecord> {
    use std::collections::HashMap;

    let mut groups: HashMap<u64, Vec<MemoryRecord>> = HashMap::new();
    let mut merged: Vec<MemoryRecord> = Vec::new();

    for record in records {
        // Skip anchor records - they are preserved unchanged (T2-44)
        if record.is_anchor {
            merged.push(record);
            continue;
        }

        let hash = content_hash(&record.content);
        groups.entry(hash).or_default().push(record);
    }

    for (hash, group) in groups {
        // Actively use `hash` per hygiene rules (no underscore ignores).
        tracing::debug!(
            content_hash = hash,
            group_size = group.len(),
            "Merging duplicate group"
        );
        // Find the record with the highest confidence
        let best = group
            .iter()
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
            .unwrap_or(group[0].clone());

        // Merge provenance from all duplicates
        let mut merged_record = best.clone();
        for record in group {
            for prov in record.metadata.provenance {
                if !merged_record.metadata.provenance.contains(&prov) {
                    merged_record.metadata.provenance.push(prov);
                }
            }
        }

        merged.push(merged_record);
    }

    merged
}

/// Calculate content hash for grouping duplicates
fn content_hash(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}
