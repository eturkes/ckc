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

Salvage caveat: stubs tagged `[S]` (tag precedes the usage figure) closed by consuming banked
`.agent/wip-*` salvage/blueprint artifacts — proven code/patches/transcription blueprints committed
by an earlier overflowed session — so their recorded context-usage measures the apply/redo session
only and materially UNDERSTATES the unit as specced; sizing anchors come from untagged stubs only.
M1's collapsed stubs cli-runner.3a.2a / cli-runner.4.1a.2 / cli-runner.4.1b.2b.1 carry the same
caveat. Pattern RETIRED (user directive): banking applyable artifacts cheats the unit — overflow
recovery = revert + respec into fresh self-contained units (memory's sizing bullet holds the
standing rule); retired wip artifacts remain in git history as provenance only.

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
  [S] 49% 97K/200K 6b61113
- [x] schemas-export.1b: committed schemas/clinical_ir.schema.json + jsonschema oracle + hash pin;
  drift-guard bless pattern. [S] 55% 110K/200K 6f4f97a
- [x] schemas-export.2: committed schemas/smt_query.grammar (BNF) + bnf Earley recognizer oracle +
  hash pin. [S] 55% 110K/200K ad10279
- [x] registry-m2.1: SchemaEntry/PromptEntry + loaders + CLI model-registry file/hash check.
  82% 164K/200K 09b58a6
- [x] registry-m2.2: experiment pipeline-set binding (dual-form pipelines/baseline_pipeline) +
  validation. 66% 131K/200K 996ddb2
- [x] run-refactor: behavior-locked compile_verify_group extraction, timing-identical.
  40% 80K/200K ed4ae3e
- [x] model-adapter.1: env-command ModelAdapter — identity probe + invoke skeleton (bare PATH
  name). 76% 151K/200K 1b61cde
- [x] model-adapter.2a: constrained invoke + k-sample derive_seed + EOF-gated capture-completeness.
  [S] 46% 92K/200K 9ae5773
- [x] model-adapter.2a-codexfix: Completed race fix + capture/seed doc honesty + engine de-leak +
  grammar re-pin. 62% 123K/200K 19f6d30
- [x] model-adapter.2b: live #[ignore]d end-to-end adapter confirm — byte-stability +
  constraint-conformance. 57% 114K/200K 6de7da7
- [x] model-cassette.1: CassettePayload (core) + CassetteStore (cli) salvage restore; load-contract
  hardened. [S] 46% 92K/200K e6d990b
- [x] model-cassette.2: live-blessed committed test cassette + runtime-absent content-hash-pinned
  replay. 72% 145K/200K b810753
- [x] stage-model-fill.1: decoupled model_fill stage core (replay/record -> accept -> ModelFill).
  [S] 51% 103K/200K da98bcc
- [x] stage-model-fill.2: repair loop (derive_seed re-prompt) + grounding terminal + both counters.
  [S] 58% 116K/200K bb18149
- [x] route-single-ir.1: pipe.m2_single_ir + m2.assemble stage + prompts.yaml/prompt + check loop.
  70% 140K/200K c9bff36
- [x] route-single-ir.2: single_ir_accept closure — strict-read + grounding pre-check.
  [S] 58% 116K/200K 5739d8d
- [x] route-single-ir.2b: single_ir_fill + 3 golden cassettes + reproduce-M1 gate (banked-patch
  redo). [S] 47% 94K/200K 93c1d18
- [x] route-single-ir.3: single_ir verdict tail — scores m1 groups vs reference (z3, full oracle
  mirror). 88% 177K/200K ea77a93
- [x] route-single-ir.4: single_ir §7.4 rejection codes over committed bad cassettes (seeds
  99/98/97 + derived). 87% 175K/200K 0feb50d
- [x] route-direct-smt.1: direct_smt registry surface — 4-stage pipeline + 2 stages + prompt.
  67% 135K/200K 1b0af4a
- [x] route-direct-smt.2: verify_pair extraction + pub verify_query_pairs (caller-minted pairs).
  69% 139K/200K 467cd85
- [x] route-direct-smt.3a: 4 golden direct cassettes (M1 query bodies verbatim) + bless +
  self-check. [S] 70% 140K/200K ff656a4
- [x] route-direct-smt.3b: direct_smt_accept + direct_smt_fill (role-namespaced sources, raw-AI
  smt_query provenance). [S] 75% 151K/200K 97cabd7
- [x] route-direct-smt.4: direct_smt_verify_group verdict tail + reference scoring
  (DIRECT_VERIFY=3, direct §4.6 event). 77% 154K/200K 3723430
- [x] route-direct-smt.5: direct §7.4 rejection codes — schema exhaustion (seed 91) +
  TargetSyntaxFailure (seed 90). [S] 63% 126K/200K cc555db

Standing M2-review flag: pre-existing rustdoc errors — 17 `private_intra_doc_links` in ckc-cli
(model.rs/replay.rs/trace.rs; a cassette.rs link since fixed) + 17 unresolved-link in ckc-core (enums.rs:50 macro doc,
one per fieldless_enum! expansion; `RUSTDOCFLAGS='-D warnings' cargo doc -p <crate> --no-deps`) —
per-unit gates hold both counts (no new), the fix lands at milestone review (pattern: memory
doc-lint bullet).

- [x] metrics-m2.1: §7.3 raw-row metrics — metrics.rs (route_metrics + 9 unit tests) + run.rs
  `route_metrics_score_recorded_two_route_run` (two-route recorded-cassette test, both arms'
  full row vectors + pipeline_id + cardinalities pinned). Pre-staged blueprint executed
  zero-re-derivation, first-run green; wip files consumed+rm'd. [S] .1a 37% 74K/200K ad174a7 ·
  .1b 60% 120K/200K 6f785b6
- [x] metrics-m2.2: k_sample_convergence row (pairwise fingerprint agreement, NA on k<2) +
  experiment_metrics baseline-delta assembly + emission_order §9 contract + Rational::sub. 87% 174K/200K
  05746ce
- [x] report-m2.1a: metrics.rs canonical layer (MetricRow/RouteMetrics/RouteDelta/
  ExperimentMetrics) — §9 raw_rows<route_deltas byte pin; salvage redo, gates == banked.
  [S] 39% 79K/200K 6c28421
- [x] report-m2.1b: Report M2 shape — failure_taxonomy/metrics/model_identity omit-None
  members + RouteTaxonomy + per-member validate rejections; M1 pins byte-identical.
  72% 143K/200K 822f924
- [x] report-m2.1c: populated_report fixture (§8.2 JA spans, settled taxonomy/metrics/identity)
  + PINNED_POPULATED_REPORT + §9 raw-before-delta pin. 54% 108K/200K be3e772
- [x] report-m2.2: assemble_report M2 population — ModelRunSections (route §7.4 ledgers→taxonomy,
  in-assembly experiment_metrics, identity verbatim). 71% 143K/200K 71058d3
- [x] report-m2.3a: report_en.md M2 sections — emission_order walk, §0-vocab lead, two pinned
  renders. 64% 127K/200K ebadf6b
- [x] report-m2.3b: report_ja.md renderer — shared Labels walk, §0 verbatim-EN in JA prose, two
  observed-output pins. 80% 160K/200K 4b5f799
- [x] run-m2.1a: two-route resolve (per-route views + RouteShape fingerprint) + exp.m2_multihop
  seed + execute M1-gate. 69% 138K/200K eb74f7d
- [x] run-m2.1b: 4-case resolve rejection battery (unsupported sequence / undefined stage /
  undefined pipeline / malformed binding), tests-only. 48% 96K/200K da51698
- [x] run-m2.1c: ModelFill attestation (accepted_cassette_hash + model_identity) + both route
  wrappers cite the accepted cassette hash (set-pinned). 74% 148K/200K a767898
- [x] run-m2.1d1: DocTrace/GroupTrace `dir` plumbing + source-node dedup + member-id+hash
  bundle→compiled edges, M1-byte-locked. 79% 159K/200K 1bfc7e0
- [x] run-m2.1d2: per-view repair_limit/is_baseline resolve extension + route_id_prefix +
  committed-registry mutation rejections (missing/overflow repair limit, sample count 2), M1 pins
  untouched. 63% 126K/200K b958cbb
- [ ] run-m2.1d3: single_ir route stage — landing + §4.6 events. New `route_document_head(root,
  entry, resolved, shell) -> Option<DocHead>` = document_pipeline's extract+segment half, shared by
  both routes (.1d4 reuses): read/extract/segment failures → today's diagnostics + None; lands
  graph+segments (M1 file names, route-prefixed artifact_ids) under `routes/{pid}/artifacts/{doc}/`
  with slot-0/1 events via the existing stage helpers (kinds at slots 0/1 match M1 — mirror
  document_pipeline); returns both wrappers + DocTrace{dir: that path, normalization None}. Rework
  `single_ir_fill`: consume DocHead; model_fill Replay + single_ir_accept unchanged;
  fill.diagnostics ride the model_fill event ONLY — Shell::processing_stage_event extends the
  ledger from event diagnostics (single path; route tests + .1d5 RouteRun ledger slices still see
  them; an added shell.diagnostic call would double-push ledger + double-merge outcome) —
  DIRECT-emitted (pattern: direct_smt_verify_group): slot 2, kind
  "model_fill", clock opened AFTER provenance gathering (M2.7 lesson), inputs [source, segments]
  hashes, outputs [accepted_cassette_hash] iff target, counters [(RECORDED_CALLS_COUNTER,
  recorded_calls), (REPAIRS_COUNTER, repairs)] (model_fill.rs 44/50), outcome =
  severity(fill.diagnostics); Err(CassetteError) → command diagnostic, NO event (infra rule). On
  target: derive tail unchanged under an assemble event via finish_processing_stage(3) (kind
  matches), bundle id `{prefix}{doc}.ir_bundle` landed beside graph/segments. Returns
  RouteDoc{trace (bundle slot set), graph wrapper, fill: Option<FillObservation> (metrics.rs
  from_fill ≈78, build before consuming fill), identity: fill.model_identity}.
  `compile_verify_group` artifact_ids gain the prefix (M1 "" → unchanged). Update call sites
  mechanically: reproduces ≈3150 (payload+content_hash pins hold; artifact_id unpinned by design),
  scores ≈3230, rejection ≈3409 (ledger asserts hold), route_metrics ≈4454 single_ir half. Gate:
  cargo test; reproduces-test extended to read events.jsonl + pin the 4 per-doc event tuples
  (kind/step_id/outcome/counters/outputs) + landed paths; M1 execute pins untouched.
- [ ] run-m2.1d4: direct route stage — landing + §4.6 events. Rework `direct_smt_fill`: consume the
  two members' DocHeads (caller builds one per UNIQUE doc via route_document_head — a doc in two
  groups lands/events once per route) replacing its internal re-read/extract/segment; per-role
  model_fill + accept unchanged (role sources stay unprefixed); land both smt_query wrappers (ids
  `{prefix}{gid}.{role}.smt_query`) at `routes/{pid}/groups/{gid}/{role}.smt_query.json`; ONE group
  model_fill event, DIRECT-emitted: slot 2, kind "model_fill", clock after provenance, inputs =
  member-order [source, segments] hashes, outputs = landed wrapper hashes (present ones), counters
  summed over both roles, outcome severity(both fills' diagnostics), diagnostics ride the event
  ONLY (ledger via Shell::processing_stage_event — .1d3's single path); Err(CassetteError) →
  command diagnostic, NO event. Returns DirectFill{pair:
  Option<(overlap, deontic)>, fills: Vec<FillObservation>, identities: Vec<ModelIdentity>}
  (observations survive terminal reject). `direct_smt_verify_group` artifact_id gains the prefix;
  keeps its direct verify event; no .smt2 materialization. Update call sites: direct reproduces
  ≈3822, non-pair ≈3959, rejection battery, route_metrics ≈4454 direct half. Gate: cargo test; one
  direct test extended to pin the group model_fill event tuple + smt_query landed paths +
  once-per-doc head events; M1 pins untouched.
- [ ] run-m2.1d5: model-route loop in `execute()` + determinism gate. Replace the model-route gate
  diagnostic (DELETE its test ≈2886): single M1Layered view → existing path verbatim; mixed
  M1+model set → command diagnostic, zero artifacts; model set → lexicon read +
  `CassetteStore::new(root)` + Z3Adapter::new (each failure → command diagnostic); base seed =
  experiment seed; then per view in set order: mark ledger start (shell.ledger().len());
  SingleIr = per-doc route_document_head →
  single_ir_fill, per-group compile_verify_group (dir `routes/{pid}/groups/{gid}`, smt under it);
  DirectSmt = per-unique-doc route_document_head, per-group direct_smt_fill →
  direct_smt_verify_group. Identity agreement: each fill's model_identity folds into one agreed
  Option<ModelIdentity> — mismatch = command diagnostic, fail-closed return (goldens agree:
  model.baseline/fixture_quant/1.0.0). Collect per view RouteRun{pipeline_id, ledger slice, fills,
  groups: Vec<GroupObservation>, samples: vec![groups.clone()]} (#[allow(dead_code)]; .1e consumes;
  observation shapes = route_metrics test ≈4454 — single_ir pairs from compiled.solver_query_plan,
  direct pairs = minted role ids). Tails run ONCE over all routes' traces (graphs deduped by
  document_id — payload-identical so content_hash equal; results per route, the settled §7.1 shape
  per .1e's populated fixture; report findings/no_conflict rows = BASELINE view's groups only —
  finding.{gid}.{seq} mints dense per group + Report::validate dedups finding ids, so two views'
  same-group results collide; model-route quality reaches the report as .1e's aggregate sections):
  trace_processing_stage + report_processing_stage gain `emit_event:
  bool` — M1 true (bytes unchanged), M2 false (trace/report = undeclared steps of route pipelines;
  tail wrapper producer = baseline view → step id UNUSED_STAGE, honest sentinel); sections stay
  None (.1e populates). Gate test: `write_m2_root` mirror (copy from repo_root(): registry/*.yaml,
  corpus lexicon + 3 html + reference yaml, rust-toolchain.toml, Cargo.lock; plant the 7 golden
  seed-42 cassettes under `<root>/cassettes/`) → executed twice: zero command diagnostics; 27
  events (single_ir 3×4+2×2=16 + direct 3×2+2×2=10 + 1 command, tails none; separate M1 baseline
  run stays 19) with
  model_fill counters (single_ir per doc 1/0; direct per group 2/0); both routes' layout present;
  trace_bundle strict-parses, shared source nodes once, direct verifier_results = legal orphan
  node (validate has no orphan check); determinism = landed artifacts byte-equal across the two
  runs + manifests byte-equal after normalizing the one `--out` token
  (manifest_inputs ≈1497 embeds out_dir.display()) + events compared on a non-timing projection.
  M1 executed() pins unchanged.
- [ ] run-m2.1e: §9 measurement record — report sections + manifests. report_processing_stage builds
  `ModelRunSections{route_diagnostics (per-route ledgers, clean route = empty slice), route_metrics
  (metrics::route_metrics per route — samples = the k=1 battery → convergence NA), baseline_pipeline_
  id, model_identity}`; `experiment_metrics` stays in-assembly (never called from run.rs). §7.1-vs-M2
  section coexistence mirrors the report-m2.1c populated_report fixture (settled shape — read it +
  Report::validate before wiring; graphs route-independent, results per route). Land report_ja.md
  beside report_en.md (`render_markdown_ja`, same read-back discipline). ManifestInputs +
  assemble_manifests gain the §9 omittable set, populated: model_identity = the run's agreed
  identity; test_source_hash = hash_bytes over the run's per-source raw-byte `sha256:<hex>` strings
  sorted + joined `\n`; reference_hash = raw bytes of the experiment's expected_outcomes file;
  schema_hash = raw bytes of registry/schemas.yaml; prompt_template_hash = raw bytes of
  registry/prompts.yaml (the registry files transitively pin the referenced files' hashes);
  model_hash/runtime_hash = None (no committed model/runtime file — env wrapper outside git; honest
  omission, run-m2.2 revisits). M1 runs keep every field None → M1 manifest pins byte-identical.
  Flagged from .1d: landed smt_query wrappers are ABSENT from manifest output_hashes + trace nodes
  (GroupTrace carries no smt_query slots) — close the gap here: extend GroupTrace + the manifest
  walk so every accepted landed artifact is manifested → replay-covered (replay.rs diffs
  expected_output_hashes vs the rerun manifest; a landed-but-unmanifested wrapper = silent replay
  hole, so omission is not an option). Sections consume .1d5's
  banked RouteRun{pipeline_id, ledger slice, fills, groups, samples}. Reading: run.rs
  report/manifest fns; report.rs ModelRunSections + populated fixture; manifests.rs; replay.rs +
  trace.rs GroupTrace for the smt_query decision. Gate: cargo test; full-run test pins report.json
  M2 sections + both md bodies + manifest fields on the .1d5 replayed run; M1 pins hold.
- [ ] run-m2.1f: `ckc run --record`. Dispatch: optional `--record` flag → `Run{record: bool}`
  (default false = replay, the committed acceptance path); record mode: one `ModelAdapter::new()`
  probe → identity into manifests/report verbatim; per route `FillSource::Record{adapter, prompt,
  constraint, ctx}` — constraint file = the route target-kind's SchemaEntry path (clinical_ir /
  smt_query), prompt = its PromptEntry file by route id (SingleIr↔route.single_ir,
  DirectSmt↔route.direct_smt), composed as template bytes + (single_ir: the doc's id line + segment
  texts in segment order; direct: a `role: overlap|deontic` line + both members' id lines + segment
  texts) — first-draft composition, run-m2.2's live recording refines wording; RecordContext per
  cassette.rs's shape (read at implementation). Replay stays the default everywhere (replay.rs
  re-executes via run::execute → never records). Reading: dispatch.rs, model.rs adapter surface,
  cassette.rs RecordContext, registry prompt/schema loaders. Gate: cargo test; default-replay
  acceptance test (record=false constructs no adapter); the Record arm stays type-enforced thin
  delegation (memory stage-model-fill) — live exercise = run-m2.2.
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
