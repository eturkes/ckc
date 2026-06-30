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
  LOCKED decisions: notation = standard BNF (engine-agnostic, the canonical base →
  transforms to a grammar-constrained-decoding dialect downstream; rejected tool-coupled alternatives).
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
  (those exact bytes stay §8.6-pinned in `emit.rs`). `bnf` = pure BNF (no extended-grammar `{}`/`[]`) → repetition
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
- [x] run-refactor: behavior-locked per-group back-end extraction into a private `run.rs` fn
  `compile_verify_group` (per-group scope, user-confirmed — `git show 93953c4` for the rejected
  per-doc/full-tail alternatives + rationale). Extract `ckc-cli` `run.rs` `group_pipeline`'s
  compile→verify body (SPEC §9 "compile→verify back end") so a later `route.single_ir` can feed its
  own validated bundles to the same back end (route-side `Resolved`/producer wiring is THAT unit's
  job — refactor-first: share internals now). PINNED SIGNATURE (complete — no threading left to
  derive): `fn compile_verify_group(group_id: &Id, members: &[&ArtifactWrapper<IrBundle>], clock:
  ProcessingStageClock, resolved: &Resolved, adapter: &Z3Adapter, shell: &mut Shell) ->
  (Option<ArtifactWrapper<CompiledArtifact>>, Option<ArtifactWrapper<VerifierResults>>)` — a private
  `fn` (only `group_pipeline` calls it now; a route unit widens visibility if it lands in another
  module). BODY = the current `let inputs`(~L487) → verify-landing block verbatim, plus: (a) at the
  helper top add `let gid = group_id; let dir = format!("groups/{gid}");` so the moved body's
  `gid`/`dir` refs stay verbatim — `dir` then becomes unused in the caller, delete it there (~L451,
  else clippy `-D warnings`); (b) the compile-fail early-return becomes `return (None, None);`; (c)
  the tail returns `(Some(compiled), verifier_results)` where `verifier_results` is the VERIFY
  `finish_processing_stage` result — drop the two `trace.* =` writes. The mid-body VERIFY
  `let clock = processing_stage_clock();`(~L524) stays inside, shadowing the `clock` param exactly as
  today. CALLER (`group_pipeline`) is unchanged through member-build, then: `let (compiled,
  verifier_results) = compile_verify_group(gid, &members, clock, resolved, adapter, shell);
  trace.compiled = compiled; trace.verifier_results = verifier_results; trace`. KEEP in the caller:
  the `trace` build (with `test_sources`), the COMPILE `clock` creation (~L459, before member-build),
  and the member-build loop incl. the member-missing `finish_processing_stage(COMPILE, clock, …)` +
  `return trace`; `clock` moves into EITHER that missing-arm finish OR the helper (mutually exclusive
  — the missing arm returns — so it compiles exactly as today). Pass the SAME compile clock (do not
  recreate it) so the compile event's `started_at` is identical → timing-identical, not merely
  artifact-identical. WHY PURE: same `finish_processing_stage` calls, same order, same args → same
  events + same artifacts; NO re-derivation (`compile` reads `m.payload.formal`/`.norm` unchanged).
  The gate PINS {`cli_shell.rs` `events.len()==19`; `run_oracle.rs` all-outcomes-Ok + §8.6
  compiled-body pins (Q1/Q2) + `assert_group_matches_reference`; lineage/trace hashes} but NOT
  compile/verify event FIELD shape (input_hashes/output_hashes/resource_counters) → correctness there
  rests on the relocation staying literal (args unchanged), so make no field edits. Per-doc derive
  fns (`derive_norm_ir`/`assemble`/`FormalIr::derive`, already pub) are OUT: route units compose them
  directly. READING: ONLY `run.rs` `group_pipeline` (~L443-565) + `finish_processing_stage`
  (~L1019-1045); `GroupTrace` = `{ group_id: Id, test_sources: Vec<Id>, compiled:
  Option<ArtifactWrapper<CompiledArtifact>>, verifier_results: Option<ArtifactWrapper<VerifierResults>>
  }` (inlined here — skip `trace.rs`); per-doc fns + ckc-smt `emit`/`verdict` sigs untouched. Gate:
  `cargo test --workspace` + `cargo fmt --check` + `cargo clippy --workspace --all-targets -- -D
  warnings` green, ZERO test edits. [Refactor-first rule: share internals before the route feature;
  per-doc head/derive sharing deferred to the route units.]
  [Done: literal relocation — `compile_verify_group` private fn placed after `group_pipeline`, body
  verbatim (git diff shows the moved block as context; only the 4 pinned boundary changes diffed:
  removed caller `dir`, delegation call, compile-fail `return (None, None)`, tuple tail); return type
  spells `ckc_smt::CompiledArtifact` qualified to match the file's non-test convention (no new import).
  Behavior-lock held with zero test edits: 392 passed / 1 ignored, fmt + clippy clean.] 40% 80K/200K
- [x] model-adapter.1: generic env-command ModelAdapter — identity + invoke skeleton. New ckc-cli
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
  [Done: `pub mod model` (landed ahead of its model-fill consumer → pub dodges clippy `--lib`
  dead_code) mirroring Z3Adapter — `with_command` (env `CKC_MODEL_COMMAND`, else neutral default
  `ckc-model-runtime`, bare-name PATH) → `--identity` probe parses `ModelIdentity`;
  `invoke(prompt, constraint, seed, budget)` → `ModelRun{outcome, stdout_bytes, stderr}` with
  `ModelOutcome` Completed{bytes}/Timeout/ExitFailure/SpawnFailure; the subprocess machinery
  (`spawn_piped`/`drain`/`run_process` + 4 budget/grace consts) MIRRORED not reused (shared-runner
  extraction deferred). NO process-fate→DiagnosticCode (stage's job); `set_var`-free env policy via
  pure `resolve_command`. 12 tests drive a committed in-source stub over every outcome + the parse
  rejections; gates green (404 passed / 1 ignored, fmt + clippy `-D warnings` clean), zero edits
  elsewhere.] 76% 151K/200K
- [x] model-adapter.2a: constrained generation + k-sample + capture-completeness (code, no live).
  Wired `crates/ckc-cli/src/model.rs` `invoke` to the §9 CLI contract (route grammar/JSON-Schema from `schemas/`
  passed as the constraint path + fixed seed; real constrained-decode VERIFIED live in .2b) + adds k-sample (`derive_seed(base_seed,i)`
  splitmix64 + `ModelSample{seed,run}` + `invoke_samples(prompt,constraint,base_seed,k,budget) ->
  Vec<ModelSample>`, k draws at `seed_i=derive_seed(base_seed,i)`, collects outputs + per-draw run) +
  HARDENS capture byte-completeness (`#![forbid(unsafe_code)]` rules out an in-crate `libc` process-group reap (a safe one needs an added syscall-wrapper dep) → gate
  `Completed` on stdout reaching EOF within DRAIN_GRACE, else new `ModelOutcome::CaptureIncomplete{bytes}`;
  the bytes are byte-stability-load-bearing). Stub-based tests ride the patch (capture-incomplete on a
  clean-exit-holds-stdout sentinel; derive_seed deterministic+distinct; invoke_samples k reproducible). NO
  live call (→ .2b). Gate: `cargo test --workspace` + `cargo fmt --all --check` + `cargo clippy
  --workspace --all-targets -- -D warnings` green.
  [Done: applied the staged VERIFIED patch clean (base blobs matched HEAD, no drift) →
  invoke/invoke_samples/CaptureIncomplete + DrainHandle EOF-gating landed as specced. Codex-review
  refinements: CaptureIncomplete + ModelRun + Completed + module docs reframed truncated→UNPROVEN
  completeness (bytes may be whole OR a prefix); ExitFailure documented usually-complete (a nonzero exit
  closes its own pipes; not EOF-gated); DRAIN_GRACE assumption + slow/large-but-finite false-negative +
  adaptive-drain follow-up stated; exact-value asserts pin `derive_seed(42,0/1/2)`, verified vs real output
  by the gate. Gates green: 409 passed / 1 ignored, fmt + clippy `-D warnings` clean. Patch deleted; memory
  derive_seed values pruned (now test-pinned).] 46% 92K/200K
- [x] model-adapter.2a-codexfix: round-2 /codex-review fixes on committed .2a (9ae5773), two commits. C1 (1b2e09d): #1 High — run_process loads stdout_eof BEFORE the stdout snapshot (race → no Completed over stale truncated bytes); #2-4 ModelRun/DRAIN_GRACE doc honesty (Timeout typically-prefix, ExitFailure completeness UNPROVEN, residual drain-thread/stdin-writer accepted under forbid(unsafe)); #5 derive_seed doc; #7 probe_holding_stdout_open_fails_construction test; #9 memory staleness pruned. C2: #8 engine-name de-leak — grammar comment scrub + roadmap 98/99/110 + memory live-hash line dropped, grammar sha256 re-pinned (d26bbd5b, 2489 B) at emit.rs + schemas.yaml; durable word-boundary audit-grep note (substring grep false-matches a Cargo.lock dep). Gates 410 passed/1 ignored, fmt + clippy -D clean. 62% 123K/200K
- [x] model-adapter.2b: live end-to-end confirm through the adapter (the §9 runtime properties).
  Add a committed `#[ignore]`d live integration test driving the .2a adapter against the env command by
  its DEFAULT bare name (covers the .1-deferred live PATH resolution): probe → identity parses; `invoke`
  twice (one inline prompt + a committed `schemas/` constraint + one seed) → byte-identical (greedy
  byte-stability = the cassette-replay assumption); `invoke_samples(base_seed,k=3)` twice → byte-identical
  `Vec` (k seeded draws replay-deterministic). Engine-agnostic asserts: byte-stability + reproducibility +
  identity-parse, PLUS constraint-CONFORMANCE — commit a simple bounded-schema fixture (enum + bool,
  additionalProperties:false; agnostic) + assert the adapter's output PARSES + SCHEMA-VALIDATES against it
  (conformance CONSISTENT WITH `--constraint` honored end-to-end → a free-running runtime ignoring the
  constraint emits non-conforming bytes + FAILS; necessary not alone sufficient — a fixed conforming object
  would pass); NOT a model-specific VALUE (that stays model-dependent). The runtime properties,
  derived seeds, env-command install, and constraint observations are OBSERVED on the local runtime
  (engine-agnostic conclusions + pinned seeds in memory `## Runtime`; machine-local specifics in
  `.agent/runtime.local.md`) → this unit RUNS the real adapter codepath live to PROVE them, closing the
  confirmation-vs-claim gap (near-zero re-derivation). Reading: THIS line + memory `## Runtime` +
  `.agent/runtime.local.md` + the .2a adapter API + SPEC §9. Gate: `cargo test --workspace` + fmt + clippy green (the ignored test compiles,
  normal runs skip it); LIVE — `cargo test <fn> -- --ignored` passes against the env command; record the
  live results in `.agent/runtime.local.md` (machine-specific) + agnostic conclusions in memory `## Runtime`. CLOSE: record context-usage; mark DONE (M2 stays IN-PROGRESS).
  DONE: `tests/model_live.rs` (`#[ignore]`d, `cargo test -p ckc-cli --test model_live -- --ignored`) +
  `tests/fixtures/bounded_verdict.schema.json` (bounded enum+bool; in `tests/` not `schemas/` — test
  artifact, not a route constraint). Live pass: cross-process SAME-seed byte-stable + schema-valid,
  k-sample seed-pinned per-index-reproducible (machine-local timing/output → `runtime.local.md`); fmt +
  clippy -D + `cargo test --workspace` green. 57% 114K/200K
- [x] model-cassette.1: §4.4/§9 cassette modules — payload + store (salvage-restore, mechanical, no
  runtime). The two modules were authored, VERIFIED, then codex-review-refined (compiled + 7 unit tests + fmt/clippy clean), reverted
  on overflow + salvaged byte-exact → restore: `cp .agent/wip-cassette-core.rs.txt crates/ckc-core/src/cassette.rs`
  (the §4.4 `CassettePayload` payload — Canonical/CanonRead + lowercase-hex codec) + `cp .agent/wip-cassette-cli.rs.txt
  crates/ckc-cli/src/cassette.rs` (the `CassetteStore` record/replay IO); wire `crates/ckc-core/src/lib.rs`
  (`mod cassette;` + `pub use cassette::{CassettePayload, InvalidCassetteHex};`, canon-adjacent slots) +
  `crates/ckc-cli/src/lib.rs` (`pub mod cassette;` — pub dodges clippy `--lib` dead_code ahead of its
  stage-model-fill consumer, model.rs precedent). LOCKED (zero re-derivation): payload in ckc-core (needs
  pub(crate) `RawText`/`emit_u64`/`read_u64`), store in ckc-cli (drives `ModelAdapter`) — mirrors
  ModelIdentity(data)/ModelAdapter(runtime); output bytes → lowercase-hex in canonical JSON (lossless any
  bytes, never lossy-decoded — recorded bytes ARE the determinism); cassette = `ArtifactWrapper<CassettePayload>`
  origin `ai_generated`/evidence `evidence_discovery_only`/effect `ai`, keyed (route, source, seed) at
  `<root>/cassettes/<route>/<source>/seed-<seed>.json`, configurable root; `record` gated (runtime + clean
  `Completed`), `replay` default (runtime-ABSENT); `load` validates + key-checks + hex-decodes on read-back. Integration
  DEFERRED (no consumer until): stage-model-fill.1 drives record/replay, run-m2.1 the `--record` surface +
  replay.rs model-artifact coverage + §9 manifest `prompt_template_hash`. Reading: ONLY this line + the two
  wip files. Gate: `cargo test --workspace` (7 unit tests pass runtime-absent — 3 payload round-trip/hex,
  4 store replay/missing/key-mismatch/malformed-hex) + `cargo fmt --check` + `cargo clippy --workspace --all-targets -- -D
  warnings` (salvage is fmt-clean → byte-exact restore passes fmt-check). CLOSE: `rm .agent/wip-cassette-core.rs.txt
  .agent/wip-cassette-cli.rs.txt` (both consumed here); record context-usage; mark DONE (M2 stays IN-PROGRESS). DONE: restored byte-exact (sha256 parity), wired both lib.rs (core `mod`+`pub use`, cli `pub mod`, + a module doc bullet each); `RawText`/`emit_u64`/`read_u64` were already pub(crate) → no canon.rs change. Gate green: fmt clean + clippy -D 0 + `cargo test --workspace`, 8 cassette tests (3 core payload + 5 cli store, all runtime-absent). 46% 92K/200K Codex-review hardened the store: `load` now enforces the fixed §4.4 cassette provenance contract (schema_id/kind/artifact_id/origin/evidence/effects/input_hashes — `validate` covers only the hashes + effect/evidence rule, so a committed cassette could otherwise lie about being an AI recording + load clean) + `record` re-reads the constraint post-invoke → `ConstraintDrift` (re-read, not relocate — a snapshot would break relative `$ref`s); path-safety comment narrowed to the real grammar guarantee (no `/`/`..`; `:` ⇒ Unix-only); `artifact_id` documented as a derived label not an identity key (the (route,source,seed) triple/path is collision-free, dotted ids aren't). +`off_contract_rejected` test.
- [x] model-cassette.2: committed test cassette via live bless + runtime-absent replay (the live unit,
  mirrors model-adapter.2b). Add `crates/ckc-cli/tests/model_cassette.rs` mirroring `tests/model_live.rs`:
  (a) an `#[ignore]`d bless `record_cassette`, guarded on `CKC_MODEL_COMMAND` unset (default bare-name
  `ModelAdapter::new()`) — `CassetteStore::new(<repo>/crates/ckc-cli/tests/fixtures)`, `RecordContext` {
  synthetic `Producer`, `prompt_template_hash = hash_bytes(<synthetic template>)`, budget 120s }, key {
  `route.fixture`, `test_source.fixture`, seed 42 }, inline prompt + committed `tests/fixtures/bounded_verdict.schema.json`
  constraint, `store.record(&adapter, &key, prompt, constraint, &ctx)` → writes the cassette; (b) a NORMAL
  `#[test] replay_committed_cassette` (runtime-ABSENT): `store.replay(&key)` → assert `content_hash ==
  CASSETTE_HASH` (pinned), `payload.output_bytes()` parses as JSON + schema-validates against the bounded
  fixture (jsonschema dev-dep), provenance (origin AiGenerated / evidence EvidenceDiscoveryOnly / effects
  [`Ai`]). LOCKED: committed cassette → `crates/ckc-cli/tests/fixtures/cassettes/route.fixture/test_source.fixture/seed-42.json`
  (.2b precedent — test artifact, not `corpus/test_sources/` (route units own those) nor `schemas/`);
  eol-irrelevant (canonical JSON is newline-free); schema-conformance is CONSISTENT WITH `--constraint`
  honored (necessary not sufficient, the .2b framing). Reading: THIS line + `tests/model_live.rs` (mirror)
  + the restored `cassette.rs` store API + memory `## Runtime`. Gate: `cargo test --workspace` + fmt +
  clippy green (the `#[ignore]`d bless compiles, the replay test runs runtime-absent). LIVE: `cargo test
  -p ckc-cli --test model_cassette record_cassette -- --ignored` records against the env command → commit
  the cassette → re-pin `CASSETTE_HASH` from the recorded wrapper's `content_hash` field (`jq -r
  .content_hash <cassette>` — the §4.4 payload-canonical hash the test compares, NOT the file-byte
  `sha256sum`) → `cargo test` green. CLOSE: record context-usage;
  mark DONE (M2 stays IN-PROGRESS). [Done: `tests/model_cassette.rs` — `#[ignore]`d `record_cassette`
  bless + normal runtime-absent `replay_committed_cassette`; live bless recorded
  `tests/fixtures/cassettes/route.fixture/test_source.fixture/seed-42.json` (bounded-verdict output
  `{"verdict":"unknown","actionable":false}`), `CASSETTE_HASH` pinned to the wrapper `content_hash`
  (`sha256:8b465db5…`). USER DECISION (asked, sets the run-m2.2 precedent): the committed cassette
  carries the runtime's REAL `model_identity` (model/quant/engine) — recorded MEASUREMENT data, exempt
  from the engine-agnostic synthetic-token audit (carve-out in memory; audit confirmed the tokens appear
  ONLY in the cassette). Replay pins output JSON+schema-conformance + ai_generated/evidence_discovery_only/[ai]
  provenance, never an identity value. Gates green: fmt + clippy -D + `cargo test --workspace`, 419 passed /
  3 ignored (record_cassette + model_live + bless_clinical_ir_schema).] 72% 145K/200K
- [x] stage-model-fill.1: model-fill processing stage — core (generic over target). DECOUPLED core
  `model_fill<T>(store, key, FillSource, parse) -> Result<ModelFill<T>, CassetteError>` (replay default /
  record gated → decode `output_bytes()` → route `parse` = §4 acceptance → `ModelFill{target, diagnostics,
  recorded_calls}`; `Err(reason)` → §7.4 `ai_schema_violation` (no target), a cassette IO/contract failure →
  distinct `CassetteError`); unreferenced `processing_stage.m2.model_fill` registry entry; the §4.6 event
  emission is deferred to run-m2.1 (builds it from `recorded_calls`). Cross-unit decisions in memory
  (`Model-fill stage core` bullet). [Done: reproduction-only redo (salvage restored byte-exact, then deleted); gate green 423 passed /
  3 ignored (4 model_fill tests + `committed_model_surface_checks_ok` by name) + fmt + clippy
  `-D warnings` clean.] 51% 103K/200K
- [x] stage-model-fill.2: model-fill repair loop + grounding (UNIFIED with .1's core). `model_fill<T>(store,
  key, FillSource, repair_limit, accept) -> Result<ModelFill<T>, CassetteError>`; `accept: impl Fn(&[u8]) ->
  Result<T, FillReject>` returns the route's typed verdict — `FillReject::Schema` → `ai_schema_violation` +
  RE-PROMPT under `derive_seed(key.seed, attempt)` up to `repair_limit`, then terminal `repair_limit_exceeded`;
  `FillReject::Grounding` → terminal `ai_hallucinated_source`, spends no repair budget. `ModelFill` gains
  `repairs` + new `REPAIRS_COUNTER` (distinct from `RECORDED_CALLS_COUNTER`; both surfaced, run-m2.1 emits
  without re-deriving). `derive_seed` → pub(crate) (k-sample + repair). 7 replay-only crafted-cassette tests;
  cross-unit + field-convention decisions in memory (`Model-fill stage core` + `DiagnosticRecord field
  convention` bullets). [Done: reproduction-only redo (salvage restored byte-exact dab2be9e…, then deleted);
  synced the lib.rs `model_fill` module doc + memory to .2; gate green 426 passed / 3 ignored + fmt + clippy
  `-D warnings` clean.] 58% 116K/200K Codex-review hardened: the stage now ASSERTS its `Grounding` carries
  ≥1 absent id (empty = a deterministic route bug → fail-closed panic, house `expect`/`unreachable` style,
  not a silent empty-id diagnostic; route-single-ir still enforces route-side as defense-in-depth) + softened
  two over-claimed "distinct committed cassette" docs (provably distinct across repairs; collision-negligible,
  not structurally excluded, against the base seed). +empty-grounding `should_panic` test.
- [ ] route-single-ir.1: `single_ir` registry + prompt surface (additive, gate-independent foundation;
  split from the original one-shot route-single-ir — respec `git show HEAD`). `candidates.yaml` +=
  `processing_stage.m2.assemble` (mirror `processing_stage.m1.assemble`, swap input
  `normalization`→`clinical_ir`): `kind: assemble` / `determinism: deterministic` /
  `input_artifact_kinds: [source_document_graph, segments, clinical_ir]` / `output_artifact_kinds:
  [ir_bundle]`. += `pipe.m2_single_ir` `processing_stages:` = `processing_stage.m1.extract`, `.m1.segment`,
  `.m2.model_fill`, `.m2.assemble`, `.m1.compile`, `.m1.verify` (chain validates: `clinical_ir` from
  `m2.model_fill` precedes `m2.assemble`; `ir_bundle` feeds `m1.compile`, which produces `compiled` +
  `smt_query` for `m1.verify`). NEW `registry/prompts.yaml` (top-level `Vec<PromptEntry>`, mirror
  `schemas.yaml` shape): one entry `id: prompt.single_ir` / `path: registry/prompts/single_ir.txt` /
  `route: route.single_ir` / `template_hash: sha256:<hash>`. NEW `registry/prompts/single_ir.txt` =
  a first-draft JA→ClinicalIR fill prompt (instruct the model to translate a JA guideline into the
  `clinical_ir` JSON-Schema, filling closed-vocab fields from the lexicon + referencing the supplied
  region/segment ids); CONTENT is not gated here (cassettes are crafted, not recorded — run-m2.2's live
  recording refines the wording) → only existence + hash + validation gate. `.gitattributes`
  `registry/prompts/*.txt eol=lf` (hash survives checkout; `schemas/` precedent). `registry_check.rs`
  `check_model_registry`: add a prompt file/hash loop SYMMETRIC to the schema loop (registry_check.rs
  76-117) — per prompt: skip when neither `path` nor `inline` is set (`PromptSource` finding covers it);
  `path` → guard `is_empty() || !is_safe_relative_path` then `hash_bytes(file_bytes)`; `inline` →
  `hash_bytes(inline.as_bytes())`; compare to `template_hash`; emit the sorted `actual`/`expected`/`prompt`
  payload (mirror the schema arm's `actual`/`expected`/`schema` + its read-error arm). Reading: THIS line;
  `registry/candidates.yaml` (the `m1.assemble` + one `pipe.*` block to mirror), `registry/schemas.yaml`
  (prompts.yaml template), `registry_check.rs` 76-117; memory `Registry model surface` covers
  `PromptEntry`/`validate_model_registry`/`parse_prompts`/`hash_bytes`/`is_safe_relative_path`. Gate:
  `cargo test`; `ckc registry check` validates `pipe.m2_single_ir` (chain) + the prompt file/hash, and
  rejects a missing / hash-mismatched prompt (mirror the schema-mismatch test); engine-agnostic (the prompt
  is human JA-instruction prose, no engine/dialect/format names); fmt + clippy.
- [ ] route-single-ir.2: per-doc model-fill → validated `IrBundle` + golden cassettes (the route's fill
  half; model-runtime-absent, z3 not needed). New route code in `run.rs` (REQUIRED there: `Resolved` +
  `compile_verify_group` are private to `mod run`, which .3 reuses). Per-doc fn (shape: `single_ir_fill(root,
  entry: &CorpusEntry, lexicon, store: &CassetteStore, seed, shell) -> Option<ArtifactWrapper<IrBundle>>`):
  `extract(&html, &config)` (mirror `run.rs` 343-388: html from `root.join(&entry.path)`, `source_family =
  static_id("synthetic_test_source_html")`, `producer(resolved, 0)`) → `segment(&source,
  &producer(resolved, 1))` → `model_fill(store, CassetteKey{route.single_ir, entry.id, seed},
  FillSource::Replay, repair_limit, accept)` → `ClinicalIr`. ACCEPT closure (T = `ClinicalIr`):
  `read_strict_canonical::<ClinicalIr>(bytes).map_err(|e| FillReject::Schema(e.to_string()))?`, then a
  grounding pre-check closed over upstream — region universe `source.payload.regions.iter().map(|r|
  &r.region_id)`, segment universe `segments.payload.segments.iter().map(|s| &s.segment_id)`; collect
  `absent` over `binding.region_ids` + `statement.exceptions[].region_ids` + `statement.source_segment_ids`
  failing membership (= `bundle.validate` steps 4+5); `!absent.is_empty()` → `Err(FillReject::Grounding(
  absent))` (route-side non-empty assert = defense-in-depth, M2.14 codex), else `Ok(clinical)`. POST-accept
  deterministic tail (mirror `assemble_bundle` run.rs 959-989, substituting the model `clinical` +
  `derive_norm_ir(&entry.id, &clinical, &segments.payload, lexicon)` for `normalization.payload.
  {clinical,norm}`): `DocIr::from_graph(&source.payload, source.diagnostics.clone())` → `assemble(doc,
  segments.payload, clinical, norm, Vec::new(), diagnostics)` → `bundle.validate(&source.payload)` → wrap.
  `bundle.validate` is EXPECTED-PASS for accepted output (parse + grounding cover the model-controllable
  failures; vocab is constrained-decoded live / lexicon-valid in golden cassettes) — a failure is a hard
  route error, NOT a §7.4 code. GOLDEN cassettes via a bless writer — per doc (a/b/control) run the
  deterministic upstream → `canonical_payload_bytes(&clinical_ir(graph, segments, lexicon))` →
  `CassettePayload::from_output(route.single_ir, source, 42, prompt, constraint_hash = the `clinical_ir`
  schema hash, prompt_template_hash = `prompt.single_ir`'s hash, model_identity = SYNTHETIC
  `model.baseline`/`fixture_quant`/`1.0.0`, &output_bytes)` → `build_wrapper` → `persist` to
  `crates/ckc-cli/tests/fixtures/cassettes/route.single_ir/<source>/seed-42.json`. SYNTHETIC identity —
  these are crafted deterministic fixtures, NOT live recordings, so the engine-agnostic audit APPLIES
  (unlike run-m2.2's exempt live cassettes). Test (model-runtime-absent): per doc, `single_ir_fill` replays
  the golden cassette → an `IrBundle` whose `content_hash` EQUALS the M1 deterministic bundle
  (`assemble_bundle` over the same doc) — proving cassette replay + closure acceptance + assembly all
  reproduce M1 (`content_hash` is payload-only → producer-independent). Reading: THIS line; `run.rs` 343-388
  (extract/segment/producer) + 959-989 (assemble_bundle); `model_fill`/`FillSource`/`FillReject` + cassette
  `build_wrapper`/`persist`/`replay` per memory `Model-fill stage core` + `Model cassette`. Gate: `cargo
  test` (per-doc bundle == M1 bundle over committed golden cassettes); engine-agnostic audit on the new
  cassettes; fmt + clippy. [Decision pinned: model fills ClinicalIR over deterministic upstream — the
  instrument supplies the grounding scaffold; hallucinated refs are measured, not fatal.]
- [ ] route-single-ir.3: per-group verdict tail + reference scoring (the route's verdict half; z3 present,
  model-runtime-absent). Extend the route: gather .2's per-doc bundles for a group's test_sources, then
  hand-build a MINIMAL `Resolved` (NO refactor — `compile_verify_group` reads only 5 fields, agent-confirmed):
  `pipeline_id = pipe.m2_single_ir`, `pipeline_step_ids: [Id; 8]` with `[4] = processing_stage.m1.compile`
  + `[5] = processing_stage.m1.verify` (the other 6 slots placeholder `Id`s), `toolchain_manifest_hash`
  (real), `budget_ms`; the tail never reads `documents: vec![]`, `groups: vec![]`, `plan: RunPlan{
  experiment_id, test_source_groups: vec![], pipelines: vec![], seed, budget: vec![]}`. Per group call
  `compile_verify_group(group_id, &format!("groups/{gid}"), &members, processing_stage_clock(), &resolved,
  &adapter, shell)` → `(compiled, verifier_results)`. `adapter = Z3Adapter::new()` (real z3, M1 precedent);
  `shell = Shell::open(static_id("run"), static_id("m2"), Some(out_dir))` (out_dir MUST be `Some` — a
  TempDir). SCORING test (z3 present): run the route over `group.m1_conflict` + `group.m1_no_conflict` (M1
  groups, `experiments.yaml`) → verdicts vs `corpus/reference/m1_expected.yaml` via the `run_oracle.rs`
  `assert_group_matches_reference` shape (conflict: exactly one `SemanticContradiction` on the deontic query,
  `unsat_core` set-equal `a.test_source.m1_guideline_a.rule.0` + `a.test_source.m1_guideline_b.rule.0`;
  no_conflict: all `SemanticNoConflict`). Low-risk: .2 proved the bundles == M1 bundles, so the verdicts ==
  M1's (already pinned by `run_oracle`). Reading: THIS line; `run.rs` 460-490 (group_pipeline member-build)
  + 500-590 (`compile_verify_group`) + 1044-1085 (`finish_processing_stage`) + 1197-1203 (`producer`);
  `tests/run_oracle.rs` (`assert_group_matches_reference`, `strict_read`); `ProcessingStageClock`/`Shell`/
  `Z3Adapter` construction is pinned above. Gate: `cargo test` (conflict + no-conflict verdicts scored vs
  `m1_expected` over the golden cassettes, model-runtime-absent); fmt + clippy.
- [ ] route-single-ir.4: rejection paths — §7.4 codes wire through (model-runtime-absent). Prove the route
  ACCEPT closure's `FillReject` → §7.4 mapping end-to-end (model_fill's repair-loop MECHANICS are already
  covered by stage-model-fill.2 — .4 adds only the route-closure coverage). Craft 2 committed bad cassettes
  keyed under `route.single_ir`/`test_source.m1_guideline_a` at distinct non-42 seeds (real upstream =
  guideline_a, so refs resolve against it): (a) HALLUCINATED — take guideline_a's golden `ClinicalIr`, mutate
  one `source_segment_id` (or a `binding.region_id`) to a fresh grammar-valid `Id` absent upstream, then
  `canonical_payload_bytes` (stays canonical → `read_strict_canonical` ACCEPTS → reaches grounding) → seed
  99; (b) MALFORMED — non-parsing bytes → `read_strict_canonical` fails → `FillReject::Schema` → seed 98.
  Tests (model-runtime-absent): hallucinated → `ModelFill.diagnostics` carries `ai_hallucinated_source` with
  ≥1 `absent_source_ids` + `target == None`; malformed with `repair_limit ≥ 1` (+ a VALID cassette at
  `derive_seed(98, 1)`, pinned in `model.rs`'s `derive_seed` test) → `ai_schema_violation` then RECOVERY (an
  accepted bundle), AND a malformed-at-every-derived-seed variant → `repair_limit_exceeded`. SYNTHETIC
  model_identity on the bad cassettes (audit applies). Reading: THIS line; .2's committed accept closure +
  cassette-craft helper; `model_fill` `FillReject`→code map + `derive_seed` re-prompt + `REPAIRS_COUNTER` per
  memory `Model-fill stage core`; the `derive_seed` test (`model.rs`). Gate: `cargo test` (hallucinated →
  `ai_hallucinated_source` with ≥1 id; malformed → `ai_schema_violation` + repair → recover / exceed);
  engine-agnostic audit on the new bad cassettes; fmt + clippy.
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
  finalized in-unit.] [When active, split like route-single-ir.1-.4 (registry+prompt / fill / verdict /
  rejection) — size from their recorded context-usage; it is simpler (model emits SMT directly, no
  IR/grounding/assemble), so likely fewer units.]
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
  model/prompt/identity hash fields), over the locked M1 inputs. Add the `ckc run --record` flag (default
  = replay committed cassettes runtime-absent; `--record` drives `CassetteStore::record` live — run-m2.2
  exercises that live path) + its default-replay acceptance. Tested via REPLAY of the route units'
  committed cassettes (deterministic, no live call) — model-fill replays runtime-absent, so replay.rs
  hash-compare covers the model-stage artifacts. Reading: `run.rs` execute (route loop), the
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
