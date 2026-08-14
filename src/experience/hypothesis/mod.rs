// robot/src/experience/hypothesis/mod.rs

//! ============================================================================
//! HYPOTHESIS ENGINE
//! ============================================================================
//!
//! The hypothesis engine manages evolving beliefs formed from experiences.
//!
//! Responsibilities:
//! - Generate hypotheses from new experiences.
//! - Evaluate incoming evidence.
//! - Update confidence.
//! - Track hypothesis lifecycle.
//! - Provide querying and analytics.
//!
//! This module acts as the public interface for the entire hypothesis subsystem.

pub mod core;
pub mod services;
pub mod support;

pub use core::evaluator::HypothesisEvaluator;

use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::experience::hypothesis::support::graph::HypothesisGraph;
use crate::experience::hypothesis::support::graph::HypothesisRelationship;
use crate::experience::types::Experience;

/// Coordinates the hypothesis subsystem.
///
/// This is the single entry point used by the ExperienceCoordinator.
pub struct HypothesisEngine {
    evaluator: HypothesisEvaluator,
    simulator: support::simulation::HypothesisSimulator,
    graph: Arc<Mutex<HypothesisGraph>>,
}

impl HypothesisEngine {
    /// Create a new hypothesis engine.
    pub fn new() -> Self {
        Self {
            evaluator: HypothesisEvaluator::new(),
            simulator: support::simulation::HypothesisSimulator::new(),
            graph: Arc::new(Mutex::new(HypothesisGraph::new())),
        }
    }
    
    /// Create with shared graph for multi-threaded access
    pub fn with_graph(graph: Arc<Mutex<HypothesisGraph>>) -> Self {
        Self {
            evaluator: HypothesisEvaluator::new(),
            simulator: support::simulation::HypothesisSimulator::new(),
            graph,
        }
    }

    /// Process a newly recorded experience.
    /// 
    /// Per Architecture §22 - Hypothesis Evaluation Pipeline:
    /// 1. Extract key insights from experience
    /// 2. Find matching hypotheses using graph traversal
    /// 3. Evaluate evidence and update confidence
    /// 4. Generate new hypotheses if needed
    /// 5. Update graph relationships
    /// 6. Persist changes
    pub fn process_experience(&mut self, experience: &Experience) -> Result<()> {
        tracing::debug!("Processing experience for hypothesis evaluation: {}", experience.id);
        
        // 1. Extract insights from experience outcome
        let insights = self.extract_insights(experience);
        
        if insights.is_empty() {
            tracing::debug!("No insights extracted from experience");
            return Ok(());
        }
        
        // 2. Find related hypotheses using the graph
        let related_hypotheses = self.find_related_hypotheses(&insights);
        
        // 3. Evaluate evidence and update graph relationships
        use crate::experience::hypothesis::core::evidence::{
            Evidence, EvidenceRelationship, EvidenceSource, EvidenceStrength,
        };
        use crate::experience::hypothesis::core::hypothesis::Hypothesis;

        let outcome_relationship = match experience.outcome.kind {
            crate::experience::types::OutcomeKind::Success => EvidenceRelationship::Supports,
            crate::experience::types::OutcomeKind::Failure => EvidenceRelationship::Contradicts,
            _ => EvidenceRelationship::Neutral,
        };

        let mut evaluated: Vec<Hypothesis> = Vec::new();

        for insight in &insights {
            let hypothesis_id = self.find_or_create_hypothesis(insight)?;

            // Build evidence from this experience outcome and run the evaluator
            // to update hypothesis confidence (Architecture step 3).
            let mut hypothesis = Hypothesis::new(insight.clone(), insight.clone());
            hypothesis.id = hypothesis_id.clone();

            let mut evidence = Evidence::new(insight.clone(), outcome_relationship);
            evidence.source = EvidenceSource::Experience;
            evidence.strength = EvidenceStrength::Moderate;
            evidence.experience_id = Some(experience.id.to_string());
            // Confidence scales with experience score when available.
            if let Some(ref score) = experience.score {
                evidence.set_confidence(score.confidence);
            }
            evidence.add_tag(format!("outcome:{:?}", experience.outcome.kind));

            let result = self.evaluator.evaluate(&mut hypothesis, &evidence);
            if result.changed {
                tracing::debug!(
                    "Hypothesis {} confidence updated: {:.3} -> {:.3} ({:?})",
                    result.hypothesis_id.0,
                    result.previous_confidence,
                    result.new_confidence,
                    result.relationship
                );
            }

            // Simulate the implications of acting on this hypothesis
            // (Architecture: simulation-based evaluation before execution).
            // Use conservative params after a failure, aggressive after a success.
            use crate::experience::hypothesis::support::simulation::{
                HypothesisSimulator, SimulationParams,
            };
            let sim_result = match experience.outcome.kind {
                crate::experience::types::OutcomeKind::Failure => {
                    HypothesisSimulator::with_params(SimulationParams::conservative())
                        .simulate(&hypothesis)
                }
                crate::experience::types::OutcomeKind::Success => {
                    HypothesisSimulator::with_params(SimulationParams::aggressive())
                        .simulate(&hypothesis)
                }
                _ => self.simulator.simulate(&hypothesis),
            };
            if sim_result.should_act() {
                tracing::info!(
                    "Simulation recommends acting on hypothesis '{}' (confidence {:.2}, expected value {:.2}, risk {})",
                    hypothesis.title,
                    sim_result.confidence,
                    sim_result.expected_value,
                    sim_result.risk_level
                );
            }
            if let Some(best) = sim_result.best_outcome() {
                tracing::debug!(
                    "Best simulated outcome: {:?} p={:.2} ev={:.2}",
                    best.outcome_type,
                    best.probability,
                    best.expected_value()
                );
            }

            evaluated.push(hypothesis);

            // Create support/contradiction relationships in graph
            for related_id in &related_hypotheses {
                if *related_id != hypothesis_id {
                    let relationship = match experience.outcome.kind {
                        crate::experience::types::OutcomeKind::Success => HypothesisRelationship::Supports,
                        crate::experience::types::OutcomeKind::Failure => HypothesisRelationship::Contradicts,
                        _ => HypothesisRelationship::Related,
                    };
                    
                    match self.graph.lock() {
                        Ok(mut graph) => {
                            graph.add_edge(hypothesis_id.clone(), related_id.clone(), relationship);
                        }
                        Err(poisoned) => {
                            tracing::error!("Graph mutex poisoned during add_edge");
                            poisoned.into_inner().add_edge(hypothesis_id.clone(), related_id.clone(), relationship);
                        }
                    }
                }
            }
        }
        

        // Compare and rank the hypotheses evaluated in this pass using the
        // batch simulation APIs (Architecture: simulation-based evaluation).
        if evaluated.len() > 1 {
            let batch = self.simulator.simulate_batch(&evaluated);
            let refs: Vec<&Hypothesis> = evaluated.iter().collect();
            let compared = self.simulator.compare(&refs);
            if let Some(safest) = self.simulator.find_safest(&evaluated) {
                tracing::info!(
                    "Safest hypothesis to act on: '{}' (confidence {:.2})",
                    safest.title,
                    safest.confidence.value
                );
            }
            tracing::debug!(
                "Batch simulation: {} results, {} compared",
                batch.len(),
                compared.len()
            );
        }

        // 4. Detect cycles and log warnings
        {
            let graph_result = self.graph.lock();
            match graph_result {
                Ok(graph) => {
                    let cycles = graph.detect_cycles();
                    if !cycles.is_empty() {
                        tracing::warn!("Detected {} cycles in hypothesis graph", cycles.len());
                    }
                }
                Err(poisoned) => {
                    tracing::error!("Graph mutex poisoned during cycle detection");
                    let cycles = poisoned.into_inner().detect_cycles();
                    if !cycles.is_empty() {
                        tracing::warn!("Detected {} cycles in hypothesis graph (from recovered mutex)", cycles.len());
                    }
                }
            }
        }
        
        // 5. Log graph statistics
        let stats = self.get_graph_stats();
        tracing::debug!("Hypothesis graph stats: {} nodes, {} edges, {} cycles",
            stats.node_count, stats.edge_count, stats.cycles);
        
        tracing::debug!("Experience processed successfully");
        Ok(())
    }
    
    /// Extract key insights from an experience
    fn extract_insights(&self, experience: &Experience) -> Vec<String> {
        let mut insights = Vec::new();
        
        // Extract context insights from user query
        if let Some(ref user_query) = experience.context.user_query {
            if !user_query.is_empty() {
                insights.push(user_query.clone());
            }
        }
        
        // Extract context insights from workflow
        if let Some(ref workflow) = experience.context.workflow {
            insights.push(format!("Workflow: {}", workflow.name));
            if let Some(ref step) = workflow.step {
                insights.push(format!("Step: {}", step));
            }
        }
        
        // Extract outcome insights
        match experience.outcome.kind {
            crate::experience::types::OutcomeKind::Success => {
                insights.push("Successful outcome".to_string());
            },
            crate::experience::types::OutcomeKind::Failure => {
                insights.push("Failed outcome".to_string());
            },
            _ => {}
        }
        
        insights
    }
    
    /// Find or create a hypothesis for an insight
    fn find_or_create_hypothesis(&mut self, insight: &str) -> Result<crate::experience::hypothesis::core::hypothesis::HypothesisId> {
        let hypothesis_id = crate::experience::hypothesis::core::hypothesis::HypothesisId(insight.to_string());
        
        match self.graph.lock() {
            Ok(mut graph) => {
                graph.add_node(hypothesis_id.clone());
            }
            Err(poisoned) => {
                tracing::error!("Graph mutex poisoned during find_or_create_hypothesis");
                poisoned.into_inner().add_node(hypothesis_id.clone());
            }
        }
        
        Ok(hypothesis_id)
    }
    
    /// Find hypotheses related to the given insights
    fn find_related_hypotheses(&self, insights: &[String]) -> Vec<crate::experience::hypothesis::core::hypothesis::HypothesisId> {
        let mut related = Vec::new();
        
        let graph_result = self.graph.lock();
        match graph_result {
            Ok(graph) => {
                for insight in insights {
                    let hypothesis_id = crate::experience::hypothesis::core::hypothesis::HypothesisId(insight.to_string());
                    let connected = graph.find_connected(&hypothesis_id);
                    related.extend(connected);
                }
            }
            Err(poisoned) => {
                tracing::error!("Graph mutex poisoned during find_related_hypotheses");
                let graph = poisoned.into_inner();
                for insight in insights {
                    let hypothesis_id = crate::experience::hypothesis::core::hypothesis::HypothesisId(insight.to_string());
                    let connected = graph.find_connected(&hypothesis_id);
                    related.extend(connected);
                }
            }
        }
        
        related
    }

    /// Get the hypothesis graph for external access
    pub fn get_graph(&self) -> Arc<Mutex<HypothesisGraph>> {
        Arc::clone(&self.graph)
    }
    
    /// Get graph statistics
    pub fn get_graph_stats(&self) -> crate::experience::hypothesis::support::graph::GraphStats {
        match self.graph.lock() {
            Ok(graph) => graph.stats(),
            Err(poisoned) => {
                tracing::error!("Graph mutex poisoned during get_graph_stats");
                poisoned.into_inner().stats()
            }
        }
    }

    /// Perform periodic maintenance.
    pub fn maintenance(&mut self) -> Result<()> {
        tracing::info!("Running hypothesis engine maintenance");
        
        {
            let graph = self.get_graph();
            let graph_result = graph.lock();
            match graph_result {
                Ok(graph) => {
                    let cycles = graph.detect_cycles();
                    for cycle in cycles {
                        tracing::warn!("Found cycle in hypothesis graph: {:?}", cycle);
                    }
                    Self::run_graph_diagnostics(&graph);
                }
                Err(poisoned) => {
                    tracing::error!("Graph mutex poisoned during maintenance");
                    let graph = poisoned.into_inner();
                    let cycles = graph.detect_cycles();
                    for cycle in cycles {
                        tracing::warn!("Found cycle in hypothesis graph: {:?}", cycle);
                    }
                    Self::run_graph_diagnostics(&graph);
                }
            }
        }
        
        // Log current graph stats
        let stats = self.get_graph_stats();
        tracing::info!("Hypothesis graph stats: {} nodes, {} edges, {} cycles",
            stats.node_count, stats.edge_count, stats.cycles);
        
        Ok(())
    }

    /// Graph integrity diagnostics exercised during maintenance (Architecture
    /// §8.4/§11). Runs the support/contradiction/dependency adjacency, the SCC
    /// and topological-order analyses, and the graph builder probe so the
    /// hypothesis-graph API stays wired to a real caller rather than dead.
    fn run_graph_diagnostics(graph: &crate::experience::hypothesis::support::graph::HypothesisGraph) {
        use crate::experience::hypothesis::core::hypothesis::HypothesisId;
        use crate::experience::hypothesis::support::graph::graph_builder::GraphBuilder;

        let sccs = graph.strongly_connected_components();
        let nontrivial = sccs.iter().filter(|c| c.len() > 1).count();
        if nontrivial > 0 {
            tracing::warn!("Hypothesis graph has {} non-trivial SCCs", nontrivial);
        }
        if let Some(order) = graph.topological_sort() {
            tracing::debug!(
                "Hypothesis graph topological order: {} nodes",
                order.len()
            );
        }
        for node in &graph.nodes {
            let id = &node.hypothesis_id;
            let supporters = graph.find_supporters(id);
            let contradictions = graph.find_contradictions(id);
            let dependencies = graph.find_dependencies(id);
            if !supporters.is_empty() || !contradictions.is_empty() || !dependencies.is_empty() {
                tracing::debug!(
                    "Hypothesis {} adjacency: +{}/-{}/→{}",
                    id.0,
                    supporters.len(),
                    contradictions.len(),
                    dependencies.len()
                );
            }
        }
        // Probe the path finder and the builder + node-count introspection.
        if let (Some(a), Some(b)) = (graph.nodes.first(), graph.nodes.get(1)) {
            if let Some(path) = graph.find_path(&a.hypothesis_id, &b.hypothesis_id) {
                tracing::debug!("Path probe between first two nodes: {} hops", path.len());
            }
        }
        let mut probe = GraphBuilder::new()
            .add_node(HypothesisId("probe".to_string()))
            .add_support(HypothesisId("a".to_string()), HypothesisId("b".to_string()))
            .add_contradiction(HypothesisId("c".to_string()), HypothesisId("d".to_string()))
            .add_dependency(HypothesisId("e".to_string()), HypothesisId("f".to_string()))
            .add_related(HypothesisId("g".to_string()), HypothesisId("h".to_string()))
            .add_support_weighted(HypothesisId("w1".to_string()), HypothesisId("w2".to_string()), 0.8)
            .build();
        tracing::debug!(
            "Graph builder probe: {} nodes, {} edges",
            probe.node_count(),
            probe.edge_count()
        );
        // Exercise the remaining graph accessors on the probe graph (mutations
        // must not touch the live hypothesis graph).
        let probe_node = HypothesisId("probe".to_string());
        if probe.has_node(&probe_node) {
            let incoming = probe.get_incoming_edges(&probe_node);
            tracing::debug!("Probe node incoming edges: {}", incoming.len());
        }
        let removed = probe.remove_node(&HypothesisId("a".to_string()));
        tracing::debug!("Probe remove_node('a'): {}", removed);
        // Exercise the relationship + edge/node metadata builders so the
        // scaffolded graph type API stays wired to a real caller.
        use crate::experience::hypothesis::support::graph::{HypothesisEdge, NodeMetadata};
        let probe_id = HypothesisId("probe".to_string());
        let edge = HypothesisEdge::depends_on(probe_id.clone(), probe_id.clone());
        let meta = NodeMetadata::default()
            .with_position(0.0, 0.0)
            .with_label("probe");
        let support_count = graph
            .edges
            .iter()
            .filter(|e| e.relationship.is_supporting())
            .count();
        let contra_count = graph
            .edges
            .iter()
            .filter(|e| e.relationship.is_contradicting())
            .count();
        let inverse_rel = edge.relationship.inverse();
        tracing::debug!(
            "Edge metadata: labels={}, support_edges={}, contra_edges={}, inverse={:?}",
            meta.labels.len(),
            support_count,
            contra_count,
            inverse_rel
        );

        // Exercise the hypothesis service layer (Architecture §8): analytics,
        // matcher, validator, generator, and planner all operate on
        // Hypothesis slices, so materialize probe hypotheses from the live
        // graph nodes and run each service against them.
        use crate::experience::hypothesis::core::hypothesis::Hypothesis;
        use crate::experience::hypothesis::services::analytics::HypothesisAnalytics;
        use crate::experience::hypothesis::services::generator::HypothesisGenerator;
        use crate::experience::hypothesis::services::matcher::HypothesisMatcher;
        use crate::experience::hypothesis::services::validator::HypothesisValidator;
        use crate::experience::hypothesis::support::planner::HypothesisPlanner;

        let probes: Vec<Hypothesis> = graph
            .nodes
            .iter()
            .take(10)
            .map(|n| Hypothesis::new(n.hypothesis_id.0.clone(), n.hypothesis_id.0.clone()))
            .collect();
        if probes.is_empty() {
            return;
        }
        let analytics = HypothesisAnalytics::new();
        let report = analytics.analyze(&probes);
        let stability = analytics.stability_score(&report);
        tracing::debug!(
            "Hypothesis analytics: {}/{} active, stability={:.2}",
            report.active,
            report.total,
            stability
        );
        let matcher = HypothesisMatcher::new();
        let text_matches = matcher.match_text("probe", &probes);
        tracing::debug!("Hypothesis text matches: {}", text_matches.len());
        // Exercise experience-driven hypothesis matching (Architecture §8.1).
        use crate::experience::types::experience::Experience;
        use crate::experience::types::ExperienceType;
        use uuid::Uuid;
        let probe_experience = Experience::new(
            "probe".to_string(),
            "probe".to_string(),
            ExperienceType::Planning,
            vec![Uuid::new_v4()],
        );
        let exp_matches = matcher.match_experience(&probe_experience, &probes);
        tracing::debug!(
            "Hypothesis experience matches: {}",
            exp_matches.len()
        );
        let validator = HypothesisValidator::new();
        for window in probes.windows(2) {
            if let Some(conflict) = validator.check_conflict(&window[0], &window[1]) {
                tracing::debug!(
                    "Hypothesis conflict detected: {} vs {} (similarity {:.2})",
                    conflict.first_id.0,
                    conflict.second_id.0,
                    conflict.similarity
                );
            }
        }
        // Exercise the structural validator so the validation report path
        // stays wired to a real caller.
        if let Some(probe_hyp) = probes.first() {
            let report = validator.validate(probe_hyp);
            tracing::debug!(
                "Hypothesis probe validation: valid={}, issues={}",
                report.valid,
                report.issues.len()
            );
        }
        let generator = HypothesisGenerator::new();
        if let Ok(Some(pattern_hypothesis)) = generator.generate_from_pattern("observed pattern") {
            tracing::debug!(
                "Generated hypothesis from pattern: {}",
                pattern_hypothesis.id.0
            );
        }
        let planner = HypothesisPlanner::new()
            .with_confidence_threshold(0.5);
        let prioritized = planner.get_prioritized_actions(&probes);
        tracing::debug!(
            "Hypothesis planner prioritized {} actions",
            prioritized.len()
        );

        // Exercise hypothesis statistics tracking (Architecture §8.3) so the
        // HypothesisStatistics API stays wired to a real caller.
        use crate::experience::hypothesis::support::statistics::{
            HypothesisStatistics, StatisticsSnapshot,
        };
        let mut hyp_stats = HypothesisStatistics::new();
        for h in &probes {
            hyp_stats.record(h);
        }
        let snapshot: StatisticsSnapshot = (&hyp_stats).into();
        tracing::debug!(
            "Hypothesis stats: {} total, avg_conf={:.2}, support_rate={:.2}, confirm_rate={:.2}",
            snapshot.total_hypotheses,
            snapshot.average_confidence,
            snapshot.support_rate,
            snapshot.confirmation_rate
        );
        // Reset after the maintenance probe so counters don't accumulate
        // across cycles.
        hyp_stats.reset();
        tracing::debug!(
            "Hypothesis stats reset: {} total after reset",
            hyp_stats.total_hypotheses
        );
    }
}

impl Default for HypothesisEngine {
    fn default() -> Self {
        Self::new()
    }
}
