You are developing CKC, a proof-carrying clinical knowledge compiler. Read the
following files to load context, then determine your session type.

## Step 1: Load context

Read `SPEC.md`, `.agent/memory.md`, `.agent/roadmap.md`, and `git show HEAD`.

## Step 2: Determine session type

Scan Phases in roadmap order. Define:

- `T` = first incomplete top-level task (may be none).
- `L` = most-recently-completed top-level task: the last `[x]` line before `T`,
  or the final `[x]` line when `T` is none.
- "subtasks present" = the task has indented subtask lines.
- "reviewed" = the task line ends with the ` reviewed` marker.

Select the first matching rule:

1. `T` has subtask lines → **Type B**.
2. `L` has subtask lines present and is not reviewed:
   - `T` exists → **Type R** (per-task review of `L`).
   - `T` is none → **Type P** (Phase review of `L`'s Phase).
3. `T` exists (so it has no subtasks) → **Type A** (open `T`).
4. `T` is none, `L` is reviewed, and SPEC.md §20 defines a further Phase →
   **Type C**.
5. Otherwise → backlog exhausted; report and stop.

So each top-level task flows: Type A opens it → Type B completes its subtasks →
one review covers it (Type R when it sits mid-Phase, Type P when it is the
Phase's last task) → the next Type A erases its subtasks.

## Type A — Open a top-level task by planning its subtasks

`T` has no subtasks. Decompose it into session-sized subtasks (see "Subtask
sizing"). Calibrate granularity from `L`'s still-present per-subtask
`(NNNK/200K)` annotations: split more finely wherever context ran hot. `L` may
sit in an earlier Phase than `T` (the Phase-boundary case); calibrating across
that boundary is correct. At project start, when no `L` carries subtasks, size
from the sizing guidance alone.

After writing `T`'s subtasks, erase the subtask lines of every completed
top-level task, keeping each `- [x] N.M Title @hash` line and any ` reviewed`
marker. Commit the plan. Then read that commit's short hash, append ` @<hash>`
to `T`'s top-level line, and commit a second time. Recording a commit's own hash
needs this fresh follow-up commit; an amend would rewrite the very hash you
recorded. End the session.

## Type B — Complete one subtask

Complete exactly the first incomplete subtask of `T`: do the work and pass its
gate. Right before staging, run `python3 .agent/context_window.py`; in the same
roadmap edit that marks the subtask `[x]`, append its `(NNNK/200K)` output
verbatim to the end of that subtask's line. Mark `T` `[x]` too when this was its
last incomplete subtask. Leave all subtask lines in place for the later review
and the next Type A. Stage everything and make a single commit covering both the
task work and the roadmap update.

## Type R — Review a completed top-level task

`L` is complete with its subtasks still present and unreviewed, and a later task
`T` exists, so `L` sits mid-Phase. Review `L`'s work over its review range:
`git show <Lhash>` for the subtask plan, `git log`/`git diff` over
`<Lhash>..HEAD` for the implementing commits, and the current state of the files
they touch.

Scrutinize for bugs, incorrect logic, patterns non-conformant with
SPEC.md/CLAUDE.md/`.agent/memory.md`, inconsistencies, token-inefficiency, and
obsolescence. The scope centers on `L`; apply any project-wide finding you hit.
Make your corrections. Append ` reviewed` to `L`'s top-level line. Make one
commit: describe the corrections when you made any, or state the review was
clean when you made none. The ` reviewed` edit ships in this commit either way.

## Type P — Review a complete Phase

Every top-level task in the final listed Phase is `[x]`, `T` is none, and that
Phase's last task `L` is unreviewed. Review the whole Phase. Run
`git show <hash>` for every top-level task's `@hash` in the Phase — this recovers
each subtask that once existed (since erased from the roadmap) plus its
implementing diff. Also scan `<first-task-hash>..HEAD` for the full Phase span.

Apply the Type R scrutiny Phase-wide; optionally implement changes. Append
` reviewed` to `L`'s line — this marks the Phase reviewed and gates Type C. Make
one commit (corrections described, or review clean).

## Type C — Create the next Phase

Every top-level task in the current Phase is `[x]` and reviewed (Type P is done),
and the next Phase has no tasks listed. Write that Phase's top-level task lines
only — derived from SPEC.md §20, with no subtasks and no `@hash`. Leave all
existing subtask lines untouched, so the previous Phase's last task keeps its
annotated subtasks: use them to gauge how much work a top-level task represents
when sizing this Phase's tasks, and the next Type A reuses them to calibrate
subtasks. Commit; this commit carries no `@hash`. End the session.

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
