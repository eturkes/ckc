# Agent Memory

Entries must add value beyond the spec, AGENTS.md, codebase, git history, and runtime
environment — project-independent tooling pitfalls (RTK, Headroom, Serena, Claude Code, web
access) live in each agent's own global guidance, not here. Exception: high-value reminders that
are derivable but easily forgotten under token pressure. Entries are consolidated aggressively;
full pre-consolidation text lives in git history.

## Policy

- Context hygiene (user directive; background: `git show 531f586`): keep every
  session lean and phrased in project vocabulary (processing stages, units, gates, artifacts) — plain
  operational words over research jargon in memory, roadmap, commits, and code alike.
  Consult `docs/` through read-only subagents so its vocabulary stays out of
  the main window. Root `.rgignore` keeps ripgrep-backed sweeps (subagent Grep, `rtk proxy
  rg`) out of `docs/`; Bash `grep -r` still enters it — scope Bash greps by path; deliberate
  docs searches use `git grep <pat> -- docs/`, `rg --no-ignore`, or explicit file paths.
  Implement sessions match patterns from the latest unit-scoped commit (`git log
  --oneline`), not bare HEAD, when HEAD is hygiene/memory work.

## Lessons

- Unit sizing rules (consolidated from roadmap `NN%` annotations and observed 200K overruns;
  per-incident case studies live in git history — `git show 6e413f0^:.agent/memory.md`). Target: one
  conceptual deliverable plus one gate, finishable AND committable in one window with margin;
  prefer more, smaller units. Plan-time obligations (a violation is a planning bug): resolve
  semantic requirements decisions INTO the roadmap line (more than ~2 left open = re-scope);
  research and pin any new external dependency (exact version + features) in the line;
  pre-split multi-deliverable stacks BEFORE scheduling — mid-session overrun recovery is
  user-initiated (stop, bring the tree clean, report); minting a split rule re-audits every
  remaining unchecked line against it in the same recovery commit; a recovery split is itself
  plan work — audit its replacement lines against every standing rule and the open-decision
  ceiling within the recovery commit. Split rules: a feature needing a refactor of existing
  code to share internals takes the refactor as its own behavior-locked unit FIRST (existing
  tests the gate, zero test edits); a format walker plus committed test-source integration =
  walker-core (inline-literal tests) then format-completion + test-source integration; a
  nontrivial algorithm plus a second authored artifact = two units; a multi-invariant
  validator plus full rejection coverage = two units; a derivation fn with its
  test-source-pinned battery plus an attachment sub-feature = two units; a type family plus
  assembly plus validation = three units; an assembly fn plus its live-pipeline pin battery =
  two units; a live-pin battery over the run binary is a unit on its own (never paired with
  assembly or processing-stage wiring); a spec-byte amendment (re-pin + reference/test mirror
  sweep) bundled with new feature code = two units — an open decision whose resolution amends
  pinned bytes is a deliverable, not a session preamble; crate foundations pair only with a
  small type surface (one payload module per foundation unit). Measured anchors (checked
  roadmap stubs carry the `NN%` figures): canonical JSON = five units; a five-layer recursive
  type family = three units; a lexicon-driven derivation half (loader / binding / builder) =
  three units; statement builder over a prebuilt binding core = one unit; exception
  attachment + determinism tests = one unit. Practices: house new type families in fresh
  modules (extending a ~2K-line module costs a full-file read); scope each split unit's
  Reading slice to exclude files its half leaves untouched; land a compiling skeleton before
  the full test battery; salvage a reverted session's compiling half as a committed
  `.agent/wip-<unit>.patch` the redo line points at (apply, verify against the line, delete in
  the closing commit) — an uncompiled draft salvages the same way flagged UNCOMPILED,
  transcription-with-verification still beating re-derivation; pin expected shapes from
  observed output, never hand-computed; cite only checked roadmap lines as measured anchors.
  At plan/re-scope time, audit any spec listing a unit must byte-reproduce: listings written
  for readability (alignment padding, inline result comments, illustrative declaration or
  conjunct order) contradict deterministic-emission rules and need a scheduled re-pin
  deliverable (caught pre-session for smt-emit.3a: §8.6 smt2 vs §6 sorted-declaration rule).
- Renaming canonical (§4.3) JSON member keys is a silent test-breaker. The object emitter buffers members then sorts them by key bytes on `finish`; the reader (`canon.rs` `member`/`optional`) is positional — it peeks the next key and demands the caller request keys in ascending byte order. So a key rename moves its sort slot: the code still compiles, but round-trip reads fail `MissingField` at runtime and pinned canonical byte-string literals mismatch. Fix = re-sort each Canonical read+emit member sequence AND every pinned byte-string to the new key order (`printf '%s\n' k1 k2 … | LC_ALL=C sort`). Related: a `#[serde(rename_all="snake_case")]` enum serializes by variant name, so a snake wire-key rename must also rename the CamelCase variant (e.g. ViewText→RenderedText) — caught by name-pin asserts, never the compiler. And hyphenated scope-IDs (`stage-extract.1`, `core-grounding`, `fixtures-m1`) in roadmap+comments are git-commit-traceability keys: keep them historical on a terminology rename (rename only dotted runtime IDs `processing_stage.m1.*` and living prose).
- Test/example producer IDs: `pipe.<qual>` (`pipeline_id`) + `processing_stage.<qual>.<step>` (`pipeline_step_id`); shared `<qual>` links a pipeline to its steps. Generic unit fixtures use `qual=test`; scenario fixtures keep their own (`m1`/`t`/`base`). Never `cand.*`/`comp.*` — those echo the pre-rename `candidate`/`component` field names the terminology cleanup removed.
- Component vs pipeline-step terminology: reserved now in identifiers AND comments (`b6e1177` + follow-up sweep) — `component` = the §5 IR `ComponentRecord`/`DocIR`/structural concept only; a registry `processing_stage` entry = a pipeline step. OPEN + deliberate (not a missed rename): SPEC §8.4 prose + `registry/candidates.yaml` still read "processing stage component(s)"; resolving it = a SPEC-level vocabulary call (route through the user), so skip auto-"fixing" it on a grep sweep.
- "Oracle" has two senses; the `terms:`/`codex:` cleanup (`b0e51b2`/`caefcbb`/`e4f983a`) renamed only the epistemic-overclaim one — `runtime-oracle` → `runtime reference` across IDs/types/prose (results are locked measurements, not an authority on real-world truth). Scope: SPEC.md + Rust + registry/corpus/reference data + IDs + config; `docs/` excluded. The commits cite a replacement map whose contents aren't recoverable from git — so "the map omitted generic `oracle`" is inference; what's verifiable is that only runtime-oracle terms were swapped. The TEST-ORACLE sense (authority deciding a test's pass/fail vs. the reference) persists in `run_oracle.rs` (file + `run_oracle_*` fn) and `rules.rs` (`// THE oracle`); those files passed through the sweep commits unrenamed — survival, not a documented approval — and the phrasing recurs in out-of-scope `docs/` ("test oracle"/"SAT oracle"/"perfect oracle") as ordinary technical usage (corroboration, not proof). SPEC.md has zero "oracle"; no instruction mandates global removal (nearest pull: the general "plain operational words over research jargon" line above). Decision: NARROW — leave the test-sense as-is. A global test-sense retirement (`run_oracle.rs`→`run_reference_check.rs`) stays an OPEN user/style call.
- ckc-smt's `serde` dep reads as unused (no `serde::`/`Serialize`/`Deserialize` in ckc-smt/src
  beyond the `fieldless_enum!` invocations) but is REQUIRED: that ckc-core macro expands to
  `::serde::Serialize`/`Deserialize` impls *in the caller's crate*, so every fieldless_enum! user
  must depend on serde — dropping it breaks the build (`E0433` unresolved `::serde`). Holds for any
  crate adopting the macro. Those serde impls go unused there (the canonical path is
  Canonical/CanonRead), an accepted KISS cost of one shared macro over per-call serde gating; don't
  "tidy" the dep away.
- M1 reviewed (REVIEWED; gates green; zero code defects in the milestone body — all nine §8.5
  mechanisms + every §8.6 byte-pin verified live, the Q1/Q2 smt pins via run_oracle's
  group.m1_conflict assertion as well as the emit-unit pins). §4.4-vs-§8.3 tension RESOLVED by SPEC
  amendment (codex-review follow-up): a processing stage's total operation result IS its §4.6
  EventRecord — the §8.3 run layout has no per-stage total artifact — and only commands materialize
  the standalone TotalOperationResult, whose value/residual/ambiguity/incoherence buckets stay empty
  until a milestone materializes typed placeholders. So do NOT add per-stage TotalOperationResults:
  inert + redundant with EventRecords until typed placeholders exist (judged technical-debt-not-gain;
  M2+ may revisit). One enhancement stays open: tests are example/byte-pin only; property-based /
  fuzzing for the canon layer (round-trip identity, reject-any-mutation) and StringPolicy
  (idempotence) is the AGENTS.md-preferred strengthening, currently unscheduled.
- Engine-agnostic DELIVERABLE (user directive): the committed SPEC/code/registry/roadmap/`schemas/`
  name NO specific LLM inference engine, grammar dialect, or model-file format. M2 elaboration picks the
  engine at build time behind the generic harness contract (greedy + fixed seed, grammar/JSON-Schema
  constraint fed by the exported `schemas/`, recorded subprocess, identity/quant/runtime-version in
  manifests); match §3's engine-neutral phrasing `the M2 local-model runtime`. The CONCRETE runtime/model
  actually used is a machine-specific environment detail recorded in `## Runtime (machine-specific)` below,
  NOT in the agnostic deliverable (the contract is the artifact; the pick is config). `docs/` research
  corpus (model-routes.md etc.) may name engines as landscape — out of scope.
- M2 plan (minimal pair; gate MET = model runtime,
  NOT a §15 gate — locked measurements stand alone). Durable decisions beyond the roadmap lines
  (which collapse at M2 review):
  - single_ir layer pick = **ClinicalIR** — free-text-free (closed-vocab fields = lexicon codes /
    enums / bounded ints) → constrained decoding tractable, deterministic leverage high. NOT fully
    closed-vocab: it carries generated IDs (`binding_id`/`statement_id`/`exception_id`) + reference
    IDs (`source_segment_ids`/`region_ids`) constrained by the Id grammar + grounding, not a
    vocabulary. The
    instrument supplies the grounding scaffold: deterministic extract+segment produce the real
    upstream ids, the model fills ClinicalIR REFERENCING them, so hallucinated `source_segment_ids`/
    `region_ids` surface as `ai_hallucinated_source` instead of corrupting the verdict. The §7.4
    codes (`ai_schema_violation`/`ai_hallucinated_source`/`repair_limit_exceeded`) and §7.3 "repair
    count" IMPLY the intended repair-loop + grounding-check architecture (an elaboration inference,
    not a §9 mandate).
  - `exp.m2_multihop` binds BOTH routes in ONE experiment — `ExperimentEntry` generalizes singular
    `pipeline` to `pipelines: Vec<Id>` + `baseline_pipeline` (baseline = the `direct_smt` pipeline);
    each route is realized as one registry pipeline (`pipe.m2_*`); one `ckc run` → one `report.json`
    with per-route raw
    rows + the baseline-delta table. Faithful to §9 "both routes execute over identical locked inputs
    (`exp.m2_multihop`)"; M3's separate `exp.*` ids are a different shape, do not back-apply here.
  - Manifest identity (§9 vs code, finder-confirmed): §9 SEPARATES model identity from prompt hashes
    → `ModelIdentity` = `{model_id, quant, runtime_version}` ONLY (mirrors `SolverIdentity`'s
    identity-only shape; no prompt hash inside). `RunManifest`/`ReplayManifest` carry only
    `corpus_hash`/`lexicon_hash` today — M2 ADDS the §9 set (test-source/reference/schema/prompt-
    template/model/runtime hashes) as OMITTABLE fields so M1 manifest bytes + pins stay unchanged
    (omit-None), M2 populates.
  - Registry `check` is referential (finder-confirmed `validate_registries`): FAILS on dangling
    experiment→pipeline / pipeline→stage refs + §8.4 ChainBreak → seed an experiment entry ONLY after
    its pipelines + stages exist (real `exp.m2_multihop` seeds in run-m2.1, not the type-extension
    unit, which gates on a synthetic fixture).
  - Engine-agnostic boundary (extends the bullet above): the runtime is an environment-provided
    COMMAND invoked Z3-style — `ModelAdapter` mirrors `Z3Adapter`; committed code carries only the CLI
    contract (prompt + constraint + seed → recorded bytes) + resolves a BARE command name on PATH (Z3
    runs `z3` by bare name, no literal path / committed config), env-overridable; the wrapper binary
    is environment-supplied outside git. Committed `schemas/` use neutral formats —
    JSON-Schema (standard) for ClinicalIR, EBNF/ABNF grammar for the SMT surface (no engine
    constraint-dialect name); the env wrapper compiles them to the runtime's constraint format.
  - "test all layer configurations" (user directive) → deferred to M3 as the §10 route-axis gradient
    seed: every meaningful single_ir IR layer + the DMN-style alt. The user chose keeping M2 the §9
    minimal pair over widening §9; the gradient is the experiment §10 ("vary and layer existing IR
    forms") was written to be.
  - Recon mechanics that right-size the units: a processing-stage `kind` is a free-form Id (no enum)
    → adding `model_fill` is registry data, not an enum change; the middle-layer derive fns live in
    ckc-cli (`segment.rs`, `normalize.rs`, `rules.rs` `derive_norm_ir`), only `DocIr::from_graph` +
    `FormalIr::derive`/`FormalConstraint::from_rule` sit on the ckc-core types → `run-refactor`
    extracts the shared ClinicalIR→verdict tail (behavior-locked, M1 tests the gate) before the
    routes reuse it.
  - Runtime-gate findings (the "gate MET" above, confirmed functionally on a real test source; concrete
    runtime/model identity in the `## Runtime (machine-specific)` section below): constrained decoding forces
    schema-VALID output
    that can be semantically WRONG (observed: a greedy run emitted a wrong enum) → the M2 report scores
    BOTH acceptance-rate (schema-validity) AND verdict-accuracy, never validity alone. The baseline
    deliberately pins a weak sub-4B model whose free-form/direct-route output degenerates → exercises §9's
    "direct-route failures common" path (pin the exact model identity in the run config; alternatives ok).
    Greedy output is byte-stable within + across processes on one host/device/quant but NOT across
    environments → the recorded-bytes cassette (engine-agnostic boundary above), not a live re-run, is the
    correctness mechanism; replay needs no model runtime present.

## Runtime (machine-specific)

Concrete M2 local-model runtime this container runs — machine-specific environment detail (paths +
measurements drift; the committed deliverable stays engine-agnostic per Policy). OpenVINO GenAI on Intel
Lunar Lake; iGPU/NPU/CPU enablement + env to source live in gitignored machine-local config. Validated
functionally on CPU (NPU/GPU structured-output support untested — NPU static shapes may not support it).

- Constrained decoding: `StructuredOutputConfig(json_schema=… | regex=… | grammar/EBNF/compound_grammar/
  structural_tags)` → JSON-Schema + grammar-constrained output. `GenerationConfig`: `do_sample=False`
  (greedy), `rng_seed`, `stop_strings`, `ignore_eos`.
- Model: Qwen2.5-1.5B-Instruct INT4 OV-IR (HF `OpenVINO/Qwen2.5-1.5B-Instruct-int4-ov`, fetch via
  `huggingface-hub`) at `/var/home/eturkes/.local/app/ckc-models/qwen2.5-1.5b-int4-ov` (867M bin + ov
  tokenizer/detokenizer). Sub-4B, JA-capable; free-form output degenerate/repetitive.
- Determinism (CPU, measured): loads ~9s; greedy output BYTE-STABLE within a process AND across separate
  processes on the same host/device/quant; cross-ENVIRONMENT determinism NOT guaranteed. JSON-Schema +
  regex-grammar constraints hold deterministically. Caveat: a constraint forces schema-VALID output that
  can be semantically WRONG (a measured greedy run picked a wrong enum value).
