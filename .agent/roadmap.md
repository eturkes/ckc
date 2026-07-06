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
run-m2.1e RESPEC (seam confirmation read run.rs+manifests.rs+report.rs+trace.rs — margin spent, so this session closes at the respec per the land-or-revert rule; nothing written): the single unit bundled 6-7 deliverables (GroupTrace landing-completeness + manifest output walk + run-level producer + manifest §9 fields + report ModelRunSections + report_ja + a full-run pin battery) across four large modules → oversized on both deliverable-count and read-cost (record-shape extension = its own unit; report/manifest each carry their own integration pins). Split at three confirmed seams, ordered A (foundational, no threading) → B (manifest §9, lands the agreed-identity + reference/registry file-read threading) → C (report sections, reuses B's threading). No new sizing RULE minted (applies the standing record-shape-extension + read-cost axioms), so no re-audit of run-m2.1f/2.2/acceptance. Facts below are source-read + banked (do not re-read the enumerated sources). · B SUB-SPLIT: run-m2.1e-B overflowed read+write with the design LOCKED in-session (zero code) → split manifest §9 into B1 (record-shape PLUMBING: fields+assembly+fixtures/byte-pins) + B2 (computation+threading) at the manifests.rs/run.rs module seam (memory sizing bullet now carries the rule: a locked design ≠ one-window fit — read-to-place-edits over a large caller + byte-pin-test authoring alone overflow). B1 leaves the crate green with §9 plumbed-but-None (omit-None → M1 AND M2 manifest bytes byte-identical to pre-B); B2 fills the model-route computation. Their lines bank EVERY apply-anchor from a full read of manifests.rs + run.rs (`manifest_inputs` 2171 / `report_processing_stage` 2032 / tail sites 255+634) → implement WITHOUT re-reading the SOURCE modules (registry.rs, plans.rs, ModelIdentity).
- [x] run-m2.1e-A: run-level landing completeness + honest producer — `GroupTrace.smt_queries: Vec<ArtifactWrapper<QueryBody>>` (single_ir empty, direct `[overlap,deontic]`); both routes push GroupTrace on artifact LANDING not verify-success, gating only GroupObservation on `results` → a landed-but-unverified compiled/pair stays replay-covered; `manifest_inputs` walks `group.smt_queries` into output_hashes (manifest-only — `assemble_trace` UNCHANGED, no smt_query trace node; provenance rests on `verifier_results.input_hashes`). Honest producer: `tail_producer(resolved, idx, emit_event)` — false→run-level `processing_stage.run.{trace,report}` step-ids (consts RUN_TRACE_STEP/RUN_REPORT_STEP), true→early-returns `producer(resolved, idx)` (M1 byte-identical). Pin: direct overlap+deontic content_hashes appear in write_m2_root manifest.json output_hashes. 99% 197K/200K
- [x] run-m2.1e-B1: manifest §9 fields plumbed (record-shape extension, None everywhere). GOAL: the 7 §9 measurement fields flow `ManifestInputs`→both manifest records, None on EVERY run (byte-identical to today); a populated byte-pin locks the new slots. NO computation/threading (=B2). FACTS (manifests.rs + the run.rs `manifest_inputs` RETURN fully read this respec — do NOT re-read source modules): `RunManifest`/`ReplayManifest` ALREADY carry the 7 (M2.1 model-types), canonical key-sort lives in ckc-core emit → the manifests.rs record LITERALS already list them (currently `None`), so literal field ORDER is cosmetic (swap in place). The 7: `model_identity: Option<ModelIdentity>` then `test_source_hash`/`reference_hash`/`schema_hash`/`prompt_template_hash`/`model_hash`/`runtime_hash`: `Option<Hash>`. `ManifestInputs` is constructed at EXACTLY 2 sites (both edited here): manifests.rs `inputs()` (l.190, ends `output_hashes: vec![hash('7'), hash('2'), hash('7')],`) + run.rs `manifest_inputs` return (ends `output_hashes,` then `})`). Helpers: `id(&str)->Id`, `hash(char)->Hash` = `Hash::new(format!("sha256:{}", c.to_string().repeat(64)))`. EDITS: (1) manifests.rs l.12 import — add `ModelIdentity` into `use ckc_core::{CanonError, Hash, Id, ReplayManifest, …}`; (2) `ManifestInputs` struct (l.20; last field `output_hashes: Vec<Hash>` l.43) — append the 7 `Option` fields; (3) `assemble_manifests` — RunManifest literal (§9 comment l.81-82, `model_identity: None`..`runtime_hash: None` l.83-89) swap each `None`→`inputs.<field>.clone()`; ReplayManifest literal (§9 comment l.101, Nones l.102-108) same — KEEP both comments; anchor each Edit on its UNIQUE §9 comment (the 7-`None` run is identical between blocks); (4) fixture `inputs()` — append the 7 fields =`None` (keeps assembly_sorts/canonical/dup/empty pins byte-identical; the l.322 field-clear mutators list stays — `Option` fields have no empty-rejection); (5) run.rs STUB — `manifest_inputs` return (after `output_hashes,`) append the 7 fields =`None` (crate compiles, §9 `None` on ALL runs incl. M2 = unchanged from pre-B; NO signature/param change — that is B2); (6) ADD a populated-§9 byte-pin test — build a `ManifestInputs` with Some §9 (SYNTHETIC: `ModelIdentity{model_id: id("model.baseline"), quant: "fixture_quant".into(), runtime_version: "1.0.0".into()}`, 6 hashes = distinct seeds `hash('a')..hash('f')`), `assemble_manifests`, byte-pin the canonical bytes of BOTH `RunManifest` AND `ReplayManifest` (memory backward-compat rule — codex once caught a missing replay pin; the existing all-None round-trips + run.rs M1 manifest pins guard the omit-None old bytes). Read ONLY: manifests.rs (struct + `assemble_manifests` + `inputs()` + test module) + the run.rs `manifest_inputs` return literal. GATE (not live-runtime — pure record plumbing, no run-level behavior change): fmt/clippy/`RUSTDOCFLAGS='-D warnings' cargo doc -p ckc-cli --no-deps`/`cargo test -p ckc-cli`. Acceptance: §9 plumbed `ManifestInputs`→both records, `None` everywhere; existing pins byte-identical; populated byte-pins lock both records' new slots. 66% 132K/200K
- [ ] run-m2.1e-B2: manifest §9 computed on the declared model routes (run.rs). GOAL: populate the 5 hashable §9 fields on model-route runs (all `None` on M1); `model_hash`/`runtime_hash` stay `None`. SPEC §794-795/854: §9 = a per-run "locked measurement record" freezing THIS run's inputs (test source/reference/schema/prompt hashes), so the fields are RUN-relevant not registry-wide (consistent with test_source_hash from `docs` + reference_hash from the run's `exp`). RULING (banked, confirmed): §9 computation lives IN `manifest_inputs` (run.rs ≈2171), NOT `report_processing_stage` — manifest_inputs already reads files via `std::fs::read(root.join(F)).map_err(|e| format!("manifests: read {F}: {e}"))?` and returns `Result<ManifestInputs, String>`, so §9 reads ride that funnel with `?` (zero re-indentation); `report_processing_stage` (returns `()`, errors via a `fail`→diagnostic chain) only THREADS the two signals (`agreed`, `model_routes`), does no reads. DESIGN — GATE (fixes codex Finding 2 — `agreed`-presence is NOT an M1/M2 discriminator: `agree_model_identity` returns true on a `None` candidate + both fills return `identity:None`/empty-`identities` on cassette failure, so an all-fills-failed MODEL run reaches the tail with `agreed==None` → gating on it mislabels the run M1): gate on the DECLARED model routes, threaded `model_routes: &[RouteShape]` (M2 = the shapes that ran; M1 = `&[]`). `manifest_inputs` gains LAST params `agreed: Option<&ModelIdentity>, model_routes: &[RouteShape]`; `if model_routes.is_empty() { 7×None (M1 path, byte-identical) } else { compute the 5 below; model_identity = agreed.cloned() — HONEST None when no identity was observed (an error run then records input hashes + None identity, distinct from M1's all-None) }`. Helper `aggregate_hashes(mut v: Vec<String>) -> Hash { v.sort(); v.dedup(); hash_bytes(v.join("\n").as_bytes()) }`. The 5: `model_identity`=`agreed.cloned()`; `test_source_hash`=`aggregate_hashes(docs.iter().map(|d| d.source_hash.as_str().to_owned()).collect())` (`docs` route-EXPANDED → sort+dedup in the helper collapses the per-route multi-count); `reference_hash`=`Some(hash_bytes(&std::fs::read(root.join(&exp.expected_outcomes))?))`, `exp`=the `ExperimentEntry` with `id == resolved.plan.experiment_id` (read EXPERIMENTS_FILE → `parse_experiments` → find); `schema_hash`/`prompt_template_hash` = ROUTE-RELEVANT (fixes codex Finding 1 — aggregating ALL registry entries over-includes, breaks the SPEC run-inputs semantic, and would churn M2's golden when a later unit grows the registry; registry.rs:657 confirms the model registry has NO pipeline cross-references yet → route→schema/prompt is by SHAPE inference): derive each `model_routes` entry's schema+prompt id — `SingleIr`→(`schema.clinical_ir`,`prompt.single_ir`), `DirectSmt`→(`schema.smt_query`,`prompt.direct_smt`) (grounded: run.rs ~2438 forms `schema.<output_kind>`; fills key `route.single_ir`/`route.direct_smt`; prompts.yaml binds those routes 1:1) → `want_schema`/`want_prompt` = those id sets; `schema_hash`=`aggregate_hashes(schemas.iter().filter(|s| want_schema.contains(&s.id)).map(|s| s.schema_hash.as_str().to_owned()).collect())`, `prompt_template_hash` same over prompts by `p.id ∈ want_prompt`; `model_hash`/`runtime_hash`=`None` (model+runtime = env bare-name PATH commands, NO committed bytes to hash → identity rides `model_identity`). CONSTS (none exist; add by run.rs l.85-89): `EXPERIMENTS_FILE="registry/experiments.yaml"` (resolve() has this as an inline literal l.698 — add the const, optionally dedup that use), `SCHEMAS_FILE="registry/schemas.yaml"`, `PROMPTS_FILE="registry/prompts.yaml"`. IMPORT (run.rs l.53): add `parse_schemas, parse_prompts` beside `parse_experiments` (`ModelIdentity` + `RouteShape` already in-module). THREADING — collect `let mut model_routes: Vec<RouteShape> = Vec::new();` in `execute_routes` alongside `route_runs`, push `resolved.shape` each view iteration (RouteShape is `Copy`); `report_processing_stage` (run.rs ≈2032) gains TWO LAST params `agreed: Option<ModelIdentity>` + `model_routes: &[RouteShape]` (after `emit_event: bool`); at its `manifest_inputs(…, resolved, shell)` call (final `.and_then`) append `agreed.as_ref(), model_routes`; M1 tail call (`execute()`, after `…&resolved, shell, true,`) append `None, &[],`; M2 tail call (`execute_routes`, after `…baseline_resolved, shell, false,`) append `agreed, &model_routes,` (`let mut agreed: Option<ModelIdentity>` at run.rs l.371, folded l.401/529, this tail = its LAST use → move OK; NOTE `adapter.identity()` in both calls is the SMT `SolverIdentity`, unrelated to `agreed`). ACCESSORS (banked — do NOT read registry.rs/plans.rs/manifests.rs): `ModelIdentity{model_id: Id, quant: String, runtime_version: String}`; `RouteShape::{SingleIr, DirectSmt, M1Layered}` (Copy); `SchemaEntry{id: Id, schema_hash: Hash}`, `PromptEntry{id: Id, template_hash: Hash}`, `ExperimentEntry.expected_outcomes: String`; `parse_schemas`/`parse_prompts`/`parse_experiments(&str)->Result<Vec<_>, RegistryError>` (need `read_to_string`); `hash_bytes(&[u8])->Hash`, `Hash::as_str()->&str`; `DocTrace.source_hash: Hash`; `resolved.plan.experiment_id` (already used l.2187). TEST (`write_m2_root` REPLAYS committed goldens → NOT live-gated): extend `m2_route_loop_lands_both_routes_namespaced` — read manifest.json + replay_manifest.json; model route asserts `model_identity`=Some(the run's own SYNTHETIC identity `model.baseline`/`fixture_quant`/`1.0.0`, safe to pin), the 4 hashes=Some, `model_hash`/`runtime_hash`=None; PIN the reference/schema/prompt hash VALUES (committed-file hashes, deterministic — schema_hash over {schema.clinical_ir, schema.smt_query}, prompt over {prompt.single_ir, prompt.direct_smt}); M1 run manifest §9 all `None` (existing M1 pins guard byte-identity). Blast radius of the gate-fix: partial-success error pins (run-m2.1d5a-2b) keep `agreed=Some` → §9 populated same as success → unchanged; only the all-fills-failed-with-manifest case shifts (not currently pinned). Read ONLY: run.rs `manifest_inputs` body + `report_processing_stage` manifest_inputs call + both tail sites + `execute_routes` route loop (add the shapes collect) + the consts block + `write_m2_root`/`m2_route_loop_lands_both_routes_namespaced` (all line #s ≈, shifted by B1 — anchor by SYMBOL). GATE: fmt/clippy/`RUSTDOCFLAGS='-D warnings' cargo doc -p ckc-cli --no-deps`/`cargo test -p ckc-cli`; engine-agnostic word-boundary grep over touched files. Acceptance: model-route runs populate the 5 §9 fields ROUTE-RELEVANTLY (model_hash/runtime_hash None), M1 all None, `write_m2_root` §9 pins hold, M1 pins byte-identical. Lands the `agreed` + `model_routes` params on report_processing_stage that C reuses.
- [ ] run-m2.1e-C: report ModelRunSections + report_ja.md. `report_processing_stage` builds `ModelRunSections{route_diagnostics (per-route ledgers, clean route=empty slice), route_metrics (metrics::route_metrics per route — samples=the k=1 battery → convergence NA), baseline_pipeline_id, model_identity}` and passes `Some` to `assemble_report` (currently `None`); land report_ja.md beside report_en.md; `experiment_metrics` stays in-assembly (never called from run.rs). FACTS: `ModelRunSections<'a>` (report.rs:551) = `{route_diagnostics:&[(Id,&[DiagnosticRecord])], route_metrics:Vec<RouteMetrics>, baseline_pipeline_id:&Id, model_identity:&ModelIdentity}`; `assemble_report` 8th param `model_run:Option<ModelRunSections>` (report.rs:601, passed at the `assemble_report` call ~run.rs:2044). SOURCES: `RouteRun{pipeline_id,ledger,fills,groups,samples}` collected in `execute_routes` (run.rs, currently `let _ = &route_runs` ~568); `route_metrics(pipeline_id,fills,groups,samples,reference)->RouteMetrics` (metrics.rs); `baseline_pipeline_id`=`baseline_resolved.pipeline_id` (run.rs:589 `baseline_resolved`); `model_identity`=agreed (in scope, Option → `expect` on a model route); `route_diagnostics`=RouteRun.ledger's DiagnosticRecords keyed by pipeline_id; `reference`=`parse_reference(read root.join(exp.expected_outcomes))` (load `exp`+`parse_reference` LOCALLY in report_processing_stage — B2 threads `agreed`+`model_routes` but NOT `exp`/reference; those reads live INSIDE `manifest_inputs`, unreachable here — the duplicate model-route-only YAML parse is trivial); report_en landing = run.rs:2064 `write_under("report_en.md", render_markdown(&report.payload)…)` with write+read-back byte-check → add `render_markdown_ja` (report.rs:889) → `write_under("report_ja.md")` same discipline. EDITS: `report_processing_stage` gains route_runs + a LOCAL `exp`/`parse_reference` + reuses B2's `agreed` param (now on report_processing_stage), builds sections, passes `Some` at the `assemble_report` call, lands report_ja; both tail call sites update (run.rs:246 M1 `model_run=None` / run.rs:598 M2 `Some`). Read: report.rs `assemble_report` M2 population body (below :700 — failure_taxonomy/metrics/model_identity fill) + populated_report fixture helpers (`m2_route_metrics()`/`baseline_model_identity()` for expected values) + `render_markdown_ja` tests (report.rs:3552/3697 for JA discipline); metrics.rs `RouteMetrics` shape; run.rs `report_processing_stage` body (2001-2110) + `execute_routes` RouteRun/registry-load region. Gate: fmt/clippy/doc/cargo test; `write_m2_root` pins report.json M2 sections (failure_taxonomy/metrics/model_identity) + report_en.md + report_ja.md bodies from observed; M1 report pins hold (M1 path `model_run=None` → byte-identical).
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
