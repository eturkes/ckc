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
`CKC_MODEL_COMMAND` (model.rs:105), `.identity()` (:163) — build ONCE in execute_routes like
`Z3Adapter::new` (~369). GUIDELINE-TEXT source = `SourceDocumentGraph.spans[].raw_text` ordered by
`reading_order` (source_linkage.rs `SourceTextSpan{span_id,node_id,raw_text,reading_order,…}`);
`ClinicalSegment`(segment_id/kind/region_ids) + `EvidenceRegion`(region_id/node_ids/span_ids/anchor_ids)
carry NO text → DECISION: the first-draft prompt joins spans' `raw_text` in `reading_order`
(full-document text); the segment→region_ids→span_ids→spans "segment order" mapping is a run-m2.2
wording refinement. Registry select mirrors `manifest_inputs` want-set (run.rs:2334-2450):
SingleIr→schema target_kind `clinical_ir` / prompt entry-id `prompt.single_ir`; DirectSmt→`smt_query`
/ `prompt.direct_smt`. `SchemaEntry{id,path,schema_hash,target_kind}` (core registry.rs:206);
`PromptEntry{id,path:Option,inline:Option,template_hash,route}` path-XOR-inline (:225); consts
`SCHEMAS_FILE`/`PROMPTS_FILE` run.rs:95/99; `parse_schemas`/`parse_prompts` (core). Slot const
`MODEL_FILL` + `producer(resolved, idx)` + `Resolved.budget_ms` all live in run.rs (used by the fills).
- [ ] run-m2.1f1: pure record-prompt selection + composition (ckc-cli, new fns in run.rs or a
  `record_prompt` section; `#[allow(dead_code)]` pre-consumers; NO I/O, NO adapter, NO dispatch —
  crate green). Deliver 4 pure fns + inline-fixture tests: `select_record_schema(&[SchemaEntry],
  RouteShape) -> Option<&SchemaEntry>` (by `target_kind`: SingleIr→clinical_ir, DirectSmt→smt_query,
  M1Layered→None); `select_record_prompt(&[PromptEntry], RouteShape) -> Option<&PromptEntry>` (by
  entry id prompt.single_ir/prompt.direct_smt, mirror manifest_inputs; M1Layered→None);
  `single_ir_prompt(template:&str, doc_id:&Id, graph:&SourceDocumentGraph) -> String` (template ++
  doc-id line ++ `graph.spans` `raw_text` joined by `reading_order`); `direct_smt_prompt(template:&str,
  gid:&Id, role:&str, members:&[(&Id, &SourceDocumentGraph)]) -> String` (template ++ `role: <role>`
  line ++ per-member doc-id + spans). Deterministic composition (order by `reading_order`; `\n` join —
  first-draft, run-m2.2 refines wording). Tests: each selector hit + miss (+ M1Layered None); each
  composer over a 2-span inline graph pins the EXACT composed string (order + doc-id + role line).
  Reading: SchemaEntry/PromptEntry/RouteShape + SourceDocumentGraph/SourceTextSpan shapes BANKED in
  the RESPEC note (do NOT re-read registry.rs/run.rs/source_linkage.rs). Gate: `cargo test -p
  ckc-cli`; fmt/clippy `--lib -D warnings`; `RUSTDOCFLAGS='-D warnings' cargo doc -p ckc-cli --no-deps`
  on the new doc comments.
- [ ] run-m2.1f2: dispatch `--record` + record-mode threading (consumes f1; B2a-style
  compute-then-green; Record arm type-enforced, live exercise = run-m2.2). (1) dispatch.rs: `--record`
  optional boolean — pre-partition it out of `rest` BEFORE `take_flags(OP_RUN,["--experiment","--out"],
  …)` (:124, fixed-arity value-flags), thread through `RawCommand::Run{experiment,out,record}` +
  `validate` (:201) + `Command::Run{…,record}` + `execute` (:309) → `crate::run::execute(Path::new("."),
  experiment, record, shell)`. (2) run.rs `execute` (:190): add `record:bool`, pass to execute_routes
  (M1-only inline path ignores it). (3) `execute_routes` (:346): when `record`, `ModelAdapter::new()`
  ONCE (Err→command diagnostic + return, mirror the Z3Adapter arm ~369), load schemas.yaml+prompts.yaml
  (mirror manifest_inputs read+parse; read/parse err→command diagnostic + return), build per-view
  `RouteRecord{adapter:&ModelAdapter, template:String (f1 select_record_prompt → `entry.inline` else
  read `root.join(entry.path)`), constraint:PathBuf (`root.join(`f1 select_record_schema`.path)`),
  ctx:RecordContext{producer(resolved,MODEL_FILL), `entry.template_hash`, budget:Duration from
  resolved.budget_ms}}`; pass `Option<&RouteRecord>` to `single_ir_fill` (:1415, call ~420) +
  `direct_smt_fill` (:1649, call ~544; map heads→`(&h.trace.document_id, &h.source.payload)`). (4)
  fills: add `record:Option<&RouteRecord>`; None→`FillSource::Replay` (unchanged), Some→build the
  prompt via f1 (single_ir: `single_ir_prompt(&r.template,&doc_id,&source.payload)` at the model_fill
  call :1456; direct: per-role `direct_smt_prompt(&r.template,gid,role,&members)` inside the role loop
  at model_fill call ~1704) + `FillSource::Record{r.adapter,&prompt,&r.constraint,&r.ctx}`. (5) update
  ALL execute/execute_routes callers to thread `record` (M1 callers pass false). (6) default-replay
  acceptance test: record=false builds NO adapter — set `CKC_MODEL_COMMAND` to a bogus command, assert
  a record=false model-route run still lands (no probe fires); thread false into
  `m2_route_loop_lands_both_routes_namespaced` (stays green). Reading: f1 fns (this crate); all
  run.rs/dispatch.rs/cassette.rs/model.rs facts BANKED in the RESPEC note (do NOT re-read). Gate:
  `cargo test -p ckc-cli`; fmt/clippy; doc-lint; type-enforced Record arm (live exercise = run-m2.2).
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
