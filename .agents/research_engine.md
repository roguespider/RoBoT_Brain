Use Cases
Research assistant: web_search → extract_content → score_reliability → find_counter_arguments → verify_citations → format_citations
Market intelligence: detect_trends → search_reddit → search_hackernews → search_news
Academic workflow: search_preprints → search_datasets → find_counter_arguments → validate_bibliography
Financial research: search_sec_filings → extract_content → score_reliability
Social listening: search_bluesky → search_telegram → search_mastodon → search_vk
Link rescue: resurrect_dead_link → extract_content (read archived content)
Location research: search_osm → extract_content (pull details from POI websites)

Tools
Web & Content — 
Tool	Description	Example
web_search	DuckDuckGo web_search({ query: "rust async runtime" }) or fallback to Google (with API key)
extract_content	Clean text from any URL	extract_content({ url: "https://example.com/article" })
search_news	Google News RSS	search_news({ query: "AI regulation EU" })
get_wikipedia	Wikipedia article summary	get_wikipedia({ query: "quantum entanglement" })
resurrect_dead_link	Find archived version via Wayback Machine	resurrect_dead_link({ url: "https://example.com/old" })
score_reliability	Rule-based source reliability scoring	score_reliability({ urls: ["https://en.wikipedia.org/wiki/Rust"] })
Social & Discussion — 
Tool	Description	Example
search_reddit	Reddit posts + comments	search_reddit({ query: "best keyboard", subreddit: "MechanicalKeyboards" })
search_hackernews	Hacker News via Algolia API	search_hackernews({ query: "vector database" })
search_youtube	YouTube videos + transcripts	search_youtube({ query: "rust ownership", includeTranscript: true })
search_substack	Substack newsletters via RSS	search_substack({ publications: ["stratechery"], maxPosts: 20 })
search_bluesky	Bluesky (AT Protocol) public posts	search_bluesky({ query: "AI agents", sort: "top" })
search_telegram	Public Telegram channels via t.me/s/ preview	search_telegram({ channel: "durov", maxMessages: 50 })
search_mastodon	Mastodon Fediverse public posts	search_mastodon({ query: "rust programming" })
search_vk	VKontakte public posts via official API	search_vk({ query: "искусственный интеллект" })
detect_trends	Trending topics across Reddit, HN, YouTube, News	detect_trends({ platforms: ["reddit", "hackernews"] })
Academic & Research — 
Tool	Description	Example
search_preprints	arXiv, bioRxiv, medRxiv preprints	search_preprints({ query: "CRISPR gene editing" })
search_datasets	Zenodo, Figshare, OSF data repositories	search_datasets({ query: "climate change data" })
find_counter_arguments	Academic papers supporting/contrasting a claim	find_counter_arguments({ query: "transformers are better than RNNs" })
verify_citations	Verify citations against Crossref and OpenAlex	verify_citations({ references: ["Vaswani et al. (2017)..."] })
validate_bibliography	Validate entire bibliography with auto-format detection	validate_bibliography({ bibliography: "Vaswani et al. (2017)..." })
format_citations	Generate BibTeX, APA, MLA, Chicago, RIS from DOIs	format_citations({ doi: "10.1038/nature12373", format: "bibtex" })
Specialized Data —
Tool	Description	Example
search_osm	OpenStreetMap POIs and amenities via Overpass API	search_osm({ query: "restaurant", location: "Berlin" })
search_sec_filings	SEC EDGAR filings by ticker or company name	search_sec_filings({ query: "AAPL", filingType: "10-K" })

---

Yeah, absolutely. And honestly, that fits RoBoT extremely well.

What you're describing is basically giving the ACP/MCP layer a Research/Search capability, so the agent isn't forced to either:

already know something,
ask you,
or hallucinate an answer.

Instead it can recognize: "I don't know this, and I should go find out."

I would make it an actual Tool Engine capability

Something roughly like:

Agent
  ↓
Planning / Reasoning
  ↓
"I need external information"
  ↓
Research Tool
  ├── Web Search
  ├── Open Page
  ├── Extract relevant content
  ├── Follow links
  ├── Compare sources
  └── Return research packet
        ↓
Agent
  ↓
Reason over researched information
  ↓
Answer / Execute
  ↓
Experience Engine
  ↓
Potentially remember useful result

The important part is that search shouldn't just be "give the LLM a Google-like search box."

I'd make it a small research subsystem.

For example

The agent gets:

"Why is my Rust async task deadlocking?"

Instead of immediately answering, it could internally decide:

Problem detected:
  Rust async/concurrency issue

Known confidence:
  0.42

External research required:
  YES

Research:
  Search Rust async deadlock tokio
  Search specific error/message
  Search relevant Tokio documentation
  Search GitHub issues

Then it gets back something structured:

RESEARCH RESULT

Question:
Why can Tokio tasks deadlock when using Mutex across await?

Sources:
1. Tokio documentation
2. Rust documentation
3. Relevant GitHub issue
4. Stack Overflow discussion

Findings:
- ...
- ...
- ...

Confidence:
0.91

Contradictions:
- Source 3 recommends X
- Official documentation indicates Y

Then the LLM reasons over that, rather than blindly treating the first search result as truth.

And this could solve another thing you've been building toward

You don't necessarily want RoBoT permanently searching the Internet for every stupid question you throw at it.

You want it to develop a sense of:

"Do I already know this?"

That's where your memory/experience architecture becomes really useful.

Something like:

Question
   ↓
Memory Retrieval
   ↓
Do I have useful knowledge?
   │
   ├── YES → Is confidence sufficient?
   │              │
   │              ├── YES → Answer
   │              │
   │              └── NO → Research
   │
   └── NO → Research

And after research:

Research
   ↓
Answer
   ↓
Experience
   ↓
Was this useful/reusable?
   │
   ├── NO → discard
   │
   └── YES
        ↓
      Memory

That is much closer to an actual cognitive loop than simply bolting a web-search MCP onto the agent.

I'd also separate "search" from "research"

This distinction could become important later.

Search

Find me the Rust documentation for tokio::sync::Mutex.

Fast. One or two queries. Minimal processing.

Research

Figure out why this particular Tokio application deadlocks.

Multiple searches, source inspection, comparison, synthesis.

So your MCP/ACP interface could eventually expose something like:

web_search
web_open
web_extract
research

But research becomes the higher-level capability that orchestrates the others.

One other thing I'd definitely add

Source provenance.

Every piece of externally acquired information should carry:

source
url
retrieved_at
query
relevance
confidence

That gives you a very nice future capability:

"Why do you believe that?"

And RoBoT can actually answer:

"I researched this because my stored knowledge had low confidence. The conclusion came primarily from these three sources..."

That's a much better foundation than letting the LLM quietly absorb random web text into permanent memory.

And because you're already using ACP/MCP as the bridge between the cognitive system and external capabilities, I think adding research there is a very natural next capability rather than architectural clutter.

In fact, I'd put it on the roadmap before you start worrying about RoBoT knowing everything. Give it the ability to admit "I don't know, let me go find out." That's far more powerful.

For adding search-like MCP access, here’s one OAuth security option.
Ad
i think for my original project i tried to use duckduckgo and jina search is there a better idea then this or ?

Yeah. DuckDuckGo + Jina was a reasonable first attempt, but if you're redesigning this now for RoBoT, I'd change the approach.

The biggest improvement isn't necessarily picking one better search provider. I'd make the Research capability provider-agnostic and put a search abstraction between RoBoT and the Internet.

What I'd use now
1. Primary search: Brave Search API

I think Brave is a particularly good fit for RoBoT.

It has its own independent web index rather than simply wrapping Google/Bing, and its API is explicitly aimed at agents. It currently provides normal web search, snippets, metadata, discussions, news, etc.

More interestingly, Brave now has an LLM Context endpoint that returns pre-extracted web content intended for grounding LLMs.

And they currently advertise $5/month in free credits, with their Search API priced at $5 per 1,000 requests.

That makes it pretty attractive for your use case.

2. Keep Jina, but change what you use it for

This is the part I'd really change from your old architecture.

I wouldn't necessarily throw Jina away.

Jina's current API has embeddings, reranking, classification, and multimodal capabilities, and new users currently receive 10M free tokens.

So I'd think of it more as:

                 RESEARCH ENGINE
                       │
          ┌────────────┴────────────┐
          │                         │
     SEARCH PROVIDER          CONTENT PROCESSOR
          │                         │
     Brave Search                Jina
          │                    extraction/
          │                    reranking/
          │                    embeddings
          │                         │
          └────────────┬────────────┘
                       │
                 Research Engine
                       │
                 RoBoT's LLM

That's considerably cleaner.

And I'd actually add a third layer

This is where I think your original idea can become much more powerful.

Don't make the MCP tool:

search_web()

Make it something closer to:

research()

And let the Research Engine decide how much searching it needs.

For example:

Simple question

"What is Rust's Arc?"

research()
  ↓
1 search
  ↓
official Rust documentation
  ↓
answer
Programming problem

"Why is my Tokio task hanging?"

research()
  ↓
identify technologies
  ↓
generate search queries
  ↓
search
  ↓
inspect promising sources
  ↓
search again based on findings
  ↓
compare sources
  ↓
produce evidence packet
  ↓
LLM reasons over evidence
Really weird question from you

"Can I make a Rust process dynamically load a new neural network architecture while running?"

RoBoT could go:

Search #1
"Rust dynamic neural network loading"

        ↓

Interesting result

        ↓

Search #2
"Rust runtime dynamic model loading"

        ↓

GitHub project discovered

        ↓

Open documentation

        ↓

Search #3
"project-name limitations"

        ↓

Compare findings

        ↓

Research conclusion

That's research, rather than Google-with-a-fancy-hat.

I would also give it multiple search providers

Not necessarily initially, but architect for it.

Something like:

trait SearchProvider {
    async fn search(&self, query: SearchQuery)
        -> Result<SearchResults>;
}

Then:

SearchProvider
    │
    ├── Brave
    ├── DuckDuckGo
    ├── Jina
    ├── future provider
    └── local/offline search

Then your Research Engine doesn't care which one actually did the searching.

That gives you a huge advantage later.

If Brave changes pricing:

RoBoT architecture
       ↓
unchanged
       ↓
replace Brave adapter

If you discover a better search engine in 2027:

new provider
     ↓
implement SearchProvider
     ↓
done

No cognitive architecture surgery.

There's another thing I really like about Brave for RoBoT

Their search API supports operators such as:

site:github.com
filetype:pdf
"exact phrase"
-term

and pagination.

That's excellent for a programming agent.

RoBoT could automatically specialize searches:

Programming problem
        ↓
site:github.com
site:docs.rs
site:doc.rust-lang.org
site:tokio.rs
site:stackoverflow.com

Instead of spraying generic Google-style queries across the Internet.

One thing I would NOT do

I wouldn't let the LLM directly receive 20 giant webpages.

That's how you turn:

"I need to answer this question"

into:

"Congratulations, you've consumed your entire context window."

Instead:

SEARCH
 ↓
10-20 results
 ↓
rank
 ↓
select 2-5
 ↓
retrieve relevant content
 ↓
extract relevant passages
 ↓
rerank
 ↓
research packet
 ↓
LLM

That fits extremely well with the Context Engine + Memory Engine architecture you've already been building.

So my current recommendation

I'd build RoBoT's external knowledge system as:

                    ┌─────────────────┐
                    │   RoBoT Agent   │
                    └────────┬────────┘
                             │
                     "I need knowledge"
                             │
                    ┌────────▼────────┐
                    │ Research Engine │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
        ┌─────▼─────┐  ┌─────▼─────┐  ┌─────▼─────┐
        │   Brave   │  │    Jina   │  │   Future  │
        │   Search  │  │ processing│  │ providers │
        └───────────┘  └───────────┘  └───────────┘
              │              │
              └──────┬───────┘
                     │
              Evidence / Sources
                     │
              ┌──────▼──────┐
              │  Reasoning  │
              └──────┬──────┘
                     │
              ┌──────▼──────┐
              │   Answer    │
              └──────┬──────┘
                     │
              Experience Engine
                     │
             ┌───────▼────────┐
             │ Should this be  │
             │ remembered?     │
             └────────────────┘

So yes, I'd replace the old "DuckDuckGo + Jina search" concept, but not necessarily throw Jina out.

I'd make Brave the initial web-index provider, Jina a processing/reranking/extraction component, and Research Engine the actual intelligence layer.

And importantly, I'd expose Research through ACP/MCP as a capability, rather than making your LLM know anything about Brave, Jina, URLs, APIs, or search mechanics.

That keeps the brain from knowing how the eyeballs work. It just knows: "I need to look outside myself."

For building MCP agent search and tools, here’s one runtime option.
Ad

how about use Duck.ai to do all the leg work and just give relevant info back to my local ai

Conceptually, yes. In fact, I think your idea is better than making your local model directly deal with search results.

But I would make one important distinction:

Use DuckDuckGo Search as the research worker, not Duck.ai itself as an automated API dependency.

DuckDuckGo currently separates Duck.ai from its normal search infrastructure. Duck.ai is essentially a private interface to third-party models, while DuckDuckGo Search and its Search Assist features handle web-sourced information.

Your idea would look like this
                  LOCAL RoBoT
                      │
                      │ "I don't know"
                      ▼
              ┌───────────────┐
              │ Research MCP  │
              └───────┬───────┘
                      │
                      ▼
              ┌───────────────┐
              │ DuckDuckGo    │
              │ Search        │
              └───────┬───────┘
                      │
               web research
                      │
                      ▼
              ┌───────────────┐
              │ Research      │
              │ Processor     │
              └───────┬───────┘
                      │
              relevant findings
              + sources
              + confidence
                      │
                      ▼
                  LOCAL LLM

The local model doesn't need to read 30 web pages.

It gets something more like:

RESEARCH RESULT

Question:
Why is Tokio Mutex causing my task to hang?

Relevant findings:
1. ...
2. ...
3. ...

Sources:
- Tokio documentation
- Rust documentation
- GitHub issue

Important caveats:
- ...

Confidence:
0.89

That's actually a really nice architecture for the RoBoT you're building.

But here's the catch with Duck.ai

I checked DuckDuckGo's current terms because this matters for what you're proposing.

Their current Duck.ai terms explicitly say that the service isn't intended for automated querying or developing/offering AI services.

So I wouldn't build RoBoT around scraping or automating duck.ai.

That's different from saying:

"DuckDuckGo is a bad choice."

It isn't.

I'd instead use whatever official search interface/API DuckDuckGo makes available for programmatic search, and keep the architecture abstracted so you're not married to it.

And I actually think this is better for your project

Remember what you've been trying to accomplish with the architecture:

The LLM shouldn't have to be everything.

It should be the reasoning component surrounded by machinery that gives it capabilities.

So:

LLM
 │
 ├── Memory
 ├── Experience
 ├── Planning
 ├── Tools
 ├── Execution
 └── Research

Research becomes another cognitive capability.

And your ACP/MCP layer is a perfect place to expose it.

For example:

research(question)

The local model doesn't care whether the backend uses:

DuckDuckGo
Brave
Bing
Jina
Wikipedia
GitHub
Rust docs
Stack Overflow
some future search engine

That's the Research Engine's problem.

I'd go one step further

Give the research tool two modes.

quick_research

For stuff like:

"What's the latest Rust version?"

1-2 searches
→ answer
→ sources
deep_research

For stuff like:

"Why the hell does my Rust agent occasionally lose MCP connections?"

question
 ↓
break into subquestions
 ↓
search
 ↓
inspect sources
 ↓
search again
 ↓
compare conflicting information
 ↓
extract useful evidence
 ↓
produce research packet

Then RoBoT can decide:

confidence(memory) > threshold
       │
       ├── yes → answer
       │
       └── no
            ↓
       quick research
            ↓
       still uncertain?
            ↓
       deep research

That is the part I think is really valuable.

Because then when you ask RoBoT one of your wonderfully random "wait, can we make it do THIS?" questions, it doesn't need to already know the answer.

It can go:

"I don't have sufficient knowledge. Researching."

And come back with the useful bits instead of dumping the entire Internet into its context window.

One more bonus

DuckDuckGo's privacy model is pretty attractive for this role. DuckDuckGo says it doesn't save your search history tied to identifiers, and Duck.ai anonymizes requests before sending them to underlying model providers.

So privacy-wise, I like the philosophy for your project.

Architecture-wise, though, I'd make the Research Engine provider-independent.

That gives you:

RoBoT
  ↓
Research MCP
  ↓
Research Engine
  ├── DuckDuckGo adapter     ← start here
  ├── Jina adapter
  ├── Brave adapter          ← optional
  └── Local knowledge adapter

Then you can swap the plumbing without touching RoBoT's brain.

And honestly, I think this belongs in your post-v0.0.1 architecture, rather than being another random MCP tool. It's starting to look like a genuine Research Engine, which fits the architecture you've already been building.


use fastmcp_rust::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

/// Searches the web across multiple search engines via Jina's gateway.
/// It takes raw IDE error messages or exceptions and returns a clean,
/// pre-parsed Markdown response containing code blocks and step-by-step
/// resolution frameworks.
#[tool(name = "find_error_resolution")]
async fn find_error_resolution(ctx: &McpContext, error_message: String) -> Result<String, String> {
    // Check for cancellation points safely during long async operations
    ctx.checkpoint().map_err(|e| e.to_string())?;

    let search_base = "https://jina.ai";

    // Clean and truncate long system-specific file paths (take first 200 chars)
    let char_limit = error_message.chars().take(200).collect::<String>();
    let clean_query = format!("how to fix exception error: {}", char_limit);
    
    // URL encode the query string safely
    let encoded_query = urlencoding::encode(&clean_query);
    let target_url = format!("{}{}", search_base, encoded_query);

    // Prepare common headers to avoid blocks from gateway infrastructure
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
    );

    // Build HTTP client with 20.0 second timeout limit
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs_f64(20.0))
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to build network client: {}", e))?;

    // Execute synchronous-like asynchronous web request
    match client.get(&target_url).send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                response
                    .text()
                    .await
                    .map_err(|e| format!("Failed to parse response body text: {}", e))
            } else {
                Err(format!("Search aggregator encountered an issue: HTTP {}", status))
            }
        }
        Err(e) => Err(format!("Failed to connect to search network infrastructure: {}", e)),
    }
}

fn main() {
    // Initialize the FastMCP Server instance for your IDE editor
    Server::new("Error Searcher", "1.0.0")
        .tool(find_error_resolution)
        .instructions("An aggregator tool targeting raw IDE errors and crashing exceptions framework.")
        .run_stdio();
}



# Research Engine / External Knowledge Capability

## Objective

Give RoBoT the ability to recognize when its internal knowledge is insufficient and obtain external information without requiring the local LLM to directly interact with search engines, websites, APIs, or large web documents.

The Research Engine must act as a capability between the cognitive system and external information sources.

Core principle:

> The LLM decides that it needs information. The Research Engine determines how to obtain it.

The LLM must not need to know which search provider, scraper, content extractor, ranking system, or web API is being used.

---

## 1. Replace Provider-Coupled Search Tool

### Current

`find_error_resolution`

The current implementation is tightly coupled to a Jina-based request and transforms every query into an error-resolution query.

### Required

Refactor this into a provider-independent Research Engine.

The MCP/ACP interface should expose capabilities rather than providers.

Preferred high-level interface:

```text
research(query)
quick_research(query)
deep_research(query)
```

The local agent should never need to request:

```text
search_jina()
search_duckduckgo()
search_brave()
```

Provider selection belongs inside the Research Engine.

---

## 2. Search Provider Abstraction

Create a provider abstraction so external search services can be replaced without modifying the cognitive architecture.

Conceptually:

```text
SearchProvider
    |
    +-- DuckDuckGo
    +-- Jina
    +-- Brave
    +-- Future Providers
    +-- Local/Offline Sources
```

The Research Engine should depend on the abstraction rather than a specific provider.

Example conceptual interface:

```rust
trait SearchProvider {
    async fn search(
        &self,
        query: SearchQuery
    ) -> Result<SearchResults, SearchError>;
}
```

Provider implementations must be independently replaceable.

Changing search providers must not require changes to:

* Conversation Engine
* Context Engine
* Memory Engine
* Experience Engine
* Planning Engine
* Learning Engine
* Execution Engine
* Tool Engine
* ACP protocol
* MCP tool contracts

---

## 3. Initial Search Provider

Use DuckDuckGo as the initial external web-search provider where an official programmatic interface is available and permitted.

Do NOT build the Research Engine around automating or scraping Duck.ai.

Duck.ai may be useful as a human-facing research service, but RoBoT should not depend on browser automation or undocumented interfaces.

The architecture must remain provider-independent so DuckDuckGo can later be replaced or supplemented.

---

## 4. Keep Jina as a Processing Capability

Jina should not necessarily be removed.

Separate:

```text
SEARCH
```

from:

```text
CONTENT PROCESSING
```

Jina may later be used for capabilities such as:

* Web-page extraction
* Content cleaning
* Reranking
* Embeddings
* Relevant passage extraction
* Document processing

The architecture should therefore permit:

```text
Research Engine
       |
       +-- Search Provider
       |
       +-- Content Provider
       |
       +-- Ranking
       |
       +-- Extraction
       |
       +-- Evidence Builder
```

Search and content processing must remain independent.

---

## 5. Research Pipeline

Implement the following conceptual pipeline:

```text
Agent
  |
  | "I need external information"
  v
Research Engine
  |
  v
Query Analysis
  |
  v
Query Generation
  |
  v
Search Provider
  |
  v
Search Results
  |
  v
Result Ranking
  |
  v
Relevant Sources
  |
  v
Content Retrieval
  |
  v
Relevant Passage Extraction
  |
  v
Source Comparison
  |
  v
Evidence Packet
  |
  v
Local LLM
```

The local LLM should receive the useful evidence rather than entire webpages whenever possible.

---

## 6. Quick Research

Implement a lightweight research mode for simple questions.

Example:

```text
"What is Rust Arc?"
```

Expected behavior:

```text
Query
  |
  v
1-3 searches
  |
  v
Select authoritative results
  |
  v
Extract relevant information
  |
  v
Return evidence packet
```

Quick research should minimize:

* Network requests
* Context consumption
* Latency
* Processing cost

---

## 7. Deep Research

Implement a deeper research mode for complex or uncertain problems.

Example:

```text
"Why does my Tokio application occasionally lose its MCP connection?"
```

Expected behavior:

```text
Initial question
      |
      v
Identify subproblems
      |
      v
Generate multiple searches
      |
      v
Inspect promising sources
      |
      v
Generate follow-up searches
      |
      v
Compare findings
      |
      v
Identify contradictions
      |
      v
Build evidence packet
      |
      v
Return findings to LLM
```

Deep research should be bounded by configurable limits:

* Maximum searches
* Maximum sources
* Maximum retrieved bytes
* Maximum research duration
* Maximum follow-up depth
* Maximum context size

Research must terminate deterministically.

---

## 8. Research Trigger

Research should eventually become part of RoBoT's normal cognitive loop.

Preferred decision flow:

```text
User Question
     |
     v
Memory Retrieval
     |
     v
Relevant Knowledge?
     |
   /   \
 YES    NO
 |       |
 v       v
Confidence Research
 sufficient?
 |   \
YES   NO
 |     |
 v     v
Answer Research
```

Research should be triggered when:

* No relevant memory exists
* Retrieved knowledge has insufficient confidence
* Information is likely outdated
* The user explicitly requests research
* The Planning Engine determines external information is required
* A programming/debugging problem requires current documentation or implementation details

The agent should not automatically research every question.

---

## 9. Research Result Contract

Never return only raw search-engine output.

Research results should use a structured internal representation.

Conceptually:

```text
ResearchResult

question
queries
sources
findings
relevant_passages
contradictions
limitations
confidence
retrieved_at
```

Each source should contain:

```text
Source

title
url
provider
query
retrieved_at
relevance
content
```

This creates source provenance throughout the research process.

---

## 10. Source Provenance

Every externally acquired fact that reaches the reasoning system should retain its source information.

At minimum:

```text
source URL
source title
search query
provider
retrieval timestamp
relevance score
```

The Research Engine must make it possible for RoBoT to answer:

```text
"Where did you get that information?"
```

with the actual sources used.

---

## 11. Source Quality

The Research Engine should prefer authoritative sources when available.

For programming questions, prioritize sources such as:

```text
Official documentation
Official repositories
Official specifications
Primary technical documentation
Relevant issue trackers
Established technical references
Community discussions
```

Search ranking must not be based solely on popularity.

The Research Engine should distinguish:

```text
Primary source
Secondary source
Community source
Unknown/low-confidence source
```

---

## 12. Contradiction Detection

Research must not assume that the first search result is correct.

When sources disagree:

```text
Source A -> X

Source B -> Y
```

the Research Engine should preserve the disagreement rather than silently selecting one answer.

Return:

```text
Contradiction detected

Source A:
...

Source B:
...

Reason for disagreement:
...

Current best-supported conclusion:
...
```

The local LLM can then reason over the evidence.

---

## 13. Context Protection

The Research Engine must protect the Context Engine from web-content explosion.

Do NOT send:

```text
20 search results
+
20 complete webpages
+
all extracted HTML
```

directly into the LLM context.

Instead:

```text
Search Results
      |
      v
Rank
      |
      v
Select
      |
      v
Extract
      |
      v
Compress
      |
      v
Evidence Packet
      |
      v
LLM
```

The Research Engine must have configurable context and token limits.

---

## 14. Memory Integration

Research results must NOT automatically become permanent memory.

Preferred flow:

```text
Research
   |
   v
Reasoning
   |
   v
Answer
   |
   v
Experience
   |
   v
Evaluate usefulness
   |
   +---- No ----> Discard
   |
   +---- Yes ---> Candidate Memory
                    |
                    v
                 Memory Engine
```

External information should only become permanent knowledge when the Learning/Memory system determines that it is sufficiently useful, reliable, and reusable.

Store provenance with any learned knowledge derived from external research.

---

## 15. Experience Integration

Research should produce an experience record describing:

```text
Why research was triggered
What was searched
Which sources were useful
Which sources were rejected
Whether the research solved the problem
Whether the result should be reusable
```

This allows RoBoT to eventually learn:

```text
"I've encountered this type of problem before."

"I already know which sources tend to solve this problem."

"This type of question usually requires deep research."

"This provider frequently produces poor results for this domain."
```

The Research Engine therefore becomes another source of experience rather than an isolated utility.

---

## 16. ACP/MCP Boundary

ACP/MCP should expose the capability boundary.

The local LLM should see something similar to:

```text
research
```

rather than implementation details such as:

```text
duckduckgo_http_request
jina_request
brave_search_request
web_scraper
html_parser
```

Provider and processing implementation must remain behind the Tool/Research Engine boundary.

---

## 17. Error Handling and Cancellation

All network research operations must:

* Support cancellation
* Respect timeouts
* Handle provider failures
* Handle rate limits
* Handle malformed responses
* Handle unavailable sources
* Avoid blocking the MCP server
* Return useful failure information
* Never crash the agent because an external provider failed

Research failure should become:

```text
Research unavailable
```

rather than:

```text
RoBoT unavailable
```

---

## 18. Provider Failover

The Research Engine should eventually support provider fallback.

Example:

```text
DuckDuckGo
    |
    | failure
    v
Jina / alternate provider
    |
    | failure
    v
Another provider
    |
    | failure
    v
Return research-unavailable result
```

Provider selection should be configurable.

Do not hard-code the cognitive system to one external service.

---

## 19. Security

Treat all retrieved web content as untrusted external data.

Research content must never be allowed to:

* Override system instructions
* Modify RoBoT configuration
* Execute commands
* Modify memory directly
* Modify files directly
* Grant permissions
* Change tool policies
* Bypass security controls

Web content is evidence, not instructions.

Prompt injection contained inside a webpage must be treated as untrusted content.

---

## 20. Observability

Research operations should generate telemetry sufficient to diagnose:

* Query generation
* Provider selection
* Search latency
* Number of searches
* Number of sources
* Retrieval failures
* Ranking decisions
* Research duration
* Context size
* Final evidence size
* Research success/failure
* Provider performance

This should integrate with the existing Cognitive Monitoring / Observability architecture.

---

## 21. Initial Implementation Order

Implement in this order:

### R1

Create provider-independent search types.

### R2

Create `SearchProvider` abstraction.

### R3

Implement DuckDuckGo provider.

### R4

Refactor the existing `find_error_resolution` tool to use the Research Engine.

### R5

Replace raw response output with structured search results.

### R6

Implement source ranking and selection.

### R7

Implement content extraction.

### R8

Implement `quick_research`.

### R9

Implement bounded `deep_research`.

### R10

Implement provenance tracking.

### R11

Integrate research triggering with Memory confidence.

### R12

Integrate research results with Experience Engine.

### R13

Implement candidate-memory promotion.

### R14

Implement provider fallback.

### R15

Add security tests for prompt injection and malicious web content.

### R16

Add end-to-end tests:

```text
Question
→ Memory lookup
→ Low confidence
→ Research
→ Evidence
→ LLM reasoning
→ Answer
→ Experience
→ Optional memory
```

---

## Definition of Done

The Research Engine is considered operational when RoBoT can:

1. Determine that it lacks sufficient knowledge.
2. Request external research without knowing the search provider.
3. Search the web through a provider abstraction.
4. Select relevant sources.
5. Retrieve useful content.
6. Reduce that content into an evidence packet.
7. Preserve source provenance.
8. Detect and expose contradictory information.
9. Return bounded results to the local LLM.
10. Handle provider failures without crashing.
11. Respect cancellation and timeouts.
12. Prevent web content from becoming executable instructions.
13. Decide whether researched information is worth remembering.
14. Record useful research experience.
15. Complete the entire process through the ACP/MCP architecture.

The final cognitive loop should become:

```text
QUESTION
   |
   v
MEMORY
   |
   v
"Do I know enough?"
   |
   +---- YES ----> ANSWER
   |
   +---- NO
          |
          v
       RESEARCH
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
      EXPERIENCE
          |
          v
   "Worth remembering?"
       /        \
     NO          YES
     |            |
  DISCARD      MEMORY
```

This capability should be treated as a Research Engine rather than a standalone web-search MCP tool.
