# Agent Memory

Cross-session scratchpad for CKC development. Contains only information that
future sessions and subagents require but cannot derive from SPEC.md, CLAUDE.md,
the codebase, or git history.

Prune entries when they become obsolete or derivable from code.

## Decisions

- [2026-05-27] License: Apache-2.0 WITH LLVM-exception. LLVM exception chosen
  because CKC is a compiler; compiled output redistributable without attribution.
- [2026-05-27] Rust edition 2024 (resolver v3). Workspace-level shared deps.
- [2026-05-27] Task runner: `just`. Tool versions: `mise.toml` + `rust-toolchain.toml`.
- [2026-05-27] Python package management: `uv` with workspace pyproject.toml.
- [2026-05-27] Memory system lives in `.agent/` — separate from `.claude/` (Claude
  Code settings) to avoid coupling to a specific agent tool.

## Lessons

(empty — first session)

## Environment

- Host has: Rust 1.95, Python 3.13, Node 20, Lean (elan), Clingo, just, mise,
  uv, pnpm. Missing: Z3 (install in Phase 0), bun, cvc5, Souffle.
- Git credentials: Emir Turkes <eturkes@bu.edu> via global gitconfig.
- Context limit: 200K tokens. Prefer subagents for large exploration tasks.

## Conventions

- Commit messages: imperative mood, optimized for LLM parsing.
- All files optimized for machine readability over human readability.
- `.gitkeep` marks directories that must exist but have no tracked content yet.
- Runs directory (`runs/`) is gitignored except `.gitkeep`; artifacts are
  content-addressed.

## Open Questions

- [2026-05-27] SPEC Section 5.1 specifies DuckDB for deterministic local joins.
  Evaluate whether DuckDB Rust bindings (duckdb-rs) or DuckDB WASM is more
  appropriate for the content-addressed store.
- [2026-05-27] Surface syntax (SPEC 5.3): design the prefix AST grammar during
  Phase 0 or defer to Phase 3? SPEC says "add only as an authoring/intermediate
  target" — likely defer.
