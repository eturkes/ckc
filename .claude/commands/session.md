---
description: Run a CKC dev session — follow the roadmap, or pass a task to override it
argument-hint: [task]
---

You are developing CKC, a staged proof-carrying compiler for Japanese clinical
text conflict review. SPEC.md is the design authority; its §11.3 build-unit
table is the single canonical build plan (unit scopes, dependencies, acceptance
gates). This command defines the session protocol around it.

## Step 1: Load context

Read `.agent/memory.md`, `.agent/roadmap.md`, and SPEC.md §0 (agent operating
contract) plus the §11.3 and §11.4 tables. Locate section boundaries with
`grep -n '^#' SPEC.md`. Load further SPEC sections only as Step 2 directs;
full-spec loading is reserved for specification-maintenance sessions.

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

The roadmap is a flat ordered checklist. Find the first unchecked line:

- A build unit (`M0.x.y`, or a split sub-line `M0.x.y.z`) → **Implement**.
- A review line (`review …`) → **Review**.
- A line marked `user-selected` → stop and confirm scope with the user first.
- No unchecked line → backlog exhausted; report and stop.

## Implement session

Read exactly the unit's §11.4 reading slice (the listed sections plus the named
Appendix A slice), and earlier accepted repository artifacts as needed. Read
`git show HEAD` to match the previous unit's patterns. Implement the unit's
§11.3 deliverable; run its acceptance gate until green; run the full workspace
test suite.

Right before staging, run `.agent/compaction.sh`; in the same roadmap edit that
marks the unit `[x]`, append its `NN% NNNK/200K` output to the unit's line.
Stage everything and make one commit covering work + roadmap, with the unit id
in the subject (e.g. `feat(m0.1.2): …`) — review sessions locate their ranges
by these ids.

### Splitting (when a unit overruns)

A unit must be finishable AND committable within one context window with margin
to spare; if mid-work you project otherwise, stop implementing. Replace the
unit's roadmap line with sub-lines `M0.x.y.1`, `M0.x.y.2`, … — each one
conceptual deliverable with explicit file paths, real identifiers from the
existing codebase, and exactly one gate command (the unit's §11.3 gate lands on
the final sub-line; give earlier sub-lines narrower test commands). Calibrate
granularity from neighbouring units' `NN%` annotations. Commit the split plan
and end the session.

## Review session

The preceding group's lines are all `[x]`. Recover the range: `git log
--oneline --reverse` from the previous review commit (or the root commit) to
HEAD, and `git show` the group's unit commits. Scrutinize for bugs, incorrect
logic, SPEC/CLAUDE.md/memory non-conformance, inconsistencies,
token-inefficiency, and obsolescence. The scope centers on the group; apply any
project-wide finding you hit. Make corrections, mark the review line `[x]`, and
make one commit: describe the corrections, or state the review was clean.

The final `review M0` line is a milestone-wide pass: apply the same scrutiny
across all groups, with fresh attention to cross-group consistency and the
§11.1/§11.2 contract surfaces.

## Task argument

$ARGUMENTS
