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
run-m2.1e RESPEC (seam confirmation read run.rs+manifests.rs+report.rs+trace.rs — margin spent, so this session closes at the respec per the land-or-revert rule; nothing written): the single unit bundled 6-7 deliverables (GroupTrace landing-completeness + manifest output walk + run-level producer + manifest §9 fields + report ModelRunSections + report_ja + a full-run pin battery) across four large modules → oversized on both deliverable-count and read-cost (record-shape extension = its own unit; report/manifest each carry their own integration pins). Split at three confirmed seams, ordered A (foundational, no threading) → B (manifest §9, lands the agreed-identity + reference/registry file-read threading) → C (report sections, reuses B's threading). No new sizing RULE minted (applies the standing record-shape-extension + read-cost axioms), so no re-audit of run-m2.1f/2.2/acceptance. Facts below are source-read + banked (do not re-read the enumerated sources). · B SUB-SPLIT: run-m2.1e-B overflowed read+write with the design LOCKED in-session (zero code) → split manifest §9 into B1 (record-shape PLUMBING: fields+assembly+fixtures/byte-pins) + B2 (computation+threading; itself later overflowed → B2a compute+fixture / B2b value-pins) at the manifests.rs/run.rs module seam (memory sizing bullet now carries the rule: a locked design ≠ one-window fit — read-to-place-edits over a large caller + byte-pin-test authoring alone overflow). B1 leaves the crate green with §9 plumbed-but-None (omit-None → M1 AND M2 manifest bytes byte-identical to pre-B); B2a/B2b fill the model-route computation. Their lines bank EVERY apply-anchor from a full read of manifests.rs + run.rs (`manifest_inputs` 2171 / `report_processing_stage` 2032 / tail sites 255+634) → implement WITHOUT re-reading the SOURCE modules (registry.rs, plans.rs, ModelIdentity). · C SUB-SPLIT (SIZE-CHECK at C start; 62% context after seam-confirmation reads spanning run.rs+report.rs+metrics.rs — margin spent, respec closes the session per land-or-revert): the pre-flagged report_ja/pin-battery seam fired — a 3-body byte-pin battery over the 7205-line run binary IS the flagged overflow pattern (B2 hit it twice), so split at the confirmed seam into C1 (sections-build wiring + report_ja landing, crate green — B2a-style compute-then-green) + C2 (the 3-body observed-output pin battery — B2b-style value-pins). No new sizing rule (applies the standing assembly-fn + pin-battery axiom), so no re-audit of run-m2.1f/2.2/acceptance. C1/C2 lines bank EVERY apply-anchor from the C-start reads → implement WITHOUT re-reading report.rs/metrics.rs/registry.rs sources.
- [x] run-m2.1e-A: run-level landing completeness + honest producer — `GroupTrace.smt_queries: Vec<ArtifactWrapper<QueryBody>>` (single_ir empty, direct `[overlap,deontic]`); both routes push GroupTrace on artifact LANDING not verify-success, gating only GroupObservation on `results` → a landed-but-unverified compiled/pair stays replay-covered; `manifest_inputs` walks `group.smt_queries` into output_hashes (manifest-only — `assemble_trace` UNCHANGED, no smt_query trace node; provenance rests on `verifier_results.input_hashes`). Honest producer: `tail_producer(resolved, idx, emit_event)` — false→run-level `processing_stage.run.{trace,report}` step-ids (consts RUN_TRACE_STEP/RUN_REPORT_STEP), true→early-returns `producer(resolved, idx)` (M1 byte-identical). Pin: direct overlap+deontic content_hashes appear in write_m2_root manifest.json output_hashes. 99% 197K/200K
- [x] run-m2.1e-B1: manifest §9 fields plumbed (record-shape extension, None everywhere). GOAL: the 7 §9 measurement fields flow `ManifestInputs`→both manifest records, None on EVERY run (byte-identical to today); a populated byte-pin locks the new slots. NO computation/threading (=B2). FACTS (manifests.rs + the run.rs `manifest_inputs` RETURN fully read this respec — do NOT re-read source modules): `RunManifest`/`ReplayManifest` ALREADY carry the 7 (M2.1 model-types), canonical key-sort lives in ckc-core emit → the manifests.rs record LITERALS already list them (currently `None`), so literal field ORDER is cosmetic (swap in place). The 7: `model_identity: Option<ModelIdentity>` then `test_source_hash`/`reference_hash`/`schema_hash`/`prompt_template_hash`/`model_hash`/`runtime_hash`: `Option<Hash>`. `ManifestInputs` is constructed at EXACTLY 2 sites (both edited here): manifests.rs `inputs()` (l.190, ends `output_hashes: vec![hash('7'), hash('2'), hash('7')],`) + run.rs `manifest_inputs` return (ends `output_hashes,` then `})`). Helpers: `id(&str)->Id`, `hash(char)->Hash` = `Hash::new(format!("sha256:{}", c.to_string().repeat(64)))`. EDITS: (1) manifests.rs l.12 import — add `ModelIdentity` into `use ckc_core::{CanonError, Hash, Id, ReplayManifest, …}`; (2) `ManifestInputs` struct (l.20; last field `output_hashes: Vec<Hash>` l.43) — append the 7 `Option` fields; (3) `assemble_manifests` — RunManifest literal (§9 comment l.81-82, `model_identity: None`..`runtime_hash: None` l.83-89) swap each `None`→`inputs.<field>.clone()`; ReplayManifest literal (§9 comment l.101, Nones l.102-108) same — KEEP both comments; anchor each Edit on its UNIQUE §9 comment (the 7-`None` run is identical between blocks); (4) fixture `inputs()` — append the 7 fields =`None` (keeps assembly_sorts/canonical/dup/empty pins byte-identical; the l.322 field-clear mutators list stays — `Option` fields have no empty-rejection); (5) run.rs STUB — `manifest_inputs` return (after `output_hashes,`) append the 7 fields =`None` (crate compiles, §9 `None` on ALL runs incl. M2 = unchanged from pre-B; NO signature/param change — that is B2); (6) ADD a populated-§9 byte-pin test — build a `ManifestInputs` with Some §9 (SYNTHETIC: `ModelIdentity{model_id: id("model.baseline"), quant: "fixture_quant".into(), runtime_version: "1.0.0".into()}`, 6 hashes = distinct seeds `hash('a')..hash('f')`), `assemble_manifests`, byte-pin the canonical bytes of BOTH `RunManifest` AND `ReplayManifest` (memory backward-compat rule — codex once caught a missing replay pin; the existing all-None round-trips + run.rs M1 manifest pins guard the omit-None old bytes). Read ONLY: manifests.rs (struct + `assemble_manifests` + `inputs()` + test module) + the run.rs `manifest_inputs` return literal. GATE (not live-runtime — pure record plumbing, no run-level behavior change): fmt/clippy/`RUSTDOCFLAGS='-D warnings' cargo doc -p ckc-cli --no-deps`/`cargo test -p ckc-cli`. Acceptance: §9 plumbed `ManifestInputs`→both records, `None` everywhere; existing pins byte-identical; populated byte-pins lock both records' new slots. 66% 132K/200K
- [x] run-m2.1e-B2a: §9 COMPUTED route-relevantly on model routes — `manifest_inputs` mode-gated (`model_routes` empty→7×None M1 / non-empty→`agreed.cloned()` honest identity + 4 hashes over THIS run's inputs: aggregate over docs / route-relevant schema+prompt entries / reference file), model_hash+runtime_hash None; `copy_committed_registry` provisions schemas+prompts (write_m2_root inherits); plans.rs §9 docs route-relevant. Existing tests green (M1 pins byte-identical, model-route §9 path computes without erroring). 82% 165K/200K
- [x] run-m2.1e-B2b: model-route §9 value pins (tests-only, bless-from-observed) — extend `m2_route_loop_lands_both_routes_namespaced` with RunManifest §9 asserts (agreed identity `model.baseline`/`fixture_quant`/`1.0.0`; model_hash+runtime_hash None; 4 hashes test_source/reference/schema/prompt_template blessed from the observed run, reference_hash cross-checked == raw sha256 of m1_expected.yaml) + ReplayManifest 7-tuple parity; M1 §9-None live-asserted in run_oracle.rs (codex follow-up — the manifests.rs `all_none_measurement_record_pins_both_manifests` byte-pin proves only the encoder, not the M1 mode-gate end-to-end; also added a `select_route_hashes` drop-case locking route-relevance vs whole-registry hashing). 53% 107K/200K
- [x] run-m2.1e-C1: report ModelRunSections wiring — model_route_metrics→assemble_report Some(sections) on a model route, None (no panic) on M1 or a no-identity degraded route; report_ja.md lands both paths in the shared stage; 3 file-set listing pins updated (M2 run.rs + 2 M1 tests); M1 5-file bytes byte-identical, M2 values deferred C2. 74% 149K/200K
- [x] run-m2.1e-C2: model-route report value pins (tests-only, bless-from-observed) — extend `m2_route_loop_lands_both_routes_namespaced` with report.json M2-section asserts (failure_taxonomy = both routes named with empty §7.4 code maps; metrics emission_order = raw rows for both routes before the lone single_ir delta + k_sample_convergence NA; agreed identity `model.baseline`/`fixture_quant`/`1.0.0`) + full report_en.md/report_ja.md body consts; the live z3 `--version` token normalized to `Z3_VERSION` under a once-per-body occurrence guard (env-independent). 77% 154K/200K
run-m2.1f RESPEC (SIZE-CHECK at f start: 57% context after a read-only seam scan spanning run.rs
execute_routes/single_ir_fill/direct_smt_fill/route_document_head/manifest_inputs + cassette.rs +
model.rs + model_fill.rs + dispatch.rs — margin spent, ZERO code, so this session closes at the
respec per land-or-revert): `ckc run --record` is a COMPUTATION+THREADING over the 7643-line run
binary (thread `record` dispatch→execute→execute_routes→both fills + build `FillSource::Record` from
registry-loaded prompt/constraint + adapter probe + all call-site updates) — the exact shape memory
flags overflow-prone (B2 overflowed it with the design LOCKED). Split at the confirmed
pure-composition/integration seam (standing "pure-computation core + integration = 2" axiom → no new
rule → no re-audit of run-m2.2/acceptance-m2): f1 = pure record-prompt selectors+composers
(inline-fixture tests, `#[allow(dead_code)]`, crate green) → f2 = dispatch `--record` + record-mode
threading consuming f1 (B2a-style compute-then-green; live exercise = run-m2.2). BANKED FACTS
(source-read this session — implement WITHOUT re-reading these sources; ≈lines drift, anchor on
NAMES): `FillSource::Record{adapter:&ModelAdapter, prompt:&str, constraint:&Path, ctx:&RecordContext}`
+ model_fill's Record arm ALREADY wired (model_fill.rs:55/157 → delegates to `store.record`) → the
units only BUILD the Record value at the fills. `RecordContext{producer:Producer,
prompt_template_hash:Hash, budget:Duration}` (cassette.rs:49). `CassetteStore::record(adapter, key,
prompt, constraint, ctx)` = subprocess + constraint re-read seal + persist (cassette.rs:101; committed
static schema files pass the seal). `ModelAdapter::new()->Result<Self,_>` probes identity on PATH /
`CKC_MODEL_COMMAND` (model.rs:105), `.identity()` (:163) — build ONCE in execute_routes mirroring
`Z3Adapter::new`'s build-once placement (~369), BUT the Err arm = command-scope `invalid_diagnostic`
(`reason` = `model adapter: {e}`), NOT `SolverExecutionFailure` (solver-specific; no model-runtime
DiagnosticCode exists → a dedicated code = later/spec concern). GUIDELINE-TEXT source = `SourceDocumentGraph.spans[].raw_text` ordered by
`reading_order` (source_linkage.rs `SourceTextSpan{span_id,node_id,raw_text,reading_order,…}`);
`ClinicalSegment`(segment_id/kind/region_ids) + `EvidenceRegion`(region_id/node_ids/span_ids/anchor_ids)
carry NO text → DECISION: the first-draft prompt joins spans' `raw_text` in `reading_order`
(full-document text); the segment→region_ids→span_ids→spans "segment order" mapping is a run-m2.2
wording refinement. Registry select mirrors `manifest_inputs` want-set (run.rs:2334-2450, selects by `s.id`/`p.id` — NOT
`target_kind`): SingleIr→schema id `schema.clinical_ir` / prompt id `prompt.single_ir`;
DirectSmt→`schema.smt_query` / `prompt.direct_smt`; M1Layered→None (manifest_inputs errs on it as a
caller-contract violation → f1 selectors return None defensively). `SchemaEntry{id,path,schema_hash,
target_kind}` (core registry.rs:206) — `target_kind` (`clinical_ir`/`smt_query`) is an OPEN classifier,
`id` is the select key; `PromptEntry{id,path:Option,inline:Option,template_hash,route}` path-XOR-inline
(:225); consts `SCHEMAS_FILE`/`PROMPTS_FILE` run.rs:95/99; `parse_schemas`/`parse_prompts` (core) do NO
validation → `validate_model_registry(&schemas,&prompts)` (core registry.rs:665; returns findings,
empty=valid; only `registry check` calls it today) enforces path-XOR + safe paths → f2 MUST call it
before any `root.join(path)`. Slot const `MODEL_FILL` + `producer(resolved, idx)` + `Resolved.budget_ms`
all live in run.rs (used by the fills); `budget_ms` is the §8.4 `solver_ms_per_query` SOLVER budget
(run.rs:687) → DECISION: f2 reuses it as the `RecordContext.budget` model-invocation placeholder
(first-draft), a dedicated model budget = run-m2.2 (record ships type-plumbing only, no live call).
- [x] run-m2.1f1: pure record-prompt selection+composition (ckc-cli run.rs, above `manifest_inputs`;
  `#[allow(dead_code)]` f2 pre-consumers, NO I/O/adapter/dispatch, crate green) —
  `select_record_{schema,prompt}(&[Entry], RouteShape)` key by `id.as_str()`
  (SingleIr→clinical_ir/single_ir, DirectSmt→smt_query/direct_smt, M1Layered→None, mirrors
  `manifest_inputs` want-set); `single_ir_prompt` = template ++ `document: <doc_id>` ++ spans,
  `direct_smt_prompt` = template ++ `group: <gid>` ++ `role: <role>` ++ per-member(doc-id ++ spans),
  spans by `reading_order` (shared `reading_order_text`), `\n`-joined (first-draft; f2 threads
  verbatim, run-m2.2 refines). Tests: selector hit/miss/M1Layered-None + each composer pins the EXACT
  string over a 2-span array-order≠reading_order graph (`SourceTextSpan::derive`); SchemaEntry/
  PromptEntry first literal builders in-crate. 84% 168K/200K
- [x] run-m2.1f2: dispatch `--record` + record-mode threading (consumes f1; B2a-style compute-then-green,
  Record arm type-enforced, live exercise = run-m2.2). dispatch.rs `take_bool_flag(op,name,args)`
  pre-partitions `--record` out of `rest` BEFORE `take_flags` (dup→`duplicate --record`; value-bearing `--record=x`→`--record takes no
  value`, rejected in `take_bool_flag` so the outcome is position-independent) → `RawCommand`/`Command::Run{…,record}` →
  `crate::run::execute(…, *record, shell)`; replay re-exec + both M1 callers thread `false`. run.rs
  threads `record` `execute`→`execute_routes`, which builds `RecordSetup{adapter,schemas,prompts}` ONCE
  `if record` else `None` (`build_record_setup`: `ModelAdapter::new()` Err→command-scope
  `invalid_diagnostic` reason=`model adapter: {e}`, NOT `SolverExecutionFailure`; loads
  SCHEMAS/PROMPTS_FILE + `validate_model_registry`, non-empty findings→diagnostic+return, guarding
  path-XOR/safe-paths before any join), then per-view `build_route_record` (f1
  `select_record_{schema,prompt}` → `template`=inline|read `root.join(path)`, `constraint`=
  `root.join(schema.path)`, `ctx`=`RecordContext{producer(resolved,MODEL_FILL), prompt.template_hash,
  budget=from_millis(budget_ms)}`; missing shape entry→command-scope+return). Both fills gain
  `record:Option<&RouteRecord>` (`single_ir_fill` +`#[allow(clippy::too_many_arguments)]`=8/7 mirroring
  `direct_smt_fill`): `None`→`FillSource::Replay`, `Some`→f1-composed prompt (single_ir
  `single_ir_prompt`, direct per-role `direct_smt_prompt` over heads→`(&document_id,&source.payload)`) +
  `FillSource::Record`. Tests: (6b) `run_record_flag_parses` direct `parse()` (no-flag→false / bare→true,
  flags intact / dup→reject / `--record=x`→reject); (6a) DEVIATION — the respec's
  `set_var(CKC_MODEL_COMMAND,bogus)` no-probe assert is void under `#![forbid(unsafe_code)]` → no-probe is
  STRUCTURAL (the `if record` gate → a replay run never constructs `ModelAdapter`), proven by threading
  `false` through the green `m2_route_loop_lands_both_routes_namespaced` (memory records the rule). Gate
  green: 286 pass/7 ignored, fmt/clippy clean, rustdoc 17 (baseline unchanged). 87% 175K/200K
run-m2.2 RESPEC (SIZE-CHECK fired pre-code: the committed templates PROMISE instrument-supplied
inputs — segment/region ids, lexicon vocabulary, `:named` label ids — that the f1 composers never
compose; honest recording REQUIRES that grounding scaffold in the prompt BEFORE any cassette is cut
(§9: route failures attribute to translation, not the instrument; cassettes embed prompt bytes →
wording changes after recording force a full re-record), and scaffold + env-wrapper grammar wiring +
live record + pin battery ≫ one window; seam-confirmation reads spent the margin → respec commit
closes the session per land-or-revert). NEW PLAN RULE (re-audited: acceptance-m2 + the three lines
below pass): a unit touching prompt templates must enumerate the supply MECHANISM for every input the
template promises — "wording refinement" hiding a composer redesign = plan bug. DECISIONS: direct
`:named` labels = NAMING-SCHEME supply, template wording ONLY, ROLE-SENSITIVE (codex catch: §8.6
reference queries + the ckc-smt artifact contract + trace.rs's per-role fallback all split the
prefix by role → template instructs overlap-query labels `ctx.<document-id>.rule.<n>`, deontic
`a.<document-id>.rule.<n>`, rules numbered 0,1,… in document order per document — a single `a.`
scheme would mis-prefix the overlap query against the instrument; exact label LISTS would import
IR-layer rule counts into the no-IR route — rejected); single_ir composer supplies per-segment lines (segment_id, kind, region_ids) + a
regions line + the lexicon vocabulary (system + concept_ids w/ interval var ids + action_ids;
enforcement stays compile-side — the schema constrains ids by grammar, wrong codes surface as compile
diagnostics = measured translation failure); model budget = NEW budget-map key `model_ms_per_call`
(600000; map is open `BTreeMap<Id,u64>` registry.rs:127 → yaml-only), `Resolved.model_ms_per_call:
Option<u64>` read unconditionally, REQUIRED at record only (`build_route_record` None→Err→command
diagnostic; replay never invokes → zero CASSETTE churn — but codex catch: the key + `Resolved` field
DO churn tests: the resolve budget-vector pin asserts the exact 3-entry map (run.rs ~4227, gains the
key) and both test `Resolved` literals (`single_ir_resolved()` ~5544, direct ~6455) gain the field). BANKED APPLY-ANCHORS (implement WITHOUT
re-reading normalize.rs/source_linkage.rs/ir.rs/cassette.rs — shapes verbatim here): composers
run.rs:2429-2465 — `single_ir_prompt(template,doc_id,graph)` = `[template, "document: {doc_id}"]` ++
`reading_order_text(graph)` (sort spans by `reading_order`, map `raw_text`), join`\n`;
`direct_smt_prompt(template,gid,role,members:&[(&Id,&SourceDocumentGraph)])` inserts `group:`/`role:`
lines then per-member `document:` + spans; `single_ir_fill` ALREADY takes `lexicon:&Lexicon` (:1455)
and `execute_routes` loads `(lexicon,lexicon_hash)` (:351) → new composer params flow from in-scope
values (verify segments/graph in scope at single_ir_fill's Record arm :1453-1520; if absent, thread
from the route loop which holds DocHead); `Lexicon{system,content_hash,concepts,actions,modality,
certainty}` (normalize.rs:67), `LexiconConcept{concept_id,surfaces,interval:Option<QuantityInterval>}`,
`LexiconAction{action_id,surfaces}`; `SourceDocumentGraph.regions:Vec<EvidenceRegion>`
(source_linkage.rs:575; `region_id` :421); `ClinicalSegment{segment_id,kind,region_ids}` (ir.rs:566);
`build_route_record` (run.rs:2535-2570) already holds the template String + `schema.path` → pre-write
BYTE-verify slots there (`hash_bytes(template.as_bytes())==prompt.template_hash`;
`hash_bytes(&read(root.join(&schema.path)))==schema.schema_hash`; mismatch→Err→command diagnostic),
closing the deferred f2 codex item; `RecordContext{producer,prompt_template_hash,budget}`
(cassette.rs:49); `store.record` already seals constraint bytes across the call (cassette.rs:109-121);
resolve() budget reads: SOLVER_BUDGET_KEY :800 required, MODEL_REPAIR_LIMIT_KEY :810
model-route-gated (mirror the Option read); f1 composer EXACT-bytes pin tests run.rs:3192+ (update
from the new composition); template edits re-pin the two prompts.yaml template_hash values
(sha256sum; committed_model_surface_checks_ok validates declared-vs-file, no const edit) AND (codex
catch) the m2 loop test's §9 aggregated `prompt_template_hash` literal (run.rs ~4560 — aggregates
the route-relevant prompts.yaml hashes; re-bless from observed); committed fixture cassettes survive
untouched — replay-mode fills are byte-source-only (`FillSource::Replay` carries no prompt, load
cross-checks no template hash). Cassette
landing (2b): CassetteStore roots at `<root>/cassettes/` (run.rs:372) → recording from the repo root
lands tracked `/cassettes/**` (payload EMBEDS the rendered prompt = deny-Read source text → close 2b
by syncing the do-not-read set: `.claude/settings.json` deny Read() + `.serena/project.yml`
ignored_paths + a block/allow Read-test pair; committed cassettes carry REAL model identity — extend
memory's audit-exclude list). Runtime facts (2b): the engine exposes a grammar-constraint slot in a
dialect near the committed one (introspected live; wiring + on-device verification = 2b's
exploratory env work — the dialect transform recipe, device notes, and output-token-cap sizing are
banked in runtime.local.md, keeping this committed file engine-neutral); the wrapper today
fail-closes non-JSON constraints; identity probe re-confirmed live this session.
- [x] run-m2.2a: prompt-scaffold completion + record-mode byte-verify (deterministic, runtime-absent).
  LANDED: `single_ir_prompt(template, doc_id, graph, segments: &[ClinicalSegment], lexicon: &Lexicon)`
  emits document line → per-segment `segment: <id> kind=<kind> regions=<ids>` → `regions: <ids>` →
  `system:` / `concept:` (`var=<interval var>` marks) / `action:` lexicon block → reading_order spans;
  DEVIATION from the banked signature: region ids read from `graph.regions`, no separate
  `regions:&[EvidenceRegion]` param — the graph carries the exact set the accept closure grounds
  against, a separate param = supply-vs-grounding divergence risk; direct composer unchanged (the
  `:named` scheme is template TEXT). Templates resynced (single_ir promise↔supply map with the
  schema-enum carve-out; direct role-sensitive `ctx.`/`a.` label scheme replacing the label-list
  promise); prompts.yaml re-pinned. FOUND: the committed ClinicalIR schema already bakes the
  lexicon-derived enums (ConceptCode/ActionKind/IntervalVar/TerminologySystem + BindingStatus/
  Direction/Strength/Certainty) → the prompt vocabulary block informs, the schema enforces, both from
  one committed lexicon — consistent by construction. `model_ms_per_call: 600000` in exp.m2_multihop
  + `Resolved.model_ms_per_call: Option<u64>` unconditional read; NEW adapter-free
  `RecordParts`/`build_record_parts` split out of `build_route_record` (selection / template load /
  byte-verify / budget testable runtime-absent) — requires the budget key (None→Err naming it), swaps
  `ctx.budget` off the solver key, and byte-verifies template bytes vs `template_hash` (inline arm
  included) + constraint file bytes vs `schema_hash`, closing f2's deferred codex item. Tests:
  scaffold composer pin (spans sorted, scaffold lines pinned pass-through order); 4×
  `build_record_parts_*` (committed-bytes happy path both routes / budget-absent / template-drift /
  schema-drift); resolve pins (4-entry budget vector, `Some(600_000)` per view, M1 `None`); §9
  `prompt_template_hash` re-blessed from observed (other 3 hashes unchanged — cross-consistent). Gate:
  290+174 pass/7 ignored, fmt/clippy clean, rustdoc 17 (baseline), touched-file engine-neutral grep
  CLEAN. 86% 171K/200K (compacted at close-out bookkeeping; close finished 38% post-compact)
- [ ] run-m2.2b: LIVE record + committed experiment cassettes (gate: run-m2.2a landed). Env wrapper
  first (machine-local, outside git): translate the committed grammar to the engine's constraint
  dialect and wire that slot, staying fail-closed on anything else; verify live against
  schemas/smt_query.grammar on the target device; size the runtime's output-token cap (~4096) —
  transform recipe + device + cap specifics banked in runtime.local.md. Then LIVE `ckc run --experiment exp.m2_multihop --out
  runs/<id> --record` from the repo root (runtime indirection: sources open inside the pipeline;
  their paths stay out of Read/Bash args); commit landed `/cassettes/**` verbatim (origin
  `ai_generated`, REAL identity — audit-exempt path, extend memory's exclude list);
  `CKC_MODEL_COMMAND=<absent-name>` `ckc replay --run <out>` matches; sync the do-not-read set for
  `/cassettes/` (banked above) + Read-test pair. Gate (LIVE): the run lands the 8-entry root layout +
  both routes' artifacts; every fill attempt persists a cassette (weak-model failures land as §7.4
  codes; a CassetteError abort = instrument fault → fix env/wrapper, re-record); replay CLI reports
  match.
- [ ] run-m2.2c: recorded-run pin battery + replay coverage (deterministic, runtime-absent; gate:
  run-m2.2b landed). New tests/ integration file: temp root = committed registry+corpus+schemas+
  prompts (write_m2_root pattern) + repo `/cassettes/**` copied in; `execute()` exp.m2_multihop →
  bless-from-observed pins: report.json M2 sections (typed strict_at structure per the C2 split —
  taxonomy route maps, metrics emission_order + k_sample NA, agreed identity == the cassettes'
  recorded identity by EQUALITY, never a literal), RunManifest+ReplayManifest §9 7-tuple (4 hashes
  blessed; reference_hash cross-checked == raw sha256 of expected_outcomes), rendered bodies via a
  run_oracle-style re-render compare (NO const body pin: solver version env-dependent, identity
  literals banned from test code); replay::execute over the run → matched() (closes the deferred
  replay.rs model-artifact coverage). Reading capped: the m2 loop test + write_m2_root helpers +
  replay.rs:105-180; report.rs/metrics.rs/manifests.rs EXCLUDED (pins bless from observed bytes).
  Gate: full battery + doc-lint green.
- [ ] acceptance-m2: §9 acceptance. Verify the §9 themes against the recorded run — both routes
  execute over identical locked inputs (`exp.m2_multihop`); recorded model I/O replays byte-stably
  (runtime absent); raw rows before the baseline-delta table; expected conflict/no-conflict per
  reference for accepted translations; the bilingual report renders deterministically from
  report.json; §0 vocabulary holds. Tag `accept/m2`. [§9 scopes acceptance to faithful measurement,
  NOT a required result sign — a null/negative delta is a valid PoC outcome.] Reading: all M2 artifacts
  + the §9 acceptance themes. Gate: all six themes pass on the recorded run; `ckc replay` byte-matches;
  tag `accept/m2`.
