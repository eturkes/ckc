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

## M2 multi-hop PoC — plan PENDING

Scope = SPEC §9: experiment 1's minimal pair on this laptop. A weak local model translates the
M1 test sources two ways — `route.direct_smt` (model emits SMT-LIB directly, the baseline) versus
`route.single_ir` (model fills one grammar-constrained IR schema, then deterministic compile) —
scored by the M1 pipeline as instrument; published as a bilingual research report. Exactly two
routes (§10 widens the route axis). Elaboration pick: `single_ir` fills **ClinicalIR** (fully
closed-vocab — lexicon codes / enums / ints, zero free text → tractable constrained decoding, max
deterministic leverage). Per the "test all layer configurations" directive, the full single_ir
layer gradient (every meaningful IR layer + the DMN-style alt) defers to M3 / §10 — recorded in
`.agent/memory.md` as the M3 route-axis seed. Milestone gate (model runtime) MET last session
(functionally confirmed); not a §15 gate — M2 results are locked measurements that stand on their
own. The local-model runtime is an environment-provided command invoked Z3-style; its
implementation lives outside git (CLAUDE.local.md), so no unit commits an engine name, dialect, or
model format. Live units feed deny-Read sources via runtime indirection (a script opens the path;
the path never appears in a Read/Bash argument).

- [ ] model-types: ckc-core model + manifest identity types. Add `ModelIdentity` (`model_id`,
  `quant`, `runtime_version`, `prompt_template_hash`) mirroring `SolverIdentity`; extend
  `RunManifest`/`ReplayManifest` with the §9 minimal measurement record — `schema_hashes`,
  `prompt_template_hashes`, `model_identity`, runtime hash beside the existing test-source /
  reference hashes (the "evaluator identity"). Add the §7.4 model-route diagnostic codes
  `ai_schema_violation`, `ai_hallucinated_source`, `repair_limit_exceeded` to the §4.4 family.
  Canonical/CanonRead + `content_hash` for the new types; byte-pin their canonical forms.
  Reading: `crates/ckc-core/src/plans.rs`, `enums.rs`; SPEC §9 manifest list, §7.4, §4.4. Gate:
  `cargo test` ckc-core green; canonical round-trip + pinned bytes for `ModelIdentity` and the
  extended manifests; new codes serialize.
- [ ] schemas-export.1: ClinicalIR JSON-Schema emitter + committed export. Hand-write an emitter
  mirroring the ClinicalIR canonical encoding (sorted-name members, omit-None optionals, §4.3-set
  vs ordered-array, `ContextAtom` tagged-union `{tag,value}`, string-quoted interval ints, derived
  `Action` key) over `ClinicalIr` + nested (`TerminologyBinding`, `ClinicalStatement`, `Action`,
  `ContextAtom`, `QuantityInterval`, `ExceptionClause`; enums `BindingStatus`/`Direction`/
  `Strength`/`Certainty`); inject lexicon-derived `enum` constraints for the controlled-vocab Id
  fields (`system`/`code`/action `kind`+`target`/concept/`var`) from `ja_core.yaml`. Land committed
  `schemas/clinical_ir.schema.json` (JSON-Schema = engine-agnostic standard) + a `schema_hash` over
  its canonical bytes. Reading: `crates/ckc-core/src/ir.rs` ClinicalIR types + canon writer;
  `corpus/lexicon/ja_core.yaml`; SPEC §9 schemas/ export, §4/§5. Gate: `cargo test`; emitted schema
  validates a known-good ClinicalIR instance; `schema_hash` stable; committed bytes pinned.
- [ ] schemas-export.2: direct_smt SMT-LIB grammar + committed export. Author a neutral-notation
  grammar (EBNF/ABNF — engine-agnostic, no dialect name) constraining output to the `emit.rs` SMT
  surface: `(set-logic QF_LRA|QF_UF)`, `(set-option …)`, `(declare-const |sym| Bool|Real)`,
  `(assert (! <term> :named |name|))`, `(check-sat)`, `(get-model)`/`(get-unsat-core)`; term grammar
  = `and`/`or`/`not`, `|c|`, the four interval relations over `|v|` and an int, negative `(- N)`,
  deontic `|pos:<action_key>|`. Land committed `schemas/smt_query.grammar` + `schema_hash`. Reading:
  `crates/ckc-smt/src/emit.rs` emitted surface; SPEC §8.6 smt pins, §6, §9 schemas/. Gate:
  `cargo test`; grammar accepts the M1 emitted Q1/Q2 bodies (pinned smt2), rejects malformed;
  `schema_hash` stable; committed bytes pinned.
- [ ] registry-m2.1: `registry/{prompts,schemas}.yaml` entry types + loaders. Add `SchemaEntry`
  (`id`, `path`, `schema_hash`, `target_kind`) + `PromptEntry` (`id`, `path`/inline,
  `template_hash`, `route`) to `registry.rs`; serde loaders + `ckc registry check` coverage
  (file existence, `schema_hash` match vs committed `schemas/`, id uniqueness). Seed
  `registry/schemas.yaml` (`clinical_ir`, `smt_query`); define `PromptEntry` loading (per-route
  prompt files authored later by the route units, which add their entry + final hash). Reading:
  `crates/ckc-core/src/registry.rs` entry types/loaders/check, `registry/*.yaml`; SPEC §14 (M2 adds
  prompts|schemas). Gate: `cargo test` registry; `ckc registry check` passes with the new files;
  loader rejects a missing / hash-mismatched schema.
- [ ] registry-m2.2: experiment multi-route binding + per-route validation. Extend `ExperimentEntry`
  to bind both M2 routes under one experiment — add `routes` (route pipeline ids) + `baseline_route`
  (designates `direct_smt` as the §7.3 delta baseline); keep single-`pipeline` M1 entries valid
  (`routes` optional). `ckc registry check` validates each route pipeline's processing-stage chain
  (input/output artifact-kind continuity) + `baseline_route` ∈ `routes`. Seed `exp.m2_multihop`
  (`routes=[pipe.m2_direct_smt, pipe.m2_single_ir]`, `baseline_route=pipe.m2_direct_smt`, the M1
  groups, seed, budget incl. k-sample count + repair limit). Amend SPEC §14/§8 registry wording for
  the route-set binding. Reading: `crates/ckc-core/src/registry.rs` `ExperimentEntry` + validation,
  `registry/{experiments,candidates}.yaml`; SPEC §8 registry-check, §14. Gate: `cargo test`;
  `ckc registry check` validates `exp.m2_multihop` + both route chains + rejects an unbalanced chain
  / missing baseline; `exp.m1_scaffold` still validates. [Decision pinned: one experiment binds both
  routes + baseline; faithful to §9 "both routes execute over identical locked inputs (`exp.m2_multihop`)".]
- [ ] run-refactor: behavior-locked deterministic-tail extraction. Refactor `ckc-cli` `run.rs` to
  expose the deterministic ClinicalIR→verdict tail as a reusable fn chaining `derive_norm_ir` →
  `FormalIr::derive` → `emit::compile` → `verdict::verify`, so both the M1 pipeline and
  `route.single_ir` call it. Zero behavior change — existing M1 tests (run oracle, §8.6 byte pins)
  are the gate, unedited. Reading: `crates/ckc-cli/src/run.rs` execute + per-group compile/verify;
  `rules.rs`/`normalize.rs`/`segment.rs` + ckc-smt `emit`/`verdict` signatures. Gate: `cargo test
  --workspace` green with ZERO test edits; `exp.m1_scaffold` run oracle + §8.6 pins unchanged.
  [Refactor-first rule: share internals before the route feature.]
- [ ] model-adapter.1: generic env-command ModelAdapter — identity + invoke skeleton. New ckc-cli
  adapter module mirroring `verify.rs` `Z3Adapter`: `ModelAdapter::with_command(<cmd from run
  config>)` probes the environment-provided runtime command (config-declared path, env-overridable)
  for its self-reported identity → `ModelIdentity` (no engine name in committed code — Z3 precedent);
  `invoke(prompt, constraint, seed, budget) -> ModelRun{outcome, stdout_bytes, stderr}` with
  `ModelOutcome` = `Completed{bytes}`/`Timeout`/`ExitFailure{code}`/`SpawnFailure{error}`; helpers
  mirror `run_process`/`spawn_piped`. Committed CLI contract: the command takes prompt + constraint
  (schema/grammar path) + seed (args/stdin) and writes generated bytes to stdout. Reading:
  `crates/ckc-smt/src/verify.rs` Z3Adapter pattern; `crates/ckc-core/src/plans.rs`
  SolverIdentity/ModelIdentity; SPEC §9 recorded-subprocess Z3 pattern. Gate: `cargo test`; probe +
  invoke drive a committed stub-command fixture deterministically; identity parses; outcome enum
  covers spawn/timeout/exit. [Decision pinned: env command + committed CLI contract; the wrapper impl
  lives outside git (CLAUDE.local.md), like `intel-accel/`.]
- [ ] model-adapter.2: constrained generation + k-sample (live). Complete `invoke` for real
  constrained decoding — pass the route's grammar/JSON-Schema (from `schemas/`), greedy, fixed seed;
  k-sample convergence draws k recorded samples via per-sample seeds (`seed_i = f(base_seed, i)`);
  collect k outputs + recorded-call count. Reading: model-adapter.1 module; `schemas/` outputs;
  CLAUDE.local.md runtime. Gate: `cargo test` (logic vs a recorded fixture); LIVE confirm via the env
  command on a real M1 source (runtime indirection) — greedy byte-stable, schema-constrained, k
  samples reproducible. [Gate MET last session; this unit re-confirms functionally.]
- [ ] model-cassette: recorded model I/O as test-source artifacts + replay. Record each model call's
  prompt + output as an `ArtifactWrapper` test-source artifact (origin `ai_generated`, evidence
  `evidence_discovery_only`, `prompt_template_hash` in the manifest), keyed by (route, source, seed);
  live calls gated behind an explicit experiment/`--record` flag, default replays the recordings →
  deterministic, runtime-absent. Extend `replay.rs` hash-compare to cover model artifacts. Reading:
  `crates/ckc-cli/src/replay.rs`, `shell.rs` artifact writes; `crates/ckc-core/src` ArtifactWrapper +
  origin/evidence enums; SPEC §9 recorded-bytes replay, §7.1. Gate: `cargo test`; a recorded sample
  replays byte-identical with the runtime command ABSENT; replay-manifest hashes match. [Acceptance:
  recorded model I/O replays byte-stably.]
- [ ] stage-model-fill: model-fill processing stage (generic over target). New stage kind
  `model_fill`: invoke `ModelAdapter` with the route prompt + constraint, parse output → target
  artifact (ClinicalIR JSON or SMT text), §4 acceptance-check the parse, run a repair loop (re-prompt
  on schema-violation up to `repair_limit` from budget, counting repairs) emitting the §7.4 codes —
  `ai_schema_violation` (parse/schema fail), `ai_hallucinated_source` (ref id absent from the
  deterministic upstream), `repair_limit_exceeded` (loop exhausted); emit a §4.6 EventRecord carrying
  recorded-call + repair counts. Target-generic (config selects ClinicalIR-parse vs SMT-passthrough).
  Reading: `crates/ckc-cli/src/run.rs` stage-event pattern; the adapter + cassette modules; ckc-core
  acceptance/validate; SPEC §7.3 repair count, §7.4. Gate: `cargo test`; stage records each code on
  crafted fixtures (valid; schema-violation+repair; hallucinated-ref; limit-exceeded);
  recorded-call accounting exact.
- [ ] route-single-ir: `single_ir@ClinicalIR` route + pipeline. `pipe.m2_single_ir` = deterministic
  extract+segment (real upstream ids/regions) → `model_fill`(target=ClinicalIR, schema=`clinical_ir`)
  → assemble `IrBundle` (model ClinicalIR + deterministic up/downstream) → §4 bundle-validate
  (acceptance; hallucinated `source_segment_ids`/`region_ids` → `ai_hallucinated_source`) →
  run-refactor deterministic tail → verdict. Author the JA→ClinicalIR prompt (`registry/prompts.yaml`,
  hashed). Reading: the refactored tail fn, stage-model-fill, schemas-export.1, segment/normalize;
  SPEC §9 single_ir, §8. Gate: `cargo test`; route produces a scoreable verdict for an accepted
  ClinicalIR over a recorded cassette; acceptance + §7.4 codes wire through; verdict scored vs
  reference for accepted translations. [Decision pinned: model fills ClinicalIR over deterministic
  upstream — the instrument supplies the grounding scaffold; hallucinated refs are measured, not fatal.]
- [ ] route-direct-smt: `direct_smt` route + pipeline (the weak baseline). `pipe.m2_direct_smt` =
  `model_fill`(target=SMT, grammar=`smt_query`) emitting the contradiction-query SMT (Q1 overlap + Q2
  deontic) per conflict pair directly, no IR → syntactic-validity check (solver parse) →
  `verdict::verify` via `Z3Adapter` + `assemble_result` → verdict. Author the JA(pair)→SMT prompt
  (hashed). Reading: schemas-export.2, stage-model-fill, ckc-smt `verify` (Z3Adapter/assemble_result)
  + `emit.rs` query structure; SPEC §9 direct_smt. Gate: `cargo test`; route runs a recorded SMT
  sample through verify to a verdict; syntactic validity recorded; verdict scored vs reference.
  [Decision pinned: per-pair Q1/Q2 emission for comparability with the M1 group verdict; packaging
  finalized in-unit.]
- [ ] metrics-m2.1: route-quality raw-row metrics. New metrics module → per-route raw rows over a
  run: schema-valid rate, acceptance rate, repair count, recorded-call counts, target syntactic
  validity (solver parse), conflict-verdict accuracy vs reference over the §8 conflict + no-conflict
  groups (no-conflict first-class); exact-fraction values, zero-denominator → `not_applicable`,
  unavailable omitted + diagnostic. Reading: run/report data shapes, reference loader; SPEC §7.3
  route quality, §9 scoring. Gate: `cargo test`; metrics correct on a recorded two-route run fixture;
  fraction / NA / omission rules hold.
- [ ] metrics-m2.2: k-sample stability + baseline-delta. k-sample verdict stability/convergence
  (per-route verdict agreement across k samples); baseline-delta table = per-metric (route −
  baseline) over identical inputs (baseline = `direct_smt` per `exp.m2_multihop`), raw rows emitted
  BEFORE the delta table. Reading: metrics-m2.1 module, the experiment baseline designation; SPEC §7.3
  baseline delta + k-sample, §9 raw-rows-before-ranking. Gate: `cargo test`; stability + delta correct
  on a fixture; raw-rows-before-delta ordering asserted. [Acceptance: raw rows emit before the
  baseline-delta table.]
- [ ] report-m2.1: report.json M2 extension. Extend `assemble_report` + the `report.json` canonical
  shape with per-route raw rows, the baseline-delta table, findings carrying quoted Japanese source
  spans + named assertions, a failure-taxonomy summary (§6 categories + §7.4 codes), model + solver
  identities, replay status, metrics (M2+). Canonical/CanonRead + byte-pin. Reading:
  `crates/ckc-cli/src/report.rs` assemble_report + Report types; the metrics modules; SPEC §7.2, §9
  report.json contents. Gate: `cargo test`; report.json assembles from a recorded run with every M2
  section; canonical bytes pinned; §0 vocabulary in the wording fields.
- [ ] report-m2.2: bilingual rendering. Render `report_en.md` (extend the M1 renderer with M2
  metrics/delta/taxonomy) + new `report_ja.md` (deterministic Japanese rendering of the same canonical
  report.json); §0 locked-measurement wording, no clinical claims; quoted JA spans verbatim. Reading:
  `report.rs` render_markdown + report-m2.1 payload; SPEC §7.2 (report_ja from M2), §0 vocabulary.
  Gate: `cargo test`; both md files render deterministically (byte-stable) from one report.json; the
  JA rendering is well-formed; §0 vocabulary asserted.
- [ ] run-m2: `ckc run --experiment exp.m2_multihop` end-to-end. Wire both route pipelines into one
  experiment run — per-route `model_fill` (cassette-recorded under the experiment flag) → scoring →
  metrics → `report.json` + `report_en.md` + `report_ja.md` + run/replay manifests, over the locked M1
  inputs. Live-pin battery over the run binary. Reading: `run.rs` execute (route loop), the routes,
  metrics, report, manifests; SPEC §1 command, §9. Gate: the command lands all artifacts; live pins on
  report.json sections + manifest evaluator-identity hashes; the recorded run is committed/referenced
  under `runs/`. LIVE: record the full cassette via the env runtime command. [Live-pin-over-run-binary
  = its own unit.]
- [ ] acceptance-m2: §9 acceptance. Verify the §9 themes against the recorded run — both routes
  execute over identical locked inputs (`exp.m2_multihop`); recorded model I/O replays byte-stably
  (runtime absent); raw rows before the baseline-delta table; expected conflict/no-conflict per
  reference for accepted translations; the bilingual report renders deterministically from
  report.json; §0 vocabulary holds. Tag `accept/m2`. Reading: all M2 artifacts + the §9 acceptance
  themes. Gate: all six themes pass on the recorded run; `ckc replay` byte-matches; tag `accept/m2`.
