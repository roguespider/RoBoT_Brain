// /src/CoObOpLoop/opportunity/mod.rs
// External opportunity intake for the CoObOpLoop system (§17).

use serde::{Deserialize, Serialize};

/// External source type for an opportunity (§17).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalSource {
    Fiverr,
    Upwork,
    GitHubIssues,
    Other(String),
}

/// Opportunity struct (§17 / T12.1).
#[derive(Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub id: String,
    pub title: String,
    pub source_type: ExternalSource,
    pub requirements: Vec<String>,
    pub expected_effort: f32,
    pub required_capabilities: Vec<crate::cooboploop::capability::CapabilityId>,
    pub expected_value: f32,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

impl std::fmt::Display for Opportunity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Opportunity({}, {})", self.id, self.title)
    }
}

/// Opportunity adapter trait (§17 / T12.2).
pub trait OpportunityAdapter {
    fn name(&self) -> &str;
    fn fetch(&self) -> Vec<Opportunity>;
    fn parse(&self, raw: &str) -> Opportunity;
}

/// Fiverr opportunity adapter (§T12.3).
pub struct FiverrAdapter {
    source: String,
}

impl FiverrAdapter {
    pub fn new(source: String) -> Self {
        Self { source }
    }
}

impl OpportunityAdapter for FiverrAdapter {
    fn name(&self) -> &str {
        "fiverr"
    }

    fn fetch(&self) -> Vec<Opportunity> {
        if self.source.trim().is_empty() {
            Vec::new()
        } else {
            vec![self.parse(&self.source)]
        }
    }

    fn parse(&self, raw: &str) -> Opportunity {
        if let Ok(mut opportunity) = serde_json::from_str::<Opportunity>(raw) {
            opportunity.source_type = ExternalSource::Fiverr;
            return opportunity;
        }
        let title: String = raw.trim().chars().take(80).collect();
        Opportunity {
            id: format!("fiverr_{}", uuid::Uuid::new_v4()),
            title,
            source_type: ExternalSource::Fiverr,
            requirements: Vec::new(),
            expected_effort: 0.0,
            required_capabilities: Vec::new(),
            expected_value: 0.0,
            deadline: None,
        }
    }
}

/// Upwork opportunity adapter (§T12.4).
pub struct UpworkAdapter {
    source: String,
}

impl UpworkAdapter {
    pub fn new(source: String) -> Self {
        Self { source }
    }
}

impl OpportunityAdapter for UpworkAdapter {
    fn name(&self) -> &str {
        "upwork"
    }

    fn fetch(&self) -> Vec<Opportunity> {
        if self.source.trim().is_empty() {
            Vec::new()
        } else {
            vec![self.parse(&self.source)]
        }
    }

    fn parse(&self, raw: &str) -> Opportunity {
        if let Ok(mut opportunity) = serde_json::from_str::<Opportunity>(raw) {
            opportunity.source_type = ExternalSource::Upwork;
            return opportunity;
        }
        let title: String = raw.trim().chars().take(80).collect();
        Opportunity {
            id: format!("upwork_{}", uuid::Uuid::new_v4()),
            title,
            source_type: ExternalSource::Upwork,
            requirements: Vec::new(),
            expected_effort: 0.0,
            required_capabilities: Vec::new(),
            expected_value: 0.0,
            deadline: None,
        }
    }
}

/// GitHub Issues opportunity adapter (§T12.5).
pub struct GitHubIssuesAdapter {
    source: String,
}

impl GitHubIssuesAdapter {
    pub fn new(source: String) -> Self {
        Self { source }
    }
}

impl OpportunityAdapter for GitHubIssuesAdapter {
    fn name(&self) -> &str {
        "github_issues"
    }

    fn fetch(&self) -> Vec<Opportunity> {
        if self.source.trim().is_empty() {
            Vec::new()
        } else {
            vec![self.parse(&self.source)]
        }
    }

    fn parse(&self, raw: &str) -> Opportunity {
        if let Ok(mut opportunity) = serde_json::from_str::<Opportunity>(raw) {
            opportunity.source_type = ExternalSource::GitHubIssues;
            return opportunity;
        }
        let title: String = raw.trim().chars().take(80).collect();
        Opportunity {
            id: format!("github_{}", uuid::Uuid::new_v4()),
            title,
            source_type: ExternalSource::GitHubIssues,
            requirements: Vec::new(),
            expected_effort: 0.0,
            required_capabilities: Vec::new(),
            expected_value: 0.0,
            deadline: None,
        }
    }
}

/// Intake decision (§17).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntakeDecision {
    Accept,
    Reject,
    Defer,
}

/// Intake result.
#[derive(Clone, Serialize, Deserialize)]
pub struct IntakeResult {
    pub opportunity_id: String,
    pub opportunity: Opportunity,
    pub capability_comparison: crate::cooboploop::capability::CapabilityComparison,
    pub resource_assessment: ResourceAssessment,
    pub risk_assessment: RiskAssessment,
    pub value_assessment: ValueAssessment,
    pub decision: IntakeDecision,
    pub reason: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResourceAssessment {
    pub required_memory_mb: u64,
    pub available_memory_mb: u64,
    pub required_storage_gb: f64,
    pub available_storage_gb: f64,
    pub sufficient: bool,
    pub reason: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub score: f32,
    pub factors: Vec<String>,
    pub acceptable: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ValueAssessment {
    pub expected_value: f32,
    pub expected_effort: f32,
    pub value_ratio: f32,
    pub worthwhile: bool,
}

/// Opportunity intake pipeline (§T12.6 — T12.15).
pub struct OpportunityIntake {
    pub default_policy_never_auto_accept: bool,
}

impl OpportunityIntake {
    pub fn new() -> Self {
        Self {
            default_policy_never_auto_accept: true, // §T12.15
        }
    }

    fn understand(&self, opportunity: &Opportunity) -> Opportunity {
        let mut understood = opportunity.clone();
        if understood.requirements.is_empty() {
            understood.requirements = understood
                .title
                .split([',', ';', '|'])
                .map(str::trim)
                .filter(|requirement| !requirement.is_empty())
                .map(str::to_string)
                .collect();
        }

        let searchable_text =
            format!("{} {}", understood.title, understood.requirements.join(" ")).to_lowercase();
        for capability_name in ["rust", "mcp", "http", "sqlite", "testing"] {
            let capability =
                crate::cooboploop::capability::CapabilityId::from_string(capability_name);
            if searchable_text.contains(capability_name)
                && !understood.required_capabilities.contains(&capability)
            {
                understood.required_capabilities.push(capability);
            }
        }

        understood
    }

    fn estimate(&self, opportunity: &mut Opportunity) {
        let requirement_weight = opportunity.requirements.len().max(1) as f32;
        let capability_weight = opportunity.required_capabilities.len() as f32 * 0.5;
        if opportunity.expected_effort <= 0.0 {
            opportunity.expected_effort = requirement_weight + capability_weight;
        }
        if opportunity.expected_value <= 0.0 {
            opportunity.expected_value = requirement_weight + capability_weight;
        }
    }

    fn decide(
        &self,
        opportunity: &Opportunity,
        has_capability_gap: bool,
        resources_sufficient: bool,
        risk: &RiskAssessment,
        value: &ValueAssessment,
    ) -> (IntakeDecision, String) {
        if opportunity
            .deadline
            .is_some_and(|deadline| deadline < chrono::Utc::now())
        {
            return (
                IntakeDecision::Reject,
                "Rejected because the opportunity deadline has passed".to_string(),
            );
        }
        if risk.score >= 0.9 {
            return (
                IntakeDecision::Reject,
                "Rejected because assessed risk is critically high".to_string(),
            );
        }
        if value.value_ratio < 0.1 {
            return (
                IntakeDecision::Reject,
                "Rejected because expected value is negligible relative to effort".to_string(),
            );
        }
        if has_capability_gap || !resources_sufficient || !risk.acceptable || !value.worthwhile {
            return (
                IntakeDecision::Defer,
                "Deferred until capability, resource, risk, or value constraints improve"
                    .to_string(),
            );
        }
        if self.default_policy_never_auto_accept {
            return (
                IntakeDecision::Defer,
                "Deferred for mandatory human review".to_string(),
            );
        }
        (
            IntakeDecision::Accept,
            "Accepted because all checks passed and policy permits automatic acceptance"
                .to_string(),
        )
    }

    fn value_check(&self, opportunity: &Opportunity) -> ValueAssessment {
        let value_ratio = if opportunity.expected_effort > 0.0 {
            opportunity.expected_value / opportunity.expected_effort
        } else {
            0.0
        };
        let worthwhile = value_ratio.is_finite() && value_ratio >= 0.5;
        ValueAssessment {
            expected_value: opportunity.expected_value,
            expected_effort: opportunity.expected_effort,
            value_ratio,
            worthwhile,
        }
    }

    fn risk_check(
        &self,
        opportunity: &Opportunity,
        capabilities: &crate::cooboploop::capability::CapabilityComparison,
        resources: &ResourceAssessment,
    ) -> RiskAssessment {
        let mut score = 0.0_f32;
        let mut factors = Vec::new();
        if !capabilities.unavailable.is_empty() {
            score += 0.4;
            factors.push("One or more required capabilities are unavailable".to_string());
        }
        if !capabilities.insufficient.is_empty() {
            score += 0.3;
            factors.push("One or more required capabilities are insufficient".to_string());
        }
        if !capabilities.uncertain.is_empty() {
            score += 0.1;
            factors.push("One or more required capabilities are uncertain".to_string());
        }
        if !resources.sufficient {
            score += 0.3;
            factors.push(resources.reason.clone());
        }
        if opportunity
            .deadline
            .is_some_and(|deadline| deadline < chrono::Utc::now())
        {
            score += 0.2;
            factors.push("Opportunity deadline has already passed".to_string());
        }
        let bounded_score = score.clamp(0.0, 1.0);
        RiskAssessment {
            score: bounded_score,
            factors,
            acceptable: bounded_score <= 0.6,
        }
    }

    fn resource_check(&self, opportunity: &Opportunity) -> ResourceAssessment {
        let required_memory_mb = (opportunity.expected_effort.ceil() as u64)
            .saturating_mul(64)
            .max(128);
        let required_storage_gb = (f64::from(opportunity.expected_effort) * 0.05).max(0.05);
        let mut discovery = crate::cooboploop::hardware::HardwareDiscovery::new();
        let Some(profile) = discovery.detect() else {
            return ResourceAssessment {
                required_memory_mb,
                available_memory_mb: 0,
                required_storage_gb,
                available_storage_gb: 0.0,
                sufficient: false,
                reason: "Hardware availability could not be detected".to_string(),
            };
        };
        let telemetry_available =
            profile.memory_available_mb > 0 && profile.storage_available_gb > 0.0;
        let sufficient = telemetry_available
            && profile.memory_available_mb >= required_memory_mb
            && profile.storage_available_gb >= required_storage_gb;
        let reason = if !telemetry_available {
            "Memory or storage availability is unknown".to_string()
        } else if sufficient {
            "Available memory and storage satisfy the estimated effort".to_string()
        } else {
            "Estimated effort exceeds available memory or storage".to_string()
        };
        ResourceAssessment {
            required_memory_mb,
            available_memory_mb: profile.memory_available_mb,
            required_storage_gb,
            available_storage_gb: profile.storage_available_gb,
            sufficient,
            reason,
        }
    }

    /// Process an opportunity through the full pipeline.
    pub fn process(
        &self,
        opp: &Opportunity,
        registry: &crate::cooboploop::capability::CapabilityRegistry,
    ) -> IntakeResult {
        // T12.7: parse — already done (raw -> Opportunity)
        let mut understood = self.understand(opp);
        self.estimate(&mut understood);
        // T12.10: capability_check
        let cap_ids: Vec<crate::cooboploop::capability::CapabilityId> =
            understood.required_capabilities.clone();
        let cap_compare = registry.compare_capabilities(&cap_ids);
        // T12.11: resource_check — assumed OK if no specific system pressure
        // T12.12: risk_check — based on capability gaps
        let has_gap = !cap_compare.insufficient.is_empty() || !cap_compare.unavailable.is_empty();
        let resource_assessment = self.resource_check(&understood);
        let sufficient_resources = resource_assessment.sufficient;
        let risk_assessment = self.risk_check(&understood, &cap_compare, &resource_assessment);
        let acceptable_risk = risk_assessment.acceptable;
        let value_assessment = self.value_check(&understood);
        let value_ratio = value_assessment.value_ratio;
        let worthwhile = value_assessment.worthwhile;
        let (decision, decision_reason) = self.decide(
            &understood,
            has_gap,
            sufficient_resources,
            &risk_assessment,
            &value_assessment,
        );
        IntakeResult {
            opportunity_id: understood.id.clone(),
            opportunity: understood,
            capability_comparison: cap_compare,
            resource_assessment,
            risk_assessment,
            value_assessment,
            decision,
            reason: format!(
                "{}; value_ratio={:.2}, worthwhile={}, has_capability_gap={}, resources_sufficient={}, risk_acceptable={}",
                decision_reason,
                value_ratio,
                worthwhile,
                has_gap,
                sufficient_resources,
                acceptable_risk
            ),
        }
    }
}

impl Default for OpportunityIntake {
    fn default() -> Self {
        Self::new()
    }
}
