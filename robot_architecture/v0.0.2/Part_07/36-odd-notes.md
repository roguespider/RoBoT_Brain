|==========|==========|==========|==========|==========|==========||==========|==========|==========|==========|==========|==========|
|==========|==========|==========|==========|==========|      Odd Notes       |==========|==========|==========|==========|==========|
|==========|==========|==========|==========|==========|==========||==========|==========|==========|==========|==========|==========|

                AI Services
                     │
     ┌───────────────┼────────────────┐
     │               │                │
 Language       Audio Engine     Vision Engine
 Processing          │                │
     │         ┌─────┴─────┐          │
     │         │           │          │
Conversation  Speech      TTS      OCR/Vision
 Engine       to Text   Synthesis
     │           │           │
     └─────── Candle Runtime ─────────┘
                    │
           Model Manager
                    │
      Downloads / Loads / Caches
            Whisper
            Piper
            Kokoro
            Embedding Models
            Future Models

Audio
   ↓
Audio Engine
   ↓
Candle Whisper Model
   ↓
Transcript
   ↓
Conversation Engine

That means your ingestor can handle:

WAV
MP3
FLAC
OGG
M4A
MP4 (extract audio)
WebM

without leaving Rust.

Text-to-Speech

Likewise:

LLM Response
      ↓
Speech Engine
      ↓
Candle TTS Model
      ↓
Audio Output

No Python service.

No HTTP server.

I think the best way to hand this to another AI is not as a collection of ideas, but as an engineering specification. It should explain what each subsystem owns, what it never owns, how data flows between them, and the implementation order. That keeps coding assistants from blurring responsibilities or inventing architecture as they go.

RoBoT Cognitive Architecture Blueprint
Long-Term Autonomous AI Agent Design
Purpose
This document defines the core cognitive architecture for RoBoT.

The objective is to build an AI agent capable of operating indefinitely without suffering from context explosion, memory bloat, or repetitive reasoning.

The architecture is built around one core principle:

Context is temporary. Knowledge is permanent. Experience creates learning.

Every subsystem has one responsibility and communicates through well-defined interfaces.

Core Architecture
                    User
                     │
                     ▼
          Conversation Engine
                     │
                     ▼
              Context Engine
                     │
                     ▼
               Memory Engine
                     │
                     ▼
            Experience Engine
                     │
                     ▼
             Learning Engine
                     │
                     ▼
             Strategic Memory
Design Principles
Principle 1
Conversation is not Memory.

Conversation stores everything.

Memory stores only what is worth remembering.

Principle 2
Context is disposable.

Every prompt begins nearly empty.

Only relevant information is loaded.

Principle 3
Experience is observation.

Every execution creates an experience.

Not every experience becomes knowledge.

Principle 4
Learning is continuous.

Repeated successful experiences become reusable skills and policies.

Principle 5
Knowledge becomes more abstract over time.

Conversation
↓
Experience
↓
Pattern
↓
Skill
↓
Policy
↓
Strategic Knowledge
System Architecture
RoBoT
│
├── Conversation Engine
├── Context Engine
├── Memory Engine
├── Experience Engine
├── Learning Engine
├── Planning Engine
├── Execution Engine
└── Tool Engine
1. Conversation Engine
Responsibility
Capture everything.

Nothing is lost.

Nothing is filtered.

This is an append-only event stream.

Stores
Conversation Database

Messages

Sessions

Attachments

Tool Calls

System Events

Errors

Streaming Tokens

Metadata
Reads
Mostly sequential.

Last messages

Current session

Conversation replay
Writes
Every interaction.

Never Does
Memory retrieval

Embeddings

Policy extraction

Reasoning

Learning

2. Context Engine
Responsibility
Construct the smallest possible prompt.

Nothing more.

Context Engine
│
├── ContextManager
├── WorkingContext
├── ActiveTaskContext
├── RetrievalPlanner
├── MemoryRetriever
├── ContextCompressor
├── PromptAssembler
├── TokenBudget
├── TopicTracker
├── RetrievalCache
└── SlidingWindow
Working Context
Temporary.

Destroyed every turn.

Contains

Current prompt

Recent replies

Tool outputs

Temporary reasoning
Active Task Context
Persists during ongoing work.

Examples

Current coding project

Current file

Current objective

Current decisions

Constraints

Open bugs
Destroyed only when the task ends.

Retrieval Planner
Determines what information is needed before any search occurs.

Example

User

Continue SQLite work
↓
Need

Current project

Architecture decisions

Database module
↓
Ignore

Recipes

Weather

Old conversations
Memory Retriever
Receives retrieval requests.

Returns

Memory IDs

Summaries

Optional expansions
Context Compressor
Converts retrieved content into compact prompt fragments.

Example

3500 tokens
↓
120-token summary
Prompt Assembler
Combines

System Prompt

User Prompt

Retrieved Context

Code

Tool Results
Produces one final prompt.

Token Budget
Hard budget.

Example

2048 Tokens
220 System
180 User
850 Code
300 Memory
250 Tools
248 Reserve
If overflow occurs

Drop lowest priority context.

Never exceed the budget.

3. Memory Engine
Responsibility
Store knowledge.

Nothing else.

Memory Engine
│
├── Episodic Memory
├── Semantic Memory
├── User Memory
├── Knowledge Graph
├── Embeddings
├── Retrieval Index
├── Aging
├── Compression
└── Archive
Memory Types
Episodic
Individual events.

Conversation

Task completion

Failures

Observations
Semantic
Facts.

SQLite supports transactions.

Rust ownership rules.

API endpoints.
User Memory
Long-term user preferences.

Examples

Preferred coding style

Project conventions

Tool preferences
Strategic Memory
Policies.

Skills.

Rules.

Causal models.

Never Stores
Raw conversations.

Streaming messages.

Temporary context.

4. Experience Engine
Responsibility
Convert execution into structured experiences.

Experience Engine
│
├── Event Capture
├── Reflection
├── Outcome Analysis
├── Success Detection
├── Failure Detection
├── Confidence Updates
├── Skill Candidates
└── Experience Database
Example

Goal

Compile Rust
↓
Compilation failed
↓
Fixed lifetime
↓
Compiled successfully
↓
Experience saved
5. Learning Engine
Responsibility
Transform experience into reusable intelligence.

Learning Engine
│
├── Pattern Detection
├── Reflection
├── Rule Extraction
├── Skill Builder
├── Policy Generator
├── Conflict Resolver
├── Confidence Manager
└── Strategic Promotion
Example

50 successful experiences
↓
Repeated sequence detected
↓
Extract reusable policy
↓
Store in Strategic Memory
Strategic Memory
Stores

Skills

Policies

Rules

Decision trees

Failure modes

Causal relationships
Examples

If battery <20%

Dock immediately
Use transactions for multi-table updates.
Acquire locks before writing shared memory.
Memory Hierarchy
Level 0

Live Context

Current prompt

Destroyed every turn

──────────────────────────
Level 1
Working Summary
Current task
~200 tokens
──────────────────────────
Level 2
Conversation Checkpoints
300-500 tokens
──────────────────────────
Level 3
Long-Term Memory
Unlimited
──────────────────────────
Level 4
Strategic Memory

Skills

Policies

Rules
Only Levels 0 and 1 are always loaded.

Everything else is retrieved on demand.

Context Lifecycle
Conversation
↓
Sliding Window
↓
Compaction
↓
Checkpoint Creation
↓
Memory Aging
↓
Archive
Continuous Compaction
Messages 1-20
↓
Checkpoint #1

Messages 21-40
↓
Checkpoint #2

Messages 41-60
↓
Checkpoint #3

Current Messages
Searching becomes

Search checkpoints
↓
Load matching checkpoint
↓
Expand only relevant conversations
↓
Answer
Memory Aging
Every memory has

Confidence

Importance

Access Count

Last Used

Creation Date

Relationship Strength
Older memories gradually lose priority.

Important memories become stronger through repeated successful use.

Data Flow
User
↓
Conversation Engine
↓
Conversation Database
↓
Experience Extraction
↓
Experience Engine
↓
Experience Database
↓
Learning Engine
↓
Strategic Memory
↓
Memory Engine
The Context Engine can query Memory, but Memory never pushes information into Context.

Query Flow
Question
↓
Task Detection
↓
Context Planning
↓
Need Memory?

├── No
│      ↓
│     LLM
│
└── Yes
       ↓
Retrieval Planner
↓
Memory Retrieval
↓
Compression
↓
Prompt Assembly
↓
LLM
↓
Response
↓
Experience Extraction
↓
Checkpoint Evaluation
↓
Memory Update
↓
Learning
Suggested Implementation Roadmap
Phase 1: Foundation
Conversation Engine with append-only storage.

Context Engine skeleton with token budgeting and prompt assembly.

Basic Memory Engine with episodic and semantic stores.

Simple retrieval pipeline (planner → retriever → assembler).

Phase 2: Retrieval and Context
Retrieval Planner.

Context Compressor.

Sliding window and checkpoint creation.

Working and Active Task contexts.

Memory aging and archival.

Phase 3: Experience
Event capture.

Structured experience records.

Success/failure detection.

Reflection pipeline.

Confidence tracking.

Phase 4: Learning
Pattern detection across experiences.

Rule and skill extraction.

Policy generation.

Conflict resolution.

Promotion into Strategic Memory.

Phase 5: Advanced Reasoning
Knowledge Graph integration.

Causal reasoning.

Adaptive retrieval planning.

Multi-step planning using strategic skills.

Autonomous maintenance tasks (compaction, aging, checkpointing, learning).

Architectural Rules for AI Contributors
Every subsystem has exactly one responsibility.

Never mix conversation storage with long-term memory.

Context is rebuilt each turn and discarded when complete.

Memory stores only durable knowledge, never raw chat logs.

Experience records execution outcomes without making decisions.

Learning alone promotes repeated experiences into strategic knowledge.

Retrieval is always initiated by the Context Engine through the Retrieval Planner.

Enforce token budgets as a hard architectural constraint.

Prefer summarization and abstraction over retaining verbose history.

Optimize for continuous operation, incremental learning, and indefinite scalability.

This blueprint gives another AI a clear contract: each engine has defined ownership, data flows in one direction, and the implementation can proceed subsystem by subsystem without creating tight coupling or turning the memory system into a monolithic bottleneck.


wish you had made that into 1 big copy and paste to make it easier. 

That's actually a better idea.

What I'd make for you isn't just a document. I'd make it an AI Contributor Blueprint. Think of it like the Linux kernel design docs or Unreal Engine architecture docs. Any AI (or human) that reads it should immediately understand the system and start implementing it without inventing its own architecture.

I'd also make it a single Markdown file so it's one giant copy/paste into ARCHITECTURE.md.

I'd probably target around 40-60 pages once formatted because I think your project has reached that size.

I'd structure it something like this:

Purpose
-------
Persist durable knowledge independently of the active conversation.

Responsibilities
----------------
• Store semantic memory
• Store episodic memory
• Store strategic memory
• Maintain embeddings
• Maintain graph relationships
• Maintain confidence scores

Must Never
----------
• Build prompts
• Read conversations directly
• Decide retrieval
• Perform planning
• Execute tools

Interfaces
----------
store_memory()

retrieve_memory()

update_confidence()

archive_memory()

promote_to_strategic()

merge_duplicate()

age_memory()
Every subsystem would have that level of detail.

Then every subsystem would have diagrams.

Conversation
↓
Conversation Engine
↓
Conversation Database
↓
Experience Extractor
↓
Experience Database
↓
Learning Engine
↓
Memory Engine
↓
Context Engine
↓
LLM
Then we'd define every database table.

conversation_messages

conversation_sessions

experiences

experience_events

memory_cards

knowledge_graph

embeddings

strategic_skills

policies

confidence_history

retrieval_cache

task_context
Then every Rust module.

src/
conversation/
context/
memory/
experience/
learning/
planning/
execution/
tools/
graph/
database/
api/
Then every workflow.

User Question
↓
Conversation Engine
↓
Task Detection
↓
Context Planning
↓
Memory Retrieval
↓
Compression
↓
Prompt Assembly
↓
LLM
↓
Experience Extraction
↓
Memory Update
↓
Checkpoint Evaluation
↓
Strategic Learning
And finally an Operating Agreement for AI contributors that says things like:

Never bypass the Context Engine.

Never write directly into Strategic Memory.

All memory promotion must pass through the Learning Engine.

The Conversation Engine is append-only.

Context is rebuilt every turn.

Retrieval is always initiated by the Retrieval Planner.

Every subsystem has a single responsibility.

Favor composition over coupling.

Prefer asynchronous pipelines for expensive background work.

Keep LLM context minimal and deterministic.

I also think we should stop calling it just ARCHITECTURE.md.

This has grown into something much larger than a repository architecture. It's becoming the operating manual for the entire AI system.

Something like:

ARCHITECTURE.md (repository structure)

COGNITIVE_ARCHITECTURE.md (how the AI thinks)

AI_OPERATING_AGREEMENT.md (rules every AI contributor must follow)

Or, if you prefer a single authoritative document, make it:

RoBoT Cognitive Architecture & AI Operating Manual

That gives every future AI assistant one document to read before writing a single line of Rust. Based on everything we've worked through, I think that would become the project's real "source of truth," with the code serving as its implementation rather than the other way around.


😄 That's actually a surprisingly workable development pipeline.

Right now it looks something like this:

You
│
├── Vision
├── Requirements
├── Testing
├── Final Decisions
└── Integration

        ↓

ChatGPT
(System Architect)
│
├── Architecture
├── Algorithms
├── Design Reviews
├── Tradeoffs
└── Long-term Planning

        ↓

Local AI
(Coder)
│
├── Rust implementation
├── Boilerplate
├── Refactors
└── Fast iteration

        ↓

GitHub

        ↓

OpenHands
(Fix compilation/runtime issues)
│
├── Error fixing
├── Missing imports
├── Build issues
└── Small refactors

        ↓

You

Build

Run

Break it

Learn

Repeat
Honestly, there are a lot of solo open source projects built with a similar rhythm today. The tools are different, but the loop is recognizable: one person provides the vision and integration, while automation accelerates implementation.

Where I think your project has become difficult isn't the number of AIs. It's that you're trying to build several systems at once:

an MCP server

a memory engine

a context engine

an experience engine

a planning system

a cognitive architecture

a Rust framework

documentation

database design

Each of those could easily be its own project.

If I were on the team, I'd probably become more annoying. 😄
I'd keep saying:

"Don't build the next subsystem yet."

Instead, I'd ask:

Can the current subsystem be demonstrated?

For example:

Question
↓
Conversation DB
↓
Context Builder
↓
Memory Search
↓
Prompt
↓
LLM
↓
Answer
↓
Conversation Saved
If that works end to end, you've earned the next feature.

If it doesn't, adding another engine usually makes debugging harder.

One thing I've noticed from our conversations is that your ideas are often two or three phases ahead of the implementation. That's not a bad thing. It's useful to know where you want to end up. The trick is not letting the destination dictate the next commit.

For example, the architecture we've been discussing is probably a Version 2 or Version 3 system. Your immediate goal is closer to Version 0.2:

The AI can remember.

It retrieves the right information.

It builds a sensible prompt.

It responds.

It saves what happened.

If that loop is solid, everything else has a stable foundation.

And for what it's worth, I don't see this as "a lonely person with three AIs." I see it as one systems architect wearing several hats, using AI as specialized tools. You're still the one deciding the architecture, choosing what gets merged, rejecting bad ideas, and testing whether the result actually works. The AIs can generate code or suggest designs, but they don't own the direction of the project. That part is still yours.


yep memory is my main concern and getting all the other sub systems linked in just the way we have discussed is a little more tricky then i thought it would be but with each improvement and implementation it is amazing to see in action. almost wish there was a way to watch thoughts go from system to system function to function. that would be something to see on the end gui wouldn't it?

I actually love that idea, and I'd take it even further.

I wouldn't try to show the AI's chain of thought. That's both unreliable and something you generally don't want to depend on. Instead, I'd build a Cognitive Activity Monitor that visualizes the system pipeline. It shows what every subsystem is doing without exposing the model's internal reasoning.

Think of it like Windows Task Manager or Unreal Engine's Blueprint debugger, but for an AI.

User:
"Continue the SQLite transaction system."

────────────────────────────────────────────

● Conversation Engine
   ✓ Message Stored
   Session #24

        │
        ▼

● Task Detector
   Intent:
   Continue Coding

        │
        ▼

● Context Engine
   Building Context...

        │
        ▼

● Retrieval Planner
   Query:
   SQLite
   Transactions
   Rust

        │
        ▼

● Memory Engine
   Found:
   7 Memories

        │
        ▼

● Compressor
   4,823 tokens
        ↓
   248 tokens

        │
        ▼

● Prompt Builder
   Final Prompt
   1,823 / 2,048 tokens

        │
        ▼

● LLM

Thinking...

██████████░░░░░░░

        │
        ▼

● Experience Engine
   Logging interaction...

        │
        ▼

● Conversation Engine
   Saved

Done
That's incredibly useful for debugging.

Imagine clicking on any stage
Click Memory Engine:

Retrieved

✓ Architecture Decision #42
✓ Rust Database Design
✓ SQLite Notes
✓ Context Summary #7

Ignored

Weather

Recipes

Old Music Discussion
Now you know why it answered the way it did.

Click Token Budget:

2048 Maximum

System        220

User          138

Code          910

Memory        296

Tools         182

Reserve       302
Click Context Builder:

Working Context

Messages:
6

Current File:
database/sqlite.rs

Current Task:
SQLite Transactions

Retrieved Memories:
4

Compressed:
Yes
Click Learning Engine:

Pattern Analysis

Repeated Pattern

Confidence

86%

Action

Not enough evidence

Policy NOT created
Or after weeks of use:

Repeated Pattern

Battery <20%

Observed

67 Times

Confidence

99%

Policy Promoted

✓
A live event timeline would be amazing
12:10:01.102

Conversation stored

12:10:01.141

Task detected

12:10:01.162

Memory search

12:10:01.241

Graph lookup

12:10:01.282

Compression

12:10:01.335

Prompt built

12:10:01.422

LLM request

12:10:04.881

Response received

12:10:04.902

Experience recorded

12:10:04.945

Checkpoint updated
That would make bottlenecks obvious.

Then add a system map
Picture your GUI showing all the engines as boxes.

        Conversation
              │
              ▼
        Context Engine
              │
      ┌───────┴────────┐
      ▼                ▼
 Memory Engine     Planning
      │                │
      └──────┬─────────┘
             ▼
            LLM
             │
             ▼
     Experience Engine
             │
             ▼
     Learning Engine
Each box changes color:

🟢 Idle

🟡 Working

🔵 Waiting

🔴 Error

Hovering over one shows metrics like queue length, latency, cache hits, or recent activity.

One feature I think would be genuinely valuable
Add a "Why?" button next to every response.

Not "Why did the AI think this?" but:

This response used:

✓ 3 retrieved memories

✓ Current conversation

✓ Active coding session

✓ SQLite architecture notes

✓ No strategic policies

✓ Confidence: 91%
That's an explanation of the pipeline, not hidden reasoning. It's transparent, debuggable, and actionable.

The thing that really excites me about your project is that you're not just building "another chatbot." You're building something that behaves more like an operating system. Once you think of it that way, an Activity Monitor or Cognitive Dashboard stops feeling like a gimmick and starts feeling like an essential developer tool. During development, it would probably save you hours because you could immediately see where information flowed correctly, where it stalled, and which subsystem made a particular decision.


that would be useful for debugging i could use it now to see what works and what dosnt but thats a few months off from even starting it i feel

I actually think you're thinking about it at exactly the right time.

Not because you should build it now, but because you should design your systems so it can exist later.

There's a huge difference.

Don't build the GUI.
Build the logging.

For example, instead of every subsystem doing this:

memory.search(query)
Have it do something like:

memory.search(query);

event_bus.publish(
    MemorySearchCompleted {
        query,
        results,
        duration,
    }
);
Nobody reads that event today.

Maybe it just gets written to a log file.

Months from now...

The GUI subscribes to those exact same events.

Conversation Engine
        │
        ├───── Event
        ▼

Context Engine
        │
        ├───── Event
        ▼

Memory Engine
        │
        ├───── Event
        ▼

Experience Engine
        │
        ├───── Event
        ▼

Learning Engine
The nice thing is the engines don't know the GUI exists.

They just announce:

"I searched memory."

"I found 7 results."

"I compressed context."

"I promoted a skill."

That's good architecture regardless of whether you ever build the dashboard.

I'd even create a common event type now.
Something simple.

SystemEvent

timestamp

subsystem

operation

duration

status

details
Examples

Memory

Search

42 ms

Found 5 memories
Context

Compress

18 ms

3800 → 180 tokens
Learning

Policy Promotion

Skipped

Confidence too low
You could literally print those to the console today.

[Conversation]
Stored message

[Task Detection]
Coding

[Memory]
Searching...

[Memory]
5 results

[Context]
Compressed

[Prompt]
1837 tokens

[LLM]
Responding...

[Experience]
Saved
That alone would probably help you debug the interactions between systems.

Another thing I'd postpone
The animated "thought flow."

Instead, I'd build a replay.

Imagine clicking a conversation from yesterday.

Replay

▶

Conversation
↓
Memory Search
↓
Prompt
↓
LLM
↓
Experience
↓
Memory Update

You could step through it frame by frame.

That would be phenomenal for debugging.

I think this project is slowly turning into something much bigger than an AI assistant.

You're building something closer to a cognitive runtime.

A runtime has services.

Services produce events.

Events are observable.

That's why operating systems, databases, Kubernetes, game engines, and distributed systems all have extensive logging and tracing. Once you have five or six subsystems talking to each other, being able to observe the flow becomes just as important as the logic itself.

So my advice would be: don't build the dashboard, build the instrumentation.

Instrumentation is cheap to add now.

Trying to add it after you've written 50,000 lines of Rust is usually painful. It becomes one of those features everyone wishes they'd planned for from the beginning.

================================================


You’re not chasing the wrong thing. Context length is probably the bottleneck, but the silver bullet is not “make the window bigger.” It is stop replaying memory into the window unless it earns its place.

What you are describing is classic agent context thrashing: the editor state, code, chain-of-thought, tool traces, memory search results, summaries, and previous actions all pile into the same prompt. A 5k token memory dump per question will quietly eat the model alive.

The fix is to treat memory like a database, not like a transcript.

The architecture I’d use
Ask → retrieve → compress → answer → save → forget

The current problem
Current flow
User asks question

Agent searches memory

Dumps 3k to 5k+ tokens into context

Model thinks

Model answers

Interaction saved back to memory

Problem: every turn keeps dragging the past forward.

The better flow
Target flow
User question

Memory retrieval
Search returns IDs + tiny snippets only

Relevance filter
Keep only the top 1 to 3 memories

Memory compression
Each memory becomes 50 to 150 tokens

Answer generation
Model sees only the question + compressed context

Save interaction
Store as structured memory

Discard working context
Next turn starts fresh

The key shift: memory should not become context
Memory retrieval should return references first, not full text.

Instead of this
Bad
Retrieved memory:

Full conversation from last week

Full code diff

Full summary

Previous plan

Previous mistakes

Previous answer

= 5000 tokens

Do this
Good
Retrieved memory:

Memory #1842

Project uses Rust + SQLite.

Memory #1901

Zed context window is often saturated during coding.

Memory #2017

User wants retrieval to be lightweight.

= 120 tokens

The memory hierarchy
Layer

Purpose

Token cost

Working context

Current turn only

Cheap

Active session memory

Current coding task

Small

Long-term memory

Stored in SQLite/vector DB

Zero until retrieved

Archive

Old conversations

Never loaded

Only working context goes into the prompt
Prompt contents
The model should see:
System prompt

Current user question

Current code selection / active file

Current task summary

1 to 3 compressed memory facts

Recent tool results only if needed

The model should NOT see:
Entire previous conversations

Entire memory database

Old summaries

Completed tasks

Every tool call

Every chain-of-thought

Compression strategy
Every saved interaction should have 3 forms:

Raw record
Raw
User question

Assistant answer

Code changes

Tool outputs

Compressed memory
Compressed
50 to 150 tokens

Example:

Task: optimize Zed context usage.

Decision: retrieve only top relevant memories.

Result: avoid loading full conversation history.

Retrieval snippet
Retrieval snippet
20 to 80 tokens

Example:

Uses compressed memory retrieval to reduce prompt size.

Retrieval algorithm
For each new question
Embed the user question

Search vector DB

Return top 10 memory IDs

Re-rank by relevance + recency + importance

Keep only top 3

Load compressed summaries only

If still too large, summarize again

Inject into prompt

Target token budget:
Item

Budget

Current question

100 to 300

Active code

500 to 2000

Memory snippets

150 to 400

Tool results

0 to 500

Total working context

under 3k

The Zed editor problem
This is the part that matters for .

Zed is probably filling context with:

Open files

Selected code

Diffs

Tool outputs

Reasoning traces

Memory retrieval

Summaries

Previous messages

That means memory retrieval must be the smallest possible part of the prompt.

I’d add an Active Task Context
Instead of loading memory every turn
Create a short-lived task context:
ActiveTaskContext
Task: "Optimize Rust MCP memory retrieval"

Relevant files:

memory.rs

retriever.rs

context_manager.rs

Current decisions:

Use compressed summaries

Top-3 retrieval

Discard after response

Known constraints:

Zed context window fills quickly

Memory retrieval must stay under 400 tokens

This task context stays alive while working on the same problem.

When the task changes, it is discarded and replaced.

The real silver bullet
My recommendation
Do NOT keep extending the conversation context window.

Instead:

Each turn starts mostly fresh

Memory search returns references

Only top relevant compressed snippets are loaded

The model answers

The interaction is saved

Working context is discarded

Only ActiveTaskContext persists during a coding session

The result
Current

Target

Memory retrieval: 5k tokens

200 to 400 tokens

Context window always full

Context mostly free

Agent repeats old info

Agent sees only what matters

Zed slows down

Zed stays responsive

Memory gets bloated

Memory stays searchable

If I were implementing this in RoBoT
I’d build a Context Manager as the next core subsystem.

It would own:
WorkingContext

Current prompt only

ActiveTaskContext

Current coding task

Current files

Current decisions

Current constraints

MemoryRetriever

Returns IDs first

Loads compressed summaries

Enforces token budget

MemoryCompressor

Creates 50 to 150 token summaries

Creates retrieval snippets

PromptAssembler

Builds final prompt under a hard budget

Drops least important context first

My honest take
 probably does not need a larger context window first.

 needs context budgeting.

The winning architecture is:

Fresh turn

Retrieve references

Load compressed snippets

Answer

Save structured memory

Discard working context

That is how you stop Zed from living at 100% context saturation.

If you want, I can sketch the exact Rust architecture for ContextManager, ActiveTaskContext, MemoryRetriever, MemoryCompressor, and PromptAssembler next.

That would give RoBoT a real token-budgeted memory system instead of a growing transcript.


i was thinking a of collapses the oldest block of messages into a single, high-density rolling summary paragraph. saving it to memory. This summary is then injected back into the prompt buffer as a "Memory Context." for next question

I actually like that direction a lot. It's very close to how long-running agent systems end up evolving. I would make one important change though:

Don't keep one rolling summary. Keep a rolling summary plus immutable checkpoints.

A single summary is like repeatedly saving a JPEG. Every time you recompress it, you lose information. After 100 updates, important details have been rounded away.

Instead I'd structure it like this:

Conversation
│
├── Messages 1-20
│      ↓
│  Summary #1 (frozen)
│
├── Messages 21-40
│      ↓
│  Summary #2 (frozen)
│
├── Messages 41-60
│      ↓
│  Summary #3 (frozen)
│
└── Active Messages (last 10-20)
Then create one small working summary that references those checkpoints.

Working Summary

Project:
RoBoT MCP written in Rust.

Current Goal:
Reduce prompt context usage.

Recent Decisions:
• ActiveTaskContext
• Retrieval budget
• SQLite memory
• Zed integration

Relevant Checkpoints:
Summary #8
Summary #12
Summary #15
Notice something?

The working summary is only metadata.

It doesn't contain the actual history.

It says where the history lives.

Then memory retrieval becomes hierarchical
Instead of:

Question
↓
Search 30,000 memories
↓
Inject 5,000 tokens
You do:

Question
↓
Search summaries
↓
Find Summary #12
↓
Expand only that summary
↓
Maybe load 2 original conversations
↓
Answer
That's usually a few hundred tokens instead of thousands.

I think RoBoT should have four memory levels
Level 0
──────────────
Live Context
(Current prompt)
↓
Level 1
──────────────
Working Summary
(~200 tokens)
↓
Level 2
──────────────
Conversation Checkpoints
(~300-500 tokens each)
↓
Level 3
──────────────
Raw Memory Database
(Unlimited)
Only Level 0 and Level 1 should be in every prompt.

Levels 2 and 3 are retrieved on demand.

I'd also add "memory aging"
Every interaction isn't equally important.

For example:

"Fixed compiler error"

importance = 0.25
"Architecture decision:
RoBoT uses SQLite instead of Postgres"

importance = 0.97
"Implemented Experience Engine"

importance = 0.99
Over time, low-importance memories can merge into broader summaries, while high-importance architectural decisions remain as standalone "anchor memories" that are never collapsed. Those become the project's institutional knowledge.

One more idea
Since RoBoT is already built around experiences, I wouldn't summarize messages. I'd summarize completed work units.

For example:

Experience #184

Goal:
Implement ExperienceCoordinator

Outcome:
Completed

Files:
experience/mod.rs
experience/coordinator.rs

Architectural Decisions:
• Coordinator owns worker pool
• Events routed asynchronously
• SQLite persists experience log

Lessons Learned:
Avoid circular module dependencies.

Confidence:
0.96
Now RoBoT remembers what it accomplished, not just what was said. That's a much denser and more useful form of memory for an engineering agent. It also makes retrieval more precise because you're searching completed experiences and architectural decisions instead of long conversational transcripts.


so i have this as blueprint or is it not right or not enough - 1. Context System
ContextManager
├── SessionContext
├── WorkingMemory
├── RetrievalCache
├── ContextCompressor
└── TopicTracker

Question
   ↓
Context Manager
   ↓
Relevant Context
   ↓
LLM

Context Management should bea first-class subsystem alongside Memory and Experience. 
Each turn starts mostly fresh
Memory search returns references
Only top relevant compressed snippets are loaded
The model answers
The interaction is saved
Working context is discarded
Only ActiveTaskContext persists during a coding session

It would own:
WorkingContext
Current prompt only
ActiveTaskContext
Current coding task
Current files
Current decisions
Current constraints
MemoryRetriever
Returns IDs first
Loads compressed summaries
Enforces token budget
MemoryCompressor
Creates 50 to 150 token summaries
Creates retrieval snippets
PromptAssembler
Builds final prompt under a hard budget of 2048 tokens
Drops least important context first
Sliding Window Pruning - Sliding window pruning acts like a First-In, First-Out (FIFO) queue for your chat logs
Continuous Compaction - a of collapses the oldest block of messages into a single, high-density rolling summary 
paragraph. saving it to memory. This summary is then injected back into the prompt buffer as a "Memory Context." for next question

Conversation
│
├── Messages 1-20
│      ↓
│  Summary #1 (frozen)
│
├── Messages 21-40
│      ↓
│  Summary #2 (frozen)
│
├── Messages 41-60
│      ↓
│  Summary #3 (frozen)
│
└── Active Messages (last 10-20)

Then create one small working summary that references those checkpoints.
Question
↓
Search summaries
↓
Find Summary #12
↓
Expand only that summary
↓
Maybe load 2 original conversations
↓
Answer

four memory levels
Level 0
──────────────
Live Context
(Current prompt)
↓
Level 1
──────────────
Working Summary
(~200 tokens)
↓
Level 2
──────────────
Conversation Checkpoints
(~300-500 tokens each)
↓
Level 3
──────────────
Raw Memory Database
(Unlimited)
Only Level 0 and Level 1 should be in every prompt.
Levels 2 and 3 are retrieved on demand.

add "memory aging"

I actually think this is one of the strongest architectural ideas you've had for RoBoT. It also fits the direction the project has been moving: Experience, Memory, and now Context become three independent systems instead of one giant memory system.

The one thing I'd change is the responsibilities. Right now you've mixed algorithms, data structures, and policies together. I'd separate them.

For example:

RoBoT Brain
│
├── Experience Engine
├── Memory Engine
└── Context Engine
The Context Engine should have a very narrow responsibility:

Build the smallest possible prompt that still allows the model to produce the correct answer.

Everything else exists to support that goal.

I'd reorganize it like this
Context Engine
│
├── ContextManager
├── WorkingContext
├── ActiveTaskContext
├── RetrievalPlanner
├── MemoryRetriever
├── ContextCompressor
├── PromptAssembler
├── TokenBudget
├── SlidingWindow
├── TopicTracker
└── RetrievalCache
Notice something?

WorkingMemory disappears.

Why?

Because WorkingMemory is really just WorkingContext.

Memory belongs to the Memory Engine.

Context belongs to the Context Engine.

That separation will keep the architecture cleaner.

I'd also rename a few pieces
Instead of

MemoryRetriever
I'd use

RetrievalPlanner
because retrieval is actually two different jobs.

Question
↓
RetrievalPlanner
↓
Which memories are worth loading?
↓
MemoryRetriever
↓
Fetch them
↓
ContextCompressor
↓
PromptAssembler
The planner decides what to retrieve.

The retriever retrieves it.

TokenBudget deserves to be its own component
This is actually more important than PromptAssembler.

TokenBudget
2048 max
System prompt
 220
User prompt
 180
Code
 850
Memory
 300
Tools
 250
Reserve
 248
Now every component knows exactly how much space it owns.

No guessing.

I would also replace "Continuous Compaction"
The name is good, but I'd make it broader.

Context Lifecycle
Inside that chapter:

Sliding Window
↓
Compaction
↓
Checkpoint Creation
↓
Memory Aging
↓
Archive
Now you've described the entire life cycle.

One thing I think is missing
This is the part I think RoBoT will eventually need.

Context Policies
Not every question should retrieve memory.

Example:

2 + 2
Should retrieve

nothing
Rename worker.rs
Should retrieve

Current task only
Continue the Experience Engine we designed yesterday
Should retrieve

Project summaries
Architecture decisions
Current task
Related source files
Different question.

Different retrieval policy.

That alone can cut token usage by half.

Another thing I would add
Since you're already tracking experiences, I'd add a Context Score to every retrieved item.

Summary #18
Similarity      0.92
Recency         0.64
Importance      0.95
Confidence      0.99
Final Score     0.91

Then PromptAssembler simply says:

Load until

TokenBudget exhausted

OR

Score < 0.60
Now context assembly becomes deterministic instead of "grab the top N."

Finally, I'd make one philosophical change
Right now your blueprint says:

Search memory → Answer → Save interaction

I would make it:

Question
↓
Task Detection
↓
Context Planning
↓
Memory Retrieval
↓
Prompt Assembly
↓
LLM
↓
Experience Extraction
↓
Memory Update
↓
Checkpoint Evaluation
That tiny change matters because not every interaction deserves to become memory.

Some responses should become:

Experiences

Skills

Architectural decisions

User preferences

Conversation history

Or nothing at all

Treating every interaction as equal is what eventually causes memory systems to bloat.

Overall, I think you're no longer designing "memory retrieval." You're designing a Context Engine, and I would absolutely make it its own major chapter in ARCHITECTURE.md, right alongside the Memory Engine and Experience Engine. In fact, I could see it becoming one of the defining features of RoBoT because it shifts the focus from "How much context can we fit?" to "How little context do we actually need?" That's a much more scalable philosophy for a long-running Rust agent.


|==========|==========|==========|==========|==========|==========||==========|==========|==========|==========|==========|==========|
RoBoT Cognitive Architecture
                 User Question
                       │
                       ▼
              Task Classification
                       │
                       ▼
                Context Engine
                       │
      ┌────────────────┼─────────────────┐
      │                │                 │
      ▼                ▼                 ▼
Working Context   Active Task      Retrieval Planner
                                       │
                                       ▼
                               Memory Retriever
                                       │
                  ┌────────────────────┼──────────────────┐
                  ▼                    ▼                  ▼
           Strategic Memory     Episodic Memory     Knowledge Graph
                  │                    │                  │
                  └──────────────┬─────┴──────────────────┘
                                 ▼
                         Context Compressor
                                 │
                           Token Budget
                                 │
                          Prompt Assembler
                                 │
                                 ▼
                                LLM
                                 │
                                 ▼
                           Action / Answer
                                 │
                                 ▼
                      Experience Extraction
                                 │
                                 ▼
                         Experience Engine
                                 │
                                 ▼
                         Memory Engine
                                 │
                                 ▼
                       Strategic Learning
The Four Independent Systems

The architecture becomes much easier to reason about if every subsystem has exactly one responsibility.

1. Context Engine

Purpose:

Build the smallest possible prompt that still allows the model to solve the current task.

The Context Engine never stores permanent information.

It only decides:

what is relevant
what should be loaded
what should be discarded
ContextEngine
│
├── ContextManager
├── WorkingContext
├── ActiveTaskContext
├── RetrievalPlanner
├── MemoryRetriever
├── ContextCompressor
├── PromptAssembler
├── TokenBudget
├── SlidingWindow
├── TopicTracker
└── RetrievalCache
Working Context

Temporary.

Contains:

current user prompt
current tool outputs
current reasoning state

Destroyed after every interaction.

Active Task Context

Lives longer.

Examples:

Current coding project
Current Rust file
Current bug
Current design discussion
Current constraints

This survives while the task remains active.

Retrieval Planner

The brain of Context.

Instead of asking Memory for everything, it asks:

What do I actually need?

Example:

Question:

Implement SQLite transactions

Planner:

Need:

Rust knowledge
Database architecture
Current repository decisions

Ignore:
Weather
Recipes
Old conversations

Memory Retriever

Planner decides.

Retriever fetches.

Never the opposite.

Returns:

IDs
compressed summaries
optional expansion
Context Compressor

Converts memory into prompt-sized knowledge.

Example:

Raw memory

4000 tokens
↓
Summary

120 tokens
Prompt Assembler

Final prompt construction.

System Prompt
+
Current Question
+
Current Code
+
Retrieved Memory
+
Tools
↓
LLM

Nothing else touches the prompt.

Token Budget

A hard budget.

Example

2048 Tokens
220 System
180 User
850 Code
300 Memory
250 Tool Results
248 Reserve

If overflow happens

Drop lowest priority.

Never exceed budget.

2. Memory Engine

Purpose:

Store information.

Nothing else.

MemoryEngine
│
├── Episodic Memory
├── Semantic Memory
├── Graph Memory
├── User Memory
├── Retrieval Index
├── Embeddings
├── Aging
├── Compression
└── Archive

Memory should never build prompts.

That belongs to Context.

3. Experience Engine

Purpose:

Learn from execution.

ExperienceEngine
│
├── Event Capture
├── Success Detection
├── Failure Detection
├── Reflection
├── Skill Extraction
├── Confidence Updates
├── Policy Generation
└── Experience Database

Every interaction produces an experience.

Not every experience becomes memory.

4. Learning Engine

Purpose:

Convert experience into reusable intelligence.

LearningEngine
│
├── Pattern Detection
├── Rule Extraction
├── Skill Builder
├── Conflict Resolver
├── Confidence Manager
├── Policy Promotion
├── Memory Consolidation
└── Knowledge Evolution

This is where intelligence grows.

Four Memory Levels

Instead of one giant chat history.

Level 0
──────────────────────
Live Context

Current prompt

Current response

Destroyed every turn

──────────────────────

Level 1
Working Summary

~200 tokens

Current task

Temporary

──────────────────────

Level 2
Conversation Checkpoints

300-500 token summaries

Frozen

Searchable

──────────────────────

Level 3
Permanent Memory

Unlimited

Raw conversations

Experiences

Documents

Knowledge Graph

Embeddings

Policies

Skills

Only Level 0 and Level 1 are loaded by default.

Everything else is retrieved.

Context Lifecycle
Conversation

↓

Sliding Window

↓

Continuous Compaction

↓

Checkpoint Creation

↓

Memory Aging

↓

Archive

This keeps the prompt small forever.

Continuous Compaction

Instead of one huge summary.

Messages 1-20
      │
      ▼
Checkpoint #1

Messages 21-40
      │
      ▼
Checkpoint #2

Messages 41-60
      │
      ▼
Checkpoint #3

Messages 61-80
      │
      ▼
Checkpoint #4

Current Messages

Searching becomes:

Question

↓

Search checkpoints

↓

Checkpoint #12 matches

↓

Expand only that checkpoint

↓

Maybe load two raw conversations

↓

Answer

No need to reload months of history.

Memory Aging

Every memory slowly changes importance.

New Memory

↓

Frequently Used

↑ confidence

↓

Rarely Used

↓

Compress

↓

Archive

↓

Delete (optional)

Importance can be calculated from:

access frequency
success rate
recency
confidence
relationship strength

Old memories never disappear automatically.

They simply become harder to retrieve unless reinforced.

Strategic Learning

The biggest improvement over traditional RAG.

Instead of remembering experiences forever:

Experience

↓

Pattern Detection

↓

Reflection

↓

Skill Extraction

↓

Policy Generation

↓

Strategic Memory

Example

Experience Log

Battery 18%

Docked

Succeeded

Battery 17%

Docked

Succeeded

Battery 15%

Docked

Succeeded

↓

Policy

IF Battery < 20%

THEN Dock Immediately

Confidence 97%

Next time

No vector search.

The rule already exists.

End-to-End Workflow
Question
    │
    ▼
Task Detection
    │
    ▼
Context Planning
    │
    ▼
Memory Retrieval
    │
    ▼
Context Compression
    │
    ▼
Prompt Assembly
    │
    ▼
LLM
    │
    ▼
Action / Response
    │
    ▼
Experience Extraction
    │
    ▼
Memory Update
    │
    ▼
Checkpoint Evaluation
    │
    ▼
Pattern Detection
    │
    ▼
Policy / Skill Promotion
Core Design Principles
Context is ephemeral. It exists only to solve the current task.
Memory is persistent. It stores knowledge but never builds prompts.
Experience is observational. Every interaction becomes structured experience.
Learning is transformative. Repeated experiences become reusable skills, policies, and causal models.
Retrieval is intentional. The planner decides what to load before any search occurs.
Compression happens continuously. Conversations evolve into checkpoints, checkpoints into knowledge, and knowledge into abstractions.
Token budgets are enforced by design. The system never relies on oversized prompts.
The architecture improves with use. The agent becomes more capable by promoting successful patterns into strategic memory rather than accumulating raw history.

engineering specification

RoBoT Cognitive Architecture Blueprint
Long-Term Autonomous AI Agent Design
Purpose

This document defines the core cognitive architecture for RoBoT.

The objective is to build an AI agent capable of operating indefinitely without suffering from context explosion, memory bloat, or repetitive reasoning.

The architecture is built around one core principle:

Context is temporary. Knowledge is permanent. Experience creates learning.

Every subsystem has one responsibility and communicates through well-defined interfaces.

Core Architecture
                    User
                     │
                     ▼
          Conversation Engine
                     │
                     ▼
              Context Engine
                     │
                     ▼
               Memory Engine
                     │
                     ▼
            Experience Engine
                     │
                     ▼
             Learning Engine
                     │
                     ▼
             Strategic Memory
Design Principles
Principle 1

Conversation is not Memory.

Conversation stores everything.

Memory stores only what is worth remembering.

Principle 2

Context is disposable.

Every prompt begins nearly empty.

Only relevant information is loaded.

Principle 3

Experience is observation.

Every execution creates an experience.

Not every experience becomes knowledge.

Principle 4

Learning is continuous.

Repeated successful experiences become reusable skills and policies.

Principle 5

Knowledge becomes more abstract over time.

Conversation
↓
Experience
↓
Pattern
↓
Skill
↓
Policy
↓

Strategic Knowledge
System Architecture
RoBoT
│
├── Conversation Engine
├── Context Engine
├── Memory Engine
├── Experience Engine
├── Learning Engine
├── Planning Engine
├── Execution Engine
└── Tool Engine
1. Conversation Engine
Responsibility

Capture everything.

Nothing is lost.

Nothing is filtered.

This is an append-only event stream.

Stores
Conversation Database

Messages

Sessions

Attachments

Tool Calls

System Events

Errors

Streaming Tokens

Metadata
Reads

Mostly sequential.

Last messages

Current session

Conversation replay
Writes

Every interaction.

Never Does

Memory retrieval

Embeddings

Policy extraction

Reasoning

Learning

2. Context Engine
Responsibility

Construct the smallest possible prompt.

Nothing more.

Context Engine
│
├── ContextManager
├── WorkingContext
├── ActiveTaskContext
├── RetrievalPlanner
├── MemoryRetriever
├── ContextCompressor
├── PromptAssembler
├── TokenBudget
├── TopicTracker
├── RetrievalCache
└── SlidingWindow
Working Context

Temporary.

Destroyed every turn.

Contains

Current prompt

Recent replies

Tool outputs

Temporary reasoning
Active Task Context

Persists during ongoing work.

Examples

Current coding project

Current file

Current objective

Current decisions

Constraints

Open bugs

Destroyed only when the task ends.

Retrieval Planner

Determines what information is needed before any search occurs.

Example

User

Continue SQLite work

↓

Need

Current project

Architecture decisions

Database module

↓

Ignore

Recipes

Weather

Old conversations
Memory Retriever

Receives retrieval requests.

Returns

Memory IDs

Summaries

Optional expansions
Context Compressor

Converts retrieved content into compact prompt fragments.

Example

3500 tokens

↓

120-token summary
Prompt Assembler

Combines

System Prompt

User Prompt

Retrieved Context

Code

Tool Results

Produces one final prompt.

Token Budget

Hard budget.

Example

2048 Tokens

220 System

180 User

850 Code

300 Memory

250 Tools

248 Reserve

If overflow occurs

Drop lowest priority context.

Never exceed the budget.

3. Memory Engine
Responsibility

Store knowledge.

Nothing else.

Memory Engine
│
├── Episodic Memory
├── Semantic Memory
├── User Memory
├── Knowledge Graph
├── Embeddings
├── Retrieval Index
├── Aging
├── Compression
└── Archive
Memory Types
Episodic

Individual events.

Conversation

Task completion

Failures

Observations
Semantic

Facts.

SQLite supports transactions.

Rust ownership rules.

API endpoints.
User Memory

Long-term user preferences.

Examples

Preferred coding style

Project conventions

Tool preferences
Strategic Memory

Policies.

Skills.

Rules.

Causal models.

Never Stores

Raw conversations.

Streaming messages.

Temporary context.

4. Experience Engine
Responsibility

Convert execution into structured experiences.

Experience Engine
│
├── Event Capture
├── Reflection
├── Outcome Analysis
├── Success Detection
├── Failure Detection
├── Confidence Updates
├── Skill Candidates
└── Experience Database

Example

Goal

Compile Rust
↓
Compilation failed
↓
Fixed lifetime
↓
Compiled successfully
↓
Experience saved
5. Learning Engine
Responsibility

Transform experience into reusable intelligence.

Learning Engine
│
├── Pattern Detection
├── Reflection
├── Rule Extraction
├── Skill Builder
├── Policy Generator
├── Conflict Resolver
├── Confidence Manager
└── Strategic Promotion

Example

50 successful experiences
↓
Repeated sequence detected
↓
Extract reusable policy
↓
Store in Strategic Memory
Strategic Memory

Stores

Skills

Policies

Rules

Decision trees

Failure modes

Causal relationships

Examples

If battery <20%

Dock immediately
Use transactions for multi-table updates.
Acquire locks before writing shared memory.
Memory Hierarchy
Level 0

Live Context

Current prompt

Destroyed every turn

──────────────────────────

Level 1

Working Summary

Current task

~200 tokens

──────────────────────────

Level 2

Conversation Checkpoints

300-500 tokens

──────────────────────────

Level 3

Long-Term Memory

Unlimited

──────────────────────────

Level 4

Strategic Memory

Skills

Policies

Rules

Only Levels 0 and 1 are always loaded.

Everything else is retrieved on demand.

Context Lifecycle
Conversation

↓

Sliding Window

↓

Compaction

↓

Checkpoint Creation

↓

Memory Aging

↓

Archive
Continuous Compaction
Messages 1-20

↓

Checkpoint #1

Messages 21-40

↓

Checkpoint #2

Messages 41-60

↓

Checkpoint #3

Current Messages

Searching becomes

Search checkpoints

↓

Load matching checkpoint

↓

Expand only relevant conversations

↓

Answer
Memory Aging

Every memory has

Confidence

Importance

Access Count

Last Used

Creation Date

Relationship Strength

Older memories gradually lose priority.

Important memories become stronger through repeated successful use.

Data Flow
User

↓

Conversation Engine

↓

Conversation Database

↓

Experience Extraction

↓

Experience Engine

↓

Experience Database

↓

Learning Engine

↓

Strategic Memory

↓

Memory Engine

The Context Engine can query Memory, but Memory never pushes information into Context.

Query Flow
Question
↓
Task Detection
↓
Context Planning
↓
Need Memory?

├── No
│      ↓
│     LLM
│
└── Yes
       ↓
Retrieval Planner

↓

Memory Retrieval

↓

Compression

↓

Prompt Assembly

↓

LLM

↓

Response

↓

Experience Extraction

↓

Checkpoint Evaluation

↓

Memory Update

↓

Learning
Suggested Implementation Roadmap
Phase 1: Foundation
Conversation Engine with append-only storage.
Context Engine skeleton with token budgeting and prompt assembly.
Basic Memory Engine with episodic and semantic stores.
Simple retrieval pipeline (planner → retriever → assembler).
Phase 2: Retrieval and Context
Retrieval Planner.
Context Compressor.
Sliding window and checkpoint creation.
Working and Active Task contexts.
Memory aging and archival.
Phase 3: Experience
Event capture.
Structured experience records.
Success/failure detection.
Reflection pipeline.
Confidence tracking.
Phase 4: Learning
Pattern detection across experiences.
Rule and skill extraction.
Policy generation.
Conflict resolution.
Promotion into Strategic Memory.
Phase 5: Advanced Reasoning
Knowledge Graph integration.
Causal reasoning.
Adaptive retrieval planning.
Multi-step planning using strategic skills.
Autonomous maintenance tasks (compaction, aging, checkpointing, learning).
Architectural Rules for AI Contributors
Every subsystem has exactly one responsibility.
Never mix conversation storage with long-term memory.
Context is rebuilt each turn and discarded when complete.
Memory stores only durable knowledge, never raw chat logs.
Experience records execution outcomes without making decisions.
Learning alone promotes repeated experiences into strategic knowledge.
Retrieval is always initiated by the Context Engine through the Retrieval Planner.
Enforce token budgets as a hard architectural constraint.
Prefer summarization and abstraction over retaining verbose history.
Optimize for continuous operation, incremental learning, and indefinite scalability.

-----------------
architecture.md update

Purpose
Responsibilities
What it owns
What it must never do
Public interfaces
Data structures
Data flow
Sequence diagrams
Rust module layout
Implementation order

So instead of saying:

Memory Engine stores memories.

It would say something like:

Memory Engine

Purpose
-------
Persist durable knowledge independently of the active conversation.

Responsibilities
----------------
• Store semantic memory
• Store episodic memory
• Store strategic memory
• Maintain embeddings
• Maintain graph relationships
• Maintain confidence scores

Must Never
----------
• Build prompts
• Read conversations directly
• Decide retrieval
• Perform planning
• Execute tools

Interfaces
----------
store_memory()
retrieve_memory()
update_confidence()
archive_memory()
promote_to_strategic()
merge_duplicate()
age_memory()

Every subsystem would have that level of detail.
Then every subsystem would have diagrams.

Conversation
↓
Conversation Engine
↓
Conversation Database
↓
Experience Extractor
↓
Experience Database
↓
Learning Engine
↓
Memory Engine
↓
Context Engine
↓
LLM
Then we'd define every database table.
conversation_messages
conversation_sessions
experiences
experience_events
memory_cards
knowledge_graph
embeddings
strategic_skills
policies
confidence_history
retrieval_cache
task_context

Then every Rust module.

src/

conversation/

context/

memory/

experience/

learning/

planning/

execution/

tools/

graph/

database/

api/

Then every workflow.

User Question
↓
Conversation Engine
↓
Task Detection
↓
Context Planning
↓
Memory Retrieval
↓
Compression
↓
Prompt Assembly
↓
LLM
↓
Experience Extraction
↓
Memory Update
↓
Checkpoint Evaluation
↓
Strategic Learning

And finally an Operating Agreement for AI contributors that says things like:

Never bypass the Context Engine.
Never write directly into Strategic Memory.
All memory promotion must pass through the Learning Engine.
The Conversation Engine is append-only.
Context is rebuilt every turn.
Retrieval is always initiated by the Retrieval Planner.
Every subsystem has a single responsibility.
Favor composition over coupling.
Prefer asynchronous pipelines for expensive background work.
Keep LLM context minimal and deterministic.


--------------------------------------------
3. Confidence Graph

Don't score only nodes.

Score relationships.

The relationship confidence becomes its own entity.

That allows planner reasoning like

"I know Rust."

"I know SQLite."

"But I have little experience combining them."

4. Event Sourcing

Instead of modifying structures directly...

everything becomes an event.

MemoryCreated

MemoryUpdated

ExperienceRecorded

ExperienceMerged

HypothesisCreated

KnowledgeValidated

SkillImproved

Current state becomes

fold(events)

Advantages:

complete history
debugging
replay
rollback
explainability

It also fits the architecture you've been building around the Experience Engine.

5. Capability System

Instead of tools...

think capabilities.

Observe

Recall

Compare

Predict

Infer

Plan

Execute

Reflect

Teach

Planner requests capabilities.

Capabilities use tools.

Much cleaner dependency direction.

6. Skill Evolution

Instead of

Skill

store

Skill
├── prerequisites
├── confidence
├── decay
├── reinforcement
├── evidence
└── last successful use

Now skills become alive instead of static.

7. Experience Compression

This is one of my favorite additions.

Instead of keeping

100 nearly identical experiences

compress them into

Pattern

Confidence

Exceptions

Exactly what humans do.

8. Hypothesis Engine ✅ **IMPLEMENTED**

The Hypothesis Engine makes RoBoT capable of learning rather than merely remembering.

```
Observation → Hypothesis → Test (Evidence) → Evaluation → Knowledge
                    ↓
              Supported | Refuted | Inconclusive | Superseded
```

**Learning Flow:**
1. **Observation** - Record successes, failures, patterns, anomalies
2. **Hypothesis** - Form testable statements from observations
3. **Test** - Add supporting or contradicting evidence
4. **Evidence** - Accumulate proof for or against hypothesis
5. **Evaluation** - Calculate status based on evidence ratio
6. **Knowledge** - Extract validated hypotheses into reusable knowledge

**MCP Tools (9):**
| Tool | Description |
|------|-------------|
| `record_observation` | Record successes, failures, patterns, anomalies |
| `list_observations` | View recorded observations |
| `create_hypothesis` | Form testable hypothesis from observations |
| `get_hypothesis` | View hypothesis with all evidence |
| `list_hypotheses` | List hypotheses (filter by domain/status) |
| `add_evidence` | Add supporting or contradicting evidence |
| `evaluate_hypothesis` | Evaluate based on evidence, update status |
| `get_knowledge` | Get extracted learned knowledge |
| `extract_knowledge` | Convert supported hypothesis → reusable knowledge |

**Database Tables (Migration 008):**
- `hypotheses` - Testable hypotheses with status and confidence
- `observations` - Raw observations that trigger learning
- `evidence` - Supporting/contradicting evidence for hypotheses
- `learned_knowledge` - Extracted knowledge from validated hypotheses

**Status Evaluation Rules:**
- 3+ evidence required to evaluate
- Supported: supporting > contradicting × 2
- Refuted: contradicting > supporting × 2
- Inconclusive: otherwise
- Knowledge extraction only from Supported hypotheses

9. Planner Feedback Loop

Instead of

Plan

Execute

Done

make it

Goal
↓
Planner
↓
Action
↓
Outcome
↓
Experience
↓
Knowledge
↓
Improved Planner

Now every task makes the planner smarter.

10. Reflection Engine

Probably the biggest architectural upgrade.

Every N experiences
Reflect
↓
Find patterns
↓
Merge memories
↓
Retire obsolete facts
↓
Create new hypotheses
↓
Adjust confidence

This is remarkably similar to sleep consolidation in biological memory.

One thing I'd change from our earlier discussions

Originally we leaned toward:

Memory

Experience

Learning

After thinking through your architecture more, I'd separate them further:

Observation Layer
↓
Working Memory
↓
Experience Engine
↓
Reflection Engine
↓
Knowledge Graph
↓
Planning
↓
Execution

That keeps every subsystem responsible for exactly one transformation. It also makes testing easier because each layer has a single job.

What I think is the single biggest missing piece

If I could add one subsystem to RoBoT_mcp, it would be the Reflection Engine.

Most AI memory systems stop at:

"Store memory. Retrieve memory."

Your architecture is already aiming higher. A Reflection Engine turns accumulated experiences into refined
knowledge, updates confidence, discovers patterns, and retires stale information. That closes the learning 
loop and makes the system improve over time rather than simply grow larger.
---
speech engines upgrade
candle for TTS and STT

 Candle with Qwen3-TTS (via Candle)

[ User Microphone / Audio WAV File ]
              │
              ▼
    ( Candle-Whisper Large-V3 )   <── Runs locally on GPU via CUDA/Metal
              │
              ▼
      [ Raw Text String ]
              │
              ▼
   ( Your LLM / Application Logic )
              │
              ▼
      [ Response Text ]
              │
              ▼
    ( Candle Qwen3-TTS )          <── Ingests Response Text + Your Speaker WAV Sample
              │
              ▼
  [ Synthesized Output Audio ]

use qwen_tts::model::loader::{ModelLoader, LoaderConfig};
use candle_core::Device;
use std::path::Path;
use cargo build --release --features cuda

[  MCP Client ]
          │
          ▼ (MCP JSON-RPC Protocol)
┌─────────────────────────────────┐
│       Local Rust MCP Agent      │  <── Only manages text, files, and state
└─────────────────────────────────┘
          │
          ▼ (Fast Binary Frame or HTTP Protocol)
┌─────────────────────────────────┐
│ qwen3-tts-candle (Inference)    │  <── Does the actual heavy math on the GPU
└─────────────────────────────────┘

toml[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"

# Pull in the Candle Qwen3 port (Enable "cuda" or "metal" depending on your GPU)
qwen3-tts = { version = "0.1", features = ["hub", "cuda"] } 

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use qwen3_tts::{Qwen3TTS, Language, auto_device}; // Utilizing native candle-backed crate
 
  ---

tools\
     ├──interaction\
     |  ├──audio <-- pass wav file directly into your execution pipeline for tts
     |  ├──chat --> output tts and print text to desktop ui
     |  ├──clipboard
     |  ├──documents
     |  ├──dragdrop
     |  ├──notifications
     |  ├──shortcuts
        
┌─────────────────────────────┐
│      Desktop UI (Rust)      │
│                             │
│ 🎤 Start Listening          │
│ 📄 Drop Files Here          │
│ 💬 Conversation             │
│ 🧠 Agent Thoughts           │
└──────────────┬──────────────┘
               │
               ▼
        RoBoT MCP Core
               │
               RoBoT Desktop (Rust)
                       │
                       ▼
               Interaction Layer
                       │
                ┌──────┴─────────┐
                ▼                ▼
               whisper-rs      F5-TTS
               (STT)            (TTS)
                       │
                       ▼
               Experience Engine
                       │
                       ▼
               Planner
                       │
                       ▼
               Memory System
               
When idle, it collapse's into a tiny floating microphone button. speak, and watch the transcript appear.
separate what the user says 'text in blue' from the agent's internal reasoning 'text in white' and what agent says 'text in lime green'.

Drop anything onto the window:
PDF
TXT
Markdown
DOCX
Images
Audio
Video
ZIP
Rust source
Entire folders
sent to ingestor which adds it to short term memory for agent usage. simply hands them to the ingestion pipeline, which routes each file to the appropriate processor.

an Interaction Layer as a peer to your Experience and Memory systems:
Interaction
├── Voice
├── Chat
├── Documents
├── Clipboard
├── Screen (future)
├── Notifications
└── Commands

--------------------------------------------------------------------------------

## Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| MCP Protocol | `rmcp` v2 | Model Context Protocol server (stdio transport) |
| Runtime | `tokio` v1 | Async runtime (`#[tokio::main]`) |
| Database | `rusqlite` v0.32 | Embedded SQLite with `load_extension` feature |
| Serialization | `serde` + `serde_json` | Data serialization (Experience, EventPayload, etc.) |
| Identity | `uuid` v1 | Unique IDs (v4) for memories and experiences |
| Time | `chrono` v0.4 | Timestamps (RFC3339) |
| File walking | `walkdir` v2 | Directory traversal for file ingestion |
| Compression | `zip` v2, `tar` v0.4, `flate2` v1 | Archive handling (zip, tar, gz) |
| Hashing | `sha2` v0.10 | File content hashing |
| Paths | `dirs` v5 | OS data directory resolution |
| Error handling | `anyhow` v1 | Result propagation throughout |

---

## Getting Started

### Prerequisites

- Rust 2024 edition (per `Cargo.toml`)
- SQLite3 development libraries (for `rusqlite`)

started ### CLI Usage

```bash
# Start the MCP server (default)
cargo run

# Run CLI commands
cargo run -- init           # Initialize database
cargo run -- status         # Check system status
cargo run -- memory list    # List memories
cargo run -- memory search <query>  # Search memories
cargo run -- memory add <content>   # Add a memory
cargo run -- memory stats    # Show memory statistics
cargo run -- experience      # Show experience statistics
cargo run -- config          # Show configuration
cargo run -- migrate         # Run database migrations
```

---

## File Ingestion (Ingestor Tools)

The ingestor tools allow you to import files from a `files_to_import/` folder into short-term memory. Files are automatically chunked and stored as memory cards.

### Supported File Formats

| Format | Extensions | Processing |
|--------|------------|------------|
| Archives | `.zip`, `.tar`, `.tar.gz`, `.tgz`, `.gz` | Extracted recursively |
| Text | `.txt`, `.md`, `.rst`, `.csv`, `.log`, `.xml`, `.html` | Direct ingestion |
| JSON | `.json`, `.jsonl` | Pretty-printed for search |
| PDF | `.pdf` | Basic text extraction |
| Audio | `.mp3`, `.wav`, `.m4a`, `.flac`, `.ogg`, `.aac` | Placeholder for transcription |

### MCP Tools

#### `ingest_files`
Import files from `files_to_import/` folder into short-term memory.

```json
{
  "folder": "files_to_import",
  "chunk_size": 1000,
  "memory_type": "file"
}
```

**Response includes:**
- `summary`: Ingestion statistics (total, successful, failed, chunks)
- `successfully_ingested`: Array of file paths that were imported
- `user_action_required`: Prompt to confirm deletion

#### `list_importable`
List files ready for import in the folder.

```json
{
  "folder": "files_to_import"
}
```

#### `list_ingested_files`
List files that have been successfully ingested and can be deleted.

```json
{
  "folder": "files_to_import",
  "limit": 100
}
```

#### `delete_ingested_files`
**Requires confirmation** - Delete files after successful ingestion.

```json
{
  "files": ["path/to/file1.txt", "path/to/file2.pdf"],
  "confirmation": "yes"
}
```

**Safety:** Without `confirmation: "yes"`, the tool runs in simulation mode showing what would be deleted.

### Workflow

```
1. Place files in ./files_to_import/

2. Call ingest_files → Files are chunked and stored in memory
   └─ Response: List of successfully imported file paths

3. Review the imported files

4. Call delete_ingested_files with confirmation to remove originals
   └─ confirmation: "yes" → Actually deletes
   └─ confirmation: anything else → Shows simulation only
```

### Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `chunk_size` | 1000 | Characters per chunk |
| `chunk_overlap` | 100 | Overlap between chunks |
| `memory_type` | "file" | Type for ingested memories |

### Build

```bash
# Development build
cargo build

# Release build (recommended for production)
cargo build --release
```

> **Note:** The project uses the system SQLite3 library. The database (`robot_brain.db`) is created automatically on first run.

### Testing

This project includes a comprehensive test suite with 54 unit tests and a full integration test binary.

```bash
# Run unit tests
cargo test

# Build and run the comprehensive integration test suite
cd robot_brain_test && cargo build && ./target/debug/robot_brain_test
```

**Test Results:**
- **Unit Tests:** 54 tests passing (0 failed, 0 skipped)
- **Integration Tests:** 103 tests passing (0 failed, 2 skipped)
- **Build:** 0 errors, 0 warnings
- **Both binaries compile and run successfully**

The test suite covers:
- Memory tools (14 tests)
- Experience tools (11 tests)
- Knowledge tools (9 tests)
- Workflow tools (14 tests)
- Planner tools (13 tests)
- Hypothesis tools (14 tests)
- Reflection tools (6 tests)
- Search tools (6 tests)
- Ingestor tools (6 tests)
- Agent tools (6 tests)
- Error handling tests (3 tests)

---

## Workflow Engine Tools

The workflow engine provides structured, executable workflows with step-by-step orchestration, variable substitution, and pause/resume capabilities. Unlike the static `get_workflow` tool (which returns guidance JSON), these tools create and run actual workflows.

### MCP Tools

#### `create_workflow`
Create a new workflow with a name and optional description.

```json
{
  "name": "My Workflow",
  "description": "A custom workflow for X task"
}
```

#### `add_workflow_step`
Add a step to an existing workflow. Steps execute in order.

```json
{
  "workflow_id": "<workflow-uuid>",
  "name": "Step 1",
  "action": "store_memory",
  "parameters": "{\"content\": \"some data\", \"memory_type\": \"note\"}"
}
```

**Supported actions:** `store_memory`, `search_memory`, `record_experience`, `create_reflection`, `ingest_files`

#### `get_workflow_status`
Get the current status and details of a workflow.

```json
{
  "workflow_id": "<workflow-uuid>"
}
```

#### `list_workflows`
List all workflows, optionally filtered by status.

```json
{
  "status": "running"
}
```

**Status values:** `draft`, `ready`, `running`, `paused`, `completed`, `failed`, `cancelled`

#### `start_workflow`
Start executing a workflow. Steps run sequentially with automatic memory reads before each action.

```json
{
  "workflow_id": "<workflow-uuid>"
}
```

#### `pause_workflow`
Pause a running workflow.

```json
{
  "workflow_id": "<workflow-uuid>"
}
```

#### `resume_workflow`
Resume a paused workflow.

```json
{
  "workflow_id": "<workflow-uuid>"
}
```

#### `cancel_workflow`
Cancel a workflow, removing it from execution.

```json
{
  "workflow_id": "<workflow-uuid>"
}
```

#### `delete_workflow`
Delete a workflow completely.

```json
{
  "workflow_id": "<workflow-uuid>"
}
```

### How It Works

```
1. create_workflow → Get workflow ID
2. add_workflow_step → Add steps (with actions like store_memory, search_memory, etc.)
3. start_workflow → Engine executes steps sequentially
   ├── Before each step: automatic memory context lookup
   ├── Execute: step action via internal tool dispatch
   ├── After: record experience for learning
   └── Variables: results can be stored and reused in subsequent steps
4. pause_workflow → Pause mid-execution
5. resume_workflow → Continue from where paused
6. get_workflow_status → Check current state
7. cancel_workflow / delete_workflow → Cleanup
```

### Key Features

| Feature | Description |
|---------|-------------|
| **Variable Substitution** | Step results can be stored as variables and referenced in later steps |
| **Automatic Memory Context** | Before each step, relevant memories are retrieved automatically |
| **Experience Recording** | After each step, the outcome is recorded as an experience for learning |
| **Pause/Resume** | Workflows can be paused and resumed mid-execution |
| **Action Dispatch** | Steps can invoke any internal tool (memory, experience, reflection, etc.) |

---

## Current Status & Gaps

| Area | Status | Details |
|------|--------|---------|
| Database layer | ✅ Functional | Schema + 8 migrations (v0→v8 via `migrations/` module), CRUD queries all implemented |
| Memory System | ✅ Complete | Working Memory, Permanent Memory, Memory Retrieval per Architecture §6.3 |
| Event System | ✅ Complete | Full event catalog per Architecture §4.04 (30+ event types) |
| Learning Pipeline | ✅ Implemented | Input→Observation→Memory→Experience→Knowledge→Planning→Decision→Action→Reflection |
| Experience types/events | ✅ Complete | Full type system for experiences, scores, reputation, event payloads |
| Observer pattern | ✅ Implemented | Trait defined with priority and filter hooks |
| Job queue + worker | ✅ Implemented | In-memory queue with async worker (mpsc channel) |
| Event bus | ✅ Implemented | Full pub/sub with broadcast channel, subscriber tracking |
| Experience coordinator | ✅ Implemented | Pipeline logic with all sub-modules wired up |
| Experience recorder | ✅ Implemented | Record/success/failure methods working with database |
| Experience repository | ✅ Implemented | Full CRUD for encounters and experiences |
| Reflection system | ✅ Complete | Core types, services (analyzer, generator, repository, validator), patterns |
| Hypothesis Engine | ✅ Implemented | Observation → Hypothesis → Test → Evidence → Knowledge pipeline with 9 MCP tools and full database support |
| Exploration system | ✅ Implemented | Exploration tracking with repository |
| Reputation system | ✅ Implemented | Full reputation tracking with decay and analytics |
| Evolution system | ✅ Implemented | Behavior creation from insights, tracking, promotion/deprecation |
| Metrics collection | ✅ Implemented | Counters, gauges, time series with aggregation |
| Scheduler | ✅ Implemented | Background task scheduling with SQLite persistence |
| MCP bridge | ✅ Implemented | RMCP, MCP, and ACP protocol implementations in `bridge/` folder |
| MCP tools | ✅ Implemented | Memory, experience, reflection, search, and ingestor tools defined |
| Planner module | ✅ Implemented | Planning engine and policy engine for task decomposition |
| Skills module | ✅ Implemented | Skill registry for managing available skills |
| Workflows module | ✅ Implemented | Workflow execution engine for multi-step tasks |
| Learning module | ✅ Implemented | Working memory, hypothesis tracking, candidate generation, lineage tracking |
| Experience Compression | ✅ Implemented | Pattern detection, exception tracking, and compression algorithms |
| CLI interface | ✅ Implemented | Command-line interface with server, memory, experience commands |
| App entry point | ✅ Implemented | App struct with coordinator and stdio server |
| Main entry point | ✅ Implemented | init_logging() and App::new().run() working |

---

## Immediate Next Steps

1. **Wire MCP tools to handlers** — Connect tool definitions to actual functionality
2. **Implement tool execution** — Make tools actually perform their operations
3. **Implement knowledge graph** — Broader knowledge representation system
4. **Add LLM integration** — Enable actual reflection generation

---

## Known Issues

- **Knowledge graph is placeholder** — Broader knowledge representation needed

## ⚖️ License & Fair-Pay Rule

This project is open-source, but it is also built on fairness. We believe that if the community helps improve this software, the community should share in its financial success.

### 1. For Open-Source Use (AGPL-3.0)
This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**. 
* You are completely free to use, modify, and share this code for personal or open-source projects.
* If you modify this code and run it as a cloud service or distribute it, **you must open-source your modifications** under the same AGPL-3.0 license.

### 2. For Commercial Use (Paid License)
Because many companies cannot or will not open-source their proprietary software, we offer a **Commercial License**. If a company wants to use this MCP server internally or in a closed-source product, they must purchase a commercial license from us.

### 3. The Fair-Pay Rule for Contributors
If you contribute code improvements to this project, you are an essential part of it. We do not believe in taking your work to enrich ourselves.
* **Revenue Sharing**: 100% of the net revenue generated from commercial licensing fees will be pooled and split among contributors.
* **How Payouts Work**: Payouts are distributed based on accepted code contributions (Pull Requests) and resolved GitHub issue bounties. 
* **Copyright**: By submitting a Pull Request, you maintain copyright over your code but grant us the right to include it in both the open-source AGPL-3.0 version and the paid commercial version, so we can legally sell it and pay you your share.
