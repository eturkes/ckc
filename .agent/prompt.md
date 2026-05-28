You are developing CKC, a proof-carrying clinical knowledge compiler. Read the
following files to load context, then determine your session type.

## Step 1: Load context

Read `SPEC.md`, `.agent/memory.md`, `.agent/roadmap.md`, and `git show HEAD`.

## Step 2: Determine session type

Find the first incomplete Phase in the roadmap. Within it, find the first
incomplete top-level task (e.g., `0.2`). The roadmap state determines exactly
one session type:

**Type A — Plan subtasks:** The next incomplete top-level task has zero
subtasks listed beneath it. This session is dedicated to decomposing it into
session-sized subtasks. Add indented `- [ ]` lines under the task in the
roadmap. Also erase all subtask lines from previously completed top-level tasks
(keep only the `- [x] N.M Title` line for each). Commit the updated roadmap
and end the session.

**Type B — Complete one subtask:** The next incomplete top-level task already
has subtasks. This session completes exactly one subtask: the first incomplete
one. Do the work, then mark the subtask `[x]` in the roadmap (and the parent
top-level task `[x]` if it was the last incomplete subtask). Stage everything
and make a single commit covering both the task work and the roadmap update.

**Type C — Create next Phase:** Every top-level task in the current Phase is
`[x]`, and the next Phase has no top-level tasks listed yet. This session is
dedicated to writing the next Phase's top-level tasks (with no subtasks).
Derive them from SPEC.md Section 20. Also erase all subtask lines from the
now-completed previous Phase (keep only top-level task lines). Commit the
updated roadmap and end the session.

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
