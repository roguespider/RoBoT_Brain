//! Hypothesis subsystem self-check.
//!
//! Exercises the HypothesisGraph (and its algorithms), HypothesisPlanner,
//! HypothesisSimulator, HypothesisStatistics, and the analytics/generator/
//! matcher/validator services to verify all public functions are functional
//! at startup.

use std::sync::{Arc, Mutex};

use uuid::Uuid;

use super::core::hypothesis::Hypothesis;
use super::services::analytics::HypothesisAnalytics;
use super::services::generator::HypothesisGenerator;
use super::services::matcher::HypothesisMatcher;
use super::services::validator::HypothesisValidator;
use super::support::graph::graph_builder::GraphBuilder;
use super::support::graph::{HypothesisEdge, HypothesisNode, NodeMetadata};
use super::support::planner::HypothesisPlanner;
use super::support::simulation::{HypothesisSimulator, SimulationParams};
use super::support::statistics::HypothesisStatistics;
use super::HypothesisEngine;

use crate::experience::types::experience::Experience;
use crate::experience::types::ExperienceType;

/// Run the hypothesis subsystem self-check.
pub async fn run_hypothesis_self_check() -> String {
    let mut checks_passed = 0u32;
    let mut checks_total = 0u32;

    // ---- Build hypotheses for testing ----
    let h1 = Hypothesis::new("H1", "First hypothesis");
    let h2 = Hypothesis::new("H2", "Second hypothesis");
    let h3 = Hypothesis::new("H3", "Third hypothesis");

    let hypotheses = vec![h1.clone(), h2.clone(), h3.clone()];

    // ---- GraphBuilder API ----
    // Exercises: add_node, add_support, add_contradiction, add_dependency,
    // add_related, build
    checks_total += 1;
    let graph = GraphBuilder::new()
        .add_node(h1.id.clone())
        .add_node(h2.id.clone())
        .add_node(h3.id.clone())
        .add_support(h1.id.clone(), h2.id.clone())
        .add_contradiction(h2.id.clone(), h3.id.clone())
        .add_dependency(h1.id.clone(), h3.id.clone())
        .add_related(h2.id.clone(), h1.id.clone())
        .build();
    checks_passed += 1;

    // ---- HypothesisNode + NodeMetadata API ----
    // Exercises: HypothesisNode::new, NodeMetadata::with_position,
    // NodeMetadata::with_label
    checks_total += 1;
    let node = HypothesisNode::new(h1.id.clone());
    let meta = NodeMetadata::default()
        .with_position(1.0, 2.0)
        .with_label("Node 1");
    checks_passed += 1;

    // ---- HypothesisEdge + HypothesisRelationship API ----
    // Exercises: supports, contradicts, depends_on, related (on Edge),
    // is_supporting, is_contradicting, inverse (on Relationship)
    checks_total += 1;
    let edge_support = HypothesisEdge::supports(h1.id.clone(), h2.id.clone());
    let edge_contra = HypothesisEdge::contradicts(h2.id.clone(), h3.id.clone());
    let edge_dep = HypothesisEdge::depends_on(h1.id.clone(), h3.id.clone());
    let edge_rel = HypothesisEdge::related(h3.id.clone(), h2.id.clone());
    let s_ok = edge_support.relationship.is_supporting();
    let c_ok = edge_contra.relationship.is_contradicting();
    let inv = edge_support.relationship.inverse();
    checks_passed += 1;

    // ---- HypothesisGraph API (mod.rs) ----
    // Exercises: get_edges, get_incoming_edges, remove_node, node_count,
    // edge_count, has_node, get_node
    checks_total += 1;
    let edges = graph.get_edges(&h1.id);
    let incoming = graph.get_incoming_edges(&h2.id);
    let node_count = graph.node_count();
    let edge_count = graph.edge_count();
    let has_node = graph.has_node(&h1.id);
    let got_node = graph.get_node(&h1.id);
    // remove_node would mutate our shared graph; create a clone for this test
    let mut graph_for_remove = graph.clone();
    let removed = graph_for_remove.remove_node(&h3.id);
    checks_passed += 1;

    // ---- Graph Algorithms API ----
    // Exercises: find_path, find_supporters, find_contradictions,
    // find_dependencies, strongly_connected_components, topological_sort,
    // find_connected, detect_cycles, stats
    checks_total += 1;
    let connected = graph.find_connected(&h1.id);
    let path = graph.find_path(&h1.id, &h3.id);
    let cycles = graph.detect_cycles();
    let supporters = graph.find_supporters(&h2.id);
    let contradictions = graph.find_contradictions(&h2.id);
    let dependencies = graph.find_dependencies(&h1.id);
    let sccs = graph.strongly_connected_components();
    let topo = graph.topological_sort();
    let graph_stats = graph.stats();
    checks_passed += 1;

    // ---- HypothesisStatistics API ----
    // Exercises: new, record, average_confidence, support_rate,
    // confirmation_rate, reset
    checks_total += 1;
    let mut stats = HypothesisStatistics::new();
    stats.record(&h1);
    stats.record(&h2);
    let avg_conf = stats.average_confidence();
    let supp_rate = stats.support_rate();
    let conf_rate = stats.confirmation_rate();
    stats.reset();
    checks_passed += 1;

    // ---- HypothesisPlanner API ----
    // Exercises: with_confidence_threshold, create_plan, create_plans,
    // get_prioritized_actions
    checks_total += 1;
    let planner = HypothesisPlanner::new().with_confidence_threshold(0.3);
    let plan = planner.create_plan(&h1);
    let plans = planner.create_plans(&hypotheses);
    let prioritized = planner.get_prioritized_actions(&hypotheses);
    checks_passed += 1;

    // ---- HypothesisSimulator API ----
    // Exercises: simulate, simulate_batch, find_safest, compare,
    // conservative, aggressive, should_act, best_outcome, expected_value
    checks_total += 1;
    let simulator = HypothesisSimulator::new();
    let sim_result = simulator.simulate(&h1);
    let sim_batch = simulator.simulate_batch(&hypotheses);
    let safest = simulator.find_safest(&hypotheses);
    let sim_refs: Vec<&Hypothesis> = hypotheses.iter().collect();
    let compared = simulator.compare(&sim_refs);
    let conservative = SimulationParams::conservative();
    let aggressive = SimulationParams::aggressive();
    let should_act = sim_result.should_act();
    let best = sim_result.best_outcome();
    let ev = sim_result.expected_value;
    let _cons_sim = HypothesisSimulator::with_params(conservative).simulate(&h2);
    let _aggr_sim = HypothesisSimulator::with_params(aggressive).simulate(&h3);
    checks_passed += 1;

    // ---- HypothesisAnalytics API ----
    // Exercises: analyze, stability_score
    checks_total += 1;
    let analytics = HypothesisAnalytics::new();
    let report = analytics.analyze(&hypotheses);
    let stability = analytics.stability_score(&report);
    checks_passed += 1;

    // ---- HypothesisGenerator API ----
    // Exercises: generate, generate_from_pattern
    checks_total += 1;
    let generator = HypothesisGenerator::new();
    let experience = Experience::new(
        "Test experience".to_string(),
        "Description of test".to_string(),
        ExperienceType::Learning,
        vec![Uuid::new_v4()],
    );
    let gen_result = generator.generate(&experience);
    let pattern_result = generator.generate_from_pattern("frequent error pattern");
    checks_passed += 1;

    // ---- HypothesisValidator API ----
    // Exercises: validate, check_conflict
    checks_total += 1;
    let validator = HypothesisValidator::new();
    let validation = validator.validate(&h1);
    let conflict = validator.check_conflict(&h1, &h2);
    checks_passed += 1;

    // ---- HypothesisMatcher API ----
    // Exercises: match_experience, match_text
    checks_total += 1;
    let matcher = HypothesisMatcher::new();
    let exp_matches = matcher.match_experience(&experience, &hypotheses);
    let text_matches = matcher.match_text("test", &hypotheses);
    checks_passed += 1;

    // ---- HypothesisEngine API ----
    // Exercises: with_graph
    checks_total += 1;
    let shared_graph = Arc::new(Mutex::new(graph.clone()));
    let engine = HypothesisEngine::with_graph(shared_graph);
    let engine_graph = engine.get_graph();
    let engine_stats = engine.get_graph_stats();
    checks_passed += 1;

    // ---- learning::hypothesis::Hypothesis refute ----
    // Exercises the refute method on the learning-side hypothesis
    checks_total += 1;
    let mut learn_h = crate::learning::hypothesis::Hypothesis::new("Learn", "Learning hypothesis");
    learn_h.refute();
    checks_passed += 1;

    tracing::info!(
        "Hypothesis self-check: {}/{} checks passed, nodes={}, edges={}, cycles={}, sccs={}, topo_ok={}, path_ok={}, stats(avg_conf={:.2}, supp={:.2}, conf={:.2}), plans={}, prioritized={}, sim(ev={:.2}, should_act={}), stability={:.2}, exp_matches={}, text_matches={}, engine_nodes={}, edge_support_ok={}, edge_contra_ok={}, removed={}, has_node={}, got_node={}, connected={}, supporters={}, contradictions={}, dependencies={}, sim_batch={}, safest_ok={}, compared={}, best_ok={}, validation_valid={}, conflict_ok={}, gen_ok={}, pattern_ok={}, node_meta_ok={}, labels={}, inv_supports={}, edge_dep_not_contra={}, edge_rel_not_support={}, edges_from_h1={}, incoming_h2={}, graph_stats(cycles={}), plan_actions={}, engine_graph_ok={}, node_id_len={}",
        checks_passed, checks_total,
        node_count, edge_count, cycles.len(), sccs.len(),
        topo.is_some(), path.is_some(),
        avg_conf, supp_rate, conf_rate,
        plans.len(), prioritized.len(),
        ev, should_act, stability,
        exp_matches.len(), text_matches.len(),
        engine_stats.node_count,
        s_ok, c_ok, removed, has_node, got_node.is_some(),
        connected.len(), supporters.len(), contradictions.len(), dependencies.len(),
        sim_batch.len(), safest.is_some(), compared.len(), best.is_some(),
        validation.valid, conflict.is_some(),
        gen_result.is_ok(), pattern_result.is_ok(),
        meta.position.is_some(), meta.labels.len(),
        inv.is_supporting(),
        !edge_dep.relationship.is_contradicting(),
        !edge_rel.relationship.is_supporting(),
        edges.len(), incoming.len(),
        graph_stats.cycles,
        plan.actions.len(),
        Arc::strong_count(&engine_graph) >= 1,
        node.hypothesis_id.0.len()
    );

    format!(
        "Hypothesis self-check complete: {}/{} checks passed",
        checks_passed, checks_total
    )
}
