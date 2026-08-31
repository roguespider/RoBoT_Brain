# Research Engine

## Purpose

The Research Engine gives RoBoT Brain the ability to search the external world when its internal knowledge (memory, knowledge base, experience, skills, reflections, workflows, world model, hypotheses) is insufficient. It is the **last resort** in a 9-tier confidence cascade. It prevents hallucination by grounding answers in sourced evidence rather than letting the LLM fabricate facts.

## Core Principle

RoBoT should never say "I don't know" or hallucinate when external information could answer the question. Instead, it recognizes the gap, triggers research, receives a structured evidence packet, reasons over it, and returns a cited answer.

---

## Architecture

### The 9-Tier Confidence Cascade

Every user query passes through these checks **in order** before research is triggered:

| Tier | Check | Tool(s) | Pass Condition |
|------|-------|---------|---------------|
| 1 | Memory | `search_memory`, `list_memories` | confidence >= 0.7 |
| 2 | Knowledge Base | `query_knowledge`, `get_knowledge` | confidence >= 0.7 |
| 3 | Experience | `list_experiences`, `get_patterns` | pattern matches |
| 4 | Skills | `search_skills`, `execute_skill` | skill available |
| 5 | Reflections | `list_reflections_by_status` | validated reflection |
| 6 | Workflows/Plans | `list_workflows`, `get_plan` | existing plan |
| 7 | World Model | `list_world_entities`, `query_world` | relevant entity |
| 8 | Hypotheses | `list_hypotheses` | relevant hypothesis |
| 9 | **Research Engine** | `research()`, `quick_research()`, `deep_research()` | **last resort** |

If any tier passes, the cascade stops. Research only fires when all 8 prior tiers fail.

### End-to-End Flow

```
User Query
  |
  v
[Tier 1-8: Internal Sources]
  |-- PASS (confidence >= 0.7) --> Answer directly
  |-- FAIL (all tiers) --> RESEARCH ENGINE
        |
        v
  SearchProvider Trait (provider-agnostic)
  |-- DuckDuckGo adapter (primary)
  |-- Brave adapter (optional fallback)
  |-- Jina adapter (content processing)
  |-- Per-source adapters (Reddit, HN, Wikipedia, arXiv, etc.)
        |
        v
  Result Ranking (pipeline.rs)
        |
        v
  Content Retrieval (2-5 sources max — context protection)
        |
        v
  Passage Extraction (Jina processing)
        |
        v
  Source Comparison / Contradiction Detection
        |
        v
  Evidence Packet (evidence.rs)
  {
    question,
    queries (all search queries used),
    sources[] (title, url, provider, query, retrieved_at, relevance, content),
    findings[],
    contradictions[],
    limitations,
    confidence (0.0-1.0),
    retrieved_at
  }
        |
        v
  LLM Reasoning (receives ONLY evidence packet — never raw HTML)
        |
        v
  Answer / Action (with source citations)
        |
        v
  Experience Engine (record: why triggered, queries, sources used, solved?)
        |
        v
  Memory Engine (promote to permanent knowledge ONLY if experience validates)
        |
        v
  Source Provenance Stored (every fact traces to URL + provider + timestamp)
```

---

## File Structure

Where each component lives in the codebase:

| File | Purpose |
|------|---------|
| `src/research/mod.rs` | Exports `ResearchEngine`, `SearchProvider` trait |
| `src/research/provider.rs` | `SearchProvider` trait + `SearchQuery`/`SearchResults` structs |
| `src/research/duckduckgo.rs` | Primary web adapter (DuckDuckGo HTML scraping) |
| `src/research/brave.rs` | Optional secondary adapter (Brave Search API) |
| `src/research/jina.rs` | Content extraction, reranking, embeddings (processing layer) |
| `src/research/pipeline.rs` | Orchestrates: search -> rank -> extract -> compare -> evidence |
| `src/research/quick_research.rs` | Bounded quick mode (1-3 sources, <5s) |
| `src/research/deep_research.rs` | Deep mode with sub-questions, comparison, 2-5 iterations |
| `src/research/evidence.rs` | `ResearchResult`, `Source`, `Finding`, `Contradiction` structs |
| `src/research/errors.rs` | Cancellation, timeout, provider failover handling |

### Integration Points

- `src/bridge/tools/search/mod.rs` — Add new tool definitions to `all()`
- `src/bridge/tools/mod.rs` — `register_tools()` chains `search_tools`; add new tools here
- `src/agent/decision.rs` — Implement 9-tier check; trigger research when all prior tiers fail
- `src/memory/` — Store evidence-derived knowledge with full provenance
- `src/experience/` — Record research experience (trigger reason, sources, outcome)

---

## Tool Catalog

### Primary Research Tools (MCP interface)

| Tool | Description | When to Use |
|------|-------------|-------------|
| `research(query)` | Full pipeline — auto-selects quick or deep based on query complexity | Default; use when unsure |
| `quick_research(query)` | Fast lookup, 1-3 sources, <5s | Simple factual gaps |
| `deep_research(query)` | Sub-questions, comparison, multi-iteration | Complex, multi-faceted questions |
| `web_search(query)` | Direct provider search (DuckDuckGo) without pipeline overhead | Raw results needed |
| `web_open(url)` | Open a specific URL for reading | Agent needs to read a specific page |
| `web_extract(url)` | Extract clean text from URL via Jina | Content needs processing |
| `find_error_resolution(error)` | Error-focused search (refactored from existing Jina tool) | When encountering errors |

### Per-Source Adapters (via SearchProvider trait)

| Adapter | Tool | When to Use |
|---------|------|-------------|
| DuckDuckGo | `web_search` | General web search (primary) |
| Brave | `web_search` | When DuckDuckGo fails (optional) |
| Jina | `web_extract`, reranking | Content extraction/processing |
| Wikipedia | `get_wikipedia(query)` | Encyclopedic facts |
| Google News | `search_news(query)` | Time-sensitive / current events |
| Reddit | `search_reddit(query, subreddit?)` | User experiences, discussions |
| Hacker News | `search_hackernews(query)` | Tech-community signals |
| YouTube | `search_youtube(query, includeTranscript?)` | Video content with transcripts |
| Substack | `search_substack(publications, maxPosts?)` | Newsletter analysis |
| Bluesky | `search_bluesky(query, sort?)` | Social/emerging trends |
| Telegram | `search_telegram(channel, maxMessages?)` | Public channel messages |
| Mastodon | `search_mastodon(query)` | Fediverse posts |
| VK | `search_vk(query)` | Russian/social network data |
| arXiv/bioRxiv/medRxiv | `search_preprints(query)` | Cutting-edge academic research |
| Zenodo/Figshare/OSF | `search_datasets(query)` | Data repository discovery |
| OpenStreetMap | `search_osm(query, location?)` | Geographic/POI data |
| SEC EDGAR | `search_sec_filings(query, filingType?)` | Financial filings |
| Wayback Machine | `resurrect_dead_link(url)` | Recover broken links |
| Crossref/OpenAlex | `verify_citations(references)` | Citation verification |
| Academic | `find_counter_arguments(query)` | Find opposing academic views |
| Academic | `validate_bibliography(bibliography)` | Validate entire reference list |
| Academic | `format_citations(doi, format)` | BibTeX/APA/MLA/Chicago/RIS formatting |
| Multi-platform | `detect_trends(platforms)` | Trending topics across platforms |
| Content | `score_reliability(urls)` | Rule-based source quality scoring |

### Use Case Routing

```
Market intelligence
  -> detect_trends -> search_reddit -> search_hackernews -> search_news

Academic workflow
  -> search_preprints -> search_datasets -> find_counter_arguments -> validate_bibliography

Financial research
  -> search_sec_filings -> extract_content -> score_reliability

Social listening
  -> search_bluesky -> search_telegram -> search_mastodon -> search_vk

Link rescue
  -> resurrect_dead_link -> extract_content (read archived content)

Research assistant
  -> web_search -> extract_content -> score_reliability -> find_counter_arguments -> verify_citations -> format_citations

Location research
  -> search_osm -> extract_content (pull details from POI websites)
```

---

## Key Design Rules

These are non-negotiable. Every implementation decision must respect them.

### 1. Last Resort Only
Research is NEVER triggered if any internal source has confidence >= 0.7. The cascade must run in full order.

### 2. Context Protection
The LLM receives ONLY the evidence packet. Never raw HTML, never full page content. Maximum 2-5 sources per query to stay within context limits.

### 3. Source Provenance
Every fact traces to: URL + provider name + timestamp. No anonymous citations. The evidence packet stores this; memory promotion preserves it.

### 4. Provider Agnostic
`SearchProvider` trait abstracts all search providers. DuckDuckGo/Brave/etc. are implementations, not the interface. Swapping providers must not require changing the pipeline.

### 5. Cancellation and Timeouts
Every research operation has a bounded timeout. No indefinite hangs. Cancellation must stop all in-flight fetches.

### 6. Graceful Degradation
Provider failures return structured errors, never crashes. Failover to secondary providers where available.

### 7. Experience Recording
Every research operation records: query, sources used, outcome (solved?), confidence. This feeds back into the experience engine for future self-improvement.

### 8. Memory Promotion Gate
Research-derived knowledge is NOT automatically stored. It is promoted to permanent memory ONLY if experience validates it (confidence >= 0.7 AND outcome = solved).

### 9. Web Content Is Data, Never Code
Every byte that comes from the external web (search snippets, extracted passages, page HTML) is hostile input. Strip and
escape it before it enters the evidence packet, never pass raw HTML across the ACP/MCP boundary, and never let provider
content be executed or interpreted as instructions. This is the injection defense behind DoD #12.

---

## Data Structures

### SearchProvider Trait

```rust
trait SearchProvider: Send + Sync {
    async fn search(&self, query: &str) -> Result<SearchResults, ResearchError>;
    fn name(&self) -> &str;
    fn supports(&self, source: SearchSource) -> bool;
}
```

### SearchQuery

```rust
struct SearchQuery {
    query: String,
    source: SearchSource,      // Web, News, Reddit, HN, Wikipedia, etc.
    max_results: usize,       // Default 10, bounded by context
    language: Option<String>, // e.g. "en", "de"
    region: Option<String>,   // e.g. "us", "uk"
}
```

### SearchResults

```rust
struct SearchResults {
    results: Vec<SearchResult>,
    provider: String,         // "duckduckgo", "brave", etc.
    query: String,
    retrieved_at: DateTime<Utc>,
}

struct SearchResult {
    title: String,
    url: String,
    snippet: String,
    relevance: f32,           // 0.0-1.0
    source: SearchSource,
}
```

### ResearchResult (Evidence Packet)

```rust
struct ResearchResult {
    question: String,
    queries: Vec<String>,           // All search queries used
    sources: Vec<Source>,
    findings: Vec<Finding>,
    contradictions: Vec<Contradiction>,
    limitations: Vec<String>,
    confidence: f32,                // 0.0-1.0
    retrieved_at: DateTime<Utc>,
}

struct Source {
    title: String,
    url: String,
    provider: String,
    query_used: String,
    retrieved_at: DateTime<Utc>,
    relevance: f32,
    content: String,                 // Extracted and cleaned
}

struct Finding {
    statement: String,
    source_url: String,
    confidence: f32,
}

struct Contradiction {
    claim_a: String,
    claim_b: String,
    source_a_url: String,
    source_b_url: String,
    resolution: Option<String>,
}
```

### ResearchError

```rust
enum ResearchError {
    Timeout { query: String, elapsed: Duration },
    ProviderUnavailable { provider: String },
    ProviderError { provider: String, message: String },
    NoResults { query: String },
    ContentExtractionFailed { url: String },
    Cancelled,
}
```

---

## Research Modes

### Quick Research

- 1-3 searches maximum
- <5 second total timeout
- Minimal sub-question generation
- Best for: factual lookups, simple definitions, straightforward answers
- Trigger: simple queries with clear intent

### Deep Research

- 2-5 iterations with sub-question generation
- Up to 60 second timeout
- Active comparison of sources
- Contradiction detection and resolution
- Best for: complex questions, multi-faceted topics, verification of claims
- Trigger: complex queries, queries with ambiguity, research assignments

### Auto-selection

The default `research(query)` tool auto-selects based on query complexity heuristics:
- Simple factual query -> quick mode
- Complex/multi-part query -> deep mode
- Agent can override by calling `quick_research()` or `deep_research()` directly

---

## Integration With Existing Subsystems

### Memory Engine
- Research-derived knowledge stored with full provenance
- Provenance = { url, provider, timestamp, query }
- Promotion gate: confidence >= 0.7 AND outcome = solved
- Used by: `search_memory`, `list_memories`, `get_embedding`

### Experience Engine
- Records every research operation:
  - `experience_type`: "research"
  - `context`: { query, sources_used[], mode, duration }
  - `outcome`: "solved" | "partial" | "failed"
  - `title`: truncated query
- Used by: `list_experiences`, `get_patterns`, `analyze_patterns`

### Knowledge Engine
- High-confidence research results promoted as knowledge
- Tags include: "web", "research", source_provider
- Used by: `query_knowledge`, `get_knowledge`

### ACP/MCP Boundary
- All research tools exposed as MCP tools via `register_tools()`
- ACP layer can route `route_acp_message` for cross-agent research sharing
- No web content passes the ACP/MCP boundary as raw HTML

---

## Implementation Order (R1-R16)

Implementation must follow this order. Each step builds on the prior.

**Prerequisite** — install the HTTP stack and keys BEFORE R3:
- Add `reqwest` (and an HTML-parsing crate, e.g. `scraper`) to `Cargo.toml`. Every provider needs this.
- Jina and Brave require API keys — wire a `JINA_API_KEY` / `BRAVE_API_KEY` config path. Jina has NO existing
  integration in `src/` today; R4 is net-new, not a refactor (correcting the stale "existing Jina pattern" claim).

### Phase A: Foundation

**R1** — Create `src/research/` directory + `mod.rs` (exports only)

**R2** — Define `SearchProvider` trait + `SearchQuery`/`SearchResults` structs in `provider.rs`

**R3** — Implement DuckDuckGo adapter (`duckduckgo.rs`) — primary provider. No external API key required. HTML scraping via reqwest.

**R4** — Implement Jina processing (`jina.rs`) — extraction, reranking, embeddings. Net-new (no existing Jina
integration in `src/` today). API-key gated via the `JINA_API_KEY` config path.

### Phase B: Core Pipeline

**R5** — Build pipeline orchestration (`pipeline.rs`):
- search -> rank -> extract -> compare -> evidence
- Bounded result count (max 10 raw, top 5 retained)
- Timeout per search operation

**R6** — Implement quick mode (`quick_research.rs`):
- 1-3 sources, <5s timeout
- Simple ranking (relevance score only)
- Direct evidence packet generation

**R7** — Implement deep mode (`deep_research.rs`):
- Sub-question generation (derive 1-3 sub-questions from query)
- Multi-iteration search
- Source comparison
- Contradiction detection

**R8** — Implement evidence packet (`evidence.rs`):
- `ResearchResult` struct with all required fields
- `Source`, `Finding`, `Contradiction` sub-structs
- Serialization for MCP tool return

### Phase C: Error Handling and Integration

**R9** — Add error handling (`errors.rs`):
- `ResearchError` enum with all variants
- Timeout handling with `tokio::time::timeout`
- Cancellation via `tokio::sync::oneshot`
- Structured error messages (never panic)

**R9b** — Web-content sanitization (prompt-injection defense): treat all scraped HTML / search content as DATA, never as
instructions. Strip HTML tags + control chars, escape embedded text, cap at 5 sources, truncate with an "and N more"
marker. Satisfies DoD #12.

**R10** — Wire into `src/agent/decision.rs`:
- Add 9-tier cascade check
- Trigger research only when tiers 1-8 all fail
- Pass confidence threshold (0.7) through the cascade

**R11** — Register tools in `src/bridge/tools/search/mod.rs`:
- Add `research`, `quick_research`, `deep_research`, `web_search`, `web_open`, `web_extract`
- Add to `all()` function
- Wire into `register_tools()` chain

**R11b** — Register the new tools in the test_suite coverage gate: add one `TestRequirement` per research tool to
`test_suite/src/function_registry/search_tools.rs` and the matching id case in
`test_suite/src/comprehensive_test/argument_builder.rs`. Keep `tool_names()`, `get_tools()`, and `execute_tool()` in
sync (a missing `get_tools()` entry creates a phantom tool — the T1-19 root cause). This keeps the "0 untested tools"
gate metric green.

### Phase D: Memory and Experience

**R12** — Integrate memory promotion (`src/memory/`):
- Store research results with provenance
- Implement promotion gate logic
- Integrate with `store_memory` and `get_embedding`

**R13** — Integrate experience recording (`src/experience/`):
- Record every research operation
- Track: query, sources used, mode, duration, outcome
- Used for pattern analysis and future recommendations

### Phase E: Hardening

**R14** — Add provider failover:
- Brave Search as secondary (requires API key, optional)
- Automatic failover on DuckDuckGo failure
- Structured error when all providers fail

**R15** — Context protection enforcement:
- Hard cap at 5 sources per evidence packet
- Token budget estimation before LLM call
- Truncation with "and N more sources" marker

**R16** — End-to-end verification:
- Add test_suite tests for the full research flow
- Verify 13-step verification path:
  1. `get_workflow` -> `search_memory` (workflow gate)
  2. Call `research("query")`
  3. Verify tiers 1-8 checked first
  4. Verify SearchProvider selected
  5. Verify results ranked
  6. Verify Jina extracted passages
  7. Verify evidence packet structure
  8. Verify LLM received only evidence packet
  9. Verify answer references sources
  10. Verify experience recorded
  11. Verify memory promoted (if confidence >= 0.7)
  12. Verify provider failure handled
  13. Verify cancellation works

---

## Definition of Done

The Research Engine is operational when RoBoT Brain can:

1. Determine that it lacks sufficient internal knowledge
2. Request external research without knowing the search provider
3. Search the web through a provider-agnostic abstraction
4. Select the most relevant sources (bounded to 2-5)
5. Retrieve clean, extracted content
6. Reduce content into a structured evidence packet
7. Preserve full source provenance (URL + provider + timestamp)
8. Detect and expose contradictory information
9. Return bounded results to the LLM (never raw HTML)
10. Handle provider failures without crashing
11. Respect cancellation and timeouts
12. Prevent web content from becoming executable instructions
13. Decide whether researched information is worth remembering
14. Record useful research experience
15. Complete the entire process through the ACP/MCP architecture

The final cognitive loop:

```
QUESTION
   |
   v
MEMORY
   |
   v
"Do I have enough confidence?"
   |-- YES (>= 0.7) --> ANSWER
   |-- NO --> KNOWLEDGE BASE
                |
                v
          EXPERIENCE
                |
                v
            SKILLS
                |
                v
          REFLECTION
                |
                v
          WORKFLOW
                |
                v
          WORLD MODEL
                |
                v
          HYPOTHESES
                |
                v
            RESEARCH <-- LAST RESORT
                |
                v
            EVIDENCE
                |
                v
          REASONING
                |
                v
            ANSWER
                |
                v
          EXPERIENCE (record)
                |
                v
       "Worth remembering?"
           /         \
         NO           YES
          |             |
       DISCARD      MEMORY
```

---

## Why This Is Not Just "a Search Tool"

A simple web search MCP tool gives the LLM a Google-like search box and lets it figure out what to do with raw HTML. This approach has problems:

- **Hallucination**: LLM may ignore sources or invent facts from snippets
- **No provenance**: Facts can't be traced back to sources
- **No context protection**: LLM receives unbounded raw content
- **No cascading**: Every query hits the web, even when memory has the answer
- **Provider lock-in**: Tool is tied to one search API
- **No experience**: Same research done repeatedly without learning

The Research Engine solves all of these by:
- Cascade-first design (internal sources always checked first)
- Evidence packet contract (structured, bounded, sourced)
- Provider abstraction (pluggable search backends)
- Experience recording (learns from research outcomes)
- Memory promotion with gates (not everything is worth remembering)
- Contradiction detection (exposes uncertainty rather than hiding it)
