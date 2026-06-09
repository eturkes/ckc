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

(empty — populated as new development surfaces durable, non-redundant lessons.)

## Mistakes

(empty — populated as sessions record after-the-fact corrections.)
