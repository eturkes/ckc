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
- [2026-06-09] RTK proxy mangles `git commit` with multiple `-m` flags whose
  values carry non-ASCII (`§`, em-dash `—`): args get dropped/split, leaving a
  bare space git reads as a pathspec ("pathspec ' ' did not match any file(s)"),
  so `git add` stages but the commit never lands — and RTK still prints
  "ok N files changed" from the add step, masking the failure. Always commit
  such messages from a file: write the message, then `git commit -F <path>` (one
  flag + one path survives the rewrite), and `rm` the temp file after. Plain
  single-`-m` ASCII commits are unaffected.
- [2026-06-09] Enabling a Serena LSP language that isn't set up yet. Symptom: a
  Serena symbolic tool (`get_symbols_overview`/`find_symbol`/reference lookups)
  errors `Active languages: ['bash']`. Fix: add the language to
  `.serena/project.yml` `languages:` (list it first = default/fallback LS). The
  MCP server reads `project.yml` only at startup (`--project-from-cwd`), so the
  edit applies only after a Serena reconnect, which you cannot trigger from a
  tool — ask the user to `/mcp` reconnect serena (or restart Claude), then
  verify with a symbol/reference call (first call may lag during indexing).
  Confirm the LS binary exists first; for rust, Serena launches the rustup
  toolchain's `rust-analyzer` directly, so `rustup update` already keeps it
  current (no `~/.local/bin/upgrade` line). `.serena/project.yml` is git-tracked
  so the set persists across fresh checkouts. Apply this whenever a needed
  Serena LSP isn't enabled.

- [2026-06-10] No gate enforces `rustfmt` or `clippy` — unit gates are
  `cargo test`, which ignores both — so `core-ids` landed fmt-dirty and
  `core-strings` folded the one-time reformatting into its commit. Always run
  `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings` before
  staging a Rust unit, so drift and lints never accumulate for the `review` line
  to sweep.

- [2026-06-10] Serena `replace_symbol_body` on a Rust item spans its preceding
  doc comment AND outer attributes (`#[derive(...)]`): the body you supply
  replaces all of them, so omitting them deletes them. Dropped a
  `#[derive(Debug, Clone, PartialEq, Eq)]` off an enum this way and broke the
  build (no `Debug` for `.unwrap()` in tests). Always include the leading `///`
  lines and every `#[...]` attribute in the replacement body, or edit the inner
  variant/field region with `replace_content` instead. Recurs with the
  derive-heavy enums and envelope/IR structs still ahead.

- [2026-06-10] M1 sizing recalibration (the core-canon-hash overrun plus a
  backlog sweep). The spec02 "canonical JSON = three units" anchor undercounted:
  its third unit (unions + strict-reading + hash, 86% in spec02) was projected to
  overrun a 200K window here, so canonical JSON is FIVE units — writer /
  collections / unions / strict-reader / hash, each ~62-66%. The seed batch
  likewise under-decomposed every unit that stacks a crate-foundation (~81%
  alone), a five-layer/recursive type family, or an (algorithm + second authored
  artifact) pair: core-ir (~120%+: IRBundle = five field-rich layers), smt-emit
  (>100%: new crate + CompiledArtifact + SMT emission + assertion_map), and
  corecli (~115-130%: new crate + full-surface schema export + six ops) all
  projected over one window and were pre-split into `<id>.z` lines. Rule: split
  such stacks BEFORE scheduling; reserve JIT-splitting for fine calibration of
  hot-but-fitting single-deliverable units (~70-85%, one gate: core-enums-
  envelope, core-plans, core-registry-types, registry-validate, registry-seed).

## Mistakes

(empty — populated as sessions record after-the-fact corrections.)
