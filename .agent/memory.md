# Agent Memory

Every entry must provide value beyond what SPEC.md, CLAUDE.md, the codebase,
git history, config files, tool `--version` output, and the runtime
environment already provide. Exception: high-value reminders that are
technically derivable but easily forgotten under token pressure.

## Lessons

- [2026-05-27] Every piece of natural language in the project (docs, comments,
  diagnostics) must use positive constraint framing per the pink elephant
  principle in CLAUDE.md. Audit before committing.
- [2026-05-27] Memory entries, roadmap descriptions, and doc preambles are the
  highest-risk locations for redundancy with SPEC.md and config files. Default
  assumption: if a fact lives in a file, it belongs only in that file.
- [2026-05-27] Spend freely on reasoning and iteration within a session, but
  all content written to files must be token-efficient for future LLM readers.
  Self-audit this balance before committing; the user expects it done
  proactively.
- [2026-05-27] When a mistake is caught after the fact, record it here so
  future sessions avoid repeating it. Verify tool operations against their
  actual output before reporting success.
- [2026-05-28] Subtask sizing directive (user): size each roadmap subtask so a
  fresh agent completes AND commits it within one context window with margin; if
  it would need compaction, split it. One conceptual deliverable + one gate per
  subtask; prefer more, smaller subtasks. Operationalized in `.agent/prompt.md`
  → "Subtask sizing". Apply when planning (Type A) and any re-planning.
- [2026-05-28] CAS determinism pitfall: `StoreManifest` entries carry
  `stored_at_epoch` (file mtime), so raw manifest canonical bytes differ across
  runs/machines. Assert cross-run determinism on per-artifact
  `content_hash`/`envelope_hash` (or manifest entries minus timestamp), as in
  ckc-store `cas_manifest_hash_is_stable` and `all_fixtures_have_deterministic_hashes`.
  Avoid asserting raw manifest-byte equality across independent runs.
- [2026-05-28] Fresh-container toolchain drift: `rust-toolchain.toml` pins
  channel `stable` (not a specific version), so a new container's rustup may
  resolve a newer stable than committed code was formatted/linted under.
  Result: `scripts/ci.sh` may fail at the fmt-check or clippy stage on
  otherwise-correct code while `cargo test --workspace` passes. Treat such
  failures as drift, not regressions; remediation is `cargo fmt --all` +
  manual clippy fixes (then commit), or pin the toolchain to a specific
  version in `rust-toolchain.toml`. Verify the toolchain via
  `cargo test --workspace` first when bringing up a fresh environment.
- [2026-05-28] Non-interactive `Bash` tool invocations do NOT source
  `~/.bashrc`. Any tool needed across Bash calls must live in the base PATH
  (which includes `~/.cargo/bin`, `~/.local/bin`, `~/.local/share/pnpm/bin`)
  or be invoked by absolute path. Appending PATH exports to `~/.bashrc` is
  invisible per-command.
- [2026-05-29] The Claude `rust-analyzer-lsp` plugin requires the real
  `rust-analyzer` binary on PATH; `~/.cargo/bin/rust-analyzer` is a rustup
  multiplexer symlink that exists even when the component is not installed,
  and invoking it without the component yields `Unknown binary 'rust-analyzer'
  in official toolchain`. The Claude LSP host swallows this and surfaces only
  a generic `Tool result missing due to internal error` after a long hang.
  Fix: `rustup component add rust-analyzer`. Reproduce-detect cheaply by
  running `rust-analyzer --version` before debugging the LSP host. Note this
  supersedes the stance in commit 2fa9393, which deferred the install when
  Claude's diagnostic loop relied only on `cargo check`/`clippy`/`test`.
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
- [2026-05-29] No standalone LSP exists as of 2026-05 for these
  SPEC verification-target formats (audited at source):
  * TLA+ — `tlaplus/vscode-tlaplus` is a TS extension that shells out to
    `tla2tools.jar`; no LSP server component, no separate language-server
    repo under the `tlaplus` or `tlaplus-community` orgs.
  * ASP/Clingo — `CaptainUnbrauchbar/ASP-Language-Support` and
    `ffrankreiter/answer-set-programming-language-support` are JS extensions
    that call the `clingo` binary directly; no LSP.
  * Categorical CQL — `CategoricalData/CQL` is a Java Swing IDE only;
    `cqframework/cql-language-server` targets HL7 Clinical Quality Language
    (FHIR), unrelated to Categorical CQL.
  Revisit per-format before the corresponding SPEC phase lands.

- [2026-05-29] Test-suite KISS gatekeeper directive (user, review pass):
  golden bytes + proptests are the durable assets; per-type unit tests should
  be one roundtrip + one optional-field-omission max. Skip these categories
  unless a specific behavior actually motivates them: (1) canonical-stability
  self-equality `assert_eq!(to_canonical_bytes(&x), to_canonical_bytes(&x))`
  — trivially true once serializer is deterministic; (2)
  distinct-types-distinct-hashes loops — assumes SHA-256 non-collision;
  (3) cross-type referential-consistency tests over hardcoded fixtures
  (`claim.rule_ids.contains(&rule.rule_id)`) — fixtures are self-consistent
  with themselves by construction; (4) serde-behavior tests
  (`invalid_variant_rejects_deserialization`) — tests the library, not CKC;
  (5) "empty arrays are valid" smokes — covered by serde derive. Cross-bundle
  reference invariants (catching fixture programmer error before regen) are
  legitimate and should be kept. Roadmap-gate phrasing like "round-trip serde
  tests for every type" means "for each type that has roundtrip", not
  "produce N×3 mechanical assertions per type". Apply this when planning
  and when implementing.
- [2026-05-29] Repo-layout scaffolding policy (this session): empty crates
  and `.gitkeep`-only directories in the SPEC §19 target layout are added
  by the roadmap phase that needs them, not pre-scaffolded. Phase 0 keeps
  only `crates/{ckc-cli,ckc-core,ckc-store,ckc-retrieve,ckc-term}/`,
  `examples/research_kernel/`, `schemas/`, `runs/`. SPEC.md §19 remains the
  target-layout source; current state lags intentionally.
- [2026-05-29] When sed-deleting a multi-line comment block, also delete the
  continuation indent line that follows it (`        //         …`) — it
  becomes an orphaned half-sentence otherwise. Verify with grep before
  committing.
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
  `cargo test` (or `scripts/ci.sh` for the full gate) — the LSP value is
  navigation, not feedback. Covered
  extensions today (cross-check `claude plugin list` if uncertain): `.rs`,
  `.py`, `.json/.jsonc`, `.yaml/.yml`, `.md`, `.toml`, `.lean`, `.ttl/.nt`,
  `.xml/.xsd/.dmn/.bpmn`, `.als`, `.pl/.pro`,
  `.smt2/.cnf/.icnf/.p/.tptp/.zf`, `.dl`, `.egg`, `.html`,
  `.css/.scss/.less`, `.svelte`. First call per server (notably
  rust-analyzer) pays one indexing cost; treat as per-session warmup, not
  a reason to fall back to grep. Fire LSP in parallel with other
  independent tools in the same response.

- [2026-05-29] serde_json f64 parse/format asymmetry: serde_json's f64
  deserializer (lexical-core, Eisel-Lemire) and serializer (ryu, shortest
  round-trip vs Rust std parser) disagree at ULP boundaries. Concretely,
  `19.212745666503906_f64` (bits `4033367680000000`) serializes to
  `"19.212745666503906"` via ryu, but `serde_json::from_str` re-parses that
  literal back to `19.212745666503903_f64` (bits `403336767fffffff`, the
  neighbouring f64). Round-trip fails by 1 ULP for some values. Rust's std
  `<str as FromStr>::parse::<f64>` does round-trip cleanly, so the asymmetry
  is purely between the two serde_json algorithms. Implication: do not write
  snapshot tests that assert `assert_eq!(live, serde_json::from_slice(disk))`
  on structs containing f64 fields — Vec equality will fire on values you
  cannot fix at the serializer. Workarounds: (a) compare via `content_hash`
  over canonical bytes in-memory only; (b) deserialize then compare
  structurally (query ids, span_id sequences, fingerprints) not by full
  struct equality; (c) quantize f64 to a fixed precision (e.g. `{:.9}`)
  before storage so values stay in the round-trippable interior. CKC took
  (b) for `crates/ckc-retrieve/tests/persistence.rs` Gate 3. Verified
  empirically in this session via a 5-line snippet against serde_json 1.x.
- [2026-05-29] Tantivy `TopDocs` tie-break is NOT cross-run-deterministic.
  Upstream docs say ties are broken by ascending doc id, which suggests
  insertion-order stability — but the default `Index::writer(budget)`
  uses multiple indexer threads, so doc ids assigned to specific spans can
  swap across independent index builds even when input order is byte-for-byte
  identical. Symptom: two independent `SparseIndex::build_from_spans(spans)`
  calls return tied hits in opposite orders (Phase-0 `q_vitals_temp`:
  `span_cell_r0c0` vs `span_cell_r1c0` swap between runs at score
  `5.094640254974365`). The original 0.7.2 sparse-index test suite only
  asserts top-k set membership and rank monotonicity, so it did not catch
  this; 0.7.6's `per_result_hashes_match_across_two_independent_runs`
  surfaced it. Fix applied in `crates/ckc-retrieve/src/sparse.rs::search`:
  post-collector stable sort by `(score DESC, span_id ASC)` then reassign
  ranks 1..N. This dominates whatever Tantivy does internally and costs
  nothing for non-tied results. Alternative fix
  (`index.writer_with_num_threads(1, budget)`) would force serial indexing
  and was not needed; keep it in mind if BM25 score values themselves ever
  drift across runs (none observed in Phase-0).

- [2026-05-29] Artifact-producer subtask sizing (calibration from the task-0.8
  re-plan after a context-exhaustion `git reset --hard`): 0.8.1/0.8.2 completed
  at 118K/102K — already past half — and the original 0.8.3 (emit + a
  per-artifact `CompiledTarget` canonical-bytes golden + regenerate, in ONE
  subtask) could not finish without compaction. The cost trap is the
  per-artifact big golden: it embeds the whole artifact text as an escaped JSON
  blob and forces a regen round-trip, paid once per emitter (×N). The artifact
  TYPE's serialization is already golden-locked once in ckc-core; per-VALUE
  locking does not belong in every emitter. Generalizable decomposition for any
  "produce N target artifacts" task (apply when planning 0.9–0.13): (1) one
  subtask for shared deterministic assembly helpers; (2) one subtask per emitter
  whose test is CHEAP — two-call `content_hash` equality +
  `artifact_text.contains(sentinel)` + `compilation_map` entry checks, with no
  committed golden; (3) one subtask to write+commit the human-readable artifact
  files with a byte-identical regen gate; (4) ONE consolidated golden over a
  compact hash manifest that byte-locks every artifact; (5) CAS
  persistence/determinism as its own subtask. Keep *committed* emit-subtask
  tests solver-independent so solver-less containers stay green — the
  PATH-guarded parse/solve checks are the 0.9 verification task per the roadmap.
  That is a test-placement choice only: install and run any solver
  (z3/cvc5/clingo/souffle/lean) freely at any time, including dev-time sanity
  checks of emitted artifacts during emit subtasks (CLAUDE.md standing
  authorization).

- [2026-05-29] Solver provisioning gotcha (fresh container): the `clingo` binary
  ships in the Debian apt package **`gringo`** (`sudo apt-get install -y gringo`
  installs `clasp` + `/usr/bin/{clingo,gringo,iclingo,oclingo,lpconvert,reify}`).
  `apt-cache policy clingo` reports `Candidate: (none)`, which misleads the
  natural `apt-get install clingo` attempt into a dead-end. Task 0.9 runs clingo;
  install it from `gringo`. (clingo 5.6.2 is installed in the current sandbox.)

- [2026-05-29] TLA+ tooling for task 0.9 verification: `java` is present in the
  sandbox; there is no apt package or LSP for TLA+ (LSP absence already noted
  above). The SANY syntax+semantic checker is class `tla2sany.SANY` inside
  `tla2tools.jar`, fetched from
  `https://github.com/tlaplus/tlaplus/releases/latest/download/tla2tools.jar`
  (the same jar `tlaplus/vscode-tlaplus` shells out to; it also bundles TLC).
  Invoke `java -cp tla2tools.jar tla2sany.SANY <file.tla>`; on success it prints
  only `Parsing file …` / `Semantic processing of module …` and exits 0, and
  emits explicit error lines on failure. Verified this session against the
  0.8.11 emitter output (`logic/tla/Conflict.tla`): parses and semantically
  processes with zero errors. The PATH/jar-guarded run belongs to 0.9 so
  solver-less containers stay green; install pattern mirrors the gringo note.

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
