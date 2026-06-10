---
description: Run a CKC dev session — follow the roadmap, or pass a task to override it
argument-hint: [task]
---

You are developing CKC. The project specification (`SPEC.md`) is the design
authority; `.agent/roadmap.md` is the canonical build plan — one milestone
header over its ordered unit checklist, with closed milestones persisting as
bare headers. This command defines the session protocol around them.

Context sizing: every session runs in a 200K context window, except Review
sessions, which run at 1M — a review's value comes from holding as much of the
codebase as possible coherently in one context.

Bookkeeping: completed checklist items record the context usage and commit
hash of the session that completed them (`NN% NNNK/200K <hash>`); milestone
headers record the commit that opened the milestone (`plan <hash>`) and the
commit that closed it (`review <hash>`), so the pair bounds the milestone's
commit range. Plan and review stamps carry hashes only, never usage. A commit
cannot contain its own hash, so hashes land lazily: a session writes `_` in
its own hash slot, and the next unit of roadmap work fills the latest `_`
within its single closing commit, resolving it from commit scopes (`git log
--oneline`; item commits are scoped `<unit-id>:`, plan commits `plan-v<n>:`,
review commits `review-v<n>:`). At most one `_` is pending at a time;
execute-task sessions neither fill nor create one. Usage comes from
`.agent/compaction.sh`, run right before staging.

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
  then commit per CLAUDE.md (one commit covering the work; leave any pending
  `_` alone). Leave the roadmap untouched unless the task itself directs an
  edit.

### Select session type (roadmap mode)

The current milestone is the last header in the roadmap:

- Its header lacks a `review` marker and an unchecked item exists →
  **Implement** the first unchecked item (a line marked `user-selected` → stop
  and confirm scope with the user first).
- Its header lacks a `review` marker and every item is checked → **Review**.
- Its header carries a `review` marker (hash or pending `_`) → the milestone
  is closed → **Plan** the next SPEC §2 milestone. No milestone left → fill a
  pending `_` if one exists (one small commit), report the spec fully
  implemented, and stop.

## Plan session

A plan session opens the next milestone. When that milestone's contract
section is still compact, elaborate it first per SPEC §1 — the spec diff
reaches the user before any unit consumes it. Then author the milestone header
and its full unit checklist.

Planning is the one task that runs as a **dynamic workflow**: unit authoring
is a discrete task that divides cleanly among subagents without loading the
main context. Author a script inline and pass it to the `Workflow` tool; this
command's instruction is your standing opt-in to multi-agent orchestration —
call `Workflow` directly, no confirmation needed. There are no saved
workflows; every script is written fresh for the task at hand. Subagent model
and effort (fable/max) come from the user's global Claude settings, so omit
per-call `model`. A judge-panel shape fits: fan out agents that each propose a
full decomposition from a different boundary (specification-section family,
artifact layer, deliverable type), score the candidates with parallel judges,
and return the winner as structured unit specs (use the `schema` option). Keep
finders read-only (`agentType: 'Explore'`); when the workflow returns, `git
status` and reconcile every stray path before staging — Explore agents are
edit-restricted but still hold `Bash`, so a finder can mutate the tree. The
main session owns every mutation — spec edits, roadmap edits — and the closing
commits.

Calibrate unit sizing against the previous milestone's checklist, still
present while you plan — its per-item usage annotations are ground truth for
what fits a window — plus the memory.md sizing anchors. Each unit line is one
conceptual deliverable with explicit file paths, real identifiers, its reading
slice, and exactly one gate command.

Closing: one commit, scoped `plan-v<n>:` — fill the previous milestone's
pending `review _`, delete its items keeping the bare header (a closed
milestone collapses to that one line; git history retains the rest), and
append the new milestone header `## <milestone> — plan _` + checklist. End the
session.

Every other session type stays single-context: ad-hoc read-only subagent
lookups (`docs/`) remain available everywhere, and the `Workflow`
tool is reserved for planning.

## Implement session

Read the unit's reading slice — the specification sections and prior artifacts
its roadmap line names — plus earlier accepted repository artifacts as needed.
Read `git show HEAD` to match the previous unit's patterns. Implement the
unit's deliverable; run its acceptance gate until green; run the full test
suite.

One closing commit covers work + roadmap — fill any pending `_`, mark the item
`[x]` with its usage annotation and a fresh `_` hash slot — with the unit id
as the scope (e.g. `<unit-id>: …`). End the session.

A unit must be finishable AND committable within one context window with
margin to spare; if mid-work you project otherwise, stop implementing, bring
the tree to a clean state, and report the overrun — recovery (restoring to
the last commit, re-scoping the roadmap) is always user-initiated.

## Review session

Every item of the current milestone is checked. Review sessions run
single-context at 1M: hold the milestone's range, its artifacts, and the
implicated spec sections together in your own context and analyze them
coherently yourself — the 1M window exists precisely so the codebase stays
whole instead of fragmenting across subagents.

Recover the range from the header: `git log --oneline <plan-hash>..HEAD`, and
`git show` the unit commits (the items' recorded hashes) to bound the scope;
read the touched artifacts in full.

Beyond trivial bug fixes, the review is a holistic analysis of codebase
cohesion and overall project direction, scrutinized along: bugs and incorrect
logic, specification non-conformance, CLAUDE.md/memory non-conformance,
inconsistencies, token-inefficiency, obsolescence. Specification improvements
are in scope: when the analysis exposes a better contract or design, edit
SPEC.md in the same session (contract-affecting amendments reach the user
first, per SPEC §1).

Close the milestone in one commit, scoped `review-v<n>:` — fill the last
item's pending `_`, carry the corrections (or state the review was clean), and
mark the header `— review _`; reviews record no usage. The next roadmap-mode
session — the plan session for the next milestone — fills the review hash.

## Task argument

$ARGUMENTS
