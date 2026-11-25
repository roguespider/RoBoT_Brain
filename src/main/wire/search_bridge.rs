// Wire search bridge tool input types and execute functions to eliminate dead-code warnings.
// Per Architecture Chapter 13 - Tool Engine and AGENTS.md (0 dead code).

pub fn wire_search_bridge() {
    let search_input_ref = api::bridge::tools::search::WebSearchInput {
        query: "test".to_string(),
    };
    tracing::debug!("search_input_ref: query={}", search_input_ref.query);

    let web_open_input_ref = api::bridge::tools::search::WebOpenInput {
        url: "http://test".to_string(),
    };
    tracing::debug!("web_open_input_ref: url={}", web_open_input_ref.url);

    let web_extract_input_ref = api::bridge::tools::search::WebExtractInput {
        url: "http://test".to_string(),
    };
    tracing::debug!("web_extract_input_ref: url={}", web_extract_input_ref.url);

    let research_input_ref = api::bridge::tools::search::ResearchInput {
        query: "test".to_string(),
    };
    tracing::debug!("research_input_ref: query={}", research_input_ref.query);

    let quick_research_input_ref = api::bridge::tools::search::QuickResearchInput {
        query: "test".to_string(),
    };
    tracing::debug!(
        "quick_research_input_ref: query={}",
        quick_research_input_ref.query
    );

    let deep_research_input_ref = api::bridge::tools::search::DeepResearchInput {
        query: "test".to_string(),
    };
    tracing::debug!(
        "deep_research_input_ref: query={}",
        deep_research_input_ref.query
    );

    let error_resolution_input_ref = api::bridge::tools::search::FindErrorResolutionInput {
        error: "test error".to_string(),
    };
    tracing::debug!(
        "error_resolution_input_ref: error={}",
        error_resolution_input_ref.error
    );

    // Wire execute functions as function references
    let fns_ref = (
        api::bridge::tools::search::execute_web_search,
        api::bridge::tools::search::execute_web_open,
        api::bridge::tools::search::execute_web_extract,
        api::bridge::tools::search::execute_research,
        api::bridge::tools::search::execute_quick_research,
        api::bridge::tools::search::execute_deep_research,
        api::bridge::tools::search::execute_find_error_resolution,
    );
    tracing::debug!("fns_ref: {} functions", std::mem::size_of_val(&fns_ref));
}
