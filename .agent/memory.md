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

- [2026-05-29] rust-analyzer `unlinked-file` on a freshly-added `tests/*.rs`
  integration file is a stale-metadata false positive, not a wiring bug. Each
  `tests/*.rs` is its own auto-discovered cargo target (crates here use default
  `autotests` with no explicit `[[test]]`), so RA links it on its next workspace
  metadata reload; the file is already a real target the moment cargo sees it.
  Authoritative check: `cargo test --test <name>` compiling+running the file
  proves linkage. Do NOT apply the lint's own suggested fix (adding
  `unlinked-file` to `rust-analyzer.diagnostics.disabled`) — that suppresses
  detection of genuinely orphaned `src/*.rs` (missing `mod`) workspace-wide.
  Expect this on every subtask that adds a new integration test; ignore it.

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

- [2026-05-29] Alloy tooling for task 0.9 verification: the org.alloytools
  distribution jar is at `~/.local/share/alloy/alloy.jar` (v6.2.0; `java` is
  present). It ships a headless CLI (Main-Class
  `org.alloytools.alloy.core.infra.Alloy`): invoke
  `java -jar alloy.jar exec <file.als>` (add `-Djava.awt.headless=true` for
  safety). It prints one line per command to **stderr** (stdout stays empty —
  confirmed task 0.9.7 when a live test read stdout and matched nothing), e.g.
  `00. check NoPriorityCycle    0    UNSAT`. For a `check`, `UNSAT` means NO
  counterexample exists (assertion holds); `SAT` would report a counterexample.
  So the 0.8.12 emitter's `logic/alloy/Priority.als` acyclicity check returns
  UNSAT — the witnessed-acyclic result, the Alloy analog of the empty Soufflé
  `cycle` relation. The demo class
  `edu.mit.csail.sdg.alloy4whole.ExampleUsingTheCompiler` is NOT in this jar;
  use the `exec` subcommand. Verified this session against the emitted stub.
  Side effect: `exec` writes a `<ModuleName>/receipt.json` (here `Priority/`)
  into the CWD, so run it from a scratch/run dir to avoid polluting the repo
  root. The jar-guarded run belongs to 0.9 so solver-less containers stay green;
  mirrors the gringo/SANY notes.

- [2026-05-29] CompiledTarget provenance invariants (task 0.8 review, applied to
  the datalog + SMT-repair emitters). Two rules for every emitter, including the
  0.9+ verification wiring and the later target-language phases: (1) a
  `CompilationMap` `ckc_node_id` must be a *resolvable* CKC node id — a
  rule/conflict/row/narrative id a `find_*` lookup can return — so a reported
  target symbol traces back to a real node. The repair emitter had used the
  repair `type` string ("add_priority"), which resolves to nothing; corrected to
  the owning conflict id, with the repair kind preserved in the target symbol
  (`repair_<type>`). (2) `source_artifact_hashes` must cover every CKC artifact
  the emitter reads, including endpoints it maps but does not "own". The datalog
  priority emitter hashed only the superiority-edge carrier; corrected to both
  graph endpoints, mirroring the alloy emitter over the same priority relation.
  When two emitters compile the same CKC view (here datalog + alloy over the
  superiority graph), cross-check them for provenance consistency. Mechanics:
  changing `compilation_map`/`source_artifact_hashes` moves the CompiledTarget
  `content_hash` but NOT `artifact_text`, so only the portfolio-manifest golden
  needs regen (`cargo test -p ckc-compile --test manifest -- --ignored
  regenerate`); committed `logic/*`+`lean/*` files and the persistence test
  (live hashes) stay correct untouched.

- [2026-05-30] Task 0.9 solver runbook (all verifiers now installed in this
  sandbox; complements the gringo/alloy/SANY provisioning notes above). Paths:
  z3 `/usr/bin/z3`, clingo `/usr/bin/clingo`, souffle `/usr/bin/souffle`, java
  `/usr/bin/java`; cvc5 `~/.local/bin/cvc5` (proof flags `--produce-proofs
  --dump-proofs [--proof-format-mode=alethe]`); lean/lake `~/.local/bin/{lean,
  lake}` via elan; jars `~/.local/share/alloy/alloy.jar` (`java -jar … exec
  <f.als>`) and `~/.local/share/tla/tla2tools.jar` (`java -cp … tla2sany.SANY
  <f.tla>`, newly fetched this session). All resolvable on the non-interactive
  base PATH. RUN GOTCHAS: clingo exits 30 (not 0) on SAT — assert on the
  `SATISFIABLE` stdout token + model atoms, accept exit ∈ {10,30}; souffle writes
  `cycle.csv` and alloy writes a `<Module>/` dir relative to CWD — run both from
  a throwaway TempDir CWD; a single import-free Lean file kernel-checks via bare
  `lean <file>` (no lakefile/lake project; only `import Mathlib`/cross-file deps
  would need one). The per-target recorded toy verdicts live in roadmap 0.9.2 /
  `verifier_outcomes.json`.
- [2026-05-30] Task 0.9 design rationale (Type A plan): determinism via a
  RECORDED-ORACLE — accepted Certificates/ExecutionWitnesses are built from
  committed canonicalized verdicts (`verifier_outcomes.json`), never from live
  solver stdout (z3 model order / cvc5 proof bytes drift run-to-run and would
  break the 0.13 replay-hash acceptance). Live solvers run ONLY in PATH-guarded
  auto-skip `tests/live_*.rs` that re-derive verdicts and compare to the oracle;
  0.9.3 introduces `runner::solver_available(bin)` (spawn `bin --version`, false
  on spawn error) as the repo's first binary-guard idiom (none existed; mirrors
  the emit-only stance of 0.8). Scope: ONE crate `crates/ckc-verify/`; SPEC §19
  `ckc-cert` deferred — its concerns live as `graph`/`assurance` modules,
  mechanical to extract when 0.13 replay warrants it. RDF/SHACL are NOT rebuilt:
  `ckc-term::{rdf,shacl}` already emit `terminology.ttl` + `shacl_report.json`
  (scenario 6: `rule_incomplete_provenance` → exactly 2 violations); 0.9.10 only
  wraps that report in a C6 certificate. Certificate/ExecutionWitness/
  AssuranceNode/CertificateClass and every persistence ArtifactKind already
  exist. CORRECTION (0.9.2, user decision): one ckc-core change WAS needed — the
  SHACL `VerifierOutcome` has no representable `target_language` (the 6-variant
  `TargetLanguage` lacked RDF/SHACL), so a `Rdf` variant was added to
  `ckc_core::enums::TargetLanguage`. Ripple stayed minimal and contained: one new
  `replay_command` arm (`Rdf => "rdf"`, the only exhaustive match) + regen of the
  two schemas embedding the enum (`compiled_target.schema.json`,
  `compile_portfolio_manifest.schema.json`); no value goldens shifted. cvc5's
  outcome is `SmtLib` (over the `.smt2`); only SHACL carries `Rdf`.
- [2026-05-30] Cargo integration tests (`crates/<c>/tests/*.rs`) can `use` the
  crate's normal `[dependencies]` directly, not just `[dev-dependencies]` —
  proven by `ckc-compile/tests/manifest.rs` doing `use ckc_core::canonical::…`
  while ckc-compile lists ckc-core only under `[dependencies]`. So a golden
  harness in `ckc-verify/tests/` reaches `ckc_core::canonical::{content_hash,
  to_canonical_bytes}` with no dev-dep or re-export needed. (Task 0.9.1 routed
  everything through `ckc_verify` re-exports, which is fine but not required;
  prefer the direct `use ckc_core::…` when copying the manifest.rs harness.)
- [2026-05-30] NF sort helpers do NOT deduplicate — correcting a recurring
  misstatement in the 0.9.x roadmap task text ("Normalize sorts/dedups").
  `NfContext::sort_ord` (used for `source_span_ids`, hash vecs, rule-id vecs)
  sorts in place and records a rewrite but never dedups; `sort_by_canonical`/
  `sort_graph` likewise sort-by-key without dedup. So any builder field that is
  conceptually a *union* must sort+dedup itself before `normalize_all`, or
  duplicates survive. Concrete trap hit in 0.9.9: the Event Calculus compiled
  target repeats its 2 narrative spans across 4 `compilation_map` entries, so
  the naive flat-mapped `source_span_ids` had 8 entries; `witness_for` calls
  `.sort()` + `.dedup()` to get the 2 distinct spans. Applies ahead to 0.9.12
  (`build_graph` nodes/edges: multiple certs/witnesses reference the same target
  or source-artifact hash → dedup `CertNode`s before/after sorting, else the
  graph carries duplicate nodes and the golden bloats) and any later
  union-shaped artifact. Verify with a quick len check on a known-overlapping
  input rather than trusting the "dedups" phrasing.

- [2026-05-30] Certificate-graph scope (task 0.9.12): `build_graph`'s
  `schemas/golden/certificate_graph.json` was locked over the FULL Phase-0 set —
  `certificates(bundle)` + `shacl_certificate` cert (11) and `witnesses(bundle)` +
  shacl witness (11) — so 0.9.14 `verify_all` MUST call `build_graph` with that
  same 11+11 set, else the graph `content_hash` diverges from this golden and from
  the 0.9.15 committed `certs/certificate_graph.json` (which 0.9.15 byte-locks
  against the live `verify_all` graph). Edge model: cert→target `checks`
  (resolved by intersecting the cert's `input_artifact_hashes` with the
  `compile_all` target hashes — exactly one hit per solver/cvc5 cert, zero for the
  in-process SHACL cert), target→source `derived_from`, cert→witness
  `witnessed_by` (matched on `certificate_id`). The norm-conflict SMT target is
  checked by BOTH z3 and cvc5, so its node + `derived_from` edges are emitted
  twice and MUST be deduped (confirmed: 1 node, 2 incoming `checks`, sources once)
  — the union-dedup trap the prior NF note flagged. Phase-0 shape: 9
  compiled_target + 5 source_artifact + 11 certificate + 11 execution_witness
  nodes; 10 checks + 14 derived_from + 11 witnessed_by edges.

- [2026-05-30] Conflict-detector invariants (task 0.10 Type-R review corrections).
  (1) The Rust Event-Calculus persistence mirror
  `ckc-conflict::detect::persists_until` must match `logic/asp/event_calculus.lp`
  exactly: initiation is STRICT (`ti < t`, the `.lp` `holdsAt` `T1 < T2` guard) and
  clipping is OPEN-INTERVAL per `stoppedIn` (`ti < tt < t`), evaluated interval-local
  to each initiation — NOT a global `<=`-initiation plus any-earlier-terminate test
  (the pre-review form, which mis-clips a terminate at/before initiation and admits
  `ti == t`). Re-verify any later temporal mirror against the `.lp` clauses, not
  intuition. (2) "Detection drives emission" for a SPEC defect defined as a
  CONJUNCTION must gate on EVERY conjunct: the #14 decision-table defect is overlap
  AND gap, so both `decision_table_overlap` and `decision_table_gap` gate
  `detect_decision_table_defects` (gap was load-bearing in name only before). Apply
  to any future multi-condition detector. Both fixes were byte-identical on the toy
  bundle (same verdict, same emitted set), so no committed cert/golden regen.

## Known issues

- [2026-05-30] (FLAGGED, escalated to user — task 0.10 review; unresolved by design
  decision, not a code bug.) Norm-conflict double-resolution in
  `examples/research_kernel/fixtures/rules.json`: the sepsis/anaphylaxis pair is
  resolved TWICE in the same direction —
  `rule_bl_anaphylaxis_contra.priority_over=[rule_sepsis_bl_recommend]` AND
  `rule_sepsis_bl_recommend.exceptions=[beta_lactam_anaphylaxis]` — yet
  `conflicts.json` / `detect_norm_contradiction` still emit it as a
  `norm_contradiction` `TrueConflict` (severity High) carrying human-review
  questions, with repair candidates `add_priority` + `add_exception` that DUPLICATE
  resolutions already present in the fixtures. Tension: an already-resolved pair is
  presented as an unresolved conflict needing adjudication. NOT patched in a
  per-task review — reconciling it is a scenario-design decision (does the demo mean
  to surface a latent-but-resolved contradiction, or a genuinely unprioritized one?)
  that ripples across committed compile (superiority graph + priority datalog/alloy),
  verify (norm-conflict certificate), and conflict (conflict + argument graph +
  repair) artifacts and needs cross-crate regen + re-review. Resolve as a dedicated
  future task before treating the norm-conflict scenario as authoritative.

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
