---
description: Run a CKC dev session — follow the roadmap, or pass a task to override it
argument-hint: [task]
---

You are developing CKC. The project specification (`SPEC.md`, once authored) is
the design authority; `.agent/roadmap.md` is the canonical build plan — a flat
ordered checklist of build units and review lines. This command defines the
session protocol around them.

## Step 1: Load context

Read `.agent/memory.md` and `.agent/roadmap.md` (when present). When a specification exists,
read its Operating contract and Build plan sections (§1–§2; locate boundaries
with `grep -n '^#' SPEC.md`), and load further sections only as Step 2 directs;
full-specification loading is reserved for specification-maintenance sessions.

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

`.agent/roadmap.md` absent → **Bootstrap**. Otherwise the roadmap is a flat ordered checklist;
find the first unchecked line:

- A build-unit line → **Implement**.
- A review line (`review …`) → **Review**.
- A line marked `user-selected` → stop and confirm scope with the user first.
- No unchecked line → backlog exhausted; report and stop.

## Bootstrap session

Create `.agent/roadmap.md` and make one commit — that is the whole session. The file opens with
a short header stating: it is a flat ordered checklist consumed by this command; `SPEC.md` is
the design authority and its §2 the build plan; completed lines gain `[x]` plus a trailing
`NN% NNNK/200K` annotation from `.agent/compaction.sh`; an empty tail means *author the next
units from the current SPEC milestone*. After the header, exactly these seed lines:

```text
- [ ] boilerplate: minimal repository skeleton per SPEC §3 — extend .gitignore as needed, root
  Cargo workspace stub (no member crates yet), corpus/{fixtures,lexicon,gold}/ and registry/
  directories. Reading: SPEC §3. Gate: `cargo test --workspace` runs clean on the empty
  workspace and the tree commits clean.
- [ ] plan-v1: author the V1 build units into this roadmap from SPEC §8.7 (dependency order;
  memory sizing anchors), run as a planning
  workflow per this command. Reading: SPEC §2, §4, §8. Gate: forward units authored below this
  line.
```

## Dynamic workflows (planning and review)

In roadmap mode, planning (unit splitting) and review tasks run as **dynamic
workflows**: the moment you reach the task, author a script inline and pass it
to the `Workflow` tool. This command's instruction is your standing opt-in to
multi-agent orchestration — call `Workflow` directly, no confirmation needed.
There are no saved workflows; every script is written fresh for the task at
hand. Subagent model and effort (fable/max) come from the user's global Claude
settings, so omit per-call `model`.

Division of labor: the workflow fans out reading and analysis and returns
structured findings (use the `schema` option); the main session owns every
mutation — corrections, roadmap edits — and the single closing commit. Keep
finders read-only (`agentType: 'Explore'`, or `isolation: 'worktree'` for any
agent that must write); when the workflow returns, `git status` and reconcile
every stray path before staging — Explore agents are edit-restricted but still
hold `Bash`, so a verifier can mutate the tree. Implement sessions otherwise
stay single-context; only their split fallback escalates to a workflow.

## Implement session

Read the unit's reading slice — the specification sections and prior artifacts
its roadmap line names — plus earlier accepted repository artifacts as needed.
Read `git show HEAD` to match the previous unit's patterns. Implement the unit's
deliverable; run its acceptance gate until green; run the full test suite.

Right before staging, run `.agent/compaction.sh`; in the same roadmap edit that
marks the unit `[x]`, append its `NN% NNNK/200K` output to the unit's line.
Stage everything and make one commit covering work + roadmap, with the unit id
as the scope (e.g. `<unit-id>: …`) — review sessions locate their ranges by
these ids.

### Splitting (when a unit overruns)

A unit must be finishable AND committable within one context window with margin
to spare; if mid-work you project otherwise, stop implementing. Produce the
split through a dynamic workflow (judge-panel shape): fan out agents that each
propose a full decomposition from a different boundary (specification-section
family, artifact layer, deliverable type), score the candidates with parallel
judges against the sizing constraints, and return the winner as structured
sub-line specs. Each sub-line is one conceptual deliverable with explicit file
paths, real identifiers from the existing codebase, and exactly one gate command
(the unit's full gate lands on the final sub-line; give earlier sub-lines
narrower test commands); calibrate granularity from neighbouring units' `NN%`
annotations. From the result the main session replaces the unit's roadmap line
with the `<unit-id>.1`, `<unit-id>.2`, … sub-lines, commits the split plan, and
ends the session.

## Review session

The preceding group's lines are all `[x]`. Recover the range: `git log
--oneline --reverse` from the previous review commit (or the root commit) to
HEAD, and `git show` the group's unit commits to bound the review scope.

Run the review through a dynamic workflow (dimensions → find → adversarially
verify → synthesize): fan out one read-only finder per scrutiny dimension —
bugs and incorrect logic, specification non-conformance, CLAUDE.md/memory
non-conformance, inconsistencies, token-inefficiency, obsolescence — over the
recovered range; pass each finding to an independent skeptic prompted to refute
it, and keep only survivors; synthesize them into a deduplicated, file-sorted
list (use the `schema` option). The scope centers on the group; carry any
project-wide finding you hit, and scale the pool to the range.

From the workflow result the main session makes the corrections, marks the
review line `[x]`, and makes one commit: describe the corrections, or state the
review was clean.

A milestone-wide review line widens the finder pool and adds
cross-group-consistency and contract-surface dimensions.

## Task argument

$ARGUMENTS
