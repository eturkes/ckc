# Prompting Claude Fable 5

Vendored from `platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-fable-5.md` (fetched 2026-06-11), trimmed per CLAUDE.md scope: guidance that enhances reasoning, long-horizon autonomy, and token-efficiency kept; user-facing communication-style and app-UX sections (readability addendum, send-to-user tool) dropped. Fenced blocks are paste-ready prompt snippets, verbatim from the source.

## Safety classifiers and refusal fallback

Fable 5 runs classifiers targeting offensive-cybersecurity techniques, biology/life-sciences content, and extraction of its summarized thinking; benign work near those domains can also trigger them (`stop_reason: "refusal"`, with server- or client-side fallback to Opus 4.8). Writing prompts/skills that tell the model to echo, transcribe, or explain its internal reasoning as response text triggers the `reasoning_extraction` refusal category and elevates fallbacks — audit instructions for show-your-thinking phrasing; consume structured thinking blocks instead.

## Capability deltas vs Opus 4.8

Long-horizon autonomy (multi-day goal-directed runs, strong instruction retention); first-shot correctness on complex well-specified problems (single-pass implementations that previously took days of iteration); denser technical-image vision at fewer output tokens; higher bug-finding recall, including search across codebases and repo history; handles ambiguous multi-threaded requests and determines next steps; significantly more dependable at dispatching and sustaining parallel subagents and at ongoing communication with long-running peers. Assign tasks at the top of the difficulty range — simple workloads undersell the capability range.

## Longer turns by default

Hard tasks run many minutes per request; autonomous runs extend for hours. Prefer asynchronous run-checking (scheduled jobs) over blocking. Anti-overplanning:

```text
When you have enough information to act, act. Do not re-derive facts already established
in the conversation, re-litigate a decision the user has already made, or narrate
options you will not pursue in user-facing messages. If you are weighing a choice, give
a recommendation, not an exhaustive survey. This does not apply to thinking blocks.
```

## Effort

Effort is the primary intelligence/latency/cost control: `high` default, `xhigh` for capability-sensitive work, `medium`/`low` still strong (often above prior models' `xhigh`). Higher effort buys the best verification and reasoning but can overgather on routine work and invite unrequested tidying:

```text
Don't add features, refactor, or introduce abstractions beyond what the task requires. A
bug fix doesn't need surrounding cleanup and a one-shot operation usually doesn't need a
helper. Don't design for hypothetical future requirements: do the simplest thing that
works well. Avoid premature abstraction and half-finished implementations. Don't add
error handling, fallbacks, or validation for scenarios that cannot happen. Trust
internal code and framework guarantees. Only validate at system boundaries (user input,
external APIs). Don't use feature flags or backwards-compatibility shims when you can
just change the code.
```

## Instruction following

Brief instructions now steer behaviors that previously needed per-behavior enumeration. Checkpoint discipline for long workflows:

```text
Pause for the user only when the work genuinely requires them: a destructive or
irreversible action, a real scope change, or input that only they can provide. If you
hit one of these, ask and end the turn, rather than ending on a promise.
```

## Ground progress claims during long runs

Nearly eliminated fabricated status reports in Anthropic's testing, even on tasks designed to elicit them:

```text
Before reporting progress, audit each claim against a tool result from this session.
Only report work you can point to evidence for; if something is not yet verified, say so
explicitly. Report outcomes faithfully: if tests fail, say so with the output; if a step
was skipped, say that; when something is done and verified, state it plainly without
hedging.
```

## State the boundaries

Guards against occasional unrequested actions (unasked-for fixes, defensive git-branch backups):

```text
When the user is describing a problem, asking a question, or thinking out loud rather
than requesting a change, the deliverable is your assessment. Report your findings and
stop. Don't apply a fix until they ask for one. Before running a command that changes
system state (restarts, deletes, config edits), check that the evidence actually
supports that specific action. A signal that pattern-matches to a known failure may have
a different cause.
```

## Parallel subagents

Use subagents freely with explicit when-to-delegate guidance; prefer asynchronous orchestrator-subagent communication over blocking on each return. Long-lived subagents that keep context across subtasks save time and cost through cache reads and avoid bottlenecking on the slowest subagent.

```text
Delegate independent subtasks to subagents and keep working while they run. Intervene
if a subagent goes off track or is missing relevant context.
```

## Memory system

Fable 5 performs particularly well when it records lessons from previous runs and references them; a Markdown file suffices:

```text
Store one lesson per file with a one-line summary at the top. Record corrections and
confirmed approaches alike, including why they mattered. Don't save what the repo or
chat history already records; update an existing note rather than creating a duplicate;
delete notes that turn out to be wrong.
```

Bootstrap from existing history: have it review past sessions with subagents, identify core themes/lessons, store them, and know to reference the store in future use.

## Early stopping (rare)

Deep in long sessions, may end a turn with a text-only statement of intent without the tool call, or ask permission it does not need; "continue" suffices. Autonomous-pipeline reminder:

```text
You are operating autonomously. The user is not watching in real time and cannot answer
questions mid-task, so asking "Want me to…?" or "Shall I…?" will block the work. For
reversible actions that follow from the original request, proceed without asking.
Offering follow-ups after the task is done is fine; asking permission after already
discussing with the user before doing the work is not. Before ending your turn, check
your last paragraph. If it is a plan, an analysis, a question, a list of next steps, or
a promise about work you have not done ("I'll…", "let me know when…"), do that work now
with tool calls. End your turn only when the task is complete or you are blocked on
input only the user can provide.
```

## Context-budget concern (rare)

Remaining-token countdowns surfaced to the model can trigger premature wrap-up (suggesting a new session, summarizing, trimming its own work). Avoid surfacing explicit counts; if the harness must, reassure:

```text
You have ample context remaining. Do not stop, summarize, or suggest a new session on
account of context limits. Continue the work.
```

## Give the reason, not only the request

Intent context outperforms bare requests, especially for long-running agents drawing on multiple workstreams:

```text
I'm working on [the larger task] for [who it's for]. They need [what the output
enables]. With that in mind: [request].
```

## Scaffolding

- Start at the top of the difficulty range: assign harder tasks than prior models got; have Fable 5 scope, ask clarifying questions, execute.
- Make self-verification explicit in long-run prompts; separate fresh-context verifier subagents outperform self-critique: `Establish a method for checking your own work at an interval of [X] as you build. Run this every [X interval], verifying your work with subagents against the specification.`
- Re-evaluate prompts and skills written for prior models: they are often too prescriptive for Fable 5 and degrade output; remove instructions that default behavior now covers. Fable 5 also updates skills on the fly from task learnings.
