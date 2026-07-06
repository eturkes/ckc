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
- [x] run-m2.1d3a: single_ir stage rework — DocHead/RouteDoc + route_document_head landing,
  direct-emitted model_fill §4.6 event (§7.3 counters, event-only diagnostics), route-prefixed
  wrapper ids, slot-3 fail-closed tail; M1 pins untouched. 85% 169K/200K 73f3c87
- [x] run-m2.1d3b: single_ir §4.6 event + landed-layout pin battery over the reproduce-M1
  test. 66% 132K/200K
- [x] run-m2.1d4a: direct route stage — direct_smt_fill consumes member DocHeads, lands each
  accepted body as a raw-AI smt_query under `groups/{gid}`, direct-emits one model_fill §4.6
  event; DirectFill{pair,fills,identities} + direct_fill_group test bridge; 6 call sites
  migrated; route prefix on smt_query/verifier_results ids. 88% 176K/200K
- [x] run-m2.1d4b: direct event + landing pin battery — reproduce/scores tests swapped onto a
  per-route unique-`DocHead` prepass (shared guideline_a heads once); pin per-group model_fill §4.6 event
  tuple (kind/step/counters summed over roles/output set) + landed `groups/{gid}/{role}.smt_query.json`
  layout + once-per-route head-event census. 77% 153K/200K
- [x] run-m2.1d5a-1: model-route loop in execute() — 3-way dispatch + execute_routes over both routes + cross-route group namespacing (routes/{pid}/groups/{gid}) + per-route landing gate; RouteRun banked for .1d5a-2 tails/.1e metrics. 88% 177K/200K
- [x] run-m2.1d5a-2: unified run-level tails over both routes (run-root trace/lineage/report + EN render + manifests); `emit_event: bool` gates the §4.6 event (M1 `true`, M2 tails `false`, fail-closed via direct `shell.diagnostic`); all_graphs seen-set dedup + all_docs bundle-first sort; landing gate pins the 8-entry root layout + `claims.len()==3`. 80% 160K/200K
- [x] run-m2.1d5a-2b: 3 run-binary error-path pins over write_m2_root variants — single_ir member-short group (dropped guideline_b cassette → fill + partial-group compile diagnostics co-occur, order fill<compile), mixed-shape [M1Layered, single_ir] → command diagnostic + assert_only_logs, model-identity disagreement → fail-closed command diagnostic. 81% 161K/200K
- [x] run-m2.1d5b: two determinism/census pins over write_m2_root's replayed model-route run — landed artifacts byte-equal across two runs, both manifests equal modulo the one `--out` token (land_record = plain canonical, no self-hash), events equal on their non-timing projection (only started_at/ended_at/duration_ms differ; event_id/seq slot-derived); §4.6 census 27 (single_ir 16 + direct 10 + 1 command; run-level tails emit_event=false → 0) + model_fill counters (single_ir 1/0 per doc, direct 2/0 per group), M1 baseline 19. 62% 123K/200K
run-m2.1e RESPEC (seam confirmation read run.rs+manifests.rs+report.rs+trace.rs — margin spent, so this session closes at the respec per the land-or-revert rule; nothing written): the single unit bundled 6-7 deliverables (GroupTrace landing-completeness + manifest output walk + run-level producer + manifest §9 fields + report ModelRunSections + report_ja + a full-run pin battery) across four large modules → oversized on both deliverable-count and read-cost (record-shape extension = its own unit; report/manifest each carry their own integration pins). Split at three confirmed seams, ordered A (foundational, no threading) → B (manifest §9, lands the agreed-identity + reference/registry file-read threading) → C (report sections, reuses B's threading). No new sizing RULE minted (applies the standing record-shape-extension + read-cost axioms), so no re-audit of run-m2.1f/2.2/acceptance. Facts below are source-read + banked (do not re-read the enumerated sources).
- [ ] run-m2.1e-A: run-level landing completeness + honest producer. Close the .1d5a-2-flagged replay hole: the direct route's overlap/deontic smt_query pair is landed (`direct_smt_fill`) yet never in `output_hashes`, and both routes push GroupTrace only on verify-success `(Some,Some)`, so a landed-but-unverified compiled/pair vanishes → replay-uncovered. FACTS: `GroupTrace` (trace.rs:561) = `{group_id:Id, test_sources:Vec<Id>, member_bundles:Vec<Id>, dir:String, compiled:Option<ArtifactWrapper<CompiledArtifact>>, verifier_results:Option<ArtifactWrapper<VerifierResults>>}` — a NON-Canonical intermediate consumed by `assemble_trace` + `report_processing_stage` + `manifest_inputs` → add `smt_queries: Vec<ArtifactWrapper<QueryBody>>` (single_ir empty; direct `[overlap,deontic]`); `DirectFill.pair` (run.rs:1571) = `Option<(ArtifactWrapper<QueryBody>,ArtifactWrapper<QueryBody>)>`. DECISION: manifest-only — do NOT emit smt_query trace nodes → `assemble_trace` UNCHANGED, trace_bundle.json bytes stable. EDITS: (1) add the field at every GroupTrace literal — trace.rs:1769/1883/1903/2033/2082/2361 (tests) + run.rs:1050 (`compile_verify_group`) + the two push sites; (2) single_ir push (run.rs ~455-475, currently GroupObservation+GroupTrace both inside `if let (Some(compiled),Some(results))`) → push GroupTrace{smt_queries:vec![], compiled:Some, verifier_results:results} when compiled lands, gate only GroupObservation on results; (3) direct push (run.rs ~519-545, `if let Some((overlap,deontic))=df.pair { …verify borrows &overlap/&deontic… if let Some(results)=direct_smt_verify_group(&gid,&dir,&overlap,&deontic,…){GroupObservation+GroupTrace}}`) → after the verify borrow, push GroupTrace{smt_queries:vec![overlap,deontic], compiled:None, verifier_results:results} on the pair landing (move the wrappers in post-borrow), gate GroupObservation on results; (4) `manifest_inputs` (run.rs:2140, walk ~2166) add `output_hashes.extend(group.smt_queries.iter().map(|q| q.content_hash.clone()))`; (5) honest producer — `producer(resolved,idx)` (run.rs:2505) reads `resolved.pipeline_step_ids[idx]`=UNUSED_STAGE for the M2 tails (run.rs:589 trace / 598 report, `emit_event:false`) → add a run-level step-id const + stamp it when `emit_event==false` (M1 `emit_event:true` keeps `resolved.pipeline_step_ids[stage]`); `producer.pipeline_step_id` is content_hash-EXCLUDED provenance → re-blesses only emitted trace_bundle/lineage_index/report.json wrapper bytes for M2, never hashes/layout/census/determinism. Read: replay.rs (the `expected_output_hashes` coverage/replay-hole mechanism); `write_m2_root` test (run.rs:3647) + its manifest/output_hashes assertions. Gate: fmt/clippy/`RUSTDOCFLAGS='-D warnings' cargo doc -p ckc-cli --no-deps`/cargo test; new pin — the two direct smt_query content_hashes appear in `write_m2_root`'s manifest.json output_hashes (replay-covered); M1 byte-identical; .1d5b determinism holds (both runs gain the two hashes); re-bless any M2 output_hashes pin from observed.
- [ ] run-m2.1e-B: manifest §9 measurement fields. `ManifestInputs` + `assemble_manifests` + `manifest_inputs` gain the §9 omittable set, populated on model routes, None on M1. FACTS (manifests.rs fully read — do not re-read): `RunManifest`/`ReplayManifest` ALREADY carry `model_identity`/`test_source_hash`/`reference_hash`/`schema_hash`/`prompt_template_hash`/`model_hash`/`runtime_hash` (M2.1 model-types); `assemble_manifests` (manifests.rs:53) hardcodes them `None` in BOTH record literals. EDITS: (1) `ManifestInputs` (manifests.rs:20) add the 7 `Option` fields (import `ModelIdentity`); (2) `assemble_manifests` replace the two hardcoded-None blocks with `inputs.<field>.clone()` (both records same values — shared provenance); (3) `manifest_inputs` (run.rs:2140) populate for M2: `model_identity`=agreed `ModelIdentity`; `test_source_hash`=`hash_bytes` over `docs[].source_hash.as_str()` sorted + joined `\n`; `reference_hash`=`hash_bytes(read root.join(exp.expected_outcomes))`; `schema_hash`=`hash_bytes(read registry/schemas.yaml)`; `prompt_template_hash`=`hash_bytes(read registry/prompts.yaml)`; `model_hash`/`runtime_hash`=None; M1 (execute() tail run.rs:246) passes all None → M1 manifest byte-identical; (4) thread the §9 values (agreed identity + the file bytes) through `report_processing_stage` (run.rs:2001, the sole `manifest_inputs` caller at run.rs:2076) — introduce `agreed: Option<ModelIdentity>` param + the reference/registry file reads here; M1 tail call site (run.rs:246) passes None; load `exp` from the registry by `views[0].plan.experiment_id` (mirror `resolve()`); (5) manifests.rs fixtures — extend `inputs()` (~manifests.rs:230) with the 7 fields =None (keeps assembly_sorts/canonical/dup/empty pins byte-identical), ADD a populated fixture byte-pinning the §9 slots for BOTH `RunManifest` AND `ReplayManifest` (memory manifest-extension rule; codex once caught a missing replay pin). Read: run.rs `manifest_inputs` body (2140-2205) + `report_processing_stage` `manifest_inputs` call + the two tail call sites' threading; registry yaml path consts (grep registry.rs/registry_check.rs for the schemas/prompts file paths); `write_m2_root` manifest assertions. Gate: fmt/clippy/doc/cargo test; `write_m2_root` pins manifest.json+replay_manifest.json §9 fields from observed; M1 manifest pins byte-identical; manifests.rs all-None + populated (both records) byte pins. Lands the agreed-identity + reference/registry threading C reuses.
- [ ] run-m2.1e-C: report ModelRunSections + report_ja.md. `report_processing_stage` builds `ModelRunSections{route_diagnostics (per-route ledgers, clean route=empty slice), route_metrics (metrics::route_metrics per route — samples=the k=1 battery → convergence NA), baseline_pipeline_id, model_identity}` and passes `Some` to `assemble_report` (currently `None`); land report_ja.md beside report_en.md; `experiment_metrics` stays in-assembly (never called from run.rs). FACTS: `ModelRunSections<'a>` (report.rs:551) = `{route_diagnostics:&[(Id,&[DiagnosticRecord])], route_metrics:Vec<RouteMetrics>, baseline_pipeline_id:&Id, model_identity:&ModelIdentity}`; `assemble_report` 8th param `model_run:Option<ModelRunSections>` (report.rs:601, passed at the `assemble_report` call ~run.rs:2044). SOURCES: `RouteRun{pipeline_id,ledger,fills,groups,samples}` collected in `execute_routes` (run.rs, currently `let _ = &route_runs` ~568); `route_metrics(pipeline_id,fills,groups,samples,reference)->RouteMetrics` (metrics.rs); `baseline_pipeline_id`=`baseline_resolved.pipeline_id` (run.rs:589 `baseline_resolved`); `model_identity`=agreed (in scope, Option → `expect` on a model route); `route_diagnostics`=RouteRun.ledger's DiagnosticRecords keyed by pipeline_id; `reference`=`parse_reference(read root.join(exp.expected_outcomes))` (reuse B's `exp`/agreed threading); report_en landing = run.rs:2064 `write_under("report_en.md", render_markdown(&report.payload)…)` with write+read-back byte-check → add `render_markdown_ja` (report.rs:889) → `write_under("report_ja.md")` same discipline. EDITS: `report_processing_stage` gains route_runs + reference (reuse B's `agreed` thread), builds sections, passes `Some` at the `assemble_report` call, lands report_ja; both tail call sites update (run.rs:246 M1 `model_run=None` / run.rs:598 M2 `Some`). Read: report.rs `assemble_report` M2 population body (below :700 — failure_taxonomy/metrics/model_identity fill) + populated_report fixture helpers (`m2_route_metrics()`/`baseline_model_identity()` for expected values) + `render_markdown_ja` tests (report.rs:3552/3697 for JA discipline); metrics.rs `RouteMetrics` shape; run.rs `report_processing_stage` body (2001-2110) + `execute_routes` RouteRun/registry-load region. Gate: fmt/clippy/doc/cargo test; `write_m2_root` pins report.json M2 sections (failure_taxonomy/metrics/model_identity) + report_en.md + report_ja.md bodies from observed; M1 report pins hold (M1 path `model_run=None` → byte-identical).
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
