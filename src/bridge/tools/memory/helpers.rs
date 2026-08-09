//! Internal conversion helpers shared across memory-tool handlers.
//!
//! These map between the string-typed `memory_type` field accepted by MCP
//! clients, the database persistence enum, and the in-memory cache enum.

use crate::database::models::MemoryType;

/// Parse a client-supplied `memory_type` string into the database enum.
///
/// Unknown strings fall back to [`MemoryType::Note`] so a malformed request
/// is still stored (with the most permissive type) rather than rejected —
/// the tool's own validation layer decides whether to reject upstream.
pub(crate) fn parse_memory_type(s: &str) -> MemoryType {
    match s.to_lowercase().as_str() {
        "fact" => MemoryType::Fact,
        "task" => MemoryType::Task,
        "file" => MemoryType::File,
        "conversation" => MemoryType::Conversation,
        "code" => MemoryType::Code,
        "decision" => MemoryType::Decision,
        "event" => MemoryType::Event,
        "encounter" => MemoryType::Encounter,
        "experience" => MemoryType::Experience,
        _ => MemoryType::Note,
    }
}

/// Convert the database persistence enum to the in-memory cache enum.
///
/// The two enums model overlapping but not identical concepts: the database
/// type captures how a memory was *recorded* (note/fact/task/...), while the
/// cache type captures the cognitive *category* it maps to
/// (experience/knowledge/skill/workflow/context/observation). This mapping is
/// lossy by design — see Architecture §6.3 for the rationale.
pub(crate) fn convert_memory_type_to_memory(
    dt: MemoryType,
) -> crate::memory::types::MemoryType {
    match dt {
        MemoryType::Note => crate::memory::types::MemoryType::Experience,
        MemoryType::Fact => crate::memory::types::MemoryType::Knowledge,
        MemoryType::Task => crate::memory::types::MemoryType::Skill,
        MemoryType::File => crate::memory::types::MemoryType::Workflow,
        MemoryType::Conversation => crate::memory::types::MemoryType::Context,
        MemoryType::Code => crate::memory::types::MemoryType::Skill,
        MemoryType::Decision => crate::memory::types::MemoryType::Experience,
        MemoryType::Event => crate::memory::types::MemoryType::Observation,
        MemoryType::Encounter => crate::memory::types::MemoryType::Observation,
        MemoryType::Experience => crate::memory::types::MemoryType::Experience,
    }
}
