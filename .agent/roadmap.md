# CKC roadmap

Build plan for /session-prompt — the session protocol, bookkeeping format, and stamp
semantics live in that command; SPEC.md is the design authority, its §2 the milestone
sequence. One milestone at a time: header `## <name> — plan <hash> — review <hash>` over an
ordered unit checklist; unchecked lines carry the full unit spec; checked items collapse to
one-line stubs `- [x] <id>: <gist>. NN% NNNK/200K <hash>`; a reviewed milestone keeps its
stubs until the next milestone is planned (that planning reads them to right-size units),
then persists as a bare header; git history retains all removed text. The active milestone's
`plan <hash>` shows `PENDING` until its first unit's closing commit fills it — the planning
commit is then known (M1's `89c4cba` was filled retroactively too).

## M1 scaffold — plan 89c4cba — accept m1 — review f6d68a0

## M2 multi-hop PoC — plan 2a4f03d

Scope = SPEC §9: experiment 1's minimal pair on this laptop. A weak local model translates the
M1 test sources two ways — `route.direct_smt` (model emits SMT-LIB directly, the baseline) versus
`route.single_ir` (model fills one grammar-constrained IR schema, then deterministic compile) —
scored by the M1 pipeline as instrument; published as a bilingual research report. Exactly two
routes (§10 widens the route axis). Each route is realized as one registry pipeline (`pipe.m2_*`);
the experiment binds the pipeline set, each pipeline scored by the shared deterministic tail.
Elaboration pick: `route.single_ir` fills **ClinicalIR** — free-text-free (closed-vocab fields =
lexicon codes / enums / bounded ints), so constrained decoding is tractable and deterministic
leverage high; it is NOT fully closed-vocab — generated IDs (`*_id`) + reference IDs
(`source_segment_ids`/`region_ids`) are constrained by the Id grammar + grounding, not a vocabulary
(the grounding scaffold handles exactly those). Per the "test all layer configurations" directive,
the full single_ir layer gradient (every meaningful IR layer + the DMN-style alt) defers to M3 / §10
— recorded in `.agent/memory.md` as the M3 route-axis seed. Milestone gate (model runtime) MET last
session (functionally confirmed); not a §15 gate — M2 results are locked measurements that stand on
their own. The local-model runtime is an environment-provided command invoked Z3-style — a bare
command name resolved on PATH (Z3 precedent), its wrapper supplied by the environment outside git;
so no unit commits an engine name, constraint dialect, or model format. Live units feed deny-Read
sources via runtime indirection (a script opens the path; the path never appears in a Read/Bash
argument).

- [x] model-types: ModelIdentity + §9 manifest measurement record + §7.4 model-route codes; M1
  pins unchanged. 71% 142K/200K d9651c4
- [x] schemas-export.1a: ClinicalIR JSON-Schema emitter-core + parse-only tests (salvage restore).
  49% 97K/200K 6b61113
- [x] schemas-export.1b: committed schemas/clinical_ir.schema.json + jsonschema oracle + hash pin;
  drift-guard bless pattern. 55% 110K/200K 6f4f97a
- [x] schemas-export.2: committed schemas/smt_query.grammar (BNF) + bnf Earley recognizer oracle +
  hash pin. 55% 110K/200K ad10279
- [x] registry-m2.1: SchemaEntry/PromptEntry + loaders + CLI model-registry file/hash check.
  82% 164K/200K 09b58a6
- [x] registry-m2.2: experiment pipeline-set binding (dual-form pipelines/baseline_pipeline) +
  validation. 66% 131K/200K 996ddb2
- [x] run-refactor: behavior-locked compile_verify_group extraction, timing-identical.
  40% 80K/200K ed4ae3e
- [x] model-adapter.1: env-command ModelAdapter — identity probe + invoke skeleton (bare PATH
  name). 76% 151K/200K 1b61cde
- [x] model-adapter.2a: constrained invoke + k-sample derive_seed + EOF-gated capture-completeness.
  46% 92K/200K 9ae5773
- [x] model-adapter.2a-codexfix: Completed race fix + capture/seed doc honesty + engine de-leak +
  grammar re-pin. 62% 123K/200K 19f6d30
- [x] model-adapter.2b: live #[ignore]d end-to-end adapter confirm — byte-stability +
  constraint-conformance. 57% 114K/200K 6de7da7
- [x] model-cassette.1: CassettePayload (core) + CassetteStore (cli) salvage restore; load-contract
  hardened. 46% 92K/200K e6d990b
- [x] model-cassette.2: live-blessed committed test cassette + runtime-absent content-hash-pinned
  replay. 72% 145K/200K b810753
- [x] stage-model-fill.1: decoupled model_fill stage core (replay/record -> accept -> ModelFill).
  51% 103K/200K da98bcc
- [x] stage-model-fill.2: repair loop (derive_seed re-prompt) + grounding terminal + both counters.
  58% 116K/200K bb18149
- [x] route-single-ir.1: pipe.m2_single_ir + m2.assemble stage + prompts.yaml/prompt + check loop.
  70% 140K/200K c9bff36
- [x] route-single-ir.2: single_ir_accept closure — strict-read + grounding pre-check.
  58% 116K/200K 5739d8d
- [x] route-single-ir.2b: single_ir_fill + 3 golden cassettes + reproduce-M1 gate (banked-patch
  redo). 47% 94K/200K 93c1d18
- [x] route-single-ir.3: single_ir verdict tail — scores m1 groups vs reference (z3, full oracle
  mirror). 88% 177K/200K ea77a93
- [x] route-single-ir.4: single_ir §7.4 rejection codes over committed bad cassettes (seeds
  99/98/97 + derived). 87% 175K/200K 0feb50d
- [x] route-direct-smt.1: direct_smt registry surface — 4-stage pipeline + 2 stages + prompt.
  67% 135K/200K 1b0af4a
- [x] route-direct-smt.2: verify_pair extraction + pub verify_query_pairs (caller-minted pairs).
  69% 139K/200K 467cd85
- [x] route-direct-smt.3a: 4 golden direct cassettes (M1 query bodies verbatim) + bless +
  self-check. 70% 140K/200K ff656a4
- [x] route-direct-smt.3b: direct_smt_accept + direct_smt_fill (role-namespaced sources, raw-AI
  smt_query provenance). 75% 151K/200K 97cabd7
- [x] route-direct-smt.4: direct_smt_verify_group verdict tail + reference scoring
  (DIRECT_VERIFY=3, direct §4.6 event). 77% 154K/200K 3723430
- [x] route-direct-smt.5: direct §7.4 rejection codes — schema exhaustion (seed 91) +
  TargetSyntaxFailure (seed 90). 63% 126K/200K cc555db

Standing M2-review flag: pre-existing 18 rustdoc `private_intra_doc_links` errors
(model.rs/cassette.rs/replay.rs/trace.rs; `RUSTDOCFLAGS='-D warnings' cargo doc -p ckc-cli
--no-deps`) — per-unit gates hold the count AT 18 (no new), the fix lands at milestone review
(pattern: memory doc-lint bullet).

- [ ] metrics-m2.1: §7.3 route-quality raw-row metrics — TWO sessions, both commit (M2.21),
  zero re-derivation: read `.agent/wip-metrics-m2-1.txt` (blueprint) and execute it. Module =
  `.agent/wip-metrics.rs.txt`, COMPLETE + gate-VERIFIED at pre-stage (9 unit tests green,
  fmt/clippy/audit clean, rustdoc count 18 = pre-existing; sha256 in blueprint §0). Session
  .1a DONE 37% 74K/200K: module restored (sha256 == §0) + lib.rs wired, gates green
  (450/7, fmt, clippy, rustdoc 18). Session .1b = blueprint §2-§3: run.rs test
  `route_metrics_score_recorded_two_route_run` over the committed two-route cassettes +
  m1_expected reference (observation recipe + expected row vectors + template/helper anchors
  all pre-derived; z3, model-runtime-absent). Gate: blueprint §1.3/§4 (cargo test --workspace
  450/7 then 451/7, fmt, clippy -D, engine-agnostic audit on touched files, rustdoc still 18).
  CLOSE at .1b: rm both wip files; unit DONE.
- [ ] metrics-m2.2: k-sample stability + baseline-delta. k-sample verdict stability/convergence
  (per-route verdict agreement across k samples); baseline-delta table = per-metric (route −
  baseline) over identical inputs (baseline = the `direct_smt` pipeline per `exp.m2_multihop`), raw
  rows emitted BEFORE the delta table. Reading: metrics-m2.1 module, the experiment baseline
  designation; SPEC §7.3 baseline delta + k-sample, §9 raw-rows-before-ranking. Gate: `cargo test`;
  stability + delta correct on a fixture; raw-rows-before-delta ordering asserted. [Acceptance: raw
  rows emit before the baseline-delta table.]
- [ ] report-m2.1: report.json M2 shape + canonical. Extend the `Report` types + `report.json`
  canonical shape (Canonical/CanonRead) with per-route raw rows, the baseline-delta table, findings
  (quoted Japanese source spans + named assertions), a failure-taxonomy summary (§6 categories + §7.4
  codes), model + solver identities, replay status, metrics (M2+). Byte-pin the canonical form on a
  HAND-BUILT, fully-populated fixture (no run needed). Reading: `crates/ckc-cli/src/report.rs` Report
  types + canon; SPEC §7.2, §9 report.json contents. Gate: `cargo test`; canonical round-trip + pinned
  bytes for the populated fixture; §0 vocabulary in the wording fields.
- [ ] report-m2.2: assemble_report M2 population. Extend `assemble_report` to populate the M2
  `report.json` from a recorded two-route run — wire the metrics modules, model + solver identities,
  replay status, the failure-taxonomy. Reading: `report.rs` assemble_report + report-m2.1 types; the
  metrics modules; SPEC §7.2, §9. Gate: `cargo test`; report.json assembles from a recorded-run
  fixture with every M2 section present + canonical-valid. [Split from report-m2.1: canonical type/pin
  vs assembly population.]
- [ ] report-m2.3: bilingual rendering. Render `report_en.md` (extend the M1 renderer with M2
  metrics/delta/taxonomy) + new `report_ja.md` (deterministic Japanese rendering of the same canonical
  report.json); §0 locked-measurement wording, no clinical claims; quoted JA spans verbatim. Reading:
  `report.rs` render_markdown + report-m2.1/.2 payload; SPEC §7.2 (report_ja from M2), §0 vocabulary.
  Gate: `cargo test`; both md files render deterministically (byte-stable) from one report.json; the
  JA rendering is well-formed; §0 vocabulary asserted.
- [ ] run-m2.1: `exp.m2_multihop` wiring + experiment entry. Seed the `exp.m2_multihop`
  `ExperimentEntry` (`pipelines=[pipe.m2_direct_smt, pipe.m2_single_ir]`, `baseline_pipeline=
  pipe.m2_direct_smt`, the M1 groups, seed, budget incl. k-sample count + repair limit) — both
  pipelines now exist, so `ckc registry check` validates the full experiment. Wire `run.rs` to execute
  both route pipelines under one experiment run → per-route `model_fill` → scoring → metrics →
  `report.json` + `report_en.md` + `report_ja.md` + run/replay manifests (populating the
  model/prompt/identity hash fields), over the locked M1 inputs. single_ir assemble-wrapper
  input_hashes: M1 cites source+segments+normalization; single_ir has no normalization wrapper →
  cite source+segments+the replayed cassette `content_hash` (the model_fill provenance; .2b's gate
  used source+segments only, F4 payload-only, so add the cassette hash here). direct_smt has no assemble
  wrapper + no `compiled`: its `model_fill_smt` `smt_query` wrappers cite source+segments (.3b) + the replayed
  cassette `content_hash` added here (same model_fill provenance), and its `verifier_results` cite the two
  `smt_query` wrapper `content_hash`es (route-direct-smt.4 — the upstream artifact, as single_ir's verify cites
  `compiled`). Generalize `resolve()` (run.rs L208): today it selects only `experiment.baseline()` + iterates
  all 8 `PROCESSING_STAGE_KINDS`, so make resolution route-aware over each pipeline's DECLARED stage list
  (`pipe.m2_direct_smt` = 4, `pipe.m2_single_ir` = 6) — else `ckc run exp.m2_multihop` cannot resolve the
  non-baseline route. Add the `ckc run --record` flag (default
  = replay committed cassettes runtime-absent; `--record` drives `CassetteStore::record` live — run-m2.2
  exercises that live path) + its default-replay acceptance. Tested via REPLAY of the route units'
  committed cassettes (deterministic, no live call) — model-fill replays runtime-absent, so replay.rs
  hash-compare covers the model-stage artifacts. Reading: `run.rs` execute (route loop), the
  routes, metrics, report, manifests; `registry/experiments.yaml`; SPEC §1 command, §9. Gate:
  `cargo test`; `ckc registry check` validates `exp.m2_multihop`; a replay-driven run lands all
  artifacts deterministically; manifests carry the populated identity/hash fields.
- [ ] run-m2.2: live-pin battery over the run binary. Record the full experiment cassette via the env
  runtime command (LIVE, runtime indirection over deny-Read sources), commit the recorded model I/O as
  tracked test-source artifacts (origin `ai_generated`); live pins on `report.json` sections + manifest
  evaluator-identity hashes; `ckc replay` byte-matches with the runtime command ABSENT. Reading:
  run-m2.1 wiring; replay/cassette modules; SPEC §9 recorded-bytes, §7.2. Gate (LIVE): the command
  lands all artifacts; live pins hold; `ckc replay` byte-matches the recorded run. [Live-pin-over-run-
  binary = its own unit; cassette stored as tracked test-source artifacts, NOT under gitignored `runs/`.]
- [ ] acceptance-m2: §9 acceptance. Verify the §9 themes against the recorded run — both routes
  execute over identical locked inputs (`exp.m2_multihop`); recorded model I/O replays byte-stably
  (runtime absent); raw rows before the baseline-delta table; expected conflict/no-conflict per
  reference for accepted translations; the bilingual report renders deterministically from
  report.json; §0 vocabulary holds. Tag `accept/m2`. [§9 scopes acceptance to faithful measurement,
  NOT a required result sign — a null/negative delta is a valid PoC outcome.] Reading: all M2 artifacts
  + the §9 acceptance themes. Gate: all six themes pass on the recorded run; `ckc replay` byte-matches;
  tag `accept/m2`.
