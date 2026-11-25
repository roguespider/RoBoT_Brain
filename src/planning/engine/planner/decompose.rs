//! Goal decomposition logic for the planning engine

use super::Planner;

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::planning::engine::types::{PlanStep, StepStatus};

impl Planner {
    /// Decompose a goal into actionable plan steps.
    ///
    /// Per Architecture §2.8: "Planning depends on the accumulated knowledge
    /// of the entire system." Without an LLM, we use rule-based decomposition:
    /// parse the goal text for action verbs and generate steps accordingly.
    /// This ensures plans have actionable steps the agent loop can select.
    pub(crate) fn decompose_goal(goal: &str) -> Vec<PlanStep> {
        let lower = goal.to_lowercase();
        let mut steps = Vec::new();

        // Detect intent from keywords and generate matching steps.
        let wants_search = lower.contains("find")
            || lower.contains("search")
            || lower.contains("lookup")
            || lower.contains("retrieve")
            || lower.contains("get");
        let wants_store = lower.contains("store")
            || lower.contains("save")
            || lower.contains("record")
            || lower.contains("remember");
        let wants_knowledge = lower.contains("knowledge")
            || lower.contains("learn")
            || lower.contains("understand")
            || lower.contains("know");
        let wants_analyze = lower.contains("analyze")
            || lower.contains("summarize")
            || lower.contains("evaluate")
            || lower.contains("assess");
        let wants_plan = lower.contains("plan")
            || lower.contains("create")
            || lower.contains("design")
            || lower.contains("build");

        let step_counter = AtomicUsize::new(0);
        let next_id = || format!("step-{}", step_counter.fetch_add(1, Ordering::Relaxed) + 1);

        if wants_search {
            steps.push(PlanStep {
                id: next_id(),
                description: format!("Search memory and knowledge for: {}", goal),
                action: "search_memory".to_string(),
                dependencies: Vec::new(),
                status: StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        if wants_knowledge {
            steps.push(PlanStep {
                id: next_id(),
                description: format!("Query knowledge base for: {}", goal),
                action: "query_knowledge".to_string(),
                dependencies: steps.last().map(|s| vec![s.id.clone()]).unwrap_or_default(),
                status: StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        if wants_analyze {
            steps.push(PlanStep {
                id: next_id(),
                description: format!("Analyze and synthesize findings for: {}", goal),
                action: "analyze_results".to_string(),
                dependencies: steps.last().map(|s| vec![s.id.clone()]).unwrap_or_default(),
                status: StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        if wants_plan {
            steps.push(PlanStep {
                id: next_id(),
                description: format!("Create a structured plan for: {}", goal),
                action: "create_plan".to_string(),
                dependencies: steps.last().map(|s| vec![s.id.clone()]).unwrap_or_default(),
                status: StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        if wants_store {
            steps.push(PlanStep {
                id: next_id(),
                description: format!("Store the result for: {}", goal),
                action: "store_memory".to_string(),
                dependencies: steps.last().map(|s| vec![s.id.clone()]).unwrap_or_default(),
                status: StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        // If no keywords matched, generate generic steps so the plan is
        // always actionable.
        if steps.is_empty() {
            steps.push(PlanStep {
                id: next_id(),
                description: format!("Retrieve relevant context for: {}", goal),
                action: "search_memory".to_string(),
                dependencies: Vec::new(),
                status: StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
            steps.push(PlanStep {
                id: next_id(),
                description: format!("Query knowledge for: {}", goal),
                action: "query_knowledge".to_string(),
                dependencies: vec!["step-1".to_string()],
                status: StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
            steps.push(PlanStep {
                id: next_id(),
                description: format!("Execute the primary action for: {}", goal),
                action: "execute_action".to_string(),
                dependencies: vec!["step-2".to_string()],
                status: StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        steps
    }
}
