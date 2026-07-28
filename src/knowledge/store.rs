// src/knowledge/store.rs

//! Knowledge store - repository for managing knowledge items


use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::types::{KnowledgeItem, KnowledgeStatus, KnowledgeType, RelationType, KnowledgeDependency, DependencyType, KnowledgeVersionInfo};

/// Knowledge store - manages all knowledge items
pub struct KnowledgeStore {
    /// All knowledge items by ID
    items: Arc<RwLock<HashMap<Uuid, KnowledgeItem>>>,
    
    /// Index by type for fast lookup
    by_type: Arc<RwLock<HashMap<KnowledgeType, Vec<Uuid>>>>,
    
    /// Index by tag for fast lookup
    by_tag: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    
    /// Dependency index: knowledge_id -> list of dependencies
    dependencies: Arc<RwLock<HashMap<Uuid, Vec<KnowledgeDependency>>>>,
    
    /// Reverse dependency index: knowledge_id -> list of dependents
    dependents: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    
    /// Version info for each knowledge item
    versions: Arc<RwLock<HashMap<Uuid, KnowledgeVersionInfo>>>,
    
    /// Maximum items to retain
    max_items: usize,
}

impl KnowledgeStore {
    /// Create a new knowledge store
    pub fn new(max_items: usize) -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
            by_type: Arc::new(RwLock::new(HashMap::new())),
            by_tag: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            dependents: Arc::new(RwLock::new(HashMap::new())),
            versions: Arc::new(RwLock::new(HashMap::new())),
            max_items,
        }
    }

    /// Add a new knowledge item
    pub async fn add(&self, item: KnowledgeItem) -> Uuid {
        let id = item.id;
        
        // Add to main store
        let mut items = self.items.write().await;
        items.insert(id, item.clone());
        
        // Update type index
        {
            let mut by_type = self.by_type.write().await;
            by_type
                .entry(item.knowledge_type.clone())
                .or_default()
                .push(id);
        }
        
        // Update tag index
        {
            let mut by_tag = self.by_tag.write().await;
            for tag in &item.tags {
                by_tag.entry(tag.clone()).or_default().push(id);
            }
        }
        
        // Enforce max items - remove lowest confidence if over limit
        if items.len() > self.max_items {
            drop(items); // Release lock before recursive call
            self.prune_low_confidence().await;
        }
        
        tracing::info!("[Knowledge] Added knowledge item: {}", id);
        id
    }

    /// Get a knowledge item by ID
    pub async fn get(&self, id: Uuid) -> Option<KnowledgeItem> {
        self.items.read().await.get(&id).cloned()
    }

    /// Update a knowledge item
    pub async fn update(&self, mut item: KnowledgeItem) -> bool {
        item.updated_at = chrono::Utc::now();
        let mut items = self.items.write().await;
        if items.contains_key(&item.id) {
            items.insert(item.id, item);
            true
        } else {
            false
        }
    }

    /// Delete a knowledge item
    pub async fn delete(&self, id: Uuid) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.remove(&id) {
            // Clean up indices
            {
                let mut by_type = self.by_type.write().await;
                if let Some(ids) = by_type.get_mut(&item.knowledge_type) {
                    ids.retain(|i| *i != id);
                }
            }
            {
                let mut by_tag = self.by_tag.write().await;
                for tag in &item.tags {
                    if let Some(ids) = by_tag.get_mut(tag) {
                        ids.retain(|i| *i != id);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Get all knowledge items
    pub async fn get_all(&self) -> Vec<KnowledgeItem> {
        self.items.read().await.values().cloned().collect()
    }

    /// Get knowledge items by type
    pub async fn get_by_type(&self, knowledge_type: &KnowledgeType) -> Vec<KnowledgeItem> {
        let by_type = self.by_type.read().await;
        let items = self.items.read().await;
        by_type
            .get(knowledge_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| items.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get knowledge items by tag
    pub async fn get_by_tag(&self, tag: &str) -> Vec<KnowledgeItem> {
        let by_tag = self.by_tag.read().await;
        let items = self.items.read().await;
        by_tag
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| items.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get active knowledge items (ready for use)
    pub async fn get_active(&self) -> Vec<KnowledgeItem> {
        self.items
            .read()
            .await
            .values()
            .filter(|item| item.status == KnowledgeStatus::Active)
            .cloned()
            .collect()
    }

    /// Get mature knowledge items (high confidence)
    pub async fn get_mature(&self) -> Vec<KnowledgeItem> {
        self.items
            .read()
            .await
            .values()
            .filter(|item| item.is_mature())
            .cloned()
            .collect()
    }

    /// Get knowledge items needing review
    pub async fn get_needing_review(&self) -> Vec<KnowledgeItem> {
        self.items
            .read()
            .await
            .values()
            .filter(|item| item.needs_review())
            .cloned()
            .collect()
    }

    /// Search knowledge by statement content
    pub async fn search(&self, query: &str) -> Vec<KnowledgeItem> {
        let query_lower = query.to_lowercase();
        self.items
            .read()
            .await
            .values()
            .filter(|item| item.statement.to_lowercase().contains(&query_lower))
            .cloned()
            .collect()
    }

    /// Get related knowledge
    pub async fn get_related(&self, id: Uuid) -> Vec<KnowledgeItem> {
        if let Some(item) = self.items.read().await.get(&id) {
            let items = self.items.read().await;
            item.relations
                .iter()
                .filter_map(|rel| items.get(&rel.target_id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Add relation between two knowledge items
    pub async fn add_relation(
        &self,
        source_id: Uuid,
        target_id: Uuid,
        relation_type: RelationType,
        strength: f32,
    ) -> bool {
        let mut items = self.items.write().await;
        
        if let Some(source) = items.get_mut(&source_id) {
            source.relations.push(super::types::KnowledgeRelation {
                target_id,
                relation_type,
                strength,
            });
            true
        } else {
            false
        }
    }

    /// Record successful application of knowledge
    pub async fn record_success(&self, id: Uuid) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(&id) {
            item.record_success();
            true
        } else {
            false
        }
    }

    /// Record failed application of knowledge
    pub async fn record_failure(&self, id: Uuid) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(&id) {
            item.record_failure();
            true
        } else {
            false
        }
    }

    /// Promote item to active status
    pub async fn activate(&self, id: Uuid) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(&id) {
            if item.status == KnowledgeStatus::New || item.status == KnowledgeStatus::Validating {
                item.status = KnowledgeStatus::Active;
                item.updated_at = chrono::Utc::now();
                return true;
            }
        }
        false
    }

    /// Suspend knowledge (needs re-evaluation)
    pub async fn suspend(&self, id: Uuid) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(&id) {
            item.status = KnowledgeStatus::Suspended;
            item.updated_at = chrono::Utc::now();
            return true;
        }
        false
    }

    /// Disprove knowledge (remove from active use)
    pub async fn disprove(&self, id: Uuid) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(&id) {
            item.status = KnowledgeStatus::Disproven;
            item.updated_at = chrono::Utc::now();
            return true;
        }
        false
    }

    /// Prune low-confidence items when over capacity
    async fn prune_low_confidence(&self) {
        let mut items = self.items.write().await;
        
        // Sort by confidence, keep highest
        let mut sorted: Vec<_> = items.values().collect();
        sorted.sort_by(|a, b| {
            b.overall_confidence()
                .partial_cmp(&a.overall_confidence())
                .unwrap()
        });
        
        // Remove bottom 10% - collect IDs first to avoid borrow issues
        let to_remove = sorted.len() / 10;
        let ids_to_remove: Vec<_> = sorted.into_iter()
            .take(to_remove)
            .map(|item| item.id)
            .collect();
        
        for id in ids_to_remove {
            items.remove(&id);
        }
        
        tracing::info!("[Knowledge] Pruned {} low-confidence items", to_remove);
    }

    /// Get knowledge statistics
    pub async fn stats(&self) -> KnowledgeStats {
        let items = self.items.read().await;
        let total = items.len();
        let active = items.values().filter(|i| i.status == KnowledgeStatus::Active).count();
        let mature = items.values().filter(|i| i.is_mature()).count();
        let needs_review = items.values().filter(|i| i.needs_review()).count();
        
        let avg_confidence: f32 = if total > 0 {
            items.values().map(|i| i.overall_confidence()).sum::<f32>() / total as f32
        } else {
            0.0
        };
        
        KnowledgeStats {
            total,
            active,
            mature,
            needs_review,
            average_confidence: avg_confidence,
        }
    }

    // ========================================================================
    // DEPENDENCY TRACKING
    // ========================================================================

    /// Add a dependency to a knowledge item
    /// 
    /// Per Architecture: Knowledge dependencies track which knowledge items
    /// rely on other knowledge items to function correctly.
    pub async fn add_dependency(
        &self,
        knowledge_id: Uuid,
        depends_on_id: Uuid,
        dependency_type: DependencyType,
    ) -> bool {
        // Verify both items exist
        {
            let items = self.items.read().await;
            if !items.contains_key(&knowledge_id) || !items.contains_key(&depends_on_id) {
                return false;
            }
        }
        
        let dependency = KnowledgeDependency::new(knowledge_id, depends_on_id, dependency_type);
        
        // Add to dependency index
        {
            let mut deps = self.dependencies.write().await;
            deps.entry(knowledge_id)
                .or_insert_with(Vec::new)
                .push(dependency);
        }
        
        // Add to reverse index (dependents)
        {
            let mut rev = self.dependents.write().await;
            rev.entry(depends_on_id)
                .or_insert_with(Vec::new)
                .push(knowledge_id);
        }
        
        tracing::debug!("Added dependency: {} -> {}", knowledge_id, depends_on_id);
        true
    }
    
    /// Remove a dependency
    pub async fn remove_dependency(&self, knowledge_id: &Uuid, depends_on_id: &Uuid) -> bool {
        let mut deps = self.dependencies.write().await;
        let mut rev = self.dependents.write().await;
        
        // Remove from dependency index
        if let Some(d) = deps.get_mut(knowledge_id) {
            d.retain(|dep| dep.dependency_id != *depends_on_id);
        }
        
        // Remove from reverse index
        if let Some(r) = rev.get_mut(depends_on_id) {
            r.retain(|&id| id != *knowledge_id);
        }
        
        true
    }
    
    /// Get all dependencies for a knowledge item
    pub async fn get_dependencies(&self, knowledge_id: &Uuid) -> Vec<KnowledgeDependency> {
        let deps = self.dependencies.read().await;
        deps.get(knowledge_id).cloned().unwrap_or_default()
    }
    
    /// Get all items that depend on this knowledge item
    pub async fn get_dependents(&self, knowledge_id: &Uuid) -> Vec<Uuid> {
        let rev = self.dependents.read().await;
        rev.get(knowledge_id).cloned().unwrap_or_default()
    }
    
    /// Check if a knowledge item's dependencies are satisfied
    pub async fn check_dependencies(&self, knowledge_id: &Uuid) -> DependencyCheckResult {
        let deps = self.dependencies.read().await;
        let items = self.items.read().await;
        
        let mut result = DependencyCheckResult {
            satisfied: Vec::new(),
            unsatisfied: Vec::new(),
            conflicts: Vec::new(),
        };
        
        if let Some(deps) = deps.get(knowledge_id) {
            for dep in deps {
                match dep.dependency_type {
                    DependencyType::Required => {
                        if items.contains_key(&dep.dependency_id) {
                            // Check version constraint if present
                            if let Some(ref constraint) = dep.version_constraint {
                                if let Some(versions) = self.versions.read().await.get(&dep.dependency_id) {
                                    if let Some(ver) = versions.get_active() {
                                        if ver.satisfies_constraint(constraint) {
                                            result.satisfied.push(dep.dependency_id);
                                        } else {
                                            result.unsatisfied.push(dep.dependency_id);
                                        }
                                    } else {
                                        result.unsatisfied.push(dep.dependency_id);
                                    }
                                } else {
                                    result.satisfied.push(dep.dependency_id); // No version info, assume satisfied
                                }
                            } else {
                                result.satisfied.push(dep.dependency_id);
                            }
                        } else {
                            result.unsatisfied.push(dep.dependency_id);
                        }
                    },
                    DependencyType::Conflict => {
                        if items.contains_key(&dep.dependency_id) {
                            result.conflicts.push(dep.dependency_id);
                        }
                    },
                    _ => {
                        // Optional dependencies are always satisfied if present
                        if items.contains_key(&dep.dependency_id) {
                            result.satisfied.push(dep.dependency_id);
                        }
                    }
                }
            }
        }
        
        result
    }
    
    /// Get all items that would be affected if this knowledge item changed
    pub async fn get_impact_set(&self, knowledge_id: &Uuid) -> Vec<Uuid> {
        let mut impact = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![*knowledge_id];
        
        while let Some(current) = queue.pop() {
            if visited.insert(current) {
                let dependents = self.get_dependents(&current).await;
                for dep in dependents {
                    if !visited.contains(&dep) {
                        impact.push(dep);
                        queue.push(dep);
                    }
                }
            }
        }
        
        impact
    }
    
    /// Validate all dependencies in the knowledge store
    pub async fn validate_all_dependencies(&self) -> Vec<DependencyValidation> {
        let mut validations = Vec::new();
        let items = self.items.read().await;
        
        for id in items.keys() {
            let result = self.check_dependencies(id).await;
            if !result.unsatisfied.is_empty() || !result.conflicts.is_empty() {
                validations.push(DependencyValidation {
                    knowledge_id: *id,
                    check_result: result,
                });
            }
        }
        
        validations
    }

    // ========================================================================
    // VERSION TRACKING
    // ========================================================================

    /// Initialize version tracking for a knowledge item
    pub async fn init_version(&self, knowledge_id: Uuid, initial_version: &str) -> bool {
        let items = self.items.read().await;
        if !items.contains_key(&knowledge_id) {
            return false;
        }
        drop(items);
        
        let version_info = KnowledgeVersionInfo::new(initial_version);
        let mut versions = self.versions.write().await;
        versions.insert(knowledge_id, version_info);
        
        tracing::debug!("Initialized version {} for knowledge {}", initial_version, knowledge_id);
        true
    }
    
    /// Create a new version of a knowledge item
    pub async fn create_version(
        &self,
        knowledge_id: Uuid,
        version: &str,
        changelog: &str,
        new_confidence: f32,
    ) -> bool {
        // Update version info
        {
            let mut versions = self.versions.write().await;
            if let Some(info) = versions.get_mut(&knowledge_id) {
                info.create_version(version, changelog, new_confidence);
            } else {
                // Initialize if not exists
                let mut info = KnowledgeVersionInfo::new(version);
                info.create_version(version, changelog, new_confidence);
                versions.insert(knowledge_id, info);
            }
        }
        
        // Update the knowledge item's confidence
        {
            let mut items = self.items.write().await;
            if let Some(item) = items.get_mut(&knowledge_id) {
                item.confidence.adjust_source_reliability(new_confidence - item.overall_confidence());
            }
        }
        
        tracing::info!("Created version {} for knowledge {}", version, knowledge_id);
        true
    }
    
    /// Get version info for a knowledge item
    pub async fn get_version_info(&self, knowledge_id: &Uuid) -> Option<KnowledgeVersionInfo> {
        let versions = self.versions.read().await;
        versions.get(knowledge_id).cloned()
    }
    
    /// Bump version number for a knowledge item
    pub async fn bump_version(&self, knowledge_id: &Uuid, bump_type: VersionBumpType) -> bool {
        let mut versions = self.versions.write().await;
        if let Some(info) = versions.get_mut(knowledge_id) {
            let current = info.current_version.clone();
            match bump_type {
                VersionBumpType::Major => info.bump_major(),
                VersionBumpType::Minor => info.bump_minor(),
                VersionBumpType::Patch => info.bump_patch(),
            }
            tracing::info!("Bumped version {} -> {} for knowledge {}", current, info.current_version, knowledge_id);
            true
        } else {
            // Initialize with 0.0.1
            let mut info = KnowledgeVersionInfo::new("0.0.1");
            match bump_type {
                VersionBumpType::Major => info.bump_major(),
                VersionBumpType::Minor => info.bump_minor(),
                VersionBumpType::Patch => info.bump_patch(),
            }
            versions.insert(*knowledge_id, info);
            true
        }
    }
}

/// Result of checking dependencies
#[derive(Debug, Clone)]
pub struct DependencyCheckResult {
    pub satisfied: Vec<Uuid>,
    pub unsatisfied: Vec<Uuid>,
    pub conflicts: Vec<Uuid>,
}

/// Validation result for a dependency
#[derive(Debug, Clone)]
pub struct DependencyValidation {
    pub knowledge_id: Uuid,
    pub check_result: DependencyCheckResult,
}

/// Type of version bump
#[derive(Debug, Clone, Copy)]
pub enum VersionBumpType {
    Major,
    Minor,
    Patch,
}

/// Knowledge statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct KnowledgeStats {
    pub total: usize,
    pub active: usize,
    pub mature: usize,
    pub needs_review: usize,
    pub average_confidence: f32,
}

impl Default for KnowledgeStore {
    fn default() -> Self {
        Self::new(10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get() {
        let store = KnowledgeStore::new(100);
        let item = KnowledgeItem::from_reflection("Test knowledge", 0.8, Uuid::new_v4());
        let id = store.add(item.clone()).await;
        
        let retrieved = store.get(id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().statement, "Test knowledge");
    }

    #[tokio::test]
    async fn test_get_mature() {
        let store = KnowledgeStore::new(100);
        
        // Add items with varying confidence
        let mut item1 = KnowledgeItem::from_reflection("Low confidence", 0.3, Uuid::new_v4());
        item1.status = KnowledgeStatus::Active;
        
        let mut item2 = KnowledgeItem::from_reflection("High confidence", 0.8, Uuid::new_v4());
        item2.status = KnowledgeStatus::Active;
        
        store.add(item1).await;
        let _id2 = store.add(item2).await;
        
        let mature = store.get_mature().await;
        assert_eq!(mature.len(), 1);
        assert_eq!(mature[0].statement, "High confidence");
    }
}
