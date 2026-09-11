// /src/CoObOpLoop/post_task.rs
// Post-task evaluation per §8.

/// Post-task evaluation struct — answers the 10 questions from §8.
#[derive(Clone)]
pub struct PostTaskEvaluation {
    /// Did the task succeed?
    pub did_succeed: bool,

    /// Was the outcome verified?
    pub verification_confirmed: bool,

    /// Unexpected problems encountered.
    pub unexpected_problems: Vec<String>,

    /// Knowledge gaps discovered.
    pub knowledge_gaps: Vec<String>,

    /// New bugs discovered.
    pub new_bugs: Vec<String>,

    /// Capability limitation encountered.
    pub capability_limitation: Option<String>,

    /// Work created by this task.
    pub created_work: Vec<String>,

    /// Efficiency score (0.0–1.0).
    pub efficiency_score: f32,

    /// Adjustment for future planning.
    pub future_planning_adjustment: Option<String>,

    /// Improvement opportunity identified.
    pub improvement_opportunity: Option<String>,
}

impl PostTaskEvaluation {
    /// Create a new post-task evaluation.
    pub fn new() -> Self {
        Self {
            did_succeed: false,
            verification_confirmed: false,
            unexpected_problems: Vec::new(),
            knowledge_gaps: Vec::new(),
            new_bugs: Vec::new(),
            capability_limitation: None,
            created_work: Vec::new(),
            efficiency_score: 0.0,
            future_planning_adjustment: None,
            improvement_opportunity: None,
        }
    }

    /// Generate new AgentGoal entries from evaluation findings.
    pub fn generate_objectives(&self) -> Vec<crate::cooboploop::queue::AgentGoal> {
        let mut goals = Vec::new();
        for problem in &self.unexpected_problems {
            goals.push(self.goal_from_description(problem, "unexpected_problem"));
        }
        for gap in &self.knowledge_gaps {
            goals.push(self.goal_from_description(gap, "knowledge_gap"));
        }
        for bug in &self.new_bugs {
            goals.push(self.goal_from_description(bug, "new_bug"));
        }
        if let Some(ref cap) = self.capability_limitation {
            goals.push(self.goal_from_description(cap, "capability_limitation"));
        }
        if let Some(ref imp) = self.improvement_opportunity {
            goals.push(self.goal_from_description(imp, "improvement_opportunity"));
        }
        // Wire created_work: each work item becomes a goal with elevated priority
        for work in &self.created_work {
            let mut goal = self.goal_from_description(work, "created_work");
            goal.priority = (goal.priority + 0.2).min(1.0);
            goals.push(goal);
        }
        // Wire future_planning_adjustment: include as a goal if set
        if let Some(ref adjustment) = self.future_planning_adjustment {
            let mut goal = self.goal_from_description(adjustment, "future_planning");
            goal.learning_value = 0.8; // planning adjustments have high learning value
            goals.push(goal);
        }
        goals
    }

    fn goal_from_description(
        &self,
        desc: &str,
        source_tag: &str,
    ) -> crate::cooboploop::queue::AgentGoal {
        use crate::cooboploop::queue::GoalStatus;
        use crate::cooboploop::sources::ObjectiveSource;
        debug_assert!(!source_tag.is_empty(), "source_tag must be non-empty");
        crate::cooboploop::queue::AgentGoal {
            id: format!("post_task_{}", std::process::id()),
            title: format!("Post-task: {}", desc.chars().take(60).collect::<String>()),
            description: desc.to_string(),
            status: GoalStatus::Discovered,
            priority: 0.5,
            source: ObjectiveSource::SystemTrigger,
            expected_value: 0.5,
            risk: 0.5,
            learning_value: 0.5,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            deadline: None,
            execution_history: Vec::new(),
            completion_state: None,
        }
    }
}

impl Default for PostTaskEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

impl PostTaskEvaluation {
    pub fn set_did_succeed(&mut self, v: bool) {
        self.did_succeed = v;
    }
    pub fn set_verification_confirmed(&mut self, v: bool) {
        self.verification_confirmed = v;
    }
    pub fn add_unexpected_problem(&mut self, s: String) {
        self.unexpected_problems.push(s);
    }
    pub fn add_knowledge_gap(&mut self, s: String) {
        self.knowledge_gaps.push(s);
    }
    pub fn add_new_bug(&mut self, s: String) {
        self.new_bugs.push(s);
    }
    pub fn set_capability_limitation(&mut self, s: String) {
        self.capability_limitation = Some(s);
    }
    pub fn add_created_work(&mut self, s: String) {
        self.created_work.push(s);
    }
    pub fn set_efficiency_score(&mut self, v: f32) {
        self.efficiency_score = v;
    }
    pub fn set_future_planning_adjustment(&mut self, s: String) {
        self.future_planning_adjustment = Some(s);
    }
    pub fn set_improvement_opportunity(&mut self, s: String) {
        self.improvement_opportunity = Some(s);
    }
    /// Wire `created_work` and `future_planning_adjustment` fields by returning a summary.
    pub fn summarize(&self) -> String {
        let work_count = self.created_work.len();
        let planning = self
            .future_planning_adjustment
            .clone()
            .unwrap_or_else(|| "none".to_string());
        format!("work={work_count} planning={planning}")
    }
}
