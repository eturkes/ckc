Run ONE improvement round on the ClinicalCNL product line (SPEC §10.6; protocol authority).

One invocation = ONE round = ONE smallest-valuable increment to the language/compiler. Rounds
are safe to halt between: every round ends with a clean tree and standalone value — a green
scoped commit, or a banked queue entry (also committed). Experiment 2's informal precursor;
development machinery, never a locked measurement (SPEC §0 honesty rule).

## Preconditions (check functionally, don't assume)

- `.agent/roadmap.md`: M3 `loop-framework` unit DONE. Not DONE → STOP and report (milestone
  units run through the normal /session-prompt workflow, never through rounds).
- `git status` clean. Dirty → STOP and report (never build a round on top of WIP).
- Conformance runner green BEFORE picking (baseline honesty; runner command lives beside the
  corpus under `clinicalcnl/clinical/`). Red baseline ⇒ this round IS fixing it (category f),
  nothing else.

## Round law

1. PICK exactly one item. Order: (1) top `.agent/cnl-queue.md` item whose size fits remaining
   context (see Ceiling); (2) else scan for the smallest real gap — a scenario the profile
   can't express, a hardcoded mapping, a missing reject test. Never two items; no
   "while I'm here" edits.
2. DO it entirely within the round: implementation + tests, runner green.
3. GATE: full conformance runner; doc lint on touched markdown; engine-agnostic grep over
   touched files (LLM engines/dialects/quant tokens — SWI-Prolog/APE/Z3 are nameable);
   `git status` clean after commit.
4. CLOSE — exactly one of:
   - LAND: commit `cnl-opt (R<n>): <what> — <standalone value>`; append one ledger line to
     `.agent/cnl-queue.md` `## Ledger` (round id, category, commit subject gist).
   - BANK: can't land green this round → revert to clean tree, write a PRECISE queue entry
     (what, where, why blocked, next action), commit `cnl-opt (R<n>): bank — <item>`.
     Banking is success, not failure.
   `<n>` = last ledger round + 1. Rounds never leave the tree dirty, never stack unfinished
   work, never push.

## Categories (pre-reset accretion stages adapted: cluster → triage → draft → gate → bless)

- a. Coverage growth: author ONE new synthetic scenario, translate to ClinicalCNL, land doc +
  expected verdicts in the corpus — or bank each gap it exposes as its own queue entry.
- b. Ulex accretion: draft ONE demand-scoped lexicon entry (or alias) for a banked gap,
  through the gate stack (load integrity, profile battery, runner). Machine-drafted entries
  pass or die by the gates; rejected surfaces go to the queue's `## Reject ledger` with
  reason + reopen trigger.
- c. Generalization (`G-MDL`): replace ONE hardcoded mapping with a data-driven/reusable
  form, conformance corpus FROZEN (behavior-locked: runner green before and after, zero
  corpus edits).
- d. Profile widening: admit ONE new registered sentence pattern WITH its reject battery.
- e. Mapping deepening: extend DRS→KB coverage for ONE construct with its oracle test
  (expected values hand-written, never derived from the mapping under test).
- f. Hardening: property/adversarial tests, upstream-fork diff audit, runner speed, queue
  triage (split oversized items, dedup, reorder by value — triage counts as a round).

## Sizing + context ceiling

A round is SMALL: one entry, one pattern, one mapping, one doc — window size never changes
round sizing (small rounds ARE the standalone-value guarantee). Pick needs more than ~15% of
the window remaining → bank a split instead. All round state lives in git + the queue, so
compaction between rounds is safe by design (loop sessions run 1M-context with autoCompact
on, user-managed). Fallback when autoCompact is off: at >80% total context usage do not
pick — compact in-flight state into the queue, commit, and stop the loop (ScheduleWakeup
stop:true) telling the user to relaunch fresh.

## Review discipline

Rounds are self-gating (deterministic gates), not self-reviewing. Human/codex review rides
the user's `/codex-review` batches over recent `cnl-opt` commits — never per-round.
Requirement-level changes (new semantics, SPEC contradictions, retiring anything, JA
surface, IR bridge, Clex import) are NOT round material: bank as `major:` queue items for a
roadmap session.

Optional focus (may be empty; a non-empty focus overrides PICK order within the round law):
$ARGUMENTS
