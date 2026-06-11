---
description: Run a CKC dev session — follow the roadmap, or pass a task to override it
argument-hint: [task]
---

You are developing CKC. The project specification (`SPEC.md`) is the design
authority; `.agent/roadmap.md` is the canonical build plan — one milestone
header over its ordered unit checklist, with closed milestones persisting as
bare headers. This command defines the session protocol around them; plan and
review sessions continue in the `.agent/protocol-*.md` file their session
type names below.

Context sizing: every session runs in a 200K context window, except Review
sessions, which run at 1M — a review's value comes from holding as much of the
codebase as possible coherently in one context. The cap is launch-set and
process-wide: every subagent runs under its session's window — the `[1m]`
model slug lifts nothing (sizing and failure mode: memory.md Lessons).

Bookkeeping: completed checklist items record the context usage and commit
hash of the session that completed them (`NN% NNNK/200K <hash>`); milestone
headers record the commit that opened the milestone (`plan <hash>`) and the
commit that closed it (`review <hash>`), so the pair bounds the milestone's
commit range. Plan and review stamps carry hashes only, never usage. A commit
cannot contain its own hash, so hashes land lazily: a session writes `_` in
its own hash slot, and the next unit of roadmap work fills the latest `_`
within its single closing commit, resolving it from commit scopes (`git log
--oneline`; item commits are scoped `<unit-id>:`, plan commits `plan-m<n>:`,
review commits `review-m<n>:`). At most one `_` is pending at a time;
execute-task sessions neither fill nor create one. Usage comes from
`.agent/compaction.sh`, run right before staging. A session that hit
compaction records `>=90% compacted/200K` in place of usage; whether compacted work
lands at all stays the user's call, per overrun recovery.

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
  **Implement** the first unchecked item per the section below (a line marked
  `user-selected` → stop and confirm scope with the user first).
- Its header lacks a `review` marker and every item is checked → **Review**:
  read `.agent/protocol-review.md` and follow it.
- Its header carries a `review` marker (hash or pending `_`) → the milestone
  is closed → **Plan** the next SPEC §2 milestone: read
  `.agent/protocol-plan.md` and follow it. No milestone left → fill a
  pending `_` if one exists (one small commit), report the spec fully
  implemented, and stop.

All sessions except Plan stay single-context: ad-hoc read-only subagent
lookups (`docs/`) remain available everywhere, and the `Workflow`
tool is reserved for plan sessions (opt-in and shape in the plan protocol).

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

## Task argument

$ARGUMENTS
