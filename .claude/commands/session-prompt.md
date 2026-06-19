Run a CKC development session. `CLAUDE.md` is the operating contract, `SPEC.md`
the design authority, and `.agent/roadmap.md` the canonical build plan: one
milestone header over its ordered unit checklist, closed milestones persisting
as bare headers. Plan and Review sessions continue in the `.agent/protocol-*.md`
file named below.

## Load context

Read `.agent/memory.md` and `.agent/roadmap.md` in full, then SPEC §1–§2
(Operating contract, Build plan; find headers with `grep -n '^#' SPEC.md`). Load
further SPEC sections only as the chosen session directs; whole-spec loading is
reserved for spec-maintenance sessions.

## Pick mode from the task argument (end of file)

- **Non-empty** ⇒ execute-task: the argument is the session's sole task and
  overrides the roadmap. Carry it out and commit per CLAUDE.md (one commit; leave
  any pending `_` untouched); touch the roadmap only if the task directs it.
- **Empty** ⇒ roadmap mode: the current milestone is the last roadmap header.
  - Lacks a `review` marker, an unchecked item remains ⇒ **Implement** the first
    unchecked item (a line marked `user-selected` ⇒ confirm scope with the user
    first).
  - Lacks a `review` marker, every item checked ⇒ **Review**: follow
    `.agent/protocol-review.md`.
  - Carries a `review` marker ⇒ milestone closed ⇒ **Plan** the next SPEC §2
    milestone: follow `.agent/protocol-plan.md`. No milestone left ⇒ fill a
    pending `_` if one exists (one small commit), report the spec fully
    implemented, and stop.

## Implement session

Read the unit's reading slice — the spec sections and prior artifacts its roadmap
line names — plus earlier accepted artifacts as needed, and `git show HEAD` to
match the previous unit's patterns. Build the deliverable, drive its acceptance
gate to green, then run the full test suite. One closing commit covers work +
roadmap, scoped `<unit-id>:`: fill any pending `_`, and mark the item `[x]` with
its usage annotation and a fresh `_` slot.

## Sizing, bookkeeping, lifecycle

- Sessions run at 200K, except Review at 1M (to hold the milestone whole). The cap
  is launch-set and process-wide: subagents inherit the session window, so the
  `[1m]` slug lifts nothing (memory.md Lessons).
- Completed items record `NN% NNNK/200K <hash>` (usage from `.agent/compaction.sh`,
  run right before staging; `>=90% compacted/200K` when compaction hit). Milestone
  headers record `plan <hash>` and `review <hash>`, bounding the milestone's commit
  range; plan and review stamps carry hashes only. Hashes land lazily: a session
  writes `_` in its own slot, and the next roadmap-work session fills the latest
  `_` in its closing commit, resolving it from commit scopes (`<unit-id>:`,
  `plan-m<n>:`, `review-m<n>:`). At most one `_` pends at once; execute-task
  sessions neither fill nor create one.
- A unit must finish AND commit within the window with margin — including margin
  for the post-session `/codex-review` pass the user runs after every session (its
  accepted fixes land in a separate `codex:`-scoped commit). If you project an
  overrun mid-work, stop, bring the tree to a clean state, and report; recovery
  (restore to last commit, re-scope the roadmap) is user-initiated.
- Every session except Plan stays single-context: ad-hoc read-only subagent
  lookups stay available, but the `Workflow` tool is reserved for Plan (its
  protocol sets the opt-in and shape).

Task (may be empty): $ARGUMENTS
