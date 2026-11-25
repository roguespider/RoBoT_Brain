//! Unified RetrievalSource type — canonical definition per Architecture §16.3.
//!
//! This module contains the single canonical `RetrievalSource` enum shared
//! across the memory and retrieval subsystems.

/// Which retrieval technique or memory type found a result.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RetrievalSource {
    /// Found via keyword/semantic search in working memory.
    KeywordWorking,
    /// Found via keyword/semantic search in permanent memory.
    KeywordPermanent,
    /// Found via vector similarity search.
    Vector,
    /// Found via graph traversal.
    Graph,
    /// Combined score from multiple sources.
    Combined,
    /// Semantic memory retrieval.
    SemanticMemory,
    /// Episodic memory retrieval.
    EpisodicMemory,
    /// Experience memory retrieval.
    ExperienceMemory,
    /// Procedural memory (skill) retrieval.
    ProceduralMemory,
    /// Archive memory retrieval.
    ArchiveMemory,
}

impl RetrievalSource {
    /// Return source label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::KeywordWorking => "KeywordWorking",
            Self::KeywordPermanent => "KeywordPermanent",
            Self::Vector => "Vector",
            Self::Graph => "Graph",
            Self::Combined => "Combined",
            Self::SemanticMemory => "SemanticMemory",
            Self::EpisodicMemory => "EpisodicMemory",
            Self::ExperienceMemory => "ExperienceMemory",
            Self::ProceduralMemory => "ProceduralMemory",
            Self::ArchiveMemory => "ArchiveMemory",
        }
    }
}
