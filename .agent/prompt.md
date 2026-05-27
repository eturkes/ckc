You are developing CKC, a proof-carrying clinical knowledge compiler. Read the
following files to orient, then determine your session type.

## Step 1: Orient

Read `SPEC.md`, `.agent/memory.md`, `.agent/roadmap.md`, and `git log --oneline -20`.

## Step 2: Determine session type

Find the first incomplete Phase in the roadmap. Within it, find the first
incomplete top-level task (e.g., `0.2`). The roadmap state determines exactly
one session type:

**Type A — Plan subtasks:** The next incomplete top-level task has zero
subtasks listed beneath it. This session is dedicated to decomposing it into
session-sized subtasks. Add indented `- [ ]` lines under the task in the
roadmap. Commit the updated roadmap and end the session.

**Type B — Complete one subtask:** The next incomplete top-level task already
has subtasks. This session completes exactly one subtask: the first incomplete
one. Do the work, commit, and mark the subtask `[x]` in the roadmap. If it
was the last incomplete subtask, also mark the parent top-level task `[x]`.
Commit the roadmap update and end the session.

**Type C — Create next Phase:** Every top-level task in the current Phase is
`[x]`, and the next Phase has no top-level tasks listed yet. This session is
dedicated to writing the next Phase's top-level tasks (with no subtasks).
Derive them from SPEC.md Section 20. Commit the updated roadmap and end the
session.
