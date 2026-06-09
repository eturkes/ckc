# Agent Memory

Every entry must provide value beyond what the project specification, CLAUDE.md,
the codebase, git history, config files, tool `--version` output, and the
runtime environment already provide. Exception: high-value reminders that are
technically derivable but easily forgotten under token pressure.

## Project history

- [2026-06-09] Branch restructure: `archive/spec02` holds the complete
  implementation of the reworked M0 specification (canonical-bytes core,
  SPEC-extraction registry and conformance checkers, runtime/replay manifests
  over a content-addressed store, CLI contract surface, source-graph schemas,
  the closure/residual engine, and the mechanized-observation layer) plus its
  full agent memory — environment and tooling lessons, work-item sizing
  corollaries, and SPEC-parser coupling notes. `archive/spec01` holds the prior
  864-line specification's Phase-0 kernel. Current `main` is a fresh orphan root
  for the next specification (spec03); the three histories share no commits, and
  `archive/specNN` numbers superseded-specification implementations in sequence.
  Consult the archives for prior art and any pre-restructure question — e.g.
  spec02's canonical serializer, golden-test harness, store layout, and
  closure/certificate patterns; spec01's Phase-0 equivalents.

## Lessons

- [2026-06-09] Work-unit sizing calibration (transferred from `archive/spec02`
  roadmap `NN%` annotations + memory corollaries, which hold the full detail).
  Target one conceptual deliverable + one gate per unit, finishable AND
  committable in one context window with margin; prefer more, smaller units.
  Empirical anchors: canonical JSON is a three-unit deliverable (writer core /
  sort-keys+collections / unions+strict-reading+hash; the last ran 86%); a unit
  that both implements a nontrivial algorithm AND authors a per-row table splits
  in two; registry entry *types* alone fit one unit (~69%); crate-foundation
  units run hot (~81%). Used to size the M1 seed batch. Calibrate finer JIT
  splits from the actual `NN%` of neighbouring units once they exist.

## Mistakes

(empty — populated as sessions record after-the-fact corrections.)
