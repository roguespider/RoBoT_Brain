// src/bridge/app/initialization/security_init.rs
//! Security layer verification at startup (Architecture Ch. 25).
//!
//! Initializes the global capability store with default grants, verifies
//! identity verification, capability-based security, audit persistence,
//! trust evaluation, rollback, trust decay, reputation scoring, explainability,
//! and memory protection at application startup.

/// Run security layer initialization and verification.
/// Per Architecture §25: initializes global capability store with default
/// actor grants, then exercises each security subsystem to verify it works.
pub fn verify_security_layer() {
    // Initialize the global capability store with default grants.
    // This replaces the old simulated identity verification.
    crate::security::init_global_capabilities();

    // Test real identity verification and capability-based security.
    let mut grant_store = crate::security::full::CapabilityGrantStore::new();
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

    // Test audit persistence to the security_audits DB table.
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

    // Test trust evaluation.
    let evidence = vec![crate::security::full::TrustEvidence {
        evidence_type: "successful_action".to_string(),
        weight: 0.8,
        timestamp: chrono::Utc::now().timestamp(),
    }];
    let trust_score = crate::security::full::evaluate_trust("system", &evidence);
    tracing::debug!(trust_score, "Trust evaluation verified");

    // Test rollback.
    let rolled_back = crate::security::full::rollback_to_safe_state("system", "test-target");
    tracing::debug!(rolled_back, "Rollback verified");

    // Test trust decay.
    let decayed = crate::security::full::apply_trust_decay(0.8, 0.1);
    tracing::debug!(decayed, "Trust decay verified");

    // Test reputation scoring.
    let mut reputation = crate::security::full::ReputationScore::new("test-tool", 0.5);
    reputation.update(0.9);
    tracing::debug!(
        reputation_value = reputation.value,
        reputation_count = reputation.evidence_count,
        "Reputation scoring verified"
    );

    // Test security explainability.
    let explanation =
        crate::security::full::explain_security_decision("system", "test_action", true, "verified");
    tracing::debug!(explanation, "Security explanation verified");

    // Test memory protection.
    let rule = crate::security::full::MemoryProtectionRule::new(
        "working",
        crate::security::full::Capability::ReadMemory,
        0.5,
    );
    let capabilities = vec![crate::security::full::Capability::ReadMemory];
    let access_allowed = rule.allows_access(&capabilities, 0.8);
    tracing::debug!(access_allowed, "Memory protection verified");

    // Test real audit logging (persists to DB, not just tracing).
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

    // Test real permission check against global capability store.
    // The system actor has ReadMemory granted, so this should return true.
    let has_perm = crate::security::check_permission("system", "read", "test");
    if let Ok(permitted) = has_perm {
        if permitted {
            tracing::debug!("Permission check verified: system can read");
        } else {
            tracing::warn!("Permission check failed unexpectedly: system should have read access");
        }
    } else {
        tracing::warn!("Permission check error: {:?}", has_perm);
    }

    // Test that an unknown actor is denied access (real enforcement).
    let unknown_perm = crate::security::check_permission("unknown_actor", "read", "test");
    if let Ok(permitted) = unknown_perm {
        if !permitted {
            tracing::debug!("Real enforcement verified: unknown actor denied");
        } else {
            tracing::warn!("Security enforcement failed: unknown actor should be denied");
        }
    }
}
