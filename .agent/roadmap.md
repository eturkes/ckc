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

- [x] model-types: ckc-core model + manifest identity types. Add `ModelIdentity` (`model_id`,
  `quant`, `runtime_version` — mirrors `SolverIdentity`'s identity-only shape; NO prompt hash inside,
  per §9 separating model identity from prompt hashes). Extend `RunManifest`/`ReplayManifest` with the
  §9 measurement record as OMITTABLE additions (None/empty → omit-None keeps M1 canonical bytes + pins
  unchanged; M2 runs populate): `model_identity` + the §9 hash set the manifests currently LACK —
  test-source, reference, schema, prompt-template, model, runtime hashes (only `corpus_hash` /
  `lexicon_hash` exist today). Add the §7.4 model-route diagnostic codes `ai_schema_violation`,
  `ai_hallucinated_source`, `repair_limit_exceeded` to the §4.4 family. Canonical/CanonRead +
  `content_hash` for the new types; byte-pin a populated `ModelIdentity` + manifest fixture. Reading:
  `crates/ckc-core/src/plans.rs`, `enums.rs`; SPEC §9 manifest list (lines 781-795), §7.4, §4.4. Gate:
  `cargo test --workspace` green with M1 manifest pins UNCHANGED (additions omitted for M1); canonical
  round-trip + pinned bytes for `ModelIdentity` and a populated manifest; new codes serialize. 71% 142K/200K
- [x] schemas-export.1a: ClinicalIR JSON-Schema emitter-core (no committed file/oracle). Restore the
  VERIFIED salvage `.agent/wip-schemas-export.rs.txt` → `crates/ckc-cli/src/schema.rs` (`cp` back;
  fix compile nits) + `pub mod schema;` in `crates/ckc-cli/src/lib.rs`; add `serde_json.workspace =
  true` to ckc-cli `[dev-dependencies]`. Emitter `clinical_ir_schema(&Lexicon) -> Vec<u8>` mirrors
  the §4.3 ClinicalIR encoding (sorted-name members, omit-None optionals, set→`uniqueItems` vs
  ordered array, `ContextAtom` 3-branch `oneOf` of `{tag,value}` consts `concept`/`concept_negated`/
  `interval`, string-int `pattern` interval bounds, derived `Action.key`) over `ClinicalIr` + nested
  (`TerminologyBinding`/`ClinicalStatement`/`Action`/`ContextAtom`/`QuantityInterval`/`ExceptionClause`;
  enums `BindingStatus`/`Direction`/`Strength`/`Certainty`) + injects lexicon `enum`s for the
  controlled-vocab fields `system`/`code`+`alternatives`/action `kind`+`target`/concept/`var`.
  Schema = structural oracle (shape+vocab+pattern) for constrained decoding + structural validation;
  the canonical-only invariants it cannot express — `Action.key`=`kind:target` derivation, set-element
  canonical sort-order — stay enforced by `read_strict_canonical`/`IrBundle::validate`, NOT the schema
  (the export never claims full canonical validity). After `cp`, KEEP the emitter + serde_json-parse tests, DROP the jsonschema/committed-file/hash tests
  (→ .1b). Tests (serde_json parse only): bytes parse as JSON; `$defs/ContextAtom` = 3-branch oneOf
  w/ the tag consts; `QuantityInterval` `required==["var"]` + bounds carry the string-int pattern;
  `Action.required` ∋ `key`; concept/action/system/var enums == lexicon sorted vocab. Reading: the
  salvage (primary); external assumptions pre-verified against `ir.rs`/`normalize.rs` this recovery
  (re-read those only to clear a compile error); lib.rs module list. Gate: `cargo test -p ckc-cli`
  green. [Salvage compiled first try (fmt-only nits); shared w/ .1b → deleted at .1b close.] 49% 97K/200K
- [x] schemas-export.1b: committed export + validation oracle + hash-pin. On .1a's emitter: add
  `jsonschema = "0.46"` to `[workspace.dependencies]` (draft-2020-12 validator = dev-only test oracle;
  default features OK for a self-contained schema, lean to `default-features=false` only if
  `validator_for` stays available) + `jsonschema.workspace = true` to ckc-cli `[dev-dependencies]`;
  `mkdir schemas/` (or `create_dir_all` in the writer). Generate committed
  `schemas/clinical_ir.schema.json` (JSON-Schema = engine-agnostic standard) via a `CKC_BLESS=clinical_ir_schema`-gated
  test (exact-value→write, else→compare = drift guard; the exact token stops an ambient/CI `CKC_BLESS`
  silently re-blessing real drift; NEW pattern, no repo precedent). Pin `const SCHEMA_HASH`
  = `hash_bytes(emitted)` (sha256 over canonical bytes). Restore the oracle tests from the salvage:
  drift compare; hash pinned; jsonschema validates a known-good `ClinicalIr` (built from lexicon vocab
  → serde_json `Value`); rejects each malformed case (missing `action`, non-lexicon `code`/`alternatives`,
  non-lexicon action `kind`, bare-number interval bound, unknown member, duplicate set element; NOT an
  out-of-`i64` magnitude bound — `INT_PATTERN` is i64-lexical not i64-bounded, `read_i64` is that backstop).
  Flow: write →
  `CKC_BLESS=clinical_ir_schema cargo test schema::` (gen file) → `sha256sum schemas/clinical_ir.schema.json` → fill
  `SCHEMA_HASH` (prefix `sha256:`) → `cargo test` (green). Reading: the salvage (oracle tests); .1a's
  committed `schema.rs`; SPEC §9 schemas/ export. Gate: `cargo test`; validates good + rejects each
  malformed; `schema_hash` stable; committed bytes pinned. Close: delete `.agent/wip-schemas-export.rs.txt`.
  [Done: `default-features=false` chosen (validator_for/is_valid present, no remote resolvers needed);
  added two malformed cases beyond the plan list — non-lexicon `alternatives` (guards the codex
  ConceptCode parity) + a non-canonical string bound `"1.5"` (proves INT_PATTERN `pattern` enforced,
  not just `string` type); salvage deleted. Codex M2.3 follow-up: bless mechanism CKC_BLESS-gate →
  drift guard (never writes) + `#[ignore]`d bless test (env-leak-proof; the durable pattern now in
  memory.md); each malformed case pins its rejection reason `(instance_path, schema_path)` via
  `iter_errors()`; valid_ir enriched (concept_negated + non-empty `alternatives` + multi-element set),
  overclaiming comment corrected.] 55% 110K/200K
- [x] schemas-export.2: direct_smt SMT-LIB grammar + committed export. PRE-DERIVED (this unit's prep
  commit): notation + oracle SOTA-selected, grammar authored + validated + hashed → the WORK-UNIT is
  mechanical wire+gate, ZERO re-derivation; read ONLY this line, then open `emit.rs` to place two tests.
  LOCKED decisions: notation = standard BNF (engine-agnostic, the canonical base of EBNF/GBNF →
  transforms to a grammar-constrained-decoding dialect downstream; rejected pest/PEG = tool-coupled).
  Oracle = `bnf` crate 0.6, dev-dep, `default-features = false` (drops the serde grammar-(de)serialize
  feature the oracle never uses; build + run validated). Recognizer = Earley, FULL-match: `let g:
  bnf::Grammar = txt.parse()?; let p = g.build_parser()?; let ok = |s: &str|
  p.parse_input(s).next().is_some();`. Grammar HAND-AUTHORED (the file IS the source — no code emitter,
  no bless/CKC_BLESS; lighter than .1b: a lone hash-pin is the whole drift guard).
  ARTIFACT staged byte-exact at `.agent/wip-smt_query.grammar`
  (sha256:fb42ee5a92d7ee445aad077095aabf0ba1016f2c56d79b1e815ff831a75d0be1, 2512 B; validated by a
  scratch `bnf` harness reading the exact file bytes — parses as BNF, builds a parser, recognizes the
  live conflict + control Q1/Q2 + the degenerate body, rejects all 14 malformed cases). Shape (review-
  only): the profile SURFACE = a union of valid constructs, NOT per-query logic/produce/result coupling
  (those exact bytes stay §8.6-pinned in `emit.rs`). `bnf` = pure BNF (no EBNF `{}`/`[]`) → repetition
  is recursion + `""` base; `<nl>`'s body is a LITERAL newline byte; `;` line-comments parse (header
  kept), `#`/`(*` do not; trailing garbage rejected (full-match).
  STEPS: (1) `cp .agent/wip-smt_query.grammar schemas/smt_query.grammar` (byte-exact restore; `schemas/`
  already exists from .1b) → confirm `sha256sum schemas/smt_query.grammar` == the hash above. (2) Deps:
  root `[workspace.dependencies]` += `bnf = { version = "0.6", default-features = false }` (comment it
  like the jsonschema entry — dev-only Earley BNF grammar oracle); ckc-smt `Cargo.toml` += a NEW
  `[dev-dependencies]` block with `bnf.workspace = true`. (3) `emit.rs` `mod tests` += `hash_bytes` on
  the existing `use ckc_core::{…}` line + two `#[test]`s:
  • `grammar_hash_is_pinned` (drift guard): `const GRAMMAR_PATH: &str =
  concat!(env!("CARGO_MANIFEST_DIR"), "/../../schemas/smt_query.grammar");` `const GRAMMAR_HASH: &str =
  "sha256:fb42ee5a92d7ee445aad077095aabf0ba1016f2c56d79b1e815ff831a75d0be1";`
  `assert_eq!(hash_bytes(&std::fs::read(GRAMMAR_PATH).unwrap()).as_str(), GRAMMAR_HASH);` (editing the
  file flips the hash → fails; mirrors `schema.rs` `schema_hash_is_pinned`).
  • `grammar_recognizes_emitted_and_rejects_malformed`: build the recognizer from
  `std::fs::read_to_string(GRAMMAR_PATH)` per the snippet above. ACCEPT (live emitter `.body`, reusing
  the test fixtures): conflict `emit_overlap_query`/`emit_deontic_query` over
  `plan_pair("group.m1_conflict", &doc_a(), &doc_b())`; control over `plan_pair("group.m1_no_conflict",
  &control(), &doc_a())`; the degenerate body — replicate `degenerate_and_numeral_forms`'s x/y build
  (`fc`/`dnf1`/`interval(…,Some(-5),…,Some(40))`/nested `or`+`and`) + its overlap emit (covers `(- n)`,
  nested or/and, bare single-atom disjunct). REJECT (`assert!(!ok(&s))`, mutate the minimal-valid base):
  unquoted-symbol (`x` for `|x|`), bare-negative (`-5` not `(- 5)`), leading-zero (`018`), bad-logic
  (`QF_NIA`), bad-sort (`Int`), missing-trailing-nl, single-arg-`(and |x|)`, unknown-command (`(push
  1)`), uppercase-id (`|X|`), one-line-spaces (`\n`→space), missing-`:named`, trailing-garbage
  (`(extra)\n` appended), declare-before-`set-logic`, missing-`(check-sat)`. Minimal-valid base =
  `"(set-logic QF_UF)\n(set-option :print-success false)\n(set-option :produce-unsat-cores
  true)\n(declare-const |x| Bool)\n(assert (! |x| :named |a.r|))\n(check-sat)\n(get-unsat-core)\n"`.
  GATE: `cargo test --workspace` (oracle + hash green, M1 §8.6 pins unchanged); `cargo fmt --check`;
  `cargo clippy --workspace --all-targets -- -D warnings`. CLOSE: `rm .agent/wip-smt_query.grammar`;
  record context-usage; mark DONE (M2 stays IN-PROGRESS — later units remain). 55% 110K/200K
- [x] registry-m2.1: `registry/{prompts,schemas}.yaml` entry types + loaders. Add `SchemaEntry`
  (`id`, `path`, `schema_hash`, `target_kind`) + `PromptEntry` (`id`, `path`/inline,
  `template_hash`, `route`) to `registry.rs`; serde loaders + `ckc registry check` coverage
  (file existence, `schema_hash` match vs committed `schemas/`, id uniqueness). Seed
  `registry/schemas.yaml` (`clinical_ir`, `smt_query` — both schema files exist by now); define
  `PromptEntry` loading (per-route prompt files authored later by the route units, which add their
  entry + final hash; none seeded here → no dangling ref). Reading: `crates/ckc-core/src/registry.rs`
  entry types/loaders/check, `crates/ckc-cli/src/registry_check.rs`, `registry/*.yaml`; SPEC §14 (M2
  adds prompts|schemas). Gate: `cargo test` registry; `ckc registry check` passes with the new files;
  the check rejects a missing / hash-mismatched schema. [Done: `Hash`-typed `schema_hash`/`template_hash`
  (grammar-checked on load); pure `validate_model_registry` (id-uniqueness + nonempty path + exactly-
  one-of path|inline via `PromptSource` finding) kept SEPARATE from `validate_registries` (no call-site
  churn); file-existence + `schema_hash`-vs-`schemas/` checks live at the CLI as `schema_invalid`/`invalid`
  diagnostics (I/O out of the pure validator); `load_optional` → absent prompts.yaml/schemas.yaml is clean
  (M1 `check` tests unchanged); `prompts.yaml` unseeded (no dangling route ref);
  `committed_model_surface_checks_ok` guards live drift.] 82% 164K/200K
- [x] registry-m2.2: experiment pipeline-set binding — type + validation + §14 wording. Generalize
  `ExperimentEntry`'s singular `pipeline: Id` to a pipeline SET — add `pipelines: Vec<Id>` +
  `baseline_pipeline: Id` (the §7.3 delta baseline); keep M1's single-pipeline entries valid (accept
  `pipeline` as a one-element set / default). `validate_registries` validates `baseline_pipeline ∈
  pipelines` (each bound pipeline's stage chain is already covered by the existing §8.4 Dangling +
  ChainBreak rules). Amend SPEC §14/§8 registry wording for the pipeline-set binding (same deliverable
  as the type — light prose, no test mirrors §14's bytes; verified no §14 byte-pin). Gate uses a
  SYNTHETIC fixture (a throwaway experiment binding throwaway pipelines) — real `exp.m2_multihop` is
  NOT seeded here (its pipelines do not exist until the route units; seeding now would dangle and fail
  `check`). Reading: `crates/ckc-core/src/registry.rs` `ExperimentEntry` + `validate_registries`,
  `crates/ckc-cli/src/registry_check.rs`; SPEC §8 registry-check, §14. Gate: `cargo test`;
  multi-pipeline validation passes on a synthetic fixture + rejects missing-baseline /
  baseline-not-in-set; `exp.m1_scaffold` still validates. [Decision pinned: experiment binds a
  pipeline set + baseline; faithful to §9 "both routes execute over identical locked inputs
  (`exp.m2_multihop`)". Real `exp.m2_multihop` seeded in run-m2.1, once both route pipelines exist.]
  [Done: dual binding forms — legacy `pipeline: Option<Id>` + set `pipelines: Vec<Id>`/`baseline_pipeline`,
  all skip-empty so the M1 `pipeline:` key stays valid + each form round-trips to its own shape;
  `baseline()`/`resolved_pipelines()` accessors normalize both; form-aware `validate_registries` (match on
  `(&pipeline, pipelines.as_slice())`) with new `BaselineNotInSet` + `PipelineBinding` findings (CLI consumes
  findings via Display → no `registry_check.rs` change); `run.rs` executes the single `baseline()` + records
  `pipelines: [baseline]`, behavior-locked to M1 (run-m2.1 completes the multi-route loop); SPEC §14 ledger
  amended, §8.4 left M1-singular (no §14 byte-pin).] 66% 131K/200K
- [ ] run-refactor: behavior-locked per-group back-end extraction (per-group scope, user-confirmed
  — `git show` this respec commit for the rejected per-doc/full-tail alternatives + rationale).
  Extract `ckc-cli` `run.rs` `group_pipeline`'s compile→verify body (SPEC §9 "compile→verify back
  end") into a reusable fn keyed on the member IrBundles — contract `compile_verify_group(group_id:
  &Id, members: &[&ArtifactWrapper<IrBundle>], resolved: &Resolved, adapter: &Z3Adapter, shell: &mut
  Shell) -> GroupTrace` (fit the exact sig — incl. stage-clock/trace threading — to the read code;
  the contract is the spec, not byte-exact). `group_pipeline` keeps building `members` from `docs`
  (the DocTrace lookup + member-missing early-return) then delegates; M2 `route.single_ir` later
  feeds its own validated bundles to the same fn. Boundary: move `inputs`(~L487) through
  `trace.compiled = Some(compiled)`(~L563) into the new fn; setup + member-build (~L443-486) stay.
  NO re-derivation — members already carry `formal`/`norm` (compile reads `m.payload.formal`/`.norm`
  unchanged), identical code path → byte-identical artifacts. Per-doc derive fns
  (`derive_norm_ir`/`assemble`/`FormalIr::derive`, already pub) are OUT of scope: route units compose
  them directly. Reading: ONLY `run.rs` `group_pipeline` (~L443-563) + `finish_processing_stage`
  (~L1019-1045) + `GroupTrace` def (`trace.rs`, to fix the return shape) — nothing else (per-doc fns + ckc-smt `emit`/`verdict` sigs untouched). Gate: `cargo
  test --workspace` green, ZERO test edits; behavior-lock = `cli_shell.rs` `events.len()==19` +
  `run_oracle.rs` §8.6 compiled-body pins (Q1/Q2) + `assert_group_matches_reference` (no
  compile/verify event-shape pin exists → this is a pure method-move). [Refactor-first rule: share
  internals before the route feature. Per-doc head/derive sharing deferred to the route units.]
- [ ] model-adapter.1: generic env-command ModelAdapter — identity + invoke skeleton. New ckc-cli
  adapter module mirroring `verify.rs` `Z3Adapter`: `ModelAdapter::with_command(name)` resolves a
  BARE command name on PATH (Z3 precedent — `Z3Adapter` runs `z3` by bare name, no literal path / no
  committed config), env-var-overridable; the committed default is a neutral role name, never an
  engine name or absolute path. Probe (a `--version`-style call) for the runtime's self-reported
  identity → `ModelIdentity`. `invoke(prompt, constraint, seed, budget) -> ModelRun{outcome,
  stdout_bytes, stderr}` with `ModelOutcome` = `Completed{bytes}`/`Timeout`/`ExitFailure{code}`/
  `SpawnFailure{error}`; helpers mirror `run_process`/`spawn_piped`. Committed CLI contract: the
  command takes prompt + constraint (schema/grammar path) + seed (args/stdin) and writes generated
  bytes to stdout. Reading: `crates/ckc-smt/src/verify.rs` Z3Adapter (PATH resolution, `spawn_piped`);
  `crates/ckc-core/src/plans.rs` SolverIdentity/ModelIdentity; SPEC §9 recorded-subprocess Z3 pattern.
  Gate: `cargo test`; probe + invoke drive a committed stub-command fixture (on PATH) deterministically;
  identity parses; outcome enum covers spawn/timeout/exit. [Decision pinned: bare PATH command name +
  committed CLI contract; the wrapper binary is environment-supplied outside git.]
- [ ] model-adapter.2: constrained generation + k-sample (live). Complete `invoke` for real
  constrained decoding — pass the route's grammar/JSON-Schema (from `schemas/`), greedy, fixed seed;
  k-sample convergence draws k recorded samples via per-sample seeds (`seed_i = f(base_seed, i)`);
  collect k outputs + recorded-call count. Reading: model-adapter.1 module; `schemas/` outputs; the
  M2 runtime (memory `## Runtime` note + `CLAUDE.local.md` env setup). Gate: `cargo test` (logic vs a recorded fixture); LIVE confirm via the env
  command on a real M1 source (runtime indirection) — greedy byte-stable, schema-constrained, k
  samples reproducible. [Gate MET last session; this unit re-confirms functionally.]
- [ ] model-cassette: recorded model I/O as test-source artifacts + replay. Record each model call's
  prompt + output as an `ArtifactWrapper` test-source artifact (tracked `corpus/test_sources/` class —
  origin `ai_generated`, evidence `evidence_discovery_only`, `prompt_template_hash` in the manifest),
  keyed by (route, source, seed); live calls gated behind an explicit experiment/`--record` flag,
  default replays the recordings → deterministic, runtime-absent. Extend `replay.rs` hash-compare to
  cover model artifacts. Reading: `crates/ckc-cli/src/replay.rs`, `shell.rs` artifact writes;
  `crates/ckc-core/src` ArtifactWrapper + origin/evidence enums; SPEC §9 recorded-bytes replay, §7.1.
  Gate: `cargo test`; a recorded sample replays byte-identical with the runtime command ABSENT;
  replay-manifest hashes match. [Acceptance: recorded model I/O replays byte-stably.]
- [ ] stage-model-fill.1: model-fill processing stage — core (generic over target). New stage kind
  `model_fill`: invoke `ModelAdapter` with the route prompt + constraint, parse output → target
  artifact (ClinicalIR JSON or SMT text; target-generic, config-selected), §4 acceptance-check the
  parse, emit `ai_schema_violation` on parse/schema fail, emit a §4.6 EventRecord carrying the
  recorded-call count. Seed the `model_fill` `ProcessingStageEntry` (`candidates.yaml`: `kind` =
  `model_fill`, input/output artifact kinds) so route pipelines can reference it. No repair loop /
  grounding yet (→ .2). Reading: `crates/ckc-cli/src/run.rs` stage-event pattern; the adapter +
  cassette modules; ckc-core acceptance/validate; `registry.rs` ProcessingStageEntry; SPEC §7.4, §8.4.
  Gate: `cargo test`; stage records a valid fill + an `ai_schema_violation` on crafted fixtures;
  recorded-call accounting exact; `model_fill` stage entry validates.
- [ ] stage-model-fill.2: model-fill repair loop + grounding. Extend the stage with a repair loop —
  re-prompt on schema-violation up to `repair_limit` (from budget), counting repairs, emitting
  `repair_limit_exceeded` on exhaustion; a grounding check — a referenced upstream id absent from the
  deterministic upstream → `ai_hallucinated_source`; the §4.6 EventRecord carries the repair count.
  Reading: stage-model-fill.1 module; ckc-core acceptance; SPEC §7.3 repair count, §7.4. Gate:
  `cargo test`; stage records repair-then-recover, `repair_limit_exceeded`, and `ai_hallucinated_source`
  on crafted fixtures; repair accounting exact. [Split from .1: core fill vs the repair/grounding
  rejection-coverage sub-feature.]
- [ ] route-single-ir: `single_ir@ClinicalIR` route + pipeline. Seed `pipe.m2_single_ir`
  `PipelineEntry` (`candidates.yaml`: deterministic extract+segment → `model_fill`(target=ClinicalIR,
  schema=`clinical_ir`) → assemble `IrBundle` → bundle-validate → the run-refactor tail; stage chain
  validates — M1 stages + `model_fill` from stage-model-fill.1). Implement the route: extract+segment
  supply real upstream ids/regions → model fills ClinicalIR over them → assemble (model ClinicalIR +
  deterministic up/downstream) → §4 bundle-validate (acceptance; hallucinated `source_segment_ids`/
  `region_ids` → `ai_hallucinated_source`) → run-refactor deterministic tail → verdict. Author the
  JA→ClinicalIR prompt (`registry/prompts.yaml`, hashed). Reading: the refactored tail fn,
  stage-model-fill, schemas-export.1a/.1b, segment/normalize; SPEC §9 single_ir, §8. Gate: `cargo test`;
  `ckc registry check` validates `pipe.m2_single_ir`; the route produces a scoreable verdict for an
  accepted ClinicalIR over a recorded cassette; acceptance + §7.4 codes wire through; verdict scored
  vs reference for accepted translations. [Decision pinned: model fills ClinicalIR over deterministic
  upstream — the instrument supplies the grounding scaffold; hallucinated refs are measured, not fatal.]
- [ ] route-direct-smt: `direct_smt` route + pipeline (the weak baseline). Seed `pipe.m2_direct_smt`
  `PipelineEntry` (`candidates.yaml`: `model_fill`(target=SMT, grammar=`smt_query`) →
  syntactic-validity → verify; stage chain validates). Implement: the model emits the
  contradiction-query SMT (Q1 overlap + Q2 deontic) per conflict pair directly, no IR →
  syntactic-validity check (solver parse) → `verdict::verify` via `Z3Adapter` + `assemble_result` →
  verdict. Author the JA(pair)→SMT prompt (hashed). Reading: schemas-export.2, stage-model-fill,
  ckc-smt `verify` (Z3Adapter/assemble_result) + `emit.rs` query structure; SPEC §9 direct_smt. Gate:
  `cargo test`; `ckc registry check` validates `pipe.m2_direct_smt`; the route runs a recorded SMT
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
  model/prompt/identity hash fields), over the locked M1 inputs. Tested via REPLAY of the route units'
  committed cassettes (deterministic, no live call). Reading: `run.rs` execute (route loop), the
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
