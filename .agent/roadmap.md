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
- [ ] run-m2.1d5a-1: model-route loop in `execute()` + per-route landing gate (unified tails +
  trace-parse gate = .1d5a-2; two-run + census = .1d5b). Split from .1d5a: the loop lands per-route
  artifacts (self-contained, gate-verifiable); the cross-route unified tails carry the design
  uncertainty (source-node dedup, GroupTrace-from-route) → .1d5a-2. DELETE the old gate test
  `m2_experiment_run_gates_until_the_route_loop_lands` (≈3244). DISPATCH after `resolve()` (edit the
  `execute()` M1-view branch, run.rs ≈158-250): `views.len()==1 && views[0].shape==M1Layered` → keep
  the existing M1 path inline verbatim; else `views.iter().any(|v| v.shape==M1Layered)` → mixed → one
  command diagnostic (`shell.diagnostic(invalid_diagnostic(vec![(static_id("reason"), …)]))`), zero
  artifacts, return; else all-model → new fn `execute_routes(root, &views, shell)` (mostly fresh
  orchestration over already-built route fns + the one `direct_smt_fill` landing-dir change per
  CROSS-ROUTE LANDING below — no broad behavior-lock refactor). `execute_routes`: mirror the M1 lexicon
  read → `(lexicon, lexicon_hash)` (command diagnostic on failure); `store = CassetteStore::new(root)`
  (INFALLIBLE, no `?` — points at `<root>/cassettes/`, so the roadmap's "each failure" applies to
  lexicon + `Z3Adapter::new()` only); `adapter = Z3Adapter::new()` (Result → command diagnostic on
  Err); `seed = views[0].plan.seed` (plan shared across the set; = experiment.seed). Per view in set
  order: `ledger_start = shell.ledger().len()`; `repair_limit = resolved.repair_limit.expect("model
  route resolves Some")`.
  - CONFIRMED call sequence (from `single_ir_route_scores_m1_groups` ≈3807 + `direct_smt_route_scores_m1_groups` ≈4953 + `route_metrics_score_recorded_two_route_run` ≈5473 — do NOT re-read these; the group-landing DIR shown is the route-namespaced TARGET per CROSS-ROUTE LANDING below, NOT the scores-test literal `groups/{gid}` — those tests run one route so never namespace):
    SingleIr → `for entry in &resolved.documents`: `let head = route_document_head(root, entry,
    resolved, shell)` (None → skip doc, diagnostic already raised); `let rd = single_ir_fill(head,
    &lexicon, &store, seed, resolved, repair_limit, shell)`; fold `rd.identity` (Option<ModelIdentity>)
    into the agreement, extend `fills` with `rd.fill` (Option<FillObservation> → 0/1; `fills: Vec<FillObservation>`, push the inner value not the Option), `bundles[entry.id] =
    rd.trace.bundle` (Option<ArtifactWrapper<IrBundle>>). Then `for group in &resolved.groups`:
    `members = group.test_sources → bundles[s]` (skip group on any None member); `let (compiled,
    results) = compile_verify_group(&group.group_id, &format!("routes/{}/groups/{}",
    resolved.pipeline_id, group.group_id), &members, processing_stage_clock(), resolved, &adapter,
    shell)`; when both Some push `GroupObservation{group_id, query_pairs =
    compiled.payload.solver_query_plan.iter().map(|p| (p.context_overlap_query_id.clone(),
    p.deontic_consistency_query_id.clone())), results: results.payload.results.clone()}`.
    DirectSmt → head prepass: unique member ids first-appearance-ordered across `resolved.groups`,
    `route_document_head` each → `BTreeMap<Id, DocHead>`. Then `for group in &resolved.groups`:
    head_refs = members' heads; `let df = direct_smt_fill(&gid, &head_refs, &store, seed, resolved,
    repair_limit, shell)`; `fills.extend(df.fills)`, fold `df.identities`; `if let Some((overlap,
    deontic)) = df.pair { let results = direct_smt_verify_group(&gid, &format!("routes/{}/groups/{}",
    pid, gid), &overlap, &deontic, resolved, &adapter, shell); when Some push GroupObservation{group_id,
    query_pairs: vec![(static_id(&format!("{gid}.overlap")), static_id(&format!("{gid}.deontic")))],
    results: results.payload.results.clone()} }`.
  - CROSS-ROUTE LANDING (Codex .1d5a review — the loop's first design decision; the respec missed it):
    every existing route test runs ONE route into its OWN out (the two-route metrics test opens a fresh
    tmpdir per arm) → bare `groups/{gid}` never collides. `execute_routes` runs BOTH routes through ONE
    shell/out: heads ARE route-namespaced (`route_document_head` → `routes/{pid}/artifacts/{doc}`,
    run.rs:885) but group artifacts are NOT — the scores tests pass `dir = groups/{gid}` (run.rs:3867,
    5039), so single_ir (`{dir}/{compiled,verifier_results}.json`) and direct
    (`{dir}/verifier_results.json`) BOTH write `groups/{gid}/verifier_results.json` → COLLISION (land
    overwrites, or fail-closed errors → breaks the gate's "zero diagnostics" + honest scoring).
    RESOLUTION (recommended; mirrors the head namespacing): land every route's group artifacts under
    `routes/{pid}/groups/{gid}`. single_ir's `compile_verify_group` + both `*_verify_group` take `dir` →
    pass `format!("routes/{}/groups/{}", resolved.pipeline_id, gid)` (the DIR the template shows). BUT
    `direct_smt_fill` hard-codes `let dir = format!("groups/{gid}")` (run.rs:1249), applying its
    `route_id_prefix` to the artifact ID only, not the path → change its dir too (it holds `resolved`).
    Then UPDATE the committed `.1d4b` direct landing pin (≈4653, asserts bare `groups/{gid}`) + any
    scores-test layout assertion to the namespaced path, and AUDIT every per-route landing dir
    (single_ir_fill's bundle too) for the same gap. This adds a `direct_smt_fill` landing change + pin
    updates → .1d5a-1 is NOT pure orchestration (correct the framing). CONFIRM the namespacing choice
    before coding (alternative: a separate out-subtree per route, but the baked `routes/{pid}/artifacts/`
    head path assumes one shared out → namespacing the group dir is the smaller change).
  - SIGNATURES (banked; re-read a body only on a call mismatch): `route_document_head(root:&Path,
    entry:&CorpusEntry, resolved:&Resolved, shell) -> Option<DocHead>`; `single_ir_fill(head:DocHead,
    lexicon:&Lexicon, store:&CassetteStore, seed:u64, resolved:&Resolved, repair_limit:u32, shell) ->
    RouteDoc`; `compile_verify_group(group_id:&Id, dir:&str, members:&[&ArtifactWrapper<IrBundle>],
    clock:ProcessingStageClock, resolved:&Resolved, adapter:&Z3Adapter, shell) ->
    (Option<ArtifactWrapper<CompiledArtifact>>, Option<ArtifactWrapper<VerifierResults>>)`;
    `direct_smt_fill(gid:&Id, heads:&[&DocHead], store, seed:u64, resolved, repair_limit:u32, shell) ->
    DirectFill`; `direct_smt_verify_group(gid:&Id, dir:&str, overlap:&ArtifactWrapper<QueryBody>,
    deontic:&ArtifactWrapper<QueryBody>, resolved, adapter:&Z3Adapter, shell) ->
    Option<ArtifactWrapper<VerifierResults>>`. Types: `DocHead{trace:DocTrace,
    source:ArtifactWrapper<SourceDocumentGraph>, segments}`; `RouteDoc{trace:DocTrace (.bundle:
    Option<ArtifactWrapper<IrBundle>>), graph, fill:Option<FillObservation>,
    identity:Option<ModelIdentity>}`; `DirectFill{pair:Option<(ArtifactWrapper<QueryBody>,
    ArtifactWrapper<QueryBody>)>, fills:Vec<FillObservation>, identities:Vec<ModelIdentity>}`. Imports:
    `use crate::metrics::{FillObservation, GroupObservation};`.
  - Identity agreement: fold each Some identity into `agreed: Option<ModelIdentity>` — first Some sets
    it, a later DIFFERING Some → one command diagnostic + fail-closed `return` (no partial run). Goldens
    agree (model.baseline/fixture_quant/1.0.0) so the clean gate never trips it.
  - `RouteRun{pipeline_id: resolved.pipeline_id.clone(), ledger: shell.ledger()[ledger_start..].to_vec()
    (Vec<DiagnosticRecord>), fills, groups: Vec<GroupObservation>, samples: vec![groups.clone()]}` —
    #[allow(dead_code)] struct; collect a local `Vec<RouteRun>` (bind + `let _ = &route_runs;` with a
    "// .1e metrics + .1d5a-2 tails consume" note so no unused warning). No unified tails here (M2 path
    lands per-route artifacts only; run-level trace/report = .1d5a-2).
  - Gate: NEW test — `write_m2_root(&Path)` mirrors `write_tiny_root` (≈2630) + `copy_committed_registry`
    (≈2745): copy from `repo_root()` registry/*.yaml, corpus lexicon + 3 html + `exp.m1_scaffold`
    expected_outcomes yaml, rust-toolchain.toml, Cargo.lock; plant the 7 golden seed-42 cassettes
    (`tests/fixtures/cassettes/route.single_ir/{test_source.m1_control,m1_guideline_a,m1_guideline_b}` +
    `route.direct_smt/{group.m1_conflict,group.m1_no_conflict}.{overlap,deontic}`) under
    `<root>/cassettes/`. `executed(&m2_root, "exp.m2_multihop")` (reads inputs from root, owns its tmp
    out): zero command diagnostics; both routes' per-route layout present under `out/routes/
    pipe.m2_single_ir/` + `out/routes/pipe.m2_direct_smt/` — heads under `…/artifacts/{doc}`, group
    artifacts under `…/groups/{gid}/` (single_ir `{compiled,verifier_results}.json`; direct
    `{overlap,deontic}.smt_query.json` + `verifier_results.json`), and NO bare `out/groups/` (the
    CROSS-ROUTE LANDING namespacing moved them). M1 `executed()` pins
    unchanged (M1 path untouched). Gate: cargo test.
  - READ (apply-anchor set; the banked SOURCES above EXCLUDED): `execute()` 158-250 (dispatch edit site +
    M1 lexicon-read to mirror); `write_tiny_root` 2630-2730 + `copy_committed_registry` 2745-2767;
    `Resolved` fields (documents:Vec<CorpusEntry>, groups:Vec<TestSourceGroup>, plan.seed, repair_limit,
    pipeline_id, shape). Do NOT re-read the route_scores tests nor the route fn bodies.
- [ ] run-m2.1d5a-2: unified run tails over both routes + trace-parse gate. `trace_processing_stage`
  (≈1506) + `report_processing_stage` (≈1612) gain a trailing `emit_event: bool` gating the §4.6
  census EVENT only — wrap `shell.processing_stage_event(ProcessingStageEvent{…})` in `if emit_event`,
  but on the `landed` Err branch STILL deliver the failure diagnostic to the ledger (e.g.
  `shell.diagnostic(diagnostic)`) even when `emit_event==false`: that diagnostic reaches the shell ONLY
  through this call's `diagnostics` field (no separate raise — run.rs:1567/1706), so gating the whole
  call fail-OPENs M2 trace/report (a failed tail → run still Ok, no diagnostic). Keep M2 fail-CLOSED;
  suppress only the clean-path census event. `emit_event` (runtime bool) → the event-only locals
  started_at/outcome/… stay conditionally-used, no unused warning; return `pair` regardless for trace.
  M1 execute() callers pass `true` (event census + landed bytes unchanged); the
  M2 tails pass `false` (trace/report = undeclared route-pipeline steps; wrapper producer stays
  `producer(baseline_resolved, TRACE|REPORT)` → step id UNUSED_STAGE since direct_smt pads slots 6/7,
  honest sentinel). After the route loop in `execute_routes`, augment the loop to collect `docs` (all
  routes' DocTraces: single_ir `rd.trace`, direct `head.trace`), `graphs` DEDUPED by document_id (both
  routes' source graphs payload-identical → content_hash equal, keep one/doc), and per-route GroupTraces
  built from the stashed (compiled, results); then run ONCE: `trace_processing_stage(&docs, &groups,
  baseline_resolved, shell, false)` → (bundle, lineage); `report_processing_stage(root, &docs, &graphs,
  &groups, &bundle, &lineage, &lexicon_hash, &adapter.identity(), baseline_resolved, shell, false)` with
  the assemble_report `sections=None` (.1e populates). `baseline_resolved` = the view whose `is_baseline`
  holds (= the direct_smt pipeline). Gate: EXTEND the .1d5a-1 test — `out/trace_bundle.json`
  strict-parses; claims come from single_ir's 2 groups ONLY — assemble_trace mints finding.{gid}.{seq}
  per verifier_result (NOT per group) for compiled+verifier_results groups, so `bundle.claims.len()==3`
  (m1_conflict + m1_no_conflict; assert 3, NOT 2 — cf. the M1 trace test trace.rs:2509); direct lands no
  compiled → mints none → no id collision, tails filter nothing; shared source nodes appear once (graphs
  deduped); direct verifier_results = legal node
  carrying only its →report out-edge (no verify in-edge — validate checks edge endpoints/rank/op, never
  node connectivity). M1 executed() pins unchanged. Gate: cargo test.
  - READ (the design reads .1d5a-1 deferred — this unit owns them): `assemble_trace` (trace.rs ≈613)
    node/finding minting (source-node dedup + finding-per-compiled-group + id formation); `GroupTrace`
    shape + how M1 `group_pipeline` (≈666) builds it from compiled+results (mirror for routes);
    `route_document_head` body (≈868) — does it route-mint the source-graph id (the "shared source nodes
    once" question, reconcile with graph dedup); the emit_event edit sites (trace final event 1506-1611,
    report final event 1612-1739, the 2 M1 callers in `execute`). `route_id_prefix` = `""` for M1, else
    `"{pipeline_id}."` (banked).
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
