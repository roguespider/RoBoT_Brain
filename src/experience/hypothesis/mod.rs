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
    graph: Arc<Mutex<HypothesisGraph>>,
}

impl HypothesisEngine {
    /// Create a new hypothesis engine.
    pub fn new() -> Self {
        Self {
            evaluator: HypothesisEvaluator::new(),
            graph: Arc::new(Mutex::new(HypothesisGraph::new())),
        }
    }
    
    /// Create with shared graph for multi-threaded access
    pub fn with_graph(graph: Arc<Mutex<HypothesisGraph>>) -> Self {
        Self {
            evaluator: HypothesisEvaluator::new(),
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
        for insight in &insights {
            let hypothesis_id = self.find_or_create_hypothesis(insight)?;
            
            // Create support/contradiction relationships in graph
            for related_id in &related_hypotheses {
                if *related_id != hypothesis_id {
                    let relationship = match experience.outcome.kind {
                        crate::experience::types::OutcomeKind::Success => HypothesisRelationship::Supports,
                        crate::experience::types::OutcomeKind::Failure => HypothesisRelationship::Contradicts,
                        _ => HypothesisRelationship::Related,
                    };
                    
                    let mut graph = self.graph.lock().unwrap();
                    graph.add_edge(hypothesis_id.clone(), related_id.clone(), relationship);
                }
            }
        }
        
        // 4. Detect cycles and log warnings
        {
            let graph = self.graph.lock().unwrap();
            let cycles = graph.detect_cycles();
            if !cycles.is_empty() {
                tracing::warn!("Detected {} cycles in hypothesis graph", cycles.len());
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
        
        {
            let mut graph = self.graph.lock().unwrap();
            graph.add_node(hypothesis_id.clone());
        }
        
        Ok(hypothesis_id)
    }
    
    /// Find hypotheses related to the given insights
    fn find_related_hypotheses(&self, insights: &[String]) -> Vec<crate::experience::hypothesis::core::hypothesis::HypothesisId> {
        let mut related = Vec::new();
        let graph = self.graph.lock().unwrap();
        
        for insight in insights {
            let hypothesis_id = crate::experience::hypothesis::core::hypothesis::HypothesisId(insight.to_string());
            let connected = graph.find_connected(&hypothesis_id);
            related.extend(connected);
        }
        
        related
    }

    /// Get the hypothesis graph for external access
    pub fn get_graph(&self) -> Arc<Mutex<HypothesisGraph>> {
        Arc::clone(&self.graph)
    }
    
    /// Get graph statistics
    pub fn get_graph_stats(&self) -> crate::experience::hypothesis::support::graph::GraphStats {
        let graph = self.graph.lock().unwrap();
        graph.stats()
    }

    /// Observe an experience (for observer pattern)
    pub fn observe(&self, experience: &Experience) -> Result<()> {
        tracing::debug!("HypothesisEngine observing experience: {}", experience.id);
        Ok(())
    }

    /// Perform periodic maintenance.
    pub fn maintenance(&mut self) -> Result<()> {
        tracing::info!("Running hypothesis engine maintenance");
        
        {
            let graph = self.graph.lock().unwrap();
            let cycles = graph.detect_cycles();
            
            for cycle in cycles {
                tracing::warn!("Found cycle in hypothesis graph: {:?}", cycle);
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
