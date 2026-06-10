---
description: Run a CKC dev session — follow the roadmap, or pass a task to override it
argument-hint: [task]
---

You are developing CKC. The project specification (`SPEC.md`) is the design
authority; `.agent/roadmap.md` is the canonical build plan — a flat ordered
checklist of build units and review lines. This command defines the session
protocol around them.

Context sizing: every session runs in a 200K context window, except Review
sessions, which run at 1M — a review's value comes from holding as much of the
codebase as possible coherently in one context. Any session that marks a
roadmap line `[x]` runs `.agent/compaction.sh` right before staging and appends
its `NN% NNNK/200K` (1M sessions: `/1M`) output to that line.

## Step 1: Load context

Read `.agent/memory.md` and `.agent/roadmap.md`. Read SPEC.md's Operating
contract and Build plan sections (§1–§2; locate boundaries with `grep -n '^#'
SPEC.md`), and load further sections only as Step 2 directs; full-specification
loading is reserved for specification-maintenance sessions.

## Step 2: Pick session mode

Read this session's task argument, shown under "Task argument" at the end of
this command.

- **Blank** ⇒ *roadmap mode*: run "Select session type" below, then follow the
  matching session.
- **Non-blank** ⇒ *execute-task mode*: the argument is this session's sole task
  and overrides the roadmap. Skip "Select session type"; carry the task out,
  then commit per CLAUDE.md (one commit covering the work). Leave the roadmap
  checklist untouched unless the task itself directs an edit.

### Select session type (roadmap mode)

The roadmap is a flat ordered checklist; find the first unchecked line:

- A planning line — its deliverable is authoring roadmap units (`plan-v<n>`,
  elaboration) → **Plan**.
- Any other build-unit line → **Implement**.
- A review line (`review …`) → **Review**.
- A line marked `user-selected` → stop and confirm scope with the user first.
- No unchecked line → the tail is empty: author the next units from the current
  SPEC milestone as a **Plan** session (when the next milestone's contract
  section is still compact, that is an elaboration session per SPEC §1, whose
  spec diff reaches the user before units are seeded).

## Plan session

Planning — authoring roadmap units — is the one task that runs as a **dynamic
workflow**: unit authoring is a discrete task that divides cleanly among
subagents without loading the main context. The moment you reach the task,
author a script inline and pass it to the `Workflow` tool; this command's
instruction is your standing opt-in to multi-agent orchestration — call
`Workflow` directly, no confirmation needed. There are no saved workflows;
every script is written fresh for the task at hand. Subagent model and effort
(fable/max) come from the user's global Claude settings, so omit per-call
`model`. A judge-panel shape fits: fan out agents that each propose a full
decomposition from a different boundary (specification-section family, artifact
layer, deliverable type), score the candidates with parallel judges against the
memory.md sizing anchors and neighbouring units' `NN%` annotations, and return
the winner as structured unit specs (use the `schema` option). Each unit line
is one conceptual deliverable with explicit file paths, real identifiers, its
reading slice, and exactly one gate command.

Division of labor: the workflow fans out reading and analysis and returns
structured results; the main session owns every mutation — roadmap edits, spec
edits — and the single closing commit. Keep finders read-only (`agentType:
'Explore'`); when the workflow returns, `git status` and reconcile every stray
path before staging — Explore agents are edit-restricted but still hold `Bash`,
so a finder can mutate the tree. From the result the main session authors the
unit lines into the roadmap, marks the planning line `[x]`, and commits.

Every other session type stays single-context: ad-hoc read-only subagent
lookups (`docs/`) remain available everywhere, and the `Workflow`
tool is reserved for planning.

## Implement session

Read the unit's reading slice — the specification sections and prior artifacts
its roadmap line names — plus earlier accepted repository artifacts as needed.
Read `git show HEAD` to match the previous unit's patterns. Implement the unit's
deliverable; run its acceptance gate until green; run the full test suite.

Mark the unit `[x]`, stage everything, and make one commit covering work +
roadmap, with the unit id as the scope (e.g. `<unit-id>: …`) — review sessions
locate their ranges by these ids.

### Splitting (when a unit overruns)

A unit must be finishable AND committable within one context window with margin
to spare; if mid-work you project otherwise, stop implementing and author the
split in-session — splitting is a normal single-context task. Decompose at the
cleanest boundary (specification-section family, artifact layer, deliverable
type). Each sub-line is one conceptual deliverable with explicit file paths,
real identifiers from the existing codebase, and exactly one gate command (the
unit's full gate lands on the final sub-line; give earlier sub-lines narrower
test commands); calibrate granularity from neighbouring units' `NN%`
annotations. Replace the unit's roadmap line with the `<unit-id>.1`,
`<unit-id>.2`, … sub-lines, commit the split plan, and end the session.

## Review session

The preceding group's lines are all `[x]`. Review sessions run single-context
at 1M: hold the recovered range, its artifacts, and the implicated spec
sections together in your own context and analyze them coherently yourself —
the 1M window exists precisely so the codebase stays whole instead of
fragmenting across subagents.

Recover the range: `git log --oneline --reverse` from the previous review
commit (or the root commit) to HEAD, and `git show` the group's unit commits to
bound the review scope; read the touched artifacts in full.

Beyond trivial bug fixes, the review is a holistic analysis of codebase
cohesion and overall project direction, scrutinized along: bugs and incorrect
logic, specification non-conformance, CLAUDE.md/memory non-conformance,
inconsistencies, token-inefficiency, obsolescence. Specification improvements
are in scope: when the analysis exposes a better contract or design, edit
SPEC.md in the same session (contract-affecting amendments reach the user
first, per SPEC §1). The scope centers on the group; carry any project-wide
finding you hit.

Make the corrections, mark the review line `[x]`, and make one commit: describe
the corrections, or state the review was clean.

A milestone-wide review line widens scope to the whole milestone and adds
cross-group-consistency and contract-surface dimensions.

## Task argument

$ARGUMENTS
