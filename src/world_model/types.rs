// src/world_model/types.rs
//! Entity types for the World Model (Architecture §14).
//!
//! "Memory stores facts. World Model stores understanding."
//!
//! The World Model represents *how the world works*: entities (objects,
//! places, people, events, time, goals, resources) and the typed relationships
//! between them. Unlike memory items (raw facts), a world-model entity carries
//! properties, a confidence in its existence/attributes, and links to other
//! entities — enabling graph-based reasoning (e.g. "is this goal blocked by a
//! resource constraint?").

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The kind of real-world thing an entity represents (Architecture §14).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityKind {
    /// Physical or digital object (a file, a tool, a device).
    Object,
    /// A location (a directory, a host, a room).
    Place,
    /// A human or agent actor.
    Person,
    /// Something that happened or will happen (a meeting, a build, a failure).
    Event,
    /// A temporal reference (a deadline, a schedule slot).
    Time,
    /// A desired end state the system or a user is pursuing.
    Goal,
    /// A consumable or finite resource (tokens, quota, memory, budget).
    Resource,
}

impl EntityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityKind::Object => "object",
            EntityKind::Place => "place",
            EntityKind::Person => "person",
            EntityKind::Event => "event",
            EntityKind::Time => "time",
            EntityKind::Goal => "goal",
            EntityKind::Resource => "resource",
        }
    }
}

/// The kind of typed relationship between two entities (Architecture §14).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationKind {
    /// `source` is located at `target` (Object→Place).
    LocatedAt,
    /// `source` owns or controls `target`.
    Owns,
    /// `source` participates in `target` (Person→Event).
    ParticipatesIn,
    /// `source` causes or caused `target`.
    Causes,
    /// `source` depends on `target` to proceed.
    DependsOn,
    /// `source` blocks `target` from proceeding.
    Blocks,
    /// `source` is a sub-part of `target`.
    PartOf,
    /// `source` is an alternative to `target`.
    AlternativeTo,
    /// `source` consumes `target` (Goal/Event→Resource).
    Consumes,
    /// `source` produces `target` (Event→Resource/Object).
    Produces,
    /// `source` occurs before `target` (temporal ordering).
    Before,
    /// Generic association for untyped links.
    RelatedTo,
}

impl RelationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationKind::LocatedAt => "located_at",
            RelationKind::Owns => "owns",
            RelationKind::ParticipatesIn => "participates_in",
            RelationKind::Causes => "causes",
            RelationKind::DependsOn => "depends_on",
            RelationKind::Blocks => "blocks",
            RelationKind::PartOf => "part_of",
            RelationKind::AlternativeTo => "alternative_to",
            RelationKind::Consumes => "consumes",
            RelationKind::Produces => "produces",
            RelationKind::Before => "before",
            RelationKind::RelatedTo => "related_to",
        }
    }
}

/// A typed relationship between two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub source: Uuid,
    pub target: Uuid,
    pub kind: RelationKind,
    /// Confidence in this relationship (0.0–1.0).
    pub confidence: f32,
    /// When the relationship was first observed.
    pub observed_at: DateTime<Utc>,
}

impl Relationship {
    pub fn new(source: Uuid, target: Uuid, kind: RelationKind) -> Self {
        Self {
            source,
            target,
            kind,
            confidence: 0.5,
            observed_at: Utc::now(),
        }
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// A node in the world model: a real-world thing with properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub kind: EntityKind,
    /// Free-form attributes (e.g. {"path": "/etc"}, {"quota": "1000"}).
    pub properties: HashMap<String, String>,
    /// Confidence that this entity exists/is-accurate (0.0–1.0).
    pub confidence: f32,
    /// How central/important this entity is to the current world view.
    pub salience: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity {
    pub fn new(name: impl Into<String>, kind: EntityKind) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind,
            properties: HashMap::new(),
            confidence: 0.5,
            salience: 0.5,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self.updated_at = Utc::now();
        self
    }
}
