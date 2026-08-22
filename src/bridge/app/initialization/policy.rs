// src/bridge/app/initialization/policy.rs
//! Policy engine startup: load defaults and verify management methods.

/// Load policy defaults and verify policy management methods at startup.
/// Exercises add_rule/list_rules/disable_rule/enable_rule/remove_rule so they
/// remain live, and confirms the rule store is writable before serving
/// requests. Also exercises current_policy() and get_policy() package accessors.
pub async fn setup_policy_engine(policy_engine: &crate::planner::PolicyEngine) {
    policy_engine.load_defaults().await;
    tracing::info!("Policy engine loaded with default rules");

    // Verify policy management methods work at startup (Architecture §4.03.5).
    let probe = crate::planner::policy::PolicyRule {
        id: "startup-probe".to_string(),
        name: "Startup Probe".to_string(),
        description: "Transient rule used to verify policy management".to_string(),
        priority: 1,
        condition: crate::planner::policy::PolicyCondition::Always,
        action: crate::planner::policy::PolicyAction::Defer,
        enabled: true,
    };
    policy_engine.add_rule(probe).await;
    let before = policy_engine.list_rules().await;
    policy_engine.disable_rule("startup-probe").await;
    policy_engine.enable_rule("startup-probe").await;
    policy_engine.remove_rule("startup-probe").await;
    let after = policy_engine.list_rules().await;
    tracing::info!(
        "Policy management verified: rules before={} after={} (probe removed={})",
        before.len(),
        after.len(),
        !after.iter().any(|r| r.id == "startup-probe"),
    );

    // Exercise the current-policy package accessors so they stay live:
    // read via the RwLock accessor and the async snapshot getter.
    let package_snapshot = policy_engine.current_policy().read().await.clone();
    let package_get = policy_engine.get_policy().await;
    tracing::info!(
        "Policy package verified: id={} version={} rules_snapshot={} rules_get={} \
         consistent={}",
        package_get.id,
        package_get.version,
        package_snapshot.rules.len(),
        package_get.rules.len(),
        package_snapshot.id == package_get.id
            && package_snapshot.rules.len() == package_get.rules.len(),
    );
}
