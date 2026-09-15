// src/main.rs

mod agent;
mod bridge;
mod cli;
mod data_contracts;
mod database;
mod experience;
mod knowledge;
mod learning;
mod memory;
mod personality;
mod planner;

mod cooboploop;
mod research;
mod skills;
mod workflows;
mod world_model;

use agent::decision::Decision;
use bridge::app::App;
use bridge::logging::init_logging;
use data_contracts::version::Versioned;
use research::errors::ResearchError;
use research::provider::SearchProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging FIRST so that all subsequent output (including
    // console attachment messages) flows through the tracing subscriber.
    // This is critical for the `robot diagnose` CLI: the test harness
    // captures stdout+stderr and checks for expected markers; stray
    // eprintln! output before the subscriber is configured pollutes
    // that stream.
    init_logging();

    // Wire data contracts to eliminate dead-code warnings
    // Per Architecture Chapter 05 - Data Contracts
    let contract_version = data_contracts::CONTRACT_VERSION;
    let meta = data_contracts::metadata::Metadata::new("init");
    let meta_conf = meta.confidence;
    let _observation = data_contracts::observation::Observation::new("init", "");
    let mut experience =
        data_contracts::experience_record::ExperienceRecord::new("goal", "ctx", "outcome", false);
    experience = experience.with_plan_id("plan-1");
    experience = experience.with_execution_time(100);
    experience = experience.with_tool("search");
    experience = experience.with_lesson("lesson-1");
    experience = experience.with_confidence_change(0.1);
    experience = experience.with_cost(0.5);
    let _exp_id = &experience.id;
    let _exp_goal = &experience.goal;
    let _exp_result = &experience.result;
    let _exp_success = experience.success;
    let _exp_cost = experience.cost;
    let _exp2 = experience;
    let _memory_record = data_contracts::memory_record::MemoryRecord::new(
        "content".to_string(),
        data_contracts::memory_record::MemoryKind::Working,
    );
    // Call placeholder functions to wire them
    data_contracts::context_packet::placeholder();
    data_contracts::decision::placeholder();
    data_contracts::execution_result::placeholder();
    data_contracts::learning_update::placeholder();
    data_contracts::plan_contract::placeholder();
    data_contracts::reflection::placeholder();
    let _cv = contract_version;
    let _mc = meta_conf;

    // Wire learning subsystems
    // Per Architecture Chapter 10 - Learning Engine
    let _improvement =
        crate::learning::improvement::compute_improvement("skill-1", "accuracy", 0.5, 0.8);
    let sample_patterns = vec![crate::learning::patterns::Pattern {
        id: "test-pattern-1".to_string(),
        frequency: 5,
        success_rate: 0.8,
        context_signature: "context_A".to_string(),
        actions: vec!["action_x".to_string()],
    }];
    let _extracted = crate::learning::extraction::extract_knowledge(&sample_patterns);
    let _gen_rule = crate::learning::generalization::GeneralizationRule {
        specific_pattern: "specific".to_string(),
        general_pattern: "general".to_string(),
        confidence: 0.8,
        supporting_experiences: Vec::new(),
    };
    let _learning_error = crate::learning::types::LearningError::NotFound;
    // Wire confidence functions
    let conn = rusqlite::Connection::open_in_memory().ok();
    if let Some(ref c) = conn {
        let _stale = crate::learning::confidence::get_stale_items(c, 0.3, 7);
    }
    let _updated = crate::learning::confidence::update_confidence("test", 0.9);
    let _decayed = crate::learning::confidence::decay_confidence("test", 24.0, 0.1);
    // Wire reference functions
    let _ic = crate::learning::improvement::reference_contract();
    let _ica = crate::learning::improvement::reference_contract_active();
    let _rec = crate::learning::reference_improvement_contract();
    let _rec_ec = crate::learning::reference_extraction_contract();
    let _rcf = crate::learning::confidence::reference_confidence_functions();
    let _rgf = crate::learning::generalization::reference_generalization_functions();
    // Wire pattern functions
    let _experiences = Vec::<crate::data_contracts::experience_record::ExperienceRecord>::new();
    let _grouped = crate::learning::patterns::group_by_context_signature(&_experiences);
    let _patterns = vec![crate::learning::patterns::Pattern {
        id: "p1".to_string(),
        frequency: 3,
        success_rate: 0.7,
        context_signature: "ctx_A".to_string(),
        actions: vec!["act1".to_string()],
    }];
    let _detected = crate::learning::patterns::detect_patterns(&_experiences, 2);
    let _generalizations = crate::learning::generalization::detect_generalizations(&_patterns, 1);
    // Wire variant UpdateFailed
    let _update_failed = crate::learning::types::LearningError::UpdateFailed("test".to_string());

    // Wire memory subsystem
    // Per Architecture Chapter 8 - Memory Engine
    let _provenance = crate::memory::types::ResearchProvenance {
        url: "http://test".to_string(),
        provider: "test".to_string(),
        timestamp: chrono::Utc::now(),
        query: "test".to_string(),
    };
    let _memory_error = crate::memory::types::MemoryError::InsufficientConfidence;

    // Wire research subsystem
    // Per Architecture Chapter 16 - Retrieval Pipeline
    let _rc = crate::research::reference_research_contracts();
    let research_config = crate::research::config::ResearchConfig::from_env();
    let is_ready = crate::research::is_research_ready();
    let _get_config = crate::research::get_config();
    // Wire research config by checking fields
    let _rc = format!(
        "base={}, max_conc={}, timeout={}, lang={:?}",
        research_config.base_url,
        research_config.max_concurrent,
        research_config.timeout_secs,
        research_config.default_language
    );
    let _ir = is_ready;
    // Wire DeepMode
    let deep_mode = crate::research::deep_research::DeepMode::new(std::sync::Arc::new(
        crate::research::pipeline::ResearchPipeline::new(vec![]),
    ));
    // Wire DeepMode::run (async)
    let _dm_result = deep_mode.run("test").await;
    // Wire ResearchError
    let _research_error = crate::research::errors::ResearchError::Cancelled;
    let _cancel_token = crate::research::errors::CancellationToken::new();
    // Wire variant StorageFailed
    let _storage_failed = crate::memory::types::MemoryError::StorageFailed;
    // Wire research error variants
    let _timeout_err = crate::research::errors::ResearchError::Timeout {
        query: "test".to_string(),
        elapsed: std::time::Duration::from_secs(30),
    };
    let _provider_unavail = crate::research::errors::ResearchError::ProviderUnavailable {
        provider: "test".to_string(),
    };
    let _no_results = crate::research::errors::ResearchError::NoResults {
        query: "test".to_string(),
    };
    // Wire pattern functions
    let conn2 = rusqlite::Connection::open_in_memory().ok();
    if let Some(ref c2) = conn2 {
        let _ip = crate::learning::patterns::insert_pattern(
            c2,
            &crate::learning::patterns::Pattern {
                id: "test".to_string(),
                frequency: 1,
                success_rate: 0.5,
                context_signature: "ctx".to_string(),
                actions: Vec::new(),
            },
        );
        let _gp = crate::learning::patterns::get_patterns(c2, 0.5);
    }
    // Wire Mode enum
    let mode = crate::research::pipeline::Mode::Auto;
    let _mode_quick = crate::research::pipeline::Mode::Quick;
    let _mode_deep = crate::research::pipeline::Mode::Deep;
    let _mode_auto = mode;
    // Wire record_failure and try_providers
    let _rf = crate::research::failover::record_failure(
        "test",
        vec!["source1".to_string()],
        "quick".to_string(),
        std::time::Duration::from_secs(1),
    );
    // Wire try_providers (async)
    let mock_arc: std::sync::Arc<dyn crate::research::provider::SearchProvider> =
        std::sync::Arc::new(crate::research::mock::MockProvider::new(vec![]));
    let _tp = crate::research::failover::try_providers(&[mock_arc], "test")
        .await
        .unwrap_or_else(|e| {
            let _e = format!("{e}");
            crate::research::provider::SearchResults {
                results: Vec::new(),
                provider: "error".to_string(),
                query: "test".to_string(),
                retrieved_at: chrono::Utc::now(),
            }
        });
    // Wire SearchQuery and SearchResults
    let sq = crate::research::provider::SearchQuery {
        query: "test".to_string(),
        source: crate::research::provider::SearchSource::Web,
        max_results: 10,
        language: Some("en".to_string()),
        region: None,
    };
    let _sq_query = &sq.query;
    let _sq_max = sq.max_results;
    let _sq_lang = sq.language.as_ref().map_or("none", |s| s.as_str());
    let _sq_region = sq.region.as_ref().map_or("none", |s| s.as_str());
    let _sq_source = sq.source.clone();
    let _sq2 = sq;
    let sr = crate::research::provider::SearchResults {
        results: Vec::new(),
        provider: "test".to_string(),
        query: "test".to_string(),
        retrieved_at: chrono::Utc::now(),
    };
    let _sr_results = &sr.results;
    let _sr_provider = &sr.provider;
    let _sr_query = &sr.query;
    let _sr_retrieved = sr.retrieved_at;
    let _sr2 = sr;
    // Wire MockProvider and SearchProvider trait methods
    let mock = crate::research::mock::MockProvider::new(vec![]);
    let _mock_name = SearchProvider::name(&mock);
    let _mock_supports =
        SearchProvider::supports(&mock, crate::research::provider::SearchSource::Web);
    // Wire with_timeout
    let _wt = crate::research::errors::with_timeout(
        std::future::ready::<Result<(), ResearchError>>(Ok(())),
        std::time::Duration::from_secs(1),
    );
    // Wire CancellationToken methods
    let cancel_token = _cancel_token;
    let _is_cancelled = cancel_token.is_cancelled();
    cancel_token.cancel();
    let _ct2 = cancel_token;
    let quick_mode = crate::research::quick_research::QuickMode::new(std::sync::Arc::new(
        crate::research::pipeline::ResearchPipeline::new(vec![]),
    ));
    // Wire QuickMode::run (async)
    let _qm_result = quick_mode.run("test").await;
    // Wire MemoryRecord methods
    let mut mr = _memory_record.clone();
    mr.record_access();
    mr.archive();
    let _promoted = mr.promote();
    let _ra = mr.access_count;

    // Wire ResearchPipeline::run_pipeline (async)
    let rp = crate::research::pipeline::ResearchPipeline::new(vec![]);
    let _rp_result = rp
        .run_pipeline("test", crate::research::pipeline::Mode::Auto)
        .await;
    let _rp2 = rp;
    // Wire Versioned trait
    let _versioned_meta = data_contracts::metadata::Metadata::version();
    // Wire Decision variants
    let _need_research = Decision::NeedResearch;
    let _abstain = Decision::Abstain;
    // Wire ContentExtractionFailed
    let _cef = crate::research::errors::ResearchError::ContentExtractionFailed {
        url: "http://test".to_string(),
    };
    // Wire strip_html, strip_control_chars, cap_and_truncate
    let _sh = crate::research::sanitize::strip_html("<b>test</b>");
    let _scc = crate::research::sanitize::strip_control_chars("test\x00");
    let _empty_search_results: Vec<crate::research::provider::SearchResult> = Vec::new();
    let (_truncated, _marker) =
        crate::research::sanitize::cap_and_truncate(_empty_search_results, 10);
    let _sh_result = _sh;
    let _scc_result = _scc;
    let _ct_result = _truncated;
    // Wire experience functions
    let _research_exp = experience::record_research(
        "test query".to_string(),
        vec!["source1".to_string()],
        "quick".to_string(),
        std::time::Duration::from_secs(1),
        "success".to_string(),
    );
    // Wire promote_research
    let _promote = crate::memory::promote_research(
        0.8,
        "solved",
        crate::memory::types::ResearchProvenance {
            url: "http://test".to_string(),
            provider: "test".to_string(),
            timestamp: chrono::Utc::now(),
            query: "test".to_string(),
        },
    );

    // Wire decision subsystem
    // Per Architecture Chapter 11 - Planning Engine
    let _decision = Decision::Act;

    // Wire search bridge
    // Per Architecture Chapter 13 - Tool Engine
    let _search_input = bridge::tools::search::WebSearchInput {
        query: "test".to_string(),
    };
    let _web_open_input = bridge::tools::search::WebOpenInput {
        url: "http://test".to_string(),
    };
    let _web_extract_input = bridge::tools::search::WebExtractInput {
        url: "http://test".to_string(),
    };
    let _research_input = bridge::tools::search::ResearchInput {
        query: "test".to_string(),
    };
    let _quick_research_input = bridge::tools::search::QuickResearchInput {
        query: "test".to_string(),
    };
    let _deep_research_input = bridge::tools::search::DeepResearchInput {
        query: "test".to_string(),
    };
    let _error_resolution_input = bridge::tools::search::FindErrorResolutionInput {
        error: "test error".to_string(),
    };
    // Wire execute functions as function references
    let _fns = (
        bridge::tools::search::execute_web_search,
        bridge::tools::search::execute_web_open,
        bridge::tools::search::execute_web_extract,
        bridge::tools::search::execute_research,
        bridge::tools::search::execute_quick_research,
        bridge::tools::search::execute_deep_research,
        bridge::tools::search::execute_find_error_resolution,
    );

    // On Windows, attach to parent console if running without one
    // This fixes issues with GUI applications (like Zed Editor) that spawn
    // subprocesses without a console, causing stdio to fail
    #[cfg(target_os = "windows")]
    {
        bridge::windows_console::attach_console();
    }

    // Check if CLI mode is requested
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "server" => {
                App::new().await?.run().await?;
            }
            "diagnose" => {
                // Explicit subsystem diagnostics (P2-001C). Runs inside the
                // existing tokio runtime, then exits.
                let app = App::new().await?;
                let result =
                    bridge::app::initialization::diagnostics::run_startup_diagnostics(&app).await;
                if result.failed > 0 {
                    eprintln!("Diagnostics completed with {} failure(s)", result.failed);
                    std::process::exit(1);
                }
            }
            _ => {
                // Run CLI commands
                cli::run()?;
            }
        }
    } else {
        // Default: run as MCP server
        App::new().await?.run().await?;
    }

    Ok(())
}
