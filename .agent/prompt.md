You are developing CKC, a proof-carrying clinical knowledge compiler. Read the
following files to load context, then determine your session type.

## Step 1: Load context

Read `SPEC.md`, `.agent/memory.md`, `.agent/roadmap.md`, and `git show HEAD`.

## Step 2: Determine session type

Scan Phases in order and locate the first incomplete top-level task `T`. The
roadmap state selects exactly one session type:

- `T` has subtask lines → **Type B**.
- `T` has zero subtask lines → **Type A**.
- No incomplete top-level task exists and SPEC.md Section 20 defines a Phase
  beyond the last one listed → **Type C**.

**Type B — Complete one subtask:** Complete exactly the first incomplete
subtask of `T`: do the work and pass its gate. Right before staging, run
`python3 .agent/context_window.py`; in the same roadmap edit that marks the
subtask `[x]`, append its `(NNNK/200K)` output verbatim to the end of that
subtask's line. Mark `T` `[x]` too when this was its last incomplete subtask.
Stage everything and make a single commit covering both the task work and the
roadmap update.

**Type A — Open a top-level task by planning its subtasks:** `T` has no
subtasks yet. Decompose it into session-sized subtasks (see "Subtask sizing").
Calibrate granularity from the most-recently-completed top-level task whose
subtask lines are still present: read its per-subtask `(NNNK/200K)` annotations
and split more finely wherever context ran hot. That completed task may sit in
an earlier Phase than `T`; using it across the Phase boundary is correct. After
writing `T`'s subtasks, erase the subtask lines of every completed top-level
task (keep each one's `- [x] N.M Title` line). At project start, when no
preceding completed task carries subtasks, size from the sizing guidance alone.
Commit the updated roadmap and end the session.

**Type C — Create next Phase:** Every top-level task in the current Phase is
`[x]` and the next Phase has no top-level tasks listed yet. Write that Phase's
top-level task lines only — derived from SPEC.md Section 20, with no subtasks.
Leave all existing subtask lines untouched, so the previous Phase's final task
keeps its annotated subtasks for the next Type A. Commit the updated roadmap
and end the session.

## Subtask sizing (Type A and any re-planning)

Size every subtask so a fresh agent finishes the work AND commits it within a
single context window with margin to spare. When completing a subtask would
require context compaction, it is too large; split it into more subtasks.

Scope each subtask to exactly one conceptual deliverable plus its gate, e.g.:
author one fixture/data set; implement one logic module with its unit tests;
wire one integration test; add persistence/determinism for one artifact kind.
Keep data authoring, pure-logic implementation, integration wiring, and
persistence/determinism in separate subtasks rather than bundling them.

Prefer many small subtasks over few large ones; default to splitting. Give each
subtask explicit file paths, the exact types/functions it touches, real
identifiers from the existing fixtures/codebase, and exactly one crisp gate
command. Number them N.M sequentially and add as many as the deliverable
honestly needs.
