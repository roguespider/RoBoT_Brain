// src/world_model/self_check.rs
//! Startup self-check for the World Model (Architecture §14).
//!
//! Exercises the entity-relationship graph end-to-end: upsert entities, link
//! them, and run the reasoning queries (blockers, dependencies, resources) so
//! the code path stays live and the queries are verified correct.

use super::store::WorldModel;
use super::types::{Entity, EntityKind, RelationKind, Relationship};

/// Run the world-model self-check, returning the number of assertions passed.
pub async fn run() -> usize {
    let mut passed = 0usize;
    let model = WorldModel::new();

    // Model a small world: a goal that depends on a resource and is blocked
    // by a failed event.
    let goal = Entity::new("ship v2.0", EntityKind::Goal).with_confidence(0.8);
    let resource = Entity::new("CI minutes", EntityKind::Resource)
        .with_property("quota", "2000")
        .with_confidence(0.9);
    let event = Entity::new("build failure", EntityKind::Event).with_confidence(0.7);
    let person = Entity::new("release engineer", EntityKind::Person).with_confidence(0.6);

    let goal_id = model.upsert_entity(goal).await;
    let resource_id = model.upsert_entity(resource).await;
    let event_id = model.upsert_entity(event).await;
    let person_id = model.upsert_entity(person).await;

    // Link them: goal depends on resource; event blocks goal; person
    // participates in event; goal consumes resource. One relationship carries
    // an explicit confidence to exercise the builder.
    let rels = [
        Relationship::new(goal_id, resource_id, RelationKind::DependsOn),
        Relationship::new(event_id, goal_id, RelationKind::Blocks).with_confidence(0.9),
        Relationship::new(person_id, event_id, RelationKind::ParticipatesIn),
        Relationship::new(goal_id, resource_id, RelationKind::Consumes),
    ];
    for rel in rels {
        if model.add_relationship(rel).await.is_ok() {
            passed += 1;
        }
    }

    // Reasoning: the goal should have one blocker (the failed build).
    let blockers = model.blockers_of(goal_id).await;
    if blockers.len() == 1 && blockers[0].name == "build failure" {
        passed += 1;
    }

    // Reasoning: the goal depends on one resource (CI minutes).
    let deps = model.dependencies_of(goal_id).await;
    if deps.len() == 1 && deps[0].kind == EntityKind::Resource {
        passed += 1;
    }

    // Reasoning: the goal consumes one resource.
    let consumed = model.resources_consumed_by(goal_id).await;
    if consumed.len() == 1 && consumed[0].name == "CI minutes" {
        passed += 1;
    }

    // Name lookup.
    if model.find_by_name("release engineer").await.is_some() {
        passed += 1;
    }

    // Direct id lookup.
    if model.get_entity(goal_id).await.is_some() {
        passed += 1;
    }

    // Kind enumeration: two resources? No — one resource, one person. Check
    // that we can enumerate by kind and get the expected count for people.
    let people = model.entities_of_kind(EntityKind::Person).await;
    if people.len() == 1 {
        passed += 1;
    }

    // Relationship traversal: the event should have two relationships
    // (participates_in person-target flipped, and blocks goal). Actually
    // event participates_in is person→event, and event→goal blocks, so event
    // is involved in both → 2 relationships.
    let event_rels = model.relationships_for(event_id).await;
    if event_rels.len() == 2 {
        passed += 1;
    }

    // Counts.
    if model.entity_count().await == 4 && model.relationship_count().await == 4 {
        passed += 1;
    }

    tracing::info!(
        "World-model self-check: {} assertions passed (entities={}, relationships={}, \
         sample_entity_kind='{}', sample_relation_kind='{}')",
        passed,
        model.entity_count().await,
        model.relationship_count().await,
        EntityKind::Goal.as_str(),
        RelationKind::Blocks.as_str()
    );

    passed
}
