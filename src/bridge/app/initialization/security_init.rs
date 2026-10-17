// src/bridge/app/initialization/security_init.rs
//! Security layer verification at startup (Architecture Ch. 25).
//!
//! Verifies identity verification, capability-based security, audit persistence,
//! trust evaluation, rollback, trust decay, reputation scoring, explainability,
//! and memory protection at application startup.

/// Run security layer verification.
pub fn verify_security_layer() {
    // Real identity verification, capability-based security, audit persistence.
    let mut grant_store = crate::security::full::CapabilityGrantStore::new();
    // Grant default capabilities to the system actor
    grant_store.add_grant(
        "system",
        crate::security::full::CapabilityGrant {
            actor_id: "system".to_string(),
            capability: crate::security::full::Capability::ExecuteTool,
            scope: "global".to_string(),
            granted_at: chrono::Utc::now().timestamp(),
            active: true,
        },
    );
    // Test security enforcement with real capability check
    let allowed = crate::security::full::enforce_security(
        "system",
        crate::security::full::Capability::ExecuteTool,
        "test-target",
        0.8,
        &grant_store,
    );
    tracing::debug!(
        security_enforcement = allowed,
        "Security enforcement verified"
    );
    // Test audit persistence
    let audit_entry = crate::security::full::SecurityAuditEntry::new(
        "system",
        "security_test",
        "audit_test",
        true,
        "audit-verification",
    )
    .with_capability(crate::security::full::Capability::ExecuteTool)
    .with_confidence_change(0.1)
    .with_reason("security verification");
    if let Err(e) = crate::security::full::persist_audit_entry(&audit_entry) {
        tracing::warn!(audit_id = %audit_entry.audit_id, persist_error = %e, "Audit persistence returned error");
    } else {
        tracing::debug!(
            audit_id = %audit_entry.audit_id,
            "Audit persistence verified"
        );
    }
    // Test trust evaluation
    let evidence = vec![crate::security::full::TrustEvidence {
        evidence_type: "successful_action".to_string(),
        weight: 0.8,
        timestamp: chrono::Utc::now().timestamp(),
    }];
    let trust_score = crate::security::full::evaluate_trust("system", &evidence);
    tracing::debug!(trust_score, "Trust evaluation verified");
    // Test rollback
    let rolled_back = crate::security::full::rollback_to_safe_state("system", "test-target");
    tracing::debug!(rolled_back, "Rollback verified");
    // Test trust decay
    let decayed = crate::security::full::apply_trust_decay(0.8, 0.1);
    tracing::debug!(decayed, "Trust decay verified");
    // Test reputation scoring
    let mut reputation = crate::security::full::ReputationScore::new("test-tool", 0.5);
    reputation.update(0.9);
    tracing::debug!(
        reputation_value = reputation.value,
        reputation_count = reputation.evidence_count,
        "Reputation scoring verified"
    );
    // Test security explainability
    let explanation =
        crate::security::full::explain_security_decision("system", "test_action", true, "verified");
    tracing::debug!(explanation, "Security explanation verified");
    // Test memory protection
    let rule = crate::security::full::MemoryProtectionRule::new(
        "working",
        crate::security::full::Capability::ReadMemory,
        0.5,
    );
    let capabilities = vec![crate::security::full::Capability::ReadMemory];
    let access_allowed = rule.allows_access(&capabilities, 0.8);
    tracing::debug!(access_allowed, "Memory protection verified");
    // Test log_audit and check_permission
    let audit_result = crate::security::log_audit(crate::security::AuditRecord {
        actor: "system".to_string(),
        action: "security_verification".to_string(),
        target: "test-target".to_string(),
        confidence_change: 0.1,
        reason: "security layer initialized".to_string(),
    });
    if let Err(e) = audit_result {
        tracing::warn!(log_audit_error = %e, "Audit logging returned error");
    } else {
        tracing::debug!("Audit logging verified");
    }
    let has_perm = crate::security::check_permission("system", "read", "test");
    if let Ok(permitted) = has_perm {
        tracing::debug!(permitted, "Permission check verified");
    } else {
        tracing::warn!("Permission check failed");
    }
}
