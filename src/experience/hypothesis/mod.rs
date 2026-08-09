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
                }
                Err(poisoned) => {
                    tracing::error!("Graph mutex poisoned during maintenance");
                    let cycles = poisoned.into_inner().detect_cycles();
                    for cycle in cycles {
                        tracing::warn!("Found cycle in hypothesis graph: {:?}", cycle);
                    }
                }
            }
        }
        
        // Log current graph stats
        let stats = self.get_graph_stats();
        tracing::info!("Hypothesis graph stats: {} nodes, {} edges, {} cycles",
            stats.node_count, stats.edge_count, stats.cycles);
        
        Ok(())
    }
}

impl Default for HypothesisEngine {
    fn default() -> Self {
        Self::new()
    }
}
