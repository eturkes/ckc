# Agent Memory

Every entry must provide value beyond what SPEC.md, CLAUDE.md, the codebase,
git history, config files, tool `--version` output, and the runtime
environment already provide. Exception: high-value reminders that are
technically derivable but easily forgotten under token pressure.

## Decisions

## Lessons

- [2026-05-27] Every piece of natural language in the project (docs, comments,
  diagnostics) must use positive constraint framing per the pink elephant
  principle in CLAUDE.md. Audit before committing.
- [2026-05-27] Memory entries, roadmap descriptions, and doc preambles are the
  highest-risk locations for redundancy with SPEC.md and config files. Default
  assumption: if a fact lives in a file, it belongs only in that file.

## Open Questions

- [2026-05-27] DuckDB for content-addressed store (SPEC 5.1): evaluate Rust
  bindings vs WASM when implementing `ckc-store`.
- [2026-05-27] Surface syntax (SPEC 5.3): defer grammar design to Phase 3?
  SPEC says "add only as an authoring/intermediate target."
