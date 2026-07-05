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
- [ ] run-m2.1d5a-2: unified run tails over both routes + trace-parse gate (respec of .1d5a-2 —
  tails-only half; error-path battery split to .1d5a-2b, seam confirmed). Add a trailing `emit_event:
  bool` to `trace_processing_stage` (run.rs ≈1799) + `report_processing_stage` (≈1905) gating the §4.6
  census EVENT only: gate the single final `shell.processing_stage_event(ProcessingStageEvent{…})` (after
  `clock.stop()`) in `if emit_event`; in the `match landed` `Err(diagnostic)` arm, when `!emit_event`
  ALSO `shell.diagnostic(diagnostic.clone())` — the diagnostic reaches the shell ONLY through the event's
  `diagnostics` field (no separate raise), so gating the whole call would fail-OPEN M2 (a failed tail →
  run still Ok, no diagnostic). Keep M2 fail-CLOSED. The runtime `emit_event` keeps
  started_at/outcome/output_hashes conditionally-used (no unused warning); trace returns `pair` unchanged.
  execute() (run.rs:163) tail callers (~245/250) append `true` (M1 census + landed bytes byte-identical);
  the M2 tails pass `false` (trace/report = undeclared route-pipeline steps; producer stays
  `producer(baseline_resolved, TRACE|REPORT)`, step id UNUSED_STAGE since a route pads slots 6/7 — honest
  sentinel). In `execute_routes` (run.rs:315): un-underscore `_lexicon_hash`→`lexicon_hash`; before `for
  resolved in views` add outer vecs `all_docs: Vec<DocTrace>`, `all_graphs:
  Vec<ArtifactWrapper<SourceDocumentGraph>>`, `all_group_traces: Vec<GroupTrace>`. single_ir arm: CLONE
  `rd.trace.bundle` into the `bundles` map (do NOT move — rd.trace keeps its bundle for the trace), then
  push `rd.graph`→all_graphs + `rd.trace`→all_docs; in the group loop capture `members.iter().map(|m|
  m.artifact_id.clone())` as `member_bundles`, keep the `(compiled, results)` wrappers, build
  GroupObservation from `&compiled`/`&results` refs FIRST, then push `GroupTrace{group_id, test_sources,
  member_bundles, dir, compiled, verifier_results: results}`. direct arm: build GroupObservation from
  `results.payload` FIRST, then push `GroupTrace{group_id: gid, test_sources: group.test_sources.clone(),
  member_bundles: Vec::new(), dir, compiled: None, verifier_results: Some(results)}`; AFTER the group loop
  drain `heads` — `for (_, head) in heads { all_graphs.push(head.source); all_docs.push(head.trace); }`.
  After the `for resolved in views` loop (after `let _ = &route_runs;`): dedup all_graphs by
  `g.payload.document.document_id` (keep first/doc — payload-identical across routes → content_hash
  equal); STABLE-SORT all_docs bundle-bearing FIRST — `all_docs.sort_by_key(|d| d.bundle.is_none())` —
  because assemble_trace's lineage lookup (trace.rs:778) `docs.iter().find(|d| d.document_id ==
  *ts).and_then(|d| d.bundle.as_ref())` takes the FIRST doc by id, and views run direct-FIRST
  (`pipelines:[direct,single_ir]`) so direct's `bundle:None` heads would shadow single_ir's `Some` → the
  lineage row is skipped → assemble_report rejects the claim `MissingLineage` (report.rs:700); the stable
  sort keeps BOTH routes' parallel chain nodes (the trace needs them) while surfacing the bundle-bearing
  doc for lineage. `let baseline_resolved = views.iter().find(|v| v.is_baseline).expect(...)`; `let Some((bundle, lineage)) =
  trace_processing_stage(&all_docs, &all_group_traces, baseline_resolved, shell, false) else { return;
  };`; `report_processing_stage(root, &all_docs, &all_graphs, &all_group_traces, &bundle, &lineage,
  &lexicon_hash, adapter.identity(), baseline_resolved, shell, false)` (assemble_report `sections=None`;
  .1e populates). CONFIRMED (do NOT re-read the sources): tails land run-level at the BARE run root
  (`trace_bundle.json`/`lineage_index.json`/`report.json`/`report_en.md`/`manifest.json`/
  `replay_manifest.json`), beside `routes/`+`logs/`. assemble_trace dedups the SOURCE node via `if
  !nodes.contains(&source)` (whole-node eq); chain nodes carry route-PREFIXED artifact_ids → distinct
  across routes, parallel chains, no DuplicateNodeId; findings mint ONLY under `if let Some(compiled)` →
  single_ir's 2 groups only → claims.len()==3. CONFIRMED acceptable (was a flagged risk):
  report_processing_stage collects `results` = ALL 4 verifier_results (single_ir 2 + direct 2);
  assemble_report indexes verifier rows by `query_id` (distinct: single_ir `q…` vs direct
  `{gid}.{overlap,deontic}`), rejects only duplicate ids, never validates group count → 4-over-2-groups
  is fine. The real cross-route hazard was the all_docs ordering (fixed above). Gate: EXTEND
  `m2_route_loop_lands_both_routes_namespaced` (run.rs:3581, which asserts `listing(routes/…)` +
  `!out.join("groups").exists()`, NOT a bare `listing(out)` → tail artifacts at root do NOT break it):
  add `strict_at::<TraceBundle>(out, "trace_bundle.json")` asserting `bundle.claims.len()==3` + existence
  of report.json/report_en.md/lineage_index.json/manifest.json/replay_manifest.json at root. M1
  executed() pins unchanged. Banked shapes (do NOT re-read): DocTrace{document_id, test_source_path,
  source_hash, dir, source_document_graph: Option<(Id,Hash)>, segments, normalization, bundle:
  Option<ArtifactWrapper<IrBundle>>}; GroupTrace{group_id, test_sources: Vec<Id>, member_bundles:
  Vec<Id>, dir: String, compiled: Option<ArtifactWrapper<CompiledArtifact>>, verifier_results:
  Option<ArtifactWrapper<VerifierResults>>}; RouteDoc{trace,graph,fill,identity}; DocHead{trace, source:
  ArtifactWrapper<SourceDocumentGraph>, segments}; SourceDocumentGraph.document: SourceDocument (source id=`.document.document_id`; source_linkage.rs:570);
  single_ir_fill(head,&lexicon,&store,seed,resolved,repair_limit,shell)->RouteDoc;
  route_document_head(root,entry,resolved,shell)->Option<DocHead>; executed(root,exp)->(
  TotalOperationResult, Vec<EventRecord>, Vec<DiagnosticRecord>, PathBuf, TempDir);
  strict_at::<P>(out,rel)->ArtifactWrapper<P>; route_id_prefix=`""` M1 / `"{pipeline_id}."` route;
  route_group_dir(resolved,gid)=`routes/{pid}/groups/{gid}`. READ FRESH only the Edit-anchor spans right
  before editing (trace/report fn tails ~1872/~2006, execute() callers ~245, execute_routes arms
  ~365-510, gate test 3581+). Gate: cargo test -p ckc-cli; if docs touched, `RUSTDOCFLAGS='-D warnings'
  cargo doc -p ckc-cli --no-deps`; fmt + clippy.
- [ ] run-m2.1d5a-2b: error-path pin battery over the loop's already-landed branches (from .1d5a-1 Codex
  review finding 3 — INDEPENDENT of .1d5a-2's tails wiring; these pin .1d5a-1's dispatch/loop branches).
  Three `#[test]`s over the run binary via `executed()`, each a crafted `write_m2_root` variant: (a)
  single_ir member-short group → assert BOTH diagnostics a dropped member raises. Drop guideline_b's
  cassette (`cassettes/route.single_ir/test_source.m1_guideline_b/seed-42.json`) — guideline_b ∈ ONLY
  group.m1_conflict, so exactly ONE group shorts (dropping guideline_a shorts BOTH: it's in m1_conflict
  AND m1_no_conflict, per registry/experiments.yaml). The dropped cassette makes single_ir_fill's
  model_fill Err → a COMMAND diagnostic `invalid_diagnostic{cassette,reason,processing_stage:"model_fill"}`
  (run.rs:1327); the member's bundle stays absent → group.m1_conflict goes member-short → ONE
  partial-group COMPILE diagnostic+event `processing_stage_diagnostic(COMPILE,"group","group.m1_conflict",
  "member … landed no ir_bundle artifact")` (run.rs:400). Assert the fill command diagnostic AND the
  single compile diagnostic+event (they CO-OCCUR — a compile short is always preceded by the member's own
  upstream fill diagnostic, per the loop comment); m1_no_conflict compiles clean (guideline_a+control both
  fill); direct route unaffected (its cassettes key by group, not source). (b) mixed-shape `[M1Layered, single_ir]` → ONE
  command diagnostic ("mixes the layered M1 pipeline with model routes") + zero artifacts
  (`assert_only_logs(out)`, run.rs:2534) + Outcome::Invalid — craft a registry variant binding BOTH the
  M1 layered pipeline AND a single_ir pipeline in one experiment (overwrite `registry/experiments.yaml`
  after `copy_committed_registry`, run.rs:3038); hits execute()'s dispatch (run.rs:171
  `views.iter().any(|v| v.shape==RouteShape::M1Layered)` with len>1). (c) identity-disagreement →
  fail-closed, ONE command diagnostic ("model routes disagree on the model identity attesting the run") +
  Outcome::Invalid — craft a variant re-blessing ONE cassette with a DIVERGENT synthetic ModelIdentity
  (model.other / fixture_quant / 1.0.0 — crafted-fixture rule, no real engine/quant/format token) so
  `agree_model_identity` (run.rs:274) trips on the second differing Some. Separate failing-input
  fixtures → .1d5b's clean-path census untouched. Banked infra: write_m2_root (run.rs:3540) copies
  copy_committed_registry + LOCKFILE + LEXICON + 3 corpus html + reference + 7 cassettes (3 single_ir
  `route.single_ir/<source>/seed-42.json`, 4 direct `route.direct_smt/<gid>.{overlap,deontic}/seed-42.json`)
  from crates/ckc-cli/tests/fixtures; a variant = a modified copy omitting/overwriting the target.
  Cassette build:
  `CassettePayload::from_output(route,source,seed,prompt,constraint_hash,template_hash,ModelIdentity{model_id,quant,runtime_version},output)`
  + `store.build_wrapper(&key,payload,producer(resolved,2))` + `store.persist(&key,wrapper)` (see
  `write_single_ir_cassette`). READ FRESH: committed registry/experiments.yaml + registry/candidates.yaml
  (for the mixed binding), write_m2_root + write_single_ir_cassette + assert_only_logs bodies. Gate: cargo
  test -p ckc-cli.
- [ ] run-m2.1d5b: two-run determinism + event census over .1d5a-1's write_m2_root mirror, after
  .1d5a-2's tails land (split from .1d5 — the pin-battery half). Execute twice into two out dirs: landed artifacts byte-equal
  across runs; manifests byte-equal after normalizing the one `--out` token (manifest_inputs ≈1589
  embeds out_dir.display()); events compared on a non-timing projection; event census = 27
  (single_ir 3×4+2×2=16 + direct 3×2+2×2=10 + 1 command, tails none; separate M1 baseline run
  stays 19) with model_fill counters (single_ir per doc 1/0; direct per group 2/0). Census +
  counters pinned from OBSERVED output sanity-checked against the .1d3a/.1d4a contracts. M1
  executed() pins unchanged. Gate: cargo test.
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
  hole, so omission is not an option). Sections consume .1d5a's
  banked RouteRun{pipeline_id, ledger slice, fills, groups, samples}. Reading: run.rs
  report/manifest fns; report.rs ModelRunSections + populated fixture; manifests.rs; replay.rs +
  trace.rs GroupTrace for the smt_query decision. Gate: cargo test; full-run test pins report.json
  M2 sections + both md bodies + manifest fields on the .1d5a replayed run; M1 pins hold.
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
