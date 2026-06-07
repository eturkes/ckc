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
  rule; the split fallback is operationalized in `.agent/prompt.md` →
  "Splitting".
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
  surfaces; re-grep after SPEC edits): 227 S-decls spec-wide, ~75 §3.1
  inventory roots, 366 collection fields, 459 hash-named fields, 61 Text
  fields in S-decl lines. Split heuristic (m0.0.3.4 rebalances, 3→5 then
  user-directed 5→7, both pre-implementation): a unit that both implements
  a nontrivial algorithm AND authors a per-inventory-row table is two
  units; a checker family that authors a per-path/per-name table or
  expects spec-defect fallout (hash-field conventions, producer mapping)
  is its OWN unit; only table-free set-comparison families pair up.
  Calibration points: authored-table units ran 87% (.4.2), fallout-bearing
  checker ran 82% (.3.3).
- [2026-06-07] `.claude/settings.json` carries the CLAUDE.md-mandated
  `permissions.deny` `Read()` list (VCS/build/dep internals, lockfiles,
  LICENSE). Verified live: permission rules hot-reload mid-session (no
  restart, unlike `env`). Bash (`cat`/`git show`) bypasses `Read()` denies —
  the escape hatch for rare legitimate reads; prefer `cargo tree` for
  dependency questions. Keep run/cert/golden/fixture trees readable as
  debugging surfaces; extend the deny list when new generated/vendored trees
  appear.

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
- [2026-06-01] Hash-record recurrence (the generic "verify against actual
  output" lesson alone did not prevent it): a fabricated placeholder short
  hash went into a roadmap line + commit message instead of the real commit
  hash. Always paste from the actual `git log -1 --format=%h` output and
  re-read the committed line before closing. Second failure mode the same
  session: when `Bash` stdout rendered empty mid-turn the same git command was
  re-fired ~5×; results were merely batched, so re-issuing an identical read
  wastes a turn — route output to a temp file and `Read` it once instead of
  retrying.
