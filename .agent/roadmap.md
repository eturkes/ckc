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
- [ ] run-m2.1d4a: direct route stage — landing + §4.6 events (production rewrite + mechanical
  call-site updates; the observed-output pin battery is .1d4b). DESIGN VERIFIED this respec against
  mirror `single_ir_fill` (run.rs ~994) + current `direct_smt_fill` (~1206) + every call-site test;
  the deltas below pin every decision / string / ordering, so implementation is TRANSCRIPTION against
  the two anchors — derive nothing fresh. Gate: cargo test; then fmt + clippy -D warnings + doc-lint +
  touched-file engine-leak audit; M1 pins untouched. On close PRUNE the [A]-[E] detail + gotchas to a
  one-line summary.

  READ FIRST (targeted — never the whole 5283-line file): mirror `single_ir_fill` body for the
  `ProcessingStageEvent` literal + `clock.stop()` + the `ModelFill { … }` destructure; current
  `direct_smt_fill` (~1206 to its closing `}` ~1360) for the role loop + `wrapper(…)` call to mutate;
  `DocHead`/`RouteDoc` (~829/~843) + `route_document_head` signature (~868) for the head fields +
  helper arg order; the 6 call sites at the lines in [E]. `ModelFill` / `FillObservation` shapes are
  pinned in [B] step 6 → skip model_fill.rs + metrics.rs unless a field mismatches.

  [A] new `struct DirectFill` directly above `direct_smt_fill`'s doc (~1194), attr
  `#[allow(dead_code)]`; fields `pair: Option<(ArtifactWrapper<QueryBody>, ArtifactWrapper<QueryBody>)>`
  (Some iff BOTH roles accepted + landed), `fills: Vec<FillObservation>`, `identities: Vec<ModelIdentity>`
  (fills + identities survive a terminal reject → the .1d5a orchestrator folds metrics + checks identity
  agreement). Doc = one line to that effect.

  [B] rewrite `direct_smt_fill` IN FULL (its doc ~1194 through the closing `}` ~1360, right before
  verify_group's `/// direct_smt route's per-group verdict tail` doc). New signature, attrs
  `#[allow(dead_code)]` + `#[allow(clippy::too_many_arguments)]`, params
  `(gid: &Id, heads: &[&DocHead], store: &CassetteStore, seed: u64, resolved: &Resolved, repair_limit: u32, shell: &mut Shell) -> DirectFill`
  (root / members / extract / segment params GONE — the caller supplies pre-built heads, landed + evented
  upstream by `route_document_head`; `direct_smt_fill` dedups nothing, so a doc in two groups heads once
  per route ONLY when the caller dedups — the .1d5a orchestrator, or .1d4b's prepass; see [D]/[E]).
  Doc must convey: consumes the group's two member heads; the
  direct route grounds nothing (raw SMT, not an IR) so heads carry only provenance forward; replays each
  role cassette through `model_fill` under `direct_smt_accept()`, wraps + LANDS each accepted body as a
  raw-AI `smt_query`; direct-emits ONE group model_fill §4.6 event; a terminal role reject breaks the
  loop; `Err(CassetteError)` → command diagnostic, NO event; `pair` Some iff both roles landed,
  fills / identities survive a reject. Body, in order:
  1. guard `heads.len() != 2` → `shell.diagnostic(invalid_diagnostic(…))` with pairs keyed
     `group`=`gid.to_string()`, `reason`=a count-mismatch message, `processing_stage`=`"model_fill_smt"`
     (`.to_owned()`) → `return DirectFill { pair: None, fills: Vec::new(), identities: Vec::new() }`.
  2. `let prefix = route_id_prefix(resolved);` then `let dir = format!("groups/{gid}");` — bare
     `groups/{gid}` co-locates the smt_query with the group's `verifier_results.json` (the
     `direct_smt_verify_group` callers pass bare `groups/{gid}`, ~4570; M1 `compile_verify_group` lands
     there too, ~809 / assert ~2484). The route `prefix` rides the artifact IDs ([B].6f / [C]) for
     cross-route uniqueness, NOT the landing dir — routes never co-land (.1d5a rejects a mixed M1+model
     set), so a flat `groups/{gid}` never collides; a uniform `routes/{pid}/…` dir for every group
     artifact is a later-milestone concern, unneeded for correctness.
  3. member-order provenance BEFORE the clock (M2.7 boundary): build `input_hashes: Vec<Hash>` by, per
     head, pushing `head.source.content_hash.clone()` then `head.segments.content_hash.clone()`.
  4. `let clock = processing_stage_clock();`.
  5. init accumulators `fills: Vec<FillObservation>`, `identities: Vec<ModelIdentity>`,
     `diagnostics: Vec<DiagnosticRecord>`, `recorded_calls: u64 = 0`, `repairs: u64 = 0`,
     `landed: Vec<ArtifactWrapper<QueryBody>>` (DELETE the old `let mut pair = Vec::new();`).
  6. role loop `for (role, logic) in [("overlap", SmtLogic::QfLra), ("deontic", SmtLogic::QfUf)]` —
     source / key / `model_fill` call UNCHANGED from current (source `static_id(&format!("{gid}.{role}"))`,
     key `CassetteKey { route: static_id("route.direct_smt"), source: source.clone(), seed }`,
     `model_fill(store, &key, FillSource::Replay, repair_limit, direct_smt_accept())`). `ModelFill` fields
     to destructure (the fill is `ModelFill<String>` — `direct_smt_accept` yields the raw SMT body `String`;
     step e wraps it into `QueryBody`): `target: Option<String>`, `accepted_cassette_hash: Option<Hash>` (Some iff target
     Some), `model_identity: Option<ModelIdentity>`, `diagnostics: Vec<DiagnosticRecord>`,
     `recorded_calls: u64`, `repairs: u64`; `FillObservation::from_fill(&fill)` borrows before the move.
     Per role:
     a. `Err(e)` → command diagnostic keyed `cassette`=`source.to_string()`, `reason`=`e.to_string()`,
        `processing_stage`=`"model_fill_smt"` → `return DirectFill { pair: None, fills, identities }`
        (carries the partial accumulators; NO event — matches the current early return).
     b. `Ok(fill)`: `fills.push(FillObservation::from_fill(&fill));` then destructure, renaming the three
        loop-colliding fields to `role_diagnostics` / `role_recorded_calls` / `role_repairs`; then
        `identities.extend(model_identity); diagnostics.extend(role_diagnostics);
        recorded_calls += role_recorded_calls; repairs += role_repairs;`.
     c. `let Some(body) = target else { break; };` — the BREAK is load-bearing (a terminal reject stops
        the pair: overlap exhausts before the deontic source is read; the rejection schema-arm ledger =
        overlap's diagnostics alone, riding the one event).
     d. `let mut role_inputs = input_hashes.clone();` then push
        `accepted_cassette_hash.expect("accepted fill carries its cassette wrapper hash")`.
     e. `let payload = QueryBody { query_id: source, logic, body };`.
     f. the current fn's `wrapper(…)` call with the SAME 8 args EXCEPT the id becomes
        `format!("{prefix}{gid}.{role}.smt_query")` (gains `{prefix}`) and the producer stage arg becomes
        `MODEL_FILL` (was literal `2`); kind stays the `"smt_query"` literal, then `producer(resolved, MODEL_FILL)`,
        `role_inputs`, `Origin::AiGenerated`, `EvidenceStatus::AcceptedEvidenceStatus`, `Vec::new()`, `payload`.
     g. land through an EXPLICIT match (not an `.and_then` closure — keeps the `&mut shell → &Shell`
        reborrow into `land` obvious): wrapper `Ok(env)` → `land(shell, &format!("{dir}/{role}.smt_query.json"), env)`
        → `Ok(w)` push `landed`, `Err(d)` → `diagnostics.push(d); break;`; wrapper `Err(e)` →
        `diagnostics.push(invalid_diagnostic(…)); break;` keyed `group`, `artifact`=the same id string,
        `reason`=`"wrap: {e}"`, `processing_stage`=`"model_fill_smt"`. Both failures ride the event (not the
        ledger) then `break` — fail-closed: `landed` can't reach 2, so stop and fall to the ONE event like the
        6c reject (NOT the current fn's `return None`, which predates the event + would skip it). Untested path
        (golden bodies wrap + land) → break vs continue perturbs no pin.
  7. after the loop: `let (started_at, ended_at, duration_ms) = clock.stop();` then
     `let output_hashes: Vec<_> = landed.iter().map(|w| w.content_hash.clone()).collect();`.
  8. emit ONE event = the mirror's `ProcessingStageEvent` literal EXCEPT three fields —
     `processing_stage: static_id(DIRECT_SMT_STAGE_KINDS[MODEL_FILL])`, `input_hashes` = the member vec
     from step 3, `output_hashes` from step 7. Identical to the mirror otherwise:
     `pipeline_id: resolved.pipeline_id.clone()`, `pipeline_step_id: resolved.pipeline_step_ids[MODEL_FILL].clone()`,
     the three clock fields, `outcome: severity(&diagnostics)` written ABOVE `diagnostics` (so the borrow
     ends before the move), `resource_counters: vec![(static_id(RECORDED_CALLS_COUNTER), recorded_calls),
     (static_id(REPAIRS_COUNTER), repairs)]`.
  9. return: `pair` = `Some((overlap, deontic))` iff `landed.len() == 2` (drain via `landed.into_iter()`,
     `.expect("overlap query wrapped")` / `.expect("deontic query wrapped")`), else `None`; then
     `DirectFill { pair, fills, identities }`.

  [C] `direct_smt_verify_group` (~1371): add first body line `let prefix = route_id_prefix(resolved);`;
  change its wrapper id `format!("{gid}.verifier_results")` → `format!("{prefix}{gid}.verifier_results")`.
  Keep its verify event + `dir` param; no .smt2. SAFE — the sole `verifier_results` artifact_id assert
  (run.rs ~2487) is M1 `compile_verify_group`, prefix empty.

  [D] add helper `direct_fill_group` near `direct_smt_resolved` (~4140), attr
  `#[allow(clippy::too_many_arguments)]`, signature = the OLD `direct_smt_fill` arg shape so call sites
  only rename + read `.pair`:
  `(root: &Path, gid: &Id, members: &[&CorpusEntry], store: &CassetteStore, seed: u64, resolved: &Resolved, repair_limit: u32, shell: &mut Shell) -> DirectFill`.
  Body: per `&m in members` push `route_document_head(root, m, resolved, shell).unwrap_or_else(|| panic!("{gid}: no head for {}", m.id))`
  into `heads: Vec<DocHead>`; `let head_refs: Vec<&DocHead> = heads.iter().collect();`; return
  `direct_smt_fill(gid, &head_refs, store, seed, resolved, repair_limit, shell)`. (Confirm
  `route_document_head`'s real arg order at ~868 when writing.) `direct_fill_group` is a SINGLE-group
  convenience — NO cross-group head dedup, so a member in N groups (or shared across arms under one shell)
  heads N×. This matters ONLY where a pin counts head events: reproduce + scores gain such pins in .1d4b →
  .1d4b swaps THAT pair onto a per-route head prepass (each unique `DocHead` built once, refs passed to
  `direct_smt_fill`) → a shared doc heads once per route. Every OTHER site keeps the plain per-group rename
  PERMANENTLY, its (double-)heading harmless (re-land overwrites; no pin there counts head events /
  landings): non_pair (1 group), the two rejection arms (ledger / last-event pins), route_metrics (3-group
  loop, one shell, guideline_a heads 3×, scores the explicit `&fills` / `&groups` ~5260).

  [E] call sites — rename `direct_smt_fill(&root, &gid, &members, …)` → `direct_fill_group(&root, &gid, &members, …)`
  (identical args), read `.pair`:
  - reproduce (~4389): `.pair` before the existing `.unwrap_or_else`; body / provenance / schema_id asserts
    hold (land round-trips bytes; the id prefix touches only the payload-immune artifact_id). One shell
    spans both groups and `m1_guideline_a` is in both → the plain per-group rename heads it twice; harmless
    (re-land overwrites, no assert counts head events), see [D]. .1d4b's prepass makes it once-per-doc.
  - non_pair (~4496) BEHAVIORAL: `let got = direct_fill_group(…);` then `assert!(got.pair.is_none(), …)` +
    `assert!(got.fills.is_empty(), "the guard precedes any cassette access — no role fill runs")`. Rewrite
    the fn doc: the OLD "short-circuits ahead of any cassette or filesystem access" premise dies (the helper
    lands the member head(s) — both cases use corpus[0], valid — BEFORE the guard); NEW: the guard still
    precedes cassette access → no role fill runs.
  - scores (~4566): `.pair.expect(…)`. Same one-shell multi-group double-head as reproduce (harmless in
    .1d4a; see [D]). Its `direct_smt_verify_group` passes bare `groups/{gid}` (~4570) = the dir the
    smt_query now shares ([B].2).
  - rejection schema-arm (~4874): `let filled = direct_fill_group(…);` +
    `assert!(filled.pair.is_none(), "schema exhaustion ends the route");`. Ledger
    `[AiSchemaViolation, AiSchemaViolation, RepairLimitExceeded]` HOLDS via break-on-first (overlap's 3
    diagnostics ride the one event → ledger); members guideline_a / guideline_b are valid → head events clean.
  - rejection syntax-arm (~4905): `.pair.expect("the shallow-accepting pair fills")`.
    `shell.ledger().is_empty()` HOLDS — both roles accept → event diagnostics empty → the clean event leaves
    the ledger untouched (head events clean too; the later `shell.events().last()` = the verify event, since
    head / model_fill events precede it).
  - route_metrics arm (~5234): a `for … in worklist` loop over 3 groups under ONE shell (2 golden
    `exp.test_source_groups` + a chained `group.m2_direct_syntax`, ~5205) → rename in the loop body,
    `.pair.unwrap_or_else(|| panic!("{gid}: direct_fill_group yielded no pair"))`. Multi-group like reproduce /
    scores → guideline_a heads 3× (harmless; see [D]): binds only the pair (no `.fills`), and
    `route_metrics(…, &fills, &groups, …)` (~5260) scores the explicit observation vecs, asserting only
    `metrics.rows` / `metrics.diagnostics.is_empty()` — nothing on shell head events.

  gotchas (verified — do not re-derive): (1) content_hash is payload-only → the id-prefix changes disturb
  NO byte pin. (2) break-on-first terminal reject is load-bearing (rejection schema-arm ledger = overlap
  only). (3) a clean event (empty diagnostics) leaves the ledger untouched (rejection syntax-arm). (4) `land`
  takes `&Shell` and auto-reborrows from `&mut shell`; the explicit-match landing form is borrow-obvious (no
  closure). (5) diagnostic / wrap MESSAGE strings are unpinned (write them freely); only codes / counters /
  ids get pinned in .1d4b. (6) the two loop exits differ BY DESIGN: a TERMINAL role reject ([B].6c
  `else { break; }`) breaks → falls through to the ONE event, which CARRIES + ledgers the accumulated
  diagnostics; a role `Err(CassetteError)` ([B].6a) records a command diagnostic + early-returns with NO
  event (mirrors single_ir's infra-error rule). On that Err a prior role's already-landed smt_query is
  ORPHANED but safe — a `pair: None` group never reaches `direct_smt_verify_group` (nothing cites it) and a
  fixed re-run overwrites it. Only `fills` / `identities` survive in the returned `DirectFill`; the prior
  role's accumulated raw `diagnostics` + the event counter totals (`recorded_calls` / `repairs`) are NOT
  emitted (no event fires), and 6a's one command diagnostic in the ledger is the sole shell record. Do NOT "fix"
  this by reading all roles before landing (deontic would accumulate past a terminal overlap reject →
  breaks the schema-arm ledger pin) or by eventing on Err (breaks the no-event infra-error rule).
- [ ] run-m2.1d4b: direct event + landing pin battery (split from .1d4). FIRST swap
  direct_smt_fill_reproduces_m1_query_bodies + direct_smt_route_scores_m1_groups OFF per-group
  `direct_fill_group` onto a per-route head prepass: build each unique `DocHead` once via
  `route_document_head`, dedup by member id (`test_source.m1_guideline_a` is in BOTH groups), pass each
  group's `&[&DocHead]` to `direct_smt_fill(gid, &refs, …)` directly → a shared doc heads ONCE per route.
  THEN extend the two tests: pin the group model_fill event tuple (kind/step_id/outcome/counters/outputs —
  counters summed over roles) + smt_query landed paths (`groups/{gid}/{role}.smt_query.json`, [B].2) + the
  once-per-doc head events (a doc in two groups heads once per route); input_hashes compared AS SETS;
  mirror .1d3b's battery shapes (strict_at landed reads, exact dir listings, slice::from_ref for
  single-hash pins — clippy); pins from OBSERVED output sanity-checked against the .1d4a contract; M1 pins
  untouched. Gate: cargo test.
- [ ] run-m2.1d5a: model-route loop in `execute()` + structural smoke gate (two-run determinism +
  event census = .1d5b). Replace the model-route gate diagnostic (DELETE its test
  m2_experiment_run_gates_until_the_route_loop_lands ≈3093): single M1Layered view → existing path
  verbatim; mixed M1+model set → command diagnostic, zero artifacts; model set → lexicon read +
  `CassetteStore::new(root)` + Z3Adapter::new (each failure → command diagnostic); base seed =
  experiment seed; then per view in set order: mark ledger start (shell.ledger().len());
  SingleIr = per-doc route_document_head →
  single_ir_fill, per-group compile_verify_group (dir `routes/{pid}/groups/{gid}`, smt under it);
  DirectSmt = per-unique-doc route_document_head, per-group direct_smt_fill →
  direct_smt_verify_group. Identity agreement: each fill's model_identity folds into one agreed
  Option<ModelIdentity> — mismatch = command diagnostic, fail-closed return (goldens agree:
  model.baseline/fixture_quant/1.0.0). Collect per view RouteRun{pipeline_id, ledger slice, fills,
  groups: Vec<GroupObservation>, samples: vec![groups.clone()]} (#[allow(dead_code)]; .1e consumes;
  observation shapes = route_metrics test ≈4665 — single_ir pairs from compiled.solver_query_plan,
  direct pairs = minted role ids). Tails run ONCE over all routes' traces (graphs deduped by
  document_id — payload-identical so content_hash equal; results per route, the settled §7.1 shape
  per .1e's populated fixture; report findings/no_conflict rows = single_ir's groups,
  structurally: assemble_trace mints claims only for compiled+verifier_results groups and direct
  lands no compiled (SPEC §9 baseline = model emits SMT directly; findings need region/quoted-span
  provenance direct cannot source) — exactly one view mints finding.{gid}.{seq}, no id collision,
  tails filter nothing; model-route quality reaches the report as .1e's aggregate sections):
  trace_processing_stage + report_processing_stage gain `emit_event:
  bool` — M1 true (bytes unchanged), M2 false (trace/report = undeclared steps of route pipelines;
  tail wrapper producer = baseline view → step id UNUSED_STAGE, honest sentinel); sections stay
  None (.1e populates). Gate test: `write_m2_root` mirror (copy from repo_root(): registry/*.yaml,
  corpus lexicon + 3 html + reference yaml, rust-toolchain.toml, Cargo.lock; plant the 7 golden
  seed-42 cassettes under `<root>/cassettes/`) → executed ONCE: zero command diagnostics; both
  routes' layout present; trace_bundle strict-parses, claims = single_ir's 2 groups only, shared
  source nodes once, direct verifier_results = legal node carrying only its →report out-edge (no
  verify in-edge — validate checks edge endpoints/rank/op, never node connectivity). M1 executed()
  pins unchanged. Gate: cargo test.
- [ ] run-m2.1d5b: two-run determinism + event census over .1d5a's write_m2_root mirror (split
  from .1d5 — the pin-battery half). Execute twice into two out dirs: landed artifacts byte-equal
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
