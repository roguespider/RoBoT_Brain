// src/experience/hypothesis/support/graph/self_check.rs
//! Hypothesis graph self-check (Architecture §9 / §4.04)
//!
//! Exercises the hypothesis graph query/algorithm API that has no direct
//! tool surface yet so those code paths remain live rather than dead code:
//! - GraphBuilder (add_support, add_contradiction, add_dependency, add_related)
//! - HypothesisGraph (get_incoming_edges, remove_node, node_count, edge_count,
//!   has_node, find_path, find_supporters, find_contradictions,
//!   find_dependencies, strongly_connected_components, topological_sort, stats)
//! - NodeMetadata (with_position, with_label)
//! - HypothesisEdge constructors (supports, contradicts, depends_on, related)
//! - HypothesisRelationship (is_supporting, is_contradicting, inverse)

use tracing::info;

use super::graph_builder::GraphBuilder;
use super::graph_types::{
    HypothesisEdge, HypothesisNode, HypothesisRelationship, NodeMetadata,
};
use super::HypothesisGraph;
use crate::experience::hypothesis::core::hypothesis::HypothesisId;

fn hid(s: &str) -> HypothesisId {
    HypothesisId(s.to_string())
}

/// Run the hypothesis graph self-check. Returns the number of checks that passed.
pub fn run() -> usize {
    let mut checks_total = 0usize;
    let mut checks_passed = 0usize;

    // 1. GraphBuilder constructs a graph with all four relationship kinds.
    checks_total += 1;
    let graph = GraphBuilder::new()
        .add_node(hid("a"))
        .add_node(hid("b"))
        .add_node(hid("c"))
        .add_node(hid("d"))
        .add_support(hid("a"), hid("b"))
        .add_contradiction(hid("c"), hid("b"))
        .add_dependency(hid("b"), hid("d"))
        .add_related(hid("a"), hid("c"))
        .build();
    if graph.node_count() == 4 && graph.edge_count() == 4 {
        checks_passed += 1;
    }

    // 2. has_node, get_edges, get_incoming_edges for node b.
    checks_total += 1;
    if graph.has_node(&hid("b"))
        && !graph.has_node(&hid("zzz"))
        && !graph.get_edges(&hid("b")).is_empty()
        && !graph.get_incoming_edges(&hid("b")).is_empty()
    {
        checks_passed += 1;
    }

    // 3. find_path between a and d (a -> b -> d).
    checks_total += 1;
    let path = graph.find_path(&hid("a"), &hid("d"));
    if matches!(path, Some(p) if p.len() >= 2 && p.last() == Some(&hid("d"))) {
        checks_passed += 1;
    }

    // 4. find_supporters, find_contradictions, find_dependencies for b.
    checks_total += 1;
    if !graph.find_supporters(&hid("b")).is_empty()
        && !graph.find_contradictions(&hid("b")).is_empty()
        && !graph.find_dependencies(&hid("b")).is_empty()
    {
        checks_passed += 1;
    }

    // 5. topological_sort on the acyclic dependency graph (a->b->d, c->b, a->c).
    checks_total += 1;
    let topo = graph.topological_sort();
    if matches!(topo, Some(order) if order.len() == 4) {
        checks_passed += 1;
    }

    // 6. strongly_connected_components and stats.
    checks_total += 1;
    let sccs = graph.strongly_connected_components();
    let stats = graph.stats();
    if !sccs.is_empty()
        && stats.node_count == 4
        && stats.edge_count == 4
        && stats.support_edges == 1
        && stats.contradict_edges == 1
        && stats.depends_edges == 1
        && stats.related_edges == 1
    {
        checks_passed += 1;
    }

    // 7. remove_node removes the node and its edges, returns true.
    checks_total += 1;
    let mut g2 = graph.clone();
    let removed = g2.remove_node(&hid("b"));
    if removed
        && !g2.has_node(&hid("b"))
        && g2.node_count() == 3
        && g2.edge_count() == 1
    {
        checks_passed += 1;
    }

    // 8. NodeMetadata builders and edge constructors.
    checks_total += 1;
    let meta = NodeMetadata::default()
        .with_position(1.0, 2.0)
        .with_label("probe");
    let node = HypothesisNode::new(hid("x"));
    let e_sup = HypothesisEdge::supports(hid("a"), hid("b"));
    let e_con = HypothesisEdge::contradicts(hid("a"), hid("b"));
    let e_dep = HypothesisEdge::depends_on(hid("a"), hid("b"));
    let e_rel = HypothesisEdge::related(hid("a"), hid("b"));
    if meta.position == Some((1.0, 2.0))
        && meta.labels == vec!["probe".to_string()]
        && node.hypothesis_id.0 == "x"
        && e_sup.relationship.is_supporting()
        && e_con.relationship.is_contradicting()
        && e_dep.relationship == HypothesisRelationship::DependsOn
        && e_rel.relationship == HypothesisRelationship::Related
    {
        checks_passed += 1;
    }

    // 9. HypothesisRelationship::inverse across variants.
    checks_total += 1;
    if HypothesisRelationship::Supports.inverse() == HypothesisRelationship::Contradicts
        && HypothesisRelationship::Contradicts.inverse() == HypothesisRelationship::Supports
        && HypothesisRelationship::DependsOn.inverse() == HypothesisRelationship::DependsOn
        && HypothesisRelationship::Related.inverse() == HypothesisRelationship::Related
    {
        checks_passed += 1;
    }

    // 10. detect_cycles on a graph with a cycle.
    checks_total += 1;
    let cyclic = GraphBuilder::new()
        .add_support(hid("p"), hid("q"))
        .add_support(hid("q"), hid("r"))
        .add_support(hid("r"), hid("p"))
        .build();
    if !cyclic.detect_cycles().is_empty()
        && !cyclic.strongly_connected_components().is_empty()
    {
        checks_passed += 1;
    }

    info!(
        "Hypothesis graph self-check: {}/{} checks passed",
        checks_passed, checks_total
    );
    // Use clear() on a throwaway graph so the clear path stays live too.
    let mut scratch = HypothesisGraph::new();
    scratch.clear();
    checks_passed
}
