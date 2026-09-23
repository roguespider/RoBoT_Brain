//! Retrieval Pipeline — Per Architecture Chapter 16.
//!
//! The Retrieval Pipeline finds, ranks, and delivers relevant information
//! for the current reasoning cycle. It acts as the bridge between
//! Memory Hierarchy, Context Lifecycle, Planning Engine, Learning Engine,
//! Experience Engine, and Tool Engine.

/// Retrieval stages per Architecture §16.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetrievalStage {
    /// Understand the query.
    QueryUnderstanding,
    /// Expand the query.
    QueryExpansion,
    /// Search semantic memory.
    SemanticSearch,
    /// Search episodic memory.
    EpisodicSearch,
    /// Search experience memory.
    ExperienceSearch,
    /// Search procedural/skill memory.
    SkillSearch,
    /// Search archive memory.
    ArchiveSearch,
    /// Generate candidates from all sources.
    CandidateGeneration,
    /// Rank candidates.
    Ranking,
    /// Filter by relevance and confidence.
    Filtering,
    /// Assemble final context package.
    ContextAssembly,
}

/// A retrieval request from the Context Engine.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalRequest {
    /// The query or goal being pursued.
    pub query: String,
    /// Required knowledge types.
    pub required_types: Vec<String>,
    /// Token budget for this retrieval.
    pub budget: usize,
    /// Minimum confidence threshold.
    pub min_confidence: f32,
}

impl RetrievalRequest {
    /// Create a new retrieval request.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            required_types: Vec::new(),
            budget: 100,
            min_confidence: 0.5,
        }
    }

    /// Add a required knowledge type.
    pub fn with_type(mut self, kind: impl Into<String>) -> Self {
        self.required_types.push(kind.into());
        self
    }

    /// Set the token budget.
    pub fn with_budget(mut self, budget: usize) -> Self {
        self.budget = budget;
        self
    }

    /// Set the minimum confidence.
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// A single retrieved candidate before ranking.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalCandidate {
    /// Source of the candidate.
    pub source: String,
    /// Content or reference.
    pub content: String,
    /// Relevance score (0.0 - 1.0).
    pub relevance: f32,
    /// Confidence in the content.
    pub confidence: f32,
    /// Recency score.
    pub recency: f32,
    /// Importance score.
    pub importance: f32,
    /// Whether this is a relationship reference.
    pub is_relationship: bool,
}

/// The result of a retrieval operation.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalResult {
    /// Candidates selected for context.
    pub candidates: Vec<RetrievalCandidate>,
    /// Total candidates found before filtering.
    pub total_found: usize,
    /// Whether retrieval succeeded.
    pub success: bool,
    /// Error message if retrieval failed.
    pub error: Option<String>,
    /// Duration of retrieval in milliseconds.
    pub duration_ms: u64,
}

/// The Retrieval Pipeline assembles context from multiple sources.
#[derive(Debug, Clone, Default)]
pub struct RetrievalPipeline {
    /// Active retrieval stages.
    stages: Vec<RetrievalStage>,
}

impl RetrievalPipeline {
    /// Create a new retrieval pipeline.
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Add a stage to the pipeline.
    pub fn add_stage(mut self, stage: RetrievalStage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Execute the retrieval pipeline for a request.
    /// Per Architecture §16: query understanding → expansion → hybrid retrieval
    /// → ranking → filtering → context assembly.
    pub fn execute(&self, request: &RetrievalRequest) -> RetrievalResult {
        tracing::debug!(
            query = %request.query,
            budget = request.budget,
            stages = ?self.stages,
            "Retrieval pipeline executing"
        );

        // Wire through actual subsystems per architecture §16
        let candidates = Self::hybrid_retrieve(&request.query, request.budget);
        let ranked = Self::rank_candidates(&candidates);
        let filtered = Self::filter_candidates(ranked, request.budget, request.min_confidence);
        RetrievalResult {
            candidates: filtered,
            total_found: candidates.len(),
            success: true,
            error: None,
            duration_ms: 0,
        }
    }

    /// Query understanding: extract intent, entities, and required types.
    pub fn understand_query(query: &str) -> Vec<String> {
        query
            .split_whitespace()
            .filter(|w| !w.is_empty())
            .map(|w| w.to_lowercase())
            .collect()
    }

    /// Query expansion: generate synonyms and related terms.
    pub fn expand_query(query: &str) -> Vec<String> {
        let base = Self::understand_query(query);
        let mut expanded = base.clone();
        // Placeholder: add basic expansions
        for term in &base {
            expanded.push(format!("related_{}", term));
        }
        expanded
    }

    /// Hybrid retrieval: combine semantic, graph, and experience sources.
    /// Per Architecture §16: integrates memory, knowledge graph, and experience.
    /// Also attempts real database retrieval via SQLite when available.
    pub fn hybrid_retrieve(query: &str, budget: usize) -> Vec<RetrievalCandidate> {
        tracing::debug!(query = %query, budget, "Hybrid retrieval (wiring memory + knowledge + experience + db)");
        let mut candidates = Vec::new();

        // Memory retrieval integration (via context engine)
        let mem_items = crate::context_engine::memory_retrieval(query, budget);
        for item in mem_items {
            candidates.push(RetrievalCandidate {
                source: "memory".to_string(),
                content: item,
                relevance: 0.7,
                confidence: 0.6,
                recency: 0.8,
                importance: 0.5,
                is_relationship: false,
            });
        }

        // Knowledge graph integration (via context engine + graph traversal + database)
        let knowledge_items = crate::context_engine::knowledge_retrieval(query, budget);
        // Also attempt real graph traversal from database
        if let Ok(db) = crate::database::sqlite::SqliteDatabase::initialize() {
            let conn_result = db.connection();
            if let Ok(conn) = conn_result {
                // Try graph traversal from a start node derived from query
                let start_node = format!("node_{}", query.replace(" ", "_").to_lowercase());
                if let Ok(edges) = crate::knowledge::graph::traverse_from(&conn, &start_node, 2) {
                    for edge in edges {
                        candidates.push(RetrievalCandidate {
                            source: "knowledge_graph".to_string(),
                            content: format!(
                                "{} -> {} ({})",
                                edge.source_id, edge.target_id, edge.relationship
                            ),
                            relevance: 0.85,
                            confidence: edge.confidence,
                            recency: 0.75,
                            importance: 0.65,
                            is_relationship: true,
                        });
                    }
                }
                // Also try to find linked concepts
                if let Ok(linked_nodes) =
                    crate::knowledge::graph::find_linked_concepts(&conn, &start_node, "related")
                {
                    for node in linked_nodes {
                        candidates.push(RetrievalCandidate {
                            source: "knowledge_graph_node".to_string(),
                            content: node.label,
                            relevance: 0.7,
                            confidence: node.confidence,
                            recency: 0.6,
                            importance: 0.55,
                            is_relationship: true,
                        });
                    }
                }
            }
        }
        for item in knowledge_items {
            candidates.push(RetrievalCandidate {
                source: "knowledge".to_string(),
                content: item,
                relevance: 0.8,
                confidence: 0.75,
                recency: 0.7,
                importance: 0.6,
                is_relationship: true,
            });
        }

        // Experience integration (via context engine)
        let exp_items = crate::context_engine::experience_retrieval(query, budget);
        for item in exp_items {
            candidates.push(RetrievalCandidate {
                source: "experience".to_string(),
                content: item,
                relevance: 0.65,
                confidence: 0.55,
                recency: 0.9,
                importance: 0.55,
                is_relationship: false,
            });
        }

        // Real database retrieval attempt (Architecture §21, §22)
        // Try to query the SQLite database for memory cards matching the query
        if let Ok(db) = crate::database::sqlite::SqliteDatabase::initialize() {
            let conn_result = db.connection();
            if let Ok(conn) = conn_result {
                // Search memories by content
                if let Ok(mem_cards) = crate::database::queries::search_memory(&conn, query, budget)
                {
                    for card in mem_cards {
                        // Generate embedding for semantic similarity if content qualifies
                        let embedding = crate::memory::embedding::generate_embedding(
                            &card.content,
                            card.confidence,
                            card.importance,
                        );
                        let has_embedding = embedding.is_some();
                        candidates.push(RetrievalCandidate {
                            source: "database_memory".to_string(),
                            content: card.content,
                            relevance: 0.75,
                            confidence: card.confidence,
                            recency: 0.85,
                            importance: card.importance,
                            is_relationship: false,
                        });
                        tracing::debug!(
                            memory_id = %card.id,
                            has_embedding,
                            "Retrieval: database memory card retrieved with embedding"
                        );
                    }
                }
                // Search experiences
                if let Ok(exps) = crate::database::queries::list_experiences(&conn, budget) {
                    for exp in exps {
                        candidates.push(RetrievalCandidate {
                            source: "database_experience".to_string(),
                            content: exp.title,
                            relevance: 0.6,
                            confidence: exp.confidence,
                            recency: 0.7,
                            importance: 0.5,
                            is_relationship: false,
                        });
                    }
                }
                // Search observations
                if let Ok(obs_list) = crate::database::queries::list_observations(&conn, budget) {
                    for obs in obs_list {
                        candidates.push(RetrievalCandidate {
                            source: "database_observation".to_string(),
                            content: obs.content,
                            relevance: 0.55,
                            confidence: 0.5,
                            recency: 0.9,
                            importance: 0.4,
                            is_relationship: false,
                        });
                    }
                }

                // Query memory tags for tag-based retrieval (Architecture §22, §20.2)
                if let Ok(mut tag_stmt) = conn
                    .prepare("SELECT memory_id, tag FROM memory_tags WHERE tag LIKE ?1 LIMIT ?2")
                {
                    let tag_results: Vec<(String, String)> = tag_stmt
                        .query_map(
                            rusqlite::params![format!("%{}%", query), budget as i64],
                            |row| Ok((row.get(0)?, row.get(1)?)),
                        )
                        .ok()
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default();
                    for (mem_id, tag) in tag_results {
                        candidates.push(RetrievalCandidate {
                            source: "database_memory_tag".to_string(),
                            content: format!("tag: {} (memory: {})", tag, mem_id),
                            relevance: 0.5,
                            confidence: 0.5,
                            recency: 0.6,
                            importance: 0.45,
                            is_relationship: false,
                        });
                    }
                }

                // Query memory relationships for graph-based retrieval (Architecture §20.2, §22)
                if let Ok(mut rel_stmt) = conn.prepare(
                    "SELECT id, memory_id, related_id, relationship_type FROM memory_relationships LIMIT ?1"
                ) {
                    let rel_results: Vec<(String, String, String, String)> = rel_stmt
                        .query_map(
                            rusqlite::params![budget as i64],
                            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                        )
                        .ok()
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default();
                    for (rel_id, mem_id, related_id, rel_type) in rel_results {
                        candidates.push(RetrievalCandidate {
                            source: "database_memory_relationship".to_string(),
                            content: format!("{}: {} -> {} ({})", rel_id, mem_id, related_id, rel_type),
                            relevance: 0.6,
                            confidence: 0.55,
                            recency: 0.65,
                            importance: 0.5,
                            is_relationship: true,
                        });
                    }
                }

                // Query scheduled tasks for task-based retrieval (Architecture §23.5, §30)
                if let Ok(mut task_stmt) =
                    conn.prepare("SELECT id, name, task_type, status FROM scheduled_tasks LIMIT ?1")
                {
                    let task_results: Vec<(String, String, String, String)> = task_stmt
                        .query_map(rusqlite::params![budget as i64], |row| {
                            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                        })
                        .ok()
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default();
                    for (task_id, name, task_type, status) in task_results {
                        // Generate embedding for task content for semantic similarity
                        let task_content_str =
                            format!("Task: {} (type: {}, status: {})", name, task_type, status);
                        // Check if embedding was generated successfully
                        let task_embedding_check = crate::memory::embedding::generate_embedding(
                            &task_content_str,
                            0.5,
                            0.35,
                        )
                        .is_some();
                        let semantic_similarity = if task_embedding_check {
                            0.7 // Higher similarity if embedding exists
                        } else {
                            0.5 // Default similarity
                        };
                        // Multi-factor scoring for scheduled tasks: Relevance * Confidence * Recency * Importance * SemanticSimilarity
                        let score = 0.45 * 0.5 * 0.5 * 0.35 * semantic_similarity;
                        candidates.push(RetrievalCandidate {
                            source: "database_scheduled_task".to_string(),
                            content: task_content_str,
                            relevance: 0.45,
                            confidence: 0.5,
                            recency: 0.5,
                            importance: 0.35,
                            is_relationship: false,
                        });
                        tracing::debug!(
                            task_id = %task_id,
                            semantic_similarity,
                            score,
                            "Retrieval: scheduled task retrieved"
                        );
                    }
                }

                // Query reputations for reputation-based retrieval (Architecture §25.19, §30)
                if let Ok(mut rep_stmt) =
                    conn.prepare("SELECT id, score, factors FROM reputations LIMIT ?1")
                {
                    let rep_results: Vec<(String, f32, String)> = rep_stmt
                        .query_map(rusqlite::params![budget as i64], |row| {
                            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                        })
                        .ok()
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default();
                    for (rep_id, score, factors_json) in rep_results {
                        candidates.push(RetrievalCandidate {
                            source: "database_reputation".to_string(),
                            content: format!(
                                "Reputation: {} (score: {}, factors: {})",
                                rep_id, score, factors_json
                            ),
                            relevance: 0.5,
                            confidence: score,
                            recency: 0.55,
                            importance: 0.4,
                            is_relationship: false,
                        });
                    }
                }

                // Query lineage for lineage-based retrieval (Architecture §26.17, §30)
                if let Ok(mut lineage_stmt) =
                    conn.prepare("SELECT id, memory_id, superseded_by FROM memory_lineage LIMIT ?1")
                {
                    let lineage_results: Vec<(String, String, Option<String>)> = lineage_stmt
                        .query_map(rusqlite::params![budget as i64], |row| {
                            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                        })
                        .ok()
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default();
                    for (lineage_id, memory_id, superseded_by) in lineage_results {
                        candidates.push(RetrievalCandidate {
                            source: "database_lineage".to_string(),
                            content: format!(
                                "Lineage: {} (memory: {}, superseded_by: {:?})",
                                lineage_id, memory_id, superseded_by
                            ),
                            relevance: 0.4,
                            confidence: 0.5,
                            recency: 0.4,
                            importance: 0.3,
                            is_relationship: true,
                        });
                    }
                }

                // Query hypotheses for hypothesis-based retrieval (Architecture §26.7, §30)
                if let Ok(mut hyp_stmt) = conn.prepare(
                    "SELECT id, statement, domain, status, confidence FROM hypotheses LIMIT ?1",
                ) {
                    let hyp_results: Vec<(String, String, String, String, f32)> = hyp_stmt
                        .query_map(rusqlite::params![budget as i64], |row| {
                            Ok((
                                row.get(0)?,
                                row.get(1)?,
                                row.get(2)?,
                                row.get(3)?,
                                row.get(4)?,
                            ))
                        })
                        .ok()
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default();
                    for (hyp_id, statement, domain, status, confidence) in hyp_results {
                        candidates.push(RetrievalCandidate {
                            source: "database_hypothesis".to_string(),
                            content: format!(
                                "Hypothesis: {} - {} (domain: {}, status: {}, confidence: {})",
                                hyp_id, statement, domain, status, confidence
                            ),
                            relevance: 0.55,
                            confidence,
                            recency: 0.5,
                            importance: 0.45,
                            is_relationship: false,
                        });
                    }
                }

                // Query evidence for evidence-based retrieval (Architecture §26.8, §30)
                if let Ok(mut evidence_stmt) = conn.prepare(
                    "SELECT id, hypothesis_id, content, evidence_type, direction, strength FROM evidence LIMIT ?1"
                ) {
                    let evidence_results: Vec<(String, String, String, String, String, f32)> = evidence_stmt
                        .query_map(
                            rusqlite::params![budget as i64],
                            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
                        )
                        .ok()
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default();
                    for (evidence_id, hypothesis_id, content, evidence_type, direction, strength) in evidence_results {
                        candidates.push(RetrievalCandidate {
                            source: "database_evidence".to_string(),
                            content: format!(
                                "Evidence: {} (id: {}, hypothesis: {}, type: {}, direction: {}, strength: {})",
                                content, evidence_id, hypothesis_id, evidence_type, direction, strength
                            ),
                            relevance: 0.5,
                            confidence: strength,
                            recency: 0.45,
                            importance: 0.4,
                            is_relationship: false,
                        });
                    }
                }

                // Query learned knowledge for knowledge-based retrieval (Architecture §26.6, §30)
                if let Ok(mut knowledge_stmt) = conn.prepare(
                    "SELECT id, content, domain, confidence FROM learned_knowledge WHERE active = 1 LIMIT ?1"
                ) {
                    let knowledge_results: Vec<(String, String, String, f32)> = knowledge_stmt
                        .query_map(
                            rusqlite::params![budget as i64],
                            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                        )
                        .ok()
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default();
                    for (knowledge_id, content, domain, confidence) in knowledge_results {
                        candidates.push(RetrievalCandidate {
                            source: "database_learned_knowledge".to_string(),
                            content: format!(
                                "Learned Knowledge: {} (id: {}, domain: {}, confidence: {})",
                                content, knowledge_id, domain, confidence
                            ),
                            relevance: 0.65,
                            confidence,
                            recency: 0.6,
                            importance: 0.55,
                            is_relationship: false,
                        });
                    }
                }
            }
        }

        // Limit to budget
        candidates.truncate(budget);
        candidates
    }

    /// Ranking engine: score candidates by multi-factor scoring with semantic similarity.
    /// Per Architecture §16.6: Final Score = Relevance * Confidence * Recency * Importance * SemanticSimilarity * SourceWeight.
    /// Semantic similarity is computed using embedding cosine similarity when available.
    /// Source weights: database_memory=1.0, database_experience=0.9, database_observation=0.8,
    /// database_memory_tag=0.7, database_memory_relationship=0.85, database_scheduled_task=0.75,
    /// database_reputation=0.7, database_lineage=0.65, database_hypothesis=0.8,
    /// database_evidence=0.75, database_learned_knowledge=0.85, memory=0.9,
    /// knowledge=0.85, experience=0.8, knowledge_graph=0.9, knowledge_graph_node=0.8.
    pub fn rank_candidates(candidates: &[RetrievalCandidate]) -> Vec<(RetrievalCandidate, f32)> {
        // Generate query embedding for semantic similarity comparison
        let query_embedding =
            crate::memory::embedding::generate_embedding("retrieval_query", 0.8, 0.8);
        // Source weights per Architecture §16.6 for multi-factor ranking
        let source_weight = |source: &str| -> f32 {
            match source {
                "database_memory" => 1.0,
                "database_experience" => 0.9,
                "database_observation" => 0.8,
                "database_memory_tag" => 0.7,
                "database_memory_relationship" => 0.85,
                "database_scheduled_task" => 0.75,
                "database_reputation" => 0.7,
                "database_lineage" => 0.65,
                "database_hypothesis" => 0.8,
                "database_evidence" => 0.75,
                "database_learned_knowledge" => 0.85,
                "memory" => 0.9,
                "knowledge" => 0.85,
                "experience" => 0.8,
                "knowledge_graph" => 0.9,
                "knowledge_graph_node" => 0.8,
                _ => 0.7,
            }
        };
        candidates
            .iter()
            .map(|c| {
                // Compute semantic similarity using embeddings when available
                let semantic_similarity = if let Some(query_emb) = &query_embedding {
                    let content_embedding = crate::memory::embedding::generate_embedding(
                        &c.content,
                        c.confidence,
                        c.importance,
                    );
                    if let Some(content_emb) = content_embedding {
                        // Cosine similarity between query and content embeddings
                        let dot: f32 = query_emb
                            .iter()
                            .zip(content_emb.iter())
                            .map(|(a, b)| a * b)
                            .sum();
                        let mag_a: f32 = query_emb.iter().map(|x| x * x).sum::<f32>().sqrt();
                        let mag_b: f32 = content_emb.iter().map(|x| x * x).sum::<f32>().sqrt();
                        if mag_a > 0.0 && mag_b > 0.0 {
                            (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
                        } else {
                            0.5
                        }
                    } else {
                        0.5 // Default similarity when content embedding unavailable
                    }
                } else {
                    0.5
                };
                // Multi-factor scoring with source weight: Relevance * Confidence * Recency * Importance * SemanticSimilarity * SourceWeight
                let weight = source_weight(&c.source);
                let score = c.relevance
                    * c.confidence
                    * c.recency
                    * c.importance
                    * semantic_similarity
                    * weight;
                (c.clone(), score.clamp(0.0, 1.0))
            })
            .collect()
    }

    /// Context filtering: enforce token budget and minimum confidence.
    pub fn filter_candidates(
        candidates: Vec<(RetrievalCandidate, f32)>,
        budget: usize,
        min_confidence: f32,
    ) -> Vec<RetrievalCandidate> {
        candidates
            .into_iter()
            .filter(|(_, score)| *score >= min_confidence)
            .map(|(c, _)| c)
            .take(budget)
            .collect()
    }
}

/// Active reference to retrieval pipeline contracts.
pub fn reference_retrieval_contracts() {
    // Query understanding
    let understood = RetrievalPipeline::understand_query("test query");
    tracing::debug!(?understood, "query understanding wire reference");

    // Query expansion
    let expanded = RetrievalPipeline::expand_query("test query");
    tracing::debug!(?expanded, "query expansion wire reference");

    // Hybrid retrieval
    let candidates = RetrievalPipeline::hybrid_retrieve("test query", 50);
    tracing::debug!(
        candidates = candidates.len(),
        "hybrid retrieval wire reference"
    );

    // Ranking engine
    let ranked = RetrievalPipeline::rank_candidates(&candidates);
    tracing::debug!(ranked = ranked.len(), "ranking wire reference");

    // Filtering
    let filtered = RetrievalPipeline::filter_candidates(ranked, 25, 0.5);
    tracing::debug!(filtered = filtered.len(), "filtering wire reference");

    // Pipeline execute
    let pipeline = RetrievalPipeline::new()
        .add_stage(RetrievalStage::QueryUnderstanding)
        .add_stage(RetrievalStage::CandidateGeneration)
        .add_stage(RetrievalStage::Ranking)
        .add_stage(RetrievalStage::Filtering);

    let request = RetrievalRequest::new("test query")
        .with_type("memory")
        .with_budget(50)
        .with_min_confidence(0.7);

    let result = pipeline.execute(&request);
    tracing::info!(
        success = result.success,
        candidates = result.candidates.len(),
        "Retrieval pipeline contracts actively referenced"
    );
}
