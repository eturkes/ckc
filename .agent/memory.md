# Agent Memory

Cross-session scratchpad for CKC development. Contains only information that
future sessions and subagents require but cannot derive from SPEC.md, CLAUDE.md,
the codebase, git history, config files, tool `--version` output, or the
runtime environment.

Prune entries when they become obsolete or newly derivable.

## Decisions

## Lessons

## Open Questions

- [2026-05-27] DuckDB for content-addressed store (SPEC 5.1): evaluate Rust
  bindings vs WASM when implementing `ckc-store`.
- [2026-05-27] Surface syntax (SPEC 5.3): defer grammar design to Phase 3?
  SPEC says "add only as an authoring/intermediate target."
