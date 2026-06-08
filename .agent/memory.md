# Agent Memory

Every entry must provide value beyond what SPEC.md, CLAUDE.md, the codebase,
git history, config files, tool `--version` output, and the runtime
environment already provide. Exception: high-value reminders that are
technically derivable but easily forgotten under token pressure.

## Project history

- [2026-06-07] Branch restructure: `archive/phase0-research-kernel` holds the
  complete Phase-0 implementation of the previous 864-line SPEC (13 reviewed
  top-level tasks: schema v0, NF, CAS store, fixtures, terminology/e-graph,
  retrieval, compiler targets, certificates, conflict detection, CLI,
  report/UI, replay) plus its full agent memory, including a known-issues
  backlog and Phase-0-specific lessons pruned from this file. Current `main`
  is an orphan root developing the reworked M0 SPEC; the two histories share
  no commits. Consult the archive for prior art — canonical-bytes serializer,
  golden-test harness (`golden_suite!`), CAS store layout,
  emitter/witness/certificate patterns — and for any pre-restructure question.

## Lessons

- [2026-05-27] Before committing, self-audit every written file (docs, comments,
  diagnostics) for token efficiency, positive (pink-elephant) framing, and
  redundancy: memory entries, roadmap annotations, and doc preambles are the
  highest-risk overlap with SPEC.md, CLAUDE.md, and config files — if a fact
  lives in a file, it belongs only there. The user expects this proactively.
- [2026-05-27] Verify tool operations against their actual output before
  reporting success; record any after-the-fact mistake here so future sessions
  avoid it.
- [2026-05-28] Work-item sizing directive (user): size each work item so a
  fresh agent completes AND commits it within one context window with margin;
  if it would need compaction, split it. One conceptual deliverable + one gate
  per item; prefer more, smaller items. SPEC §11.3 rows are pre-sized to this
  rule; the split fallback is operationalized in the `/session-prompt` command
  (`.claude/commands/session-prompt.md`) → "Splitting".
- [2026-05-28] Non-interactive `Bash` tool invocations do NOT source
  `~/.bashrc`. Any tool needed across Bash calls must live in the base PATH
  (which includes `~/.cargo/bin`, `~/.local/bin`, `~/.local/share/pnpm/bin`)
  or be invoked by absolute path. Appending PATH exports to `~/.bashrc` is
  invisible per-command.
- [2026-05-28] Fresh-container toolchain drift: pinning `rust-toolchain.toml`
  to channel `stable` (the Phase-0 choice) lets a new container's rustup
  resolve a newer stable than committed code was formatted/linted under, so
  fmt-check/clippy gates can fail on otherwise-correct code while
  `cargo test --workspace` passes. Treat such failures as drift, not
  regressions: `cargo fmt --all` + manual clippy fixes (then commit), or pin a
  specific version. The unit that creates the workspace decides the pin;
  verify a fresh environment via `cargo test --workspace` first.
- [2026-05-29] The Claude `rust-analyzer-lsp` plugin requires the real
  `rust-analyzer` binary on PATH; `~/.cargo/bin/rust-analyzer` is a rustup
  multiplexer symlink that exists even when the component is not installed,
  and invoking it without the component yields `Unknown binary 'rust-analyzer'
  in official toolchain`. The Claude LSP host swallows this and surfaces only
  a generic `Tool result missing due to internal error` after a long hang.
  Fix: `rustup component add rust-analyzer`. Reproduce-detect cheaply by
  running `rust-analyzer --version` before debugging the LSP host.
- [2026-05-29] Project-local LSP marketplace lives at
  `.claude/marketplace/.claude-plugin/marketplace.json` and is wired in via
  project-scope `.claude/settings.json` (relative path,
  `.claude/marketplace`). The Claude LSP host caches plugin discovery at
  session start, so any new `*-lsp@ckc-lsps` plugin requires a Claude Code
  restart before `LSP` calls find it. Use `claude plugin list` to confirm
  enablement state cheaply without restarting. `claude plugin marketplace
  add … --scope project` + `claude plugin install <name> --scope project`
  is the supported automation path. Per-LSP install steps and gotchas live
  in `.claude/marketplace/plugins/<name>-lsp/README.md`.
- [2026-05-29] `bgcmd` (single-file bash script from
  github.com/izabera/bgcmd) is installed at `~/.local/bin/bgcmd` for driving
  interactive REPLs across separate `Bash` tool calls (solvers like
  z3/cvc5/clingo in interactive mode, lean REPL, duckdb, psql, gdb/rr, python).
  Required env: `BGCMDPROMPT` must equal the REPL's exact prompt string;
  `BGCMDDIR` (defaults `$HOME/.bgcmd`) holds the in/out fifos + pid file.
  Pattern: `BGCMDDIR=path BGCMDPROMPT='>>> ' bgcmd START python3 -i -q` then
  `BGCMDDIR=path BGCMDPROMPT='>>> ' bgcmd '<line>'` per turn; the prompt env
  vars must be re-exported on each call because non-interactive `Bash` does not
  preserve shell state. Per-REPL wrapper scripts (README pattern) eliminate
  this repetition when used heavily. To run multiple concurrent REPLs, give
  each a distinct `BGCMDDIR`. REPLs that suppress the prompt under non-TTY
  stdout require a force-interactive flag (e.g. python `-i`, bash
  `--noediting -i`). No pty is spawned, so ANSI escapes do not contaminate
  output. Cleanup: `kill "$(cat $BGCMDDIR/pid)"` then `rm -rf $BGCMDDIR`.
- [2026-05-29] Test-suite KISS gatekeeper directive (user, review pass):
  golden bytes + proptests are the durable assets; per-type unit tests should
  be one roundtrip + one optional-field-omission max. Skip these categories
  unless a specific behavior actually motivates them: (1) canonical-stability
  self-equality `assert_eq!(to_canonical_bytes(&x), to_canonical_bytes(&x))`
  — trivially true once serializer is deterministic; (2)
  distinct-types-distinct-hashes loops — assumes SHA-256 non-collision;
  (3) cross-type referential-consistency tests over hardcoded fixtures —
  fixtures are self-consistent with themselves by construction; (4)
  serde-behavior tests (`invalid_variant_rejects_deserialization`) — tests
  the library, not CKC; (5) "empty arrays are valid" smokes — covered by
  serde derive. Cross-bundle reference invariants (catching fixture
  programmer error before regen) are legitimate and should be kept. Gate
  phrasing like "round-trip serde tests for every type" means "for each type
  that has roundtrip", not "produce N×3 mechanical assertions per type".
  Apply this when planning and when implementing.
- [2026-05-29] LSP-first reflex for code navigation (user feedback: LSP is
  under-used). When a session needs to find symbols, callers, definitions,
  types, or call graphs in a file whose extension is covered by an installed
  `*-lsp` plugin, default to the `LSP` tool over `grep` and full-file `Read`.
  The Claude `LSP` tool exposes navigation/structure operations only — no
  diagnostics, no completion: `goToDefinition`, `findReferences`, `hover`,
  `documentSymbol`, `workspaceSymbol`, `goToImplementation`,
  `prepareCallHierarchy`, `incomingCalls`, `outgoingCalls`. Map to the grep
  patterns LLM sessions reach for first:
  * `workspaceSymbol` replaces multi-dir `grep` for "where is `Foo`
    defined" — name-only, no position needed.
  * `documentSymbol` replaces `grep '^\(fn\|struct\|impl\|trait\)'` for
    enumerating items in a file — path-only, no position needed.
  * `findReferences` replaces `grep '\bfoo\b'` for callers/usages —
    semantic, cross-file, immune to identifier collisions in unrelated
    scopes.
  * `goToDefinition` + `hover` replace `Read`ing a possibly-large defining
    file when only the signature or jump target is needed.
  * `incomingCalls`/`outgoingCalls` give a real call graph with no grep
    equivalent — run before refactoring a function to size blast radius.
  Position-based ops (`hover`, `findReferences`, `goToDefinition`,
  `goToImplementation`, `prepareCallHierarchy`) need 1-based `line` and
  `character`. Use `Read` once to locate the identifier, then issue LSP
  in the next response (or in parallel with other tools once the position
  is known). For diagnostics keep using `cargo check`/`cargo clippy`/
  `cargo test` — the LSP value is navigation, not feedback. Covered
  extensions today (cross-check `claude plugin list` if uncertain): `.rs`,
  `.py`, `.json/.jsonc`, `.yaml/.yml`, `.md`, `.toml`, `.lean`, `.ttl/.nt`,
  `.xml/.xsd/.dmn/.bpmn`, `.als`, `.pl/.pro`,
  `.smt2/.cnf/.icnf/.p/.tptp/.zf`, `.dl`, `.egg`, `.html`,
  `.css/.scss/.less`, `.ts/.js`, `.svelte`. First call per server (notably
  rust-analyzer) pays one indexing cost; treat as per-session warmup, not
  a reason to fall back to grep. Fire LSP in parallel with other
  independent tools in the same response.
- [2026-05-29] When sed-deleting a multi-line comment block, also delete the
  continuation indent line that follows it (`        //         …`) — it
  becomes an orphaned half-sentence otherwise. Verify with grep before
  committing.
- [2026-05-29] rust-analyzer `unlinked-file` on a freshly-added `tests/*.rs`
  integration file is a stale-metadata false positive, not a wiring bug. Each
  `tests/*.rs` is its own auto-discovered cargo target (with default
  `autotests` and no explicit `[[test]]`), so RA links it on its next workspace
  metadata reload; the file is already a real target the moment cargo sees it.
  Authoritative check: `cargo test --test <name>` compiling+running the file
  proves linkage. Keep `unlinked-file` enabled — the lint's own suggested fix
  (adding it to `rust-analyzer.diagnostics.disabled`) would also suppress
  detection of genuinely orphaned `src/*.rs` (missing `mod`) workspace-wide.
  Expect this on every unit that adds a new integration test; ignore it.
- [2026-05-29] serde_json float traps (two distinct, both verified empirically
  in Phase 0): (a) parse/format ULP asymmetry — the f64 deserializer
  (lexical-core, Eisel-Lemire) and serializer (ryu) disagree at ULP
  boundaries, e.g. bits `4033367680000000` serializes to
  `"19.212745666503906"` which re-parses to the neighbouring f64
  `403336767fffffff`; round-trip can fail by 1 ULP and is unfixable at the
  serializer. (b) integer-valued floats — an RFC 8785-style canonical
  serializer renders 38.0 as `"38"` (ECMAScript form), which serde_json
  re-reads as the INTEGER 38, so equality through an UNTYPED
  `serde_json::Value` breaks while a TYPED f64 field stays safe (`"38"`
  parses to 38.0_f64). Rules: compare via content hash over canonical bytes
  or structurally, never by full-struct equality through untyped Values that
  may hold floats; typed-struct round-trip equality is fine when no untyped
  Value can hold an integer-valued float. SPEC §1.3 rational normalization
  exists partly to dodge this class.
- [2026-05-30] Cargo integration tests (`crates/<c>/tests/*.rs`) can `use` the
  crate's normal `[dependencies]` directly, not just `[dev-dependencies]` —
  no re-export shim needed to reach e.g. core canonicalization helpers from a
  sibling-crate-consuming test.
- [2026-05-30] Verifier-binary inventory (installed during Phase 0, still
  present in this sandbox; M0's kernel checker is internal Rust, so these
  serve §12/GATED external-backend work and ad-hoc artifact sanity checks):
  z3, clingo, souffle at `/usr/bin` — clingo ships in the apt package
  **`gringo`** (`apt-cache policy clingo` reports no candidate) and exits
  10/30 on SAT, so assert on the `SATISFIABLE` stdout token and accept exit
  ∈ {10,30}; cvc5 at `~/.local/bin` (proof flags `--produce-proofs
  --dump-proofs [--proof-format-mode=alethe]`); lean/lake at `~/.local/bin`
  via elan (a single import-free file kernel-checks via bare `lean <file>`;
  only `import Mathlib`/cross-file deps need a lake project); alloy jar at
  `~/.local/share/alloy/alloy.jar` (`java -jar … exec <f.als>`; for a
  `check`, UNSAT means the assertion holds; prints to stderr, stdout stays
  empty) and tla2tools.jar at `~/.local/share/tla` (`java -cp …
  tla2sany.SANY <f.tla>`). souffle and alloy write outputs relative to CWD —
  run both from a throwaway dir. All resolvable on the non-interactive base
  PATH.
- [2026-05-31] Review-workflow tree-mutation hygiene (reviews via Workflow
  tool): `agentType: 'Explore'` subagents are read-only by EDIT permission but
  still hold `Bash`, so a verifier proving "the fix keeps gates green" can
  write files and invalidate `Cargo.lock`. Always `git status` after a review
  workflow returns and reconcile every stray path before staging: discard
  experiments you reject, re-derive deliberately the ones you accept. To
  prevent mutation entirely, run mutation-capable review agents under
  `isolation: 'worktree'`.
- [2026-05-31] Subagent model enforcement (user feedback: spawned subagents
  kept running on Haiku, violating the CLAUDE.md "always latest+largest Opus"
  directive). Durable lever is the env var `CLAUDE_CODE_SUBAGENT_MODEL` in the
  settings.json `env` block: it forces the model for ALL subagents + agent
  teams and OUTRANKS the per-call `model` param, the agent-definition
  frontmatter, AND parent inheritance (value `inherit` disables it). Set to
  `"opus"` in project `.claude/settings.json` — the alias tracks the latest
  Opus, no version drift. Root cause it closes: with no `.claude/agents/` defs
  and no env override, each built-in `subagent_type` uses its OWN bundled
  default — `Explore` and `claude-code-guide` are pinned to Haiku, while
  `general-purpose` and `Plan` inherit the parent — so an omitted param
  silently ran the search/guide agents on Haiku. No settings.json KEY sets a
  default subagent model; it must be this env var. The env var is read at
  startup, so in any session started before it was set, pass `model: "opus"`
  explicitly on every `Agent`/`Workflow agent()` dispatch. Precedence +
  built-in defaults verified against code.claude.com/docs/en/{sub-agents,
  model-config}.
- [2026-05-31] Reasoning-effort enforcement (companion to the subagent-model
  lever above; user: "ensure we are using max"). Opus 4.8 effort tiers are
  low/medium/high/xhigh/max, DEFAULT `high`, and switching INTO Opus 4.8
  resets effort to that default — so persistence matters. `max` = deepest
  tier, no token-spend cap; it supersedes `MAX_THINKING_TOKENS` and the
  `ultrathink` keyword on adaptive models (neither raises it further — set
  neither). PERSISTENCE NUANCE: `max` is session-only via the `--effort`
  flag, `/effort`, and the `effortLevel` settings KEY (that key accepts only
  low/medium/high/xhigh) — it survives across sessions ONLY through env var
  `CLAUDE_CODE_EFFORT_LEVEL`, which also outranks the key, `/effort`, and
  per-agent `effort` frontmatter. Set `"max"` in project
  `.claude/settings.json` `env`. PROPAGATION: no dedicated subagent-effort
  var exists; subagent/teammate `effort` frontmatter defaults to
  inherit-from-session, so the session env var drives them to max too (our
  `.claude/agents/` is empty → nothing overrides down). Binary-confirmed
  (2.1.158): `CLAUDE_CODE_EFFORT_LEVEL` is the READ control; the live
  `CLAUDE_EFFORT` var is an EXPORT for hooks/`${CLAUDE_EFFORT}` templates
  (not an input). Applies after restart; fast mode (/fast) is orthogonal and
  does not lower effort.
- [2026-06-01] Hash-chained artifact cascade: editing any generated artifact's
  text moves its content hash, which flows into every artifact that embeds it
  (certificates, manifests, reports, graphs) and renames any
  content-addressed FILENAME derived from it — a rename leaves an orphan
  file; `git rm` it, since regen writes the new name, never the deletion.
  Before editing, `git grep <old-hash>` to bound the radius; regenerate via
  the gates' regen paths; confirm zero stale-hash refs after. Phase 0 hit a
  14-file cascade from one prose reframe; the M0 design is even more
  hash-chained, so expect this on any generated-content edit.
- [2026-06-07] Fixture strings that must byte-match SPEC.md (all Appendix A
  slices): extract programmatically from SPEC.md (python regex → generated
  Rust const block pasted into the test) instead of retyping. A.1 mixes
  visually identical fullwidth/ASCII variants (，U+FF0C vs , — and Ｘ vs X)
  that retyping silently corrupts; assert expected count and charset
  inventory after extraction. First use: M0.0.1 `t_unicode_idempotency.rs`.
- [2026-06-07] New `E` enums / `S` records route through ckc-core's
  `bare_enum!` / `canonical_record!` (canon.rs declaration-macros section)
  instead of hand-written Canonical/Deserialize impls; bare-enum `Ord` is
  opt-in per invocation (`#[derive(PartialOrd, Ord)]`, needed for BTreeSet
  elements).
- [2026-06-07] Registry-surface sizing constants (calibrate splits of
  registry-adjacent units, e.g. M0.0.4 schema equivalence walks the same
  surfaces; re-grep after SPEC edits): 229 S-decls spec-wide, 75 §3.1
  inventory roots, 367 collection fields, 431 hash-named fields, 64 Text
  fields in S-decl lines (2026-06-08 re-grep, post-.5.2.3.1; counts are
  `grep '^S '`-scoped: S-decl lines; `Set[|List[|Map[`;
  `_hash|_hashes|_digest|_digests` before `:`; `Text<`).
  Split heuristic (m0.0.3.4 rebalances, 3→5 then
  user-directed 5→7, both pre-implementation): a unit that both implements
  a nontrivial algorithm AND authors a per-inventory-row table is two
  units; a checker family that authors a per-path/per-name table or
  expects spec-defect fallout (hash-field conventions, producer mapping)
  is its OWN unit; only table-free set-comparison families pair up.
  Calibration points: authored-table units ran 87% (.4.2), fallout-bearing
  checker ran 82% (.3.3). Corollary (m0.0.3.4.5 split 1→2, user-directed
  pre-implementation): a unit stacking BOTH patterns — authored table AND
  spec-defect fallout — over a large surface (hash-named walk: 260 paths,
  171 distinct terminal names) exceeds the window; split along the
  SPEC-edit boundary (classify + author table, defects → Unresolved rows /
  burn down Unresolved with SPEC corrections + wiring) so the
  unpredictable-fallout half starts from a committed, bounded list. Second
  corollary (m0.0.3.4.5.1 split 1→2, user-directed pre-implementation): a
  per-name judgment sweep alone (171 distinct terminal names, each needing
  its S-decl context) still exceeds the window even with suffix defaults
  carrying the bulk and SPEC untouched; split at the lexicographic
  terminal-name median (a-l 85 names/123 paths + classifier mechanics /
  m-z 86/137 + finalized count assertions) so each name is judged exactly
  once in exactly one half. Third corollary (m0.0.3.4.5.2 split 1→3,
  user-directed pre-implementation): a defect burn-down (42 Unresolved
  names/49 paths, each needing S-decl context + a SPEC correction) splits
  by SPEC-section family, not lexicographic median — names in one section
  share one fix and one context read (§1.x/§4.x 14 / §6.2-§9.1
  digest-semantics 17 / §6.4 pre-acceptance 11 + wiring on the final
  sub-unit), so no fix family straddles sessions with divergent styles.
  Fourth corollary (m0.0.3.4.5.2.x re-split 3→7, user-directed
  pre-implementation): section-family clusters at 14/17/11 names still
  exceed the window; budget ≈5 heterogeneous fixes per session (one S-decl
  context + one SPEC correction each) — names sharing one fix or one
  section read batch as one unit (8-name §6.2 grammar family, 11-name §6.4
  shared-convention family), and checker wiring/finalization stays its own
  unit. Assign names to clusters by grepped S-decl site, not defect
  framing: canonical_bytes_hash sat in the §1.x cluster but lives on
  ProofNode/ProofDAG (§7.2) beside checker_hashes; dual-site names
  (termination_argument_hash §4.3+§7.1, input_hash §4.4+§9.1) resolve once
  in the primary-section unit with the second site's context loaded.
  Judgment sweeps split lexicographically; burn-downs split by section at
  ≈5 heterogeneous fixes per session. Fifth corollary (m0.0.4 split 1→4,
  user-directed pre-implementation): a multi-artifact equivalence unit
  splits along artifact-layer boundaries — model+spec-derivation /
  second-source agreement / generated-format / rewiring+checker+gate —
  keeping the fallout-bearing rewiring+gate layer solo. Sixth corollary
  (m0.0.5 split 1→4, user-directed pre-implementation): a multi-schema
  runtime/replay foundation row (generic envelope + 7 manifest records + 2
  ops + boundary algorithm + new ckc-store crate + gate) splits along
  data-substrate layers — generic-envelope+store foundation /
  runtime-manifest cluster+ValidateRuntimeManifests / replay-manifest
  cluster / boundary-algorithm+gate — isolating the manual-Canonical generic
  envelope (first non-canonical_record! type, ObjectEmitter-hand-written) as
  the foundation unit and keeping the algorithm+gate solo; mechanical
  canonical_record! clusters stay light (3-4 records/unit) because each
  record still costs a descriptor-agreement reconciliation against its
  spec-derived S-decl. Seventh corollary (m0.0.6 split 1→3, user-directed
  pre-implementation): a CLI/contract row whose gate checks a runtime surface
  against a SPEC table an earlier unit already extracts (§11.1 `command_table`
  from m0.0.3.2-.3) reuses that extraction — author a runtime model+parser the
  gate proves bijects with the table, never a re-authored copy; split by
  deliverable layer (surface-model+parser / diagnostic-writer+repo-layout /
  contract-checker+gate), pairing the two light table-free deliverables and
  keeping the fallout-bearing gate solo. CLI command/operation types are
  CLI-internal (absent from §3.1 inventory) → no descriptor/registry
  registration, agreement gates untouched (unlike m0.0.5's schema rows).
  Eighth corollary (m0.1.2 split 1→5, user-directed pre-implementation): a
  schema unit whose deliverable also authors a FULL-corpus fixture (A.1
  U1-U27 SourceGraph + table/caption/footnote/crossref substructure) is
  ≈5 units — the fixture alone is ≥2: split leaf-content (per-unit
  spans/anchors/leaf nodes, an authored ~85% table) from
  structure-wiring+assembly (container nodes + all edge kinds + the
  SourceGraph value), and keep the predicate+gate unit solo; schemas split
  3-4 records/unit as usual. Applies to every later fixture-extending unit
  (m0.1.3 closure, m0.2 mech-obs grow this same SourceGraph). Two durable
  design facts surfaced: (a) a §1.5 sort key reading a downstream-crate
  record (source_order_key over ckc-source SourceSpan/SourceAnchor) lives in
  ckc-core as a fn over a ckc-core view struct (SourceOrderView, optional
  fields, missing = type canonical minimum) with the downstream crate impl'ing
  the view — ckc-core cannot dep ckc-source; (b) the M0.1 enum-placement rule
  resolves source enums case-by-case, not by name family: SourceNodeKind/
  SourceEdgeKind are §2 vocabulary → ckc-core beside outcome.rs, but Lang is
  §4.2-local → ckc-source.
- [2026-06-08] A missing/renamed `bare_enum!` variant surfaces its first
  rustc/RA diagnostic (E0599) at ckc-core canon.rs — the macro-definition
  site — not the real use site. Grep the variant name across crates before
  debugging ckc-core.
- [2026-06-07] `.claude/settings.json` carries the CLAUDE.md-mandated
  `permissions.deny` `Read()` list (VCS/build/dep internals, lockfiles,
  LICENSE). Verified live: permission rules hot-reload mid-session (no
  restart, unlike `env`). Bash (`cat`/`git show`) bypasses `Read()` denies —
  the escape hatch for rare legitimate reads; prefer `cargo tree` for
  dependency questions. Keep run/cert/golden/fixture trees readable as
  debugging surfaces; extend the deny list when new generated/vendored trees
  appear.

- [2026-06-08] The recurring pre-implementation split directive ("the
  upcoming unit is too large") names its target by the too-large property,
  not by roadmap position: first verify the first unchecked unit's
  remaining surface against code state (gates green, prerequisites
  complete, NN% calibration); when it verifiably fits, the target is the
  first unchecked unit that cannot fit — the next bare §11.3 row. Applied
  2026-06-08: .2.3.2 verified wiring-only (burn-down 42/42 committed) →
  target M0.0.4; 2026-06-08(2): .4.7 verified bounded check+gate (fits,
  already detailed) and .4.1-.4 sized last session → target M0.0.5; 2026-06-08(3): chain
  .4.7/.4.1-4/.5.1-4 all detailed-and-fitting → target M0.0.6 (next bare
  row, split 1→3); 2026-06-08(4): chain .4.7/.4.1-4/.5.1-4/.6.1-3 all
  detailed-and-fitting → target M0.1.1 (first bare M0.1 row, split 1→2
  along schema-layer / Residual+projection+gate); 2026-06-08(5): chain
  extends through .1.1.1-2 → target M0.1.2 (next bare row, split 1→5); 2026-06-08(6): chain extends
  through .1.2.1-5 → target M0.1.3 (next bare row, split 1→6: §8.7
  diagnostic-family / region-schemas / HandleBoundOverflow / closure-engine
  +contains / relational-closure+residuals / cert-admissibility+fixture+gate
  — a deferred cross-cutting op (HandleBoundOverflow, first consumer here)
  and its prerequisite §8.7 schema family each become their own sub-unit
  ahead of the closure that consumes them); 2026-06-08(7): chain extends
  through .1.3.1-6 → target M0.2.1 (next bare row, first M0.2 §11.3 row,
  split 1→3: ckc-observe-crate+Authority+extraction/analyzer-manifests /
  lexicon+payload+MechObsKind / fixtures+authority-invariant+gate). New-crate
  creation pairs with the §2 Authority enum (first consumer → ckc-core) as the
  foundation sub-unit (M0.1.2.1 §2-enum+skeleton precedent); MechObsKind is
  §4.4 schema-local → ckc-observe; ExtractionManifest folds in per M0.1.2.4's
  defer; mechanical-schema 3-4-records/unit rule holds (5 records → schema unit
  splits 2+3, gate solo).
- [2026-06-08] From M0.1 on, implementing a §3.1-inventory schema in Rust
  needs NO build.rs/registry/checker edit: build_v0_registry, spec-derived
  descriptors, hash-classification, and producer-mapping all parse SPEC.md,
  so they already cover every inventory row (whole-spec coverage since
  M0.0.3/.4). The only rust-side cost per new type is its
  CanonicalType/descriptor() emission appended to the crate's descriptor
  registry + keeping ckc-schema descriptor_agreement green (rust-emitted ==
  spec-derived; fix the rust side, SPEC is authority). New §2-vocabulary
  enums land in ckc-core beside outcome.rs (first-consumer rule, e.g.
  SourceClass at SourceEdition); §-section-local enums/records land in the
  consuming crate; cross-crate-shared diagnostics (Residual family) land in
  ckc-core. (M0.0.4/.5/.6 not yet built — planning inference from their
  roadmap lines; confirm the agreement API when M0.0.4 lands.)
- [2026-06-08] A SPEC.md prose-only edit can break the implemented
  spec-parsing checkers — run `cargo test --workspace` after any SPEC.md edit.
  Two non-obvious couplings a token-efficiency compression pass tripped (both
  easily forgotten under token pressure): (a) the §6.2 builtin-definition
  parser (spec.rs) splits a co-defined line on the literal ` and ` separator
  (commas occur inside definition prose, so splitting on commas is impossible),
  so a global `and`->`,` substitution silently drops every builtin after the
  first and breaks the BuiltinName<->§6.2-definition bijection under
  T-Registry-Referential-Integrity; keep ` and ` joining co-defined builtins
  (`normalize_context and ctx_compatible: §8.1.`). (b) check.rs
  `check_duplicate_and_dangling_refs_reject` splices the real spec onto a
  synthetic section block, so that block's section number must stay above the
  spec's range (moved 13->97 when the spec gained a real §13; a real section
  colliding with it emits a spurious duplicate section_anchor). Standalone
  synthetic-spec tests that do not splice the real spec (build.rs, the §3.1
  mini in check.rs) tolerate any number.

## Mistakes

- [2026-05-27] `replace_all` is case-sensitive. A single replace_all pass can
  silently miss case variants. Always read the result or verify match count
  covers all intended occurrences.
- [2026-05-27] User gave a directive and the agent acknowledged it verbally
  but failed to persist it to memory. Any user directive meant for future
  sessions must be written to memory immediately, in the same turn.
- [2026-05-29] Missing CLI tool → install it in the same turn instead of
  reaching for a workaround. When `jq` was absent the agent fell back to ad-hoc
  `python3 -c` one-liners and only installed `jq` after the user interrupted,
  despite CLAUDE.md granting full permission to install/download anything.
  Treat "make full use of your capabilities and environment" as standing
  authorization: always provision the tool the moment it is missing. A standard
  system utility (jq and the like) is fine to install system-wide despite the
  prefer-project-local directive; distro/package-manager specifics now live in
  CLAUDE.md.
- [2026-06-07] Escape sequences authored through tool JSON args (`\t`,
  `\n`, …) inside Write/Edit string parameters decode to the literal control
  character before landing in the file — the file then byte-differs from the
  intended backslash-escape text and the tool-result echo renders both forms
  identically. After writing any file that must contain literal
  backslash-escape text, verify with `grep -nP '\t'`/`cat -A` and patch via
  a python heredoc composing `chr(92)` explicitly.
- [2026-06-07] A session closed leaving untracked scratch
  (crates/ckc-schema/tests/tmp_dump.rs, a HashNamed-path dump probe). Run
  `git status` before the closing commit and reconcile every stray path —
  stage it, delete it, or gitignore it; scratch probes belong under /tmp.
- [2026-06-01] Hash-record recurrence (the generic "verify against actual
  output" lesson alone did not prevent it): a fabricated placeholder short
  hash went into a roadmap line + commit message instead of the real commit
  hash. Always paste from the actual `git log -1 --format=%h` output and
  re-read the committed line before closing. Second failure mode the same
  session: when `Bash` stdout rendered empty mid-turn the same git command was
  re-fired ~5×; results were merely batched, so re-issuing an identical read
  wastes a turn — route output to a temp file and `Read` it once instead of
  retrying.
