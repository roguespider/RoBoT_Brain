//! Storage Architecture — Storage layers, storage policies, storage audit,
//! storage versioning (Architecture Chapter 21).
//!
//! Per Architecture §21.1-21.4:
//! - Storage layers: working (hot), session (warm), experience (warm),
//!   semantic (cold), skill (cold), knowledge graph (cold), archive (frozen)
//! - Storage policies: retention, promotion, demotion, cleanup
//! - Storage audit: provenance tracking, version tracking
//! - Storage versioning: schema versions, data versions
//! - Wiring: storage_architecture/ -> database/ -> memory/ + experience/ + knowledge/

/// Storage layer types per Architecture §21.2.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StorageLayer {
    /// Hot storage: working memory, active session.
    Hot,
    /// Warm storage: recent experience, active knowledge.
    Warm,
    /// Cold storage: long-term knowledge, archived experience.
    Cold,
    /// Frozen storage: archived historical data.
    Frozen,
}

impl StorageLayer {
    /// Return layer label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Hot => "Hot",
            Self::Warm => "Warm",
            Self::Cold => "Cold",
            Self::Frozen => "Frozen",
        }
    }

    /// Check if layer supports write operations.
    pub fn supports_write(&self) -> bool {
        matches!(self, Self::Hot | Self::Warm)
    }

    /// Check if layer supports read operations.
    pub fn supports_read(&self) -> bool {
        true
    }
}

/// Storage policy defining retention and promotion rules.
/// Per Architecture §21.3 (Storage Policies).
#[derive(Debug, Clone, PartialEq)]
pub struct StoragePolicy {
    /// Layer this policy applies to.
    pub layer: StorageLayer,
    /// Retention period in days.
    pub retention_days: u32,
    /// Promotion threshold (confidence score).
    pub promotion_threshold: f32,
    /// Demotion threshold (access count).
    pub demotion_access_threshold: u32,
    /// Cleanup enabled.
    pub cleanup_enabled: bool,
}

impl StoragePolicy {
    /// Create a new storage policy.
    pub fn new(layer: StorageLayer, retention_days: u32, promotion_threshold: f32) -> Self {
        Self {
            layer,
            retention_days,
            promotion_threshold,
            demotion_access_threshold: 3,
            cleanup_enabled: true,
        }
    }

    /// Check if data should be promoted.
    pub fn should_promote(&self, confidence: f32) -> bool {
        confidence >= self.promotion_threshold
    }

    /// Check if data should be demoted.
    pub fn should_demote(&self, access_count: u32) -> bool {
        access_count < self.demotion_access_threshold
    }
}

/// Storage audit record tracking provenance and versions.
/// Per Architecture §21.3 (Storage Audit) and §21.4 (Storage Versioning).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageAuditRecord {
    /// Audit entry ID.
    pub audit_id: String,
    /// Data identifier.
    pub data_id: String,
    /// Storage layer.
    pub layer: StorageLayer,
    /// Action (store, retrieve, promote, demote, archive, delete).
    pub action: String,
    /// Actor performing action.
    pub actor: String,
    /// Timestamp.
    pub timestamp: i64,
    /// Schema version.
    pub schema_version: String,
    /// Data version.
    pub data_version: String,
    /// Provenance chain.
    pub provenance: Vec<String>,
}

impl StorageAuditRecord {
    /// Create a new storage audit record.
    pub fn new(data_id: &str, layer: StorageLayer, action: &str, actor: &str) -> Self {
        Self {
            audit_id: uuid::Uuid::new_v4().to_string(),
            data_id: data_id.to_string(),
            layer,
            action: action.to_string(),
            actor: actor.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            schema_version: "v0.0.2.1".to_string(),
            data_version: "1.0".to_string(),
            provenance: Vec::new(),
        }
    }

    /// Add provenance entry.
    pub fn add_provenance(&mut self, source: &str) {
        self.provenance.push(source.to_string());
    }
}

/// Storage version tracking.
/// Per Architecture §21.4 (Storage Versioning).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StorageVersion {
    /// Schema version.
    Schema(String),
    /// Data version.
    Data(String),
    /// Migration version.
    Migration(String),
}

impl StorageVersion {
    /// Return version label.
    pub fn label(&self) -> String {
        match self {
            Self::Schema(s) => s.clone(),
            Self::Data(s) => s.clone(),
            Self::Migration(s) => s.clone(),
        }
    }
}

/// Storage layer manager coordinating storage operations.
/// Per Architecture §21.1 (Storage Architecture Overview).
#[derive(Debug, Clone, Default)]
pub struct StorageArchitecture {
    /// Active storage policies by layer.
    policies: std::collections::HashMap<String, StoragePolicy>,
    /// Audit records.
    audit_records: Vec<StorageAuditRecord>,
    /// Storage versions tracked.
    versions: std::collections::HashMap<String, StorageVersion>,
}

impl StorageArchitecture {
    /// Create a new storage architecture manager.
    pub fn new() -> Self {
        Self {
            policies: std::collections::HashMap::new(),
            audit_records: Vec::new(),
            versions: std::collections::HashMap::new(),
        }
    }

    /// Set a storage policy for a layer.
    pub fn set_policy(&mut self, layer_name: &str, policy: StoragePolicy) {
        self.policies.insert(layer_name.to_string(), policy);
    }

    /// Get policy for a layer.
    pub fn get_policy(&self, layer_name: &str) -> Option<&StoragePolicy> {
        self.policies.get(layer_name)
    }

    /// Record a storage audit entry.
    pub fn audit(&mut self, record: StorageAuditRecord) {
        self.audit_records.push(record);
    }

    /// Track a storage version.
    pub fn track_version(&mut self, id: &str, version: StorageVersion) {
        self.versions.insert(id.to_string(), version);
    }

    /// Get audit records for a data item.
    pub fn get_audit_for_data(&self, data_id: &str) -> Vec<&StorageAuditRecord> {
        self.audit_records
            .iter()
            .filter(|r| r.data_id == data_id)
            .collect()
    }

    /// Check if cleanup should run for a layer.
    pub fn should_cleanup(&self, layer_name: &str) -> bool {
        self.get_policy(layer_name)
            .map(|p| p.cleanup_enabled)
            .unwrap_or(false)
    }
}

/// Storage layer initialization.
/// Per Architecture §21.2 (Storage Layers): initialize hot, warm, cold, frozen layers.
pub fn initialize_storage_layers() -> StorageArchitecture {
    let mut architecture = StorageArchitecture::new();
    architecture.set_policy("hot", StoragePolicy::new(StorageLayer::Hot, 7, 0.9));
    architecture.set_policy("warm", StoragePolicy::new(StorageLayer::Warm, 30, 0.7));
    architecture.set_policy("cold", StoragePolicy::new(StorageLayer::Cold, 365, 0.5));
    architecture.set_policy("frozen", StoragePolicy::new(StorageLayer::Frozen, 0, 0.0));
    architecture
}

/// Active reference to storage architecture contracts.
pub fn reference_storage_architecture() {
    let mut arch = initialize_storage_layers();
    let audit = StorageAuditRecord::new("data-1", StorageLayer::Hot, "store", "agent");
    arch.audit(audit);
    arch.track_version("data-1", StorageVersion::Schema("v0.0.2.1".to_string()));
    tracing::debug!(
        policies = arch.policies.len(),
        audit_count = arch.audit_records.len(),
        versions = arch.versions.len(),
        "Storage architecture referenced"
    );
}
