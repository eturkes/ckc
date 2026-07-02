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
- LSP coverage map (ckc inventory; the Serena-vs-marketplace wiring and diagnostic-delivery
  mechanism live in each agent's own global guidance, not here). ckc's hand-authored/byte-pinned
  formats → provider: rust, bash, json, yaml, toml, markdown (Marksman), html, lean4 are
  Serena-served (in `.serena/project.yml` `languages:`); xml, smt2 (dolmen), alloy, egglog come
  from the `global` LSP marketplace. §13 formal targets: alloy + egglog covered (marketplace
  plugins); lean4 listed but its server starts only once `.lean` files exist. No standalone LSP
  (audited): TLA+, ASP/Clingo, categorical-CQL; Isabelle lacks solidlsp (marketplace gap plugin at
  adoption), Python is solidlsp-covered (add to `languages:` at adoption) — §13 additional-targets,
  §13.1 adapter boundary. Compendium families present only as registry YAML data carry no LSP.

## Lessons

- Unit sizing rules (per-incident case studies in git — `git show 6e413f0^:.agent/memory.md`). Target:
  one conceptual deliverable + one gate, finishable AND committable in one window with margin; prefer
  more, smaller units. PLAN-TIME obligations (a violation is a planning bug): resolve semantic decisions
  INTO the roadmap line (>~2 left open = re-scope); research + pin any new external dependency (exact
  version + features) in the line; pre-split multi-deliverable stacks BEFORE scheduling (mid-session
  overrun recovery is user-initiated — stop, clean the tree, report); minting a split rule re-audits
  every remaining unchecked line against it in the same recovery commit; a recovery split is itself plan
  work → audit its replacement lines against every standing rule + the open-decision ceiling within that
  commit. SPLIT RULES: refactor-to-share-internals → the refactor is its OWN behavior-locked unit FIRST
  (existing tests the gate, zero test edits); format walker + test-source integration = walker-core
  (inline-literal tests) then format-completion + integration; nontrivial algorithm + a 2nd authored
  artifact = 2; multi-invariant validator + full rejection coverage = 2; pure-computation module (full
  §-semantics + unit tests) + its recorded-run integration test = 2 (metrics-m2.1 overflowed deriving
  both in one window); canonical-emit layer over an existing type family (one module) + a byte-pinned
  record-shape extension consuming it (a second module) = 2, split at the module seam (report-m2.1
  overflowed: read+derive+first-module-green consumed the window before module 2 started); derivation fn + its
  test-source-pinned battery + an attachment sub-feature = 2; type family + assembly + validation = 3;
  assembly fn + its live-pipeline pin battery = 2; a live-pin battery over the run binary is its OWN unit
  (never paired with assembly or stage wiring); spec-byte amendment (re-pin + reference/test mirror sweep)
  + new feature code = 2 (an open decision that amends pinned bytes is a deliverable, not a preamble);
  crate foundations pair only with a small type surface (one payload module each); deterministic code + a
  SLOW/exploratory live confirm over an external runtime = 2 (code stub-gated + mechanical; the live
  confirm its own unit) → apply to EVERY live-runtime-gated unit at plan time, not only the obviously-slow,
  and on recovery discharge the one-time exploration into memory `## Runtime` + persist any
  session-scratchpad tool the live unit needs to a stable machine-local path (on PATH for a bare-name
  command) so the redo is a checklist. MEASURED ANCHORS (checked stubs carry `NN%`): canonical JSON = 5;
  five-layer recursive type family = 3; lexicon-driven derivation half (loader/binding/builder) = 3;
  statement builder over a prebuilt binding core = 1; exception attachment + determinism tests = 1.
  PRACTICES: house new type families in fresh modules (extending a ~2K-line module costs a full-file
  read); scope each split's Reading slice to exclude files its half leaves untouched; land a compiling
  skeleton before the full test battery; pin expected shapes from observed output, never hand-computed;
  cite only checked roadmap lines as anchors. At plan/re-scope time audit any spec a unit must
  byte-reproduce — readability listings (alignment padding, inline result comments, illustrative
  declaration/conjunct order) contradict deterministic-emission rules and need a scheduled re-pin
  deliverable (smt-emit.3a: §8.6 smt2 vs §6 sorted-declaration). SALVAGE (a reverted session's compiling
  half → committed `.agent/wip-<unit>.patch` the redo line points at; apply, verify vs the line, delete at
  the closing commit): a recovery with margin PROVES the salvage green before reverting (apply full set,
  run gate, fix, revert) → the redo is reproduction-only, gate pre-proven + pass counts banked in the redo
  line. A SOURCE-DIFF salvage banks the whole proven change as a `git diff` + a thin procedure-only
  `.txt` → redo = `git apply`→bless→gate→commit, transcribing NOTHING; a codex-review of a salvage targets
  the wip as the real deliverable + folds accepted NEW TESTS into it pre-redo (else the "reproduction-only"
  redo re-derives them) → re-prove green, re-bank sha + pass counts. An UNCOMPILED draft salvages the same
  way flagged UNCOMPILED — recovery verifies its external assumptions (referenced types/APIs/field names)
  against source, an assumption-verified target not a blind preserve; a whole-NEW-file draft salvages as a
  byte-verified `.rs.txt` copy not a diff (dodges RTK diff-compression + LSP indexing; pre-format it —
  rustfmt reflows an unformatted draft so a byte-exact restore else fails `cargo fmt --check`). A
  DERIVATION overflow (SOTA notation/tool selection + empirical external-crate validation) salvages the
  same — the redo line banks the LOCKED decision, the validated + hashed artifact (any committed file →
  byte-exact `.agent/wip-<file>`, not only `.rs`), AND the wiring APIs pre-transcribed from source, so the
  redo reads nothing but the line. Latent bugs surface in recovery not redo (e.g. a missing `Debug` on a
  public result type); a salvage shared by a multi-unit split is deleted at the LAST consuming unit's close.
- Read-cost is a unit-sizing axis distinct from deliverable count (route-single-ir.2 overflowed
  a 200K window during READING, ZERO code written → nothing to salvage). A unit framed 'one
  deliverable + one gate' still overflows when its test/bless/fixture scaffolding needs
  byte-exact shapes — signatures, sorted-field orders, enum variants, harness helpers, the
  `Resolved`-style stamp structs — assembled across many modules; a deterministic-REPRODUCTION
  gate reads the WHOLE upstream type + helper set. Detect at PLAN time: count the modules a
  unit's gate/bless scaffolding must read for exact shapes, not just its conceptual pieces. A
  nothing-written overflow recovers FORWARD (not via a backward `.patch`/`.rs.txt` salvage):
  (a) SPLIT the production fn from its golden-fixture + gate when separable (route-single-ir.2
  = accept closure; .2b = fill+bless+gate); (b) pre-derive exact code + CONFIRMED signatures +
  the verified equality-premise facts (e.g. clinical_ir diagnostics empty for the 3 docs) +
  insertion anchors into a throwaway `.agent/wip-<unit>.txt` the impl line POINTS at — read
  THAT not the N files, targeted reads only at flagged VERIFY points; delete it in the closing
  commit. A self-checking gate (`content_hash == reference`) bounds transcription risk on the
  PAYLOAD path ONLY: a content-hash-affecting line fails loudly; off-payload lines don't (wrong
  signature → compile error; producer/wrapper/input_hash fields compile AND pass silently → still
  targeted-read those). Mark gate-IRRELEVANT fields (producer
  stamps / step-ids / wrapper-level fields under a payload-only `content_hash`) explicit so the
  session skips pinning them.
- Renaming canonical (§4.3) JSON member keys is a silent test-breaker. The object emitter buffers members then sorts them by key bytes on `finish`; the reader (`canon.rs` `member`/`optional`) is positional — it peeks the next key and demands the caller request keys in ascending byte order. So a key rename moves its sort slot: the code still compiles, but round-trip reads fail `MissingField` at runtime and pinned canonical byte-string literals mismatch. Fix = re-sort each Canonical read+emit member sequence AND every pinned byte-string to the new key order (`printf '%s\n' k1 k2 … | LC_ALL=C sort`). Related: a `#[serde(rename_all="snake_case")]` enum serializes by variant name, so a snake wire-key rename must also rename the CamelCase variant (e.g. ViewText→RenderedText) — caught by name-pin asserts, never the compiler. And hyphenated scope-IDs (`stage-extract.1`, `core-grounding`, `fixtures-m1`) in roadmap+comments are git-commit-traceability keys: keep them historical on a terminology rename (rename only dotted runtime IDs `processing_stage.m1.*` and living prose).
- Backward-compatible canonical-record extension (proven M2.1 model-types, inverse of the rename
  break above): adding fields to a byte-pinned §4.3 record without disturbing pins = make them
  `Option<T>`, emit `obj.optional(name, self.f.as_ref(), |b,v| v.emit_canonical(b))`, read
  `obj.optional(name, T::read)?`, each in the field's sorted-key slot. Omit-None emits nothing →
  prior pins stay byte-identical (the unchanged-expected-bytes M1 pin tests are the regression
  guard — never edit their literals). Emitter sorts on `finish` (emit-call order cosmetic) but the
  positional reader REQUIRES the `obj.optional` call in ascending-key position (peek next key:
  `<name`→UnknownField, `==`→consume, `>name`/absent→None) → a misplaced optional misreads. Pin
  BOTH an all-None fixture (locks old bytes) AND a fully-populated one (locks the new members'
  slots) — once per extended record type, not one exemplar for the family (a populated round-trip
  proves read/write inverse but only a byte-pin locks canonical order/content → each record,
  RunManifest AND ReplayManifest, needs its own populated pin; M2.1 codex follow-up caught the
  missing replay pin). `content_hash` = the generic `content_hash<T: Canonical>` free fn → every Canonical type
  gets it with zero per-type code (a roadmap "content_hash for the new types" clause needs no
  extra impl).
- Behavior-locked extraction past a timed interval (M2.7 run-refactor, codex follow-up): a
  `ProcessingStageClock` opens in the CALLER before the extracted call → pure setup (`format!`/alloc)
  left in the callee body runs INSIDE the timed interval, falsifying a timing-identity claim that byte
  pins CAN'T catch (`duration_ms` is normalized → tests stay green while the guarantee breaks). Audit
  clock boundaries when extracting: hoist pre-clock setup to the caller, pass it in — `compile_verify_group`
  takes `dir: &str` so its `format!("groups/{gid}")` stays outside COMPILE timing (route.single_ir
  supplies its own dir + clock the same way). The call-boundary overhead itself is inherent + below
  ms/normalization resolution — only named setup is worth hoisting.
- Doc-lint gate (Rust): the per-unit test+fmt+clippy gate MISSES rustdoc → run `RUSTDOCFLAGS='-D
  warnings' cargo doc -p <crate> --no-deps` whenever a unit touches doc comments. Two failure shapes:
  a public item's ``[`priv`]`` intra-doc link to a PRIVATE item (`private_intra_doc_links`) → plain
  ticks `` `priv` ``; a link to a type not `use`d in the module (unresolved) → qualified-path
  `` [`T`](crate::T) `` (a docs-only `use` trips `unused_imports`). Caught by codex on M2.20 .2.
- Contract-tense docs (codex flagged TWICE: M2.22 emission_order re-tense, M2.23 .1a follow-up):
  a doc claim about pending wiring must be unit-attributed — "report-m2.1b embeds X in
  `report.json`" holds before and after the unit lands; present-state phrasing ("carriers today:
  report.json bytes agree") overreaches until the wiring commit. House pattern: "run-m2.1 wires
  the observations". Apply at write time — each violation costs a codex follow-up commit.
- Model-runtime adapter (§9, `ckc-cli/src/model.rs`, mirrors `ckc-smt` Z3Adapter; DONE .1/.2a/.2b).
  Live facts beyond code/git: `pub mod model` — a pre-consumer skeleton must be pub or clippy `--lib
  -D warnings` flags dead_code (no-cfg-test lib build; recurs for cassette/route fns). MIRRORS not
  reuses Z3's subprocess machinery — a shared cross-crate runner is a DEFERRED unit (don't
  ad-hoc-dedup) that also absorbs the two codex-REJECTED fixes (`Instant+budget` overflow-panic +
  ETXTBSY vacuous-window; rejected Z3-mirrored, non-realistic, fix-both-not-one) AND the cap/reap of the
  STILL-unbounded post-grace detached drain (a descendant holding stdout open appends to its Vec forever;
  accepted meanwhile for the local trusted runtime, no-unsafe/no-extra-dep). `Completed{bytes}`
  duplicates `stdout_bytes` on clean exit; PARTIAL capture on Timeout/ExitFailure/SpawnFailure diverges;
  stdout stays RAW, never lossy-decoded (byte-stability = cassette determinism). NO
  process-fate→DiagnosticCode here (§7.4 `ai_*` = output-parse, stage-model-fill's job). `set_var`
  forbidden (`#![forbid(unsafe_code)]`) → pure `resolve_command(Option<String>)`; default neutral
  `ckc-model-runtime`, `CKC_MODEL_COMMAND`-override. argv `&[&OsStr]` not `&str` → constraint PATH
  reaches the runtime verbatim (`to_string_lossy` corrupts non-UTF-8 → silent open-fail; 0xFF-tested);
  identity-probe strict `from_utf8`→`IdentityUnparsed` (recorded = true bytes, stderr lossy). COMMITTED
  CLI CONTRACT (module consts+docs; run-m2.*/cassette/stage-model-fill/env-wrapper bind): probe
  `--identity` → `key=value` model_id/quant/runtime_version (order-independent/first-wins/all-nonempty/
  model_id a grammatical Id); generation `--constraint <path> --seed <u64>` + prompt on stdin → bytes on
  stdout.
- Model cassette (§4.4/§9, model-cassette.1 modules + .2 live bless). Crate split mirrors
  ModelIdentity(data, ckc-core)/ModelAdapter(runtime, ckc-cli): `CassettePayload` in ckc-core (needs
  pub(crate) `RawText`/`emit_u64`/`read_u64`), `CassetteStore` record/replay IO in ckc-cli (drives
  `ModelAdapter`). Recorded bytes → lowercase-hex in canonical JSON: lossless for any bytes incl. non-UTF-8,
  and NEVER lossy-decoded — the recorded bytes ARE the determinism (greedy is byte-stable on a fixed host
  but not cross-environment, §9 → replay the committed cassette, never re-invoke). Cassette =
  `ArtifactWrapper<CassettePayload>` origin `ai_generated`/evidence `evidence_discovery_only`/effect `ai`,
  keyed (route, source, seed) at `<root>/cassettes/<route>/<source>/seed-<seed>.json`; `replay` (default)
  runtime-ABSENT, `record` (gated) needs the runtime + a clean `Completed`. Committed TEST cassette →
  `crates/ckc-cli/tests/fixtures/cassettes/...` (.2b precedent — test artifact, NOT `corpus/test_sources/`
  (route units own those) NOR `schemas/`), blessed via an `#[ignore]`d `CKC_MODEL_COMMAND`-unset-guarded
  test mirroring `tests/model_live.rs`, content-hash-pinned. DEFERRED (run-m2.1): the `--record`
  surface + replay.rs model-artifact coverage + §9 manifest `prompt_template_hash` (stage-model-fill.1
  now drives replay/record via `FillSource`, next bullet).
- Model-fill stage core (§7.4/§9, stage-model-fill.1 core + .2 repair/grounding,
  `ckc-cli/src/model_fill.rs`). DECOUPLED core
  `model_fill<T>(store, key, source: FillSource, repair_limit, accept) -> Result<ModelFill<T>, CassetteError>` →
  `ModelFill<T>{target: Option<T>, diagnostics, recorded_calls, repairs}` — a plain value, NOT a §4.6
  event/`ArtifactWrapper`. `FillSource::Replay` (default, runtime-absent) / `Record{adapter,prompt,
  constraint,ctx}` (gated) gets each attempt's cassette via `CassetteStore`, decodes `output_bytes()`, runs
  the route's `accept: impl Fn(&[u8])->Result<T, FillReject>` = the §4 acceptance check (route supplies the
  ClinicalIR/SMT parser+grounding; target + acceptance stay route-side). The `FillReject` variant picks the
  §7.4 code AND repair-vs-terminal: `Schema(reason)` → `ai_schema_violation` and RE-PROMPTS under
  `derive_seed(base, attempt)` (each attempt its own derived-seed cassette) up to `repair_limit`, then
  terminal `repair_limit_exceeded`; `Grounding(absent)` → terminal `ai_hallucinated_source`, spends NO
  repair. The stage ASSERTS the closure's `Grounding` carries ≥1 absent id (empty = a deterministic route
  bug → fail-closed panic, house `expect`/`unreachable` style, not a silent empty-`absent_source_ids`
  diagnostic); route-single-ir still enforces route-side too (defense-in-depth, M2.14 codex follow-up). A
  cassette IO/contract failure stays `Err(CassetteError)`, DISTINCT (route tells a broken recording from a
  bad model output). Two §7.3 counters both surfaced so run-m2.1 emits both without re-deriving:
  `RECORDED_CALLS_COUNTER="recorded_calls"` (one per attempt), `REPAIRS_COUNTER="repairs"`
  (`=recorded_calls-1` here, single draw per attempt). EVENT NOT emitted here — M1 `finish_processing_stage`
  is index-coupled (`PROCESSING_STAGE_KINDS[index]`/`pipeline_step_ids[index]`) → run-m2.1 generalizes
  emission + builds the §4.6 event from the counters. `derive_seed` is pub(crate) (shared: k-sample draws +
  repair re-prompts). `CassetteStore::{build_wrapper, persist}` → pub(crate) so the stage (+ its tests) seed
  cassettes through the store's own contract-valid builder. Tests = replay-only (8): valid (0 repairs),
  schema→repair→recover, repair_limit_exceeded, zero-budget exhaust, hallucinated-terminal, grounding-on-
  repair (multi-id sort+dedup), missing-cassette → `Io`, empty-grounding → panic (should_panic). The
  `Record` arm is type-enforced thin delegation
  to `store.record` (a mis-wired arm cannot compile; `record`'s subprocess plumbing is cassette-layer,
  live-bless-validated), so the shared decode→accept path is covered via `Replay` — no stub-runtime test
  duplicated here. Registry: ONE single_ir-shaped `processing_stage.m2.model_fill` (nondeterministic,
  `[source_document_graph,segments]→[clinical_ir]`), UNREFERENCED (no chain check fires until a route
  pipeline references it); route-direct-smt adds its OWN smt_query-output entry.
- Committed-artifact + hash-pin pattern (`schemas-export.1b`, reused for any committed regenerable
  artifact — report-m2 fixtures, cassettes). EMITTER-BACKED (file regenerable from code): two tests
  beat one env-gated `CKC_BLESS` write-in-test (its token leaking into CI masks drift) — a drift guard
  that NEVER writes (`assert_eq!` committed bytes vs emitter output) + an `#[ignore]`d bless that
  regenerates (`create_dir_all`+write, run manually). Pin `const <X>_HASH = hash_bytes(bytes).as_str()`
  (`sha256:<hex>`, byte-identical to `sha256sum`; re-pin after bless). jsonschema oracle = dev-only,
  `default-features=false` (drops remote-$ref/TLS, keeps `validator_for`/`is_valid`/`pattern`); pin the
  rejection REASON via `iter_errors()` `(instance_path, schema_path)` — a failed `oneOf` reports at the
  parent `.../oneOf`, so prove the nested split (pattern vs type) by the baseline accepting the
  canonical value. HAND-AUTHORED variant (no emitter — grammar / prompt files): the file IS the source,
  its oracle is the format's own recognizer (working `bnf` Earley form in `emit.rs`; two facts that are
  NOT: `Grammar::parse_input` is DEPRECATED → `-D warnings` forbids it, recognize via `g.build_parser()?`
  + `p.parse_input(s).next().is_some()`; `parse_input` borrows `input: &'gram str` from the parser's
  grammar → rebuild the parser per call to free input lifetimes, free for a tiny grammar) → skip bless +
  cross-check; the lone `hash_bytes(file) == <X>_HASH` pin IS the whole drift guard. DESIGN LESSON
  (any grammar/schema oracle): oracle = SOUND SUPERSET of the emitter image, NOT its exact shape — a
  CFG can't bind cross-field coupling / assertion cardinality / declare-before-use, so §8.6 byte pins
  own those; keep the grammar the construct-surface union (grammar-constrained decoding wants the
  union), cover every production incl. the empty-context→`true` collapse, and prove full-match ONLY via
  a trailing-garbage case. Byte-pinned text file → `.gitattributes eol=lf` (sha256 + literal-LF `<nl>`
  survive checkout).
- Schema↔canonical coupling (maintenance): the oracle validates `canonical_payload_bytes(ir)` parsed as
  JSON against the emitted schema, so any §4.3 canonical-encoding change (key rename, integer formatting,
  union shape, a new field) silently breaks good-instance validation unless `schema.rs` tracks it —
  `schema_accepts_canonical_clinical_ir` is that guard (M3 ClinicalStatement additions must extend both).
  Non-obvious anchor: canonical integers are STRING-quoted (`emit_int`→`emit_string`), so interval bounds
  are schema `string`+INT_PATTERN (a bare JSON number is rejected), not `number`.
- Registry model surface (§14): `schemas.yaml` (`SchemaEntry`=id/path/schema_hash/target_kind) +
  `prompts.yaml` (`PromptEntry`=id/path-xor-inline/template_hash/route); both OPTIONAL via
  `load_optional` (absent→empty, additive — M1 counts unchanged). Hash fields are `Hash`-typed →
  grammar-validated on load (Id/Hash use `#[serde(try_from="String")]`; a plain derived Deserialize would
  NOT validate). Validation is SEPARATE — `validate_model_registry`, NOT folded into
  `validate_registries` (no §8.4 cross-refs yet; fold in only when a stage→schema/prompt dangling check
  is wanted, else 18 call sites churn for nothing). Layer split: pure findings (id uniqueness, path
  nonempty, path-xor-inline → `PromptSource`/`Empty`) in core; FILE existence + `schema_hash`/`template_hash`
  match are I/O → CLI `check_model_registry` emits sorted-key `actual`/`expected`/`schema`|`prompt` (or
  `reason`/…) diagnostics, NOT `RegistryFinding`s. Adding a route (prompt + pipeline + stages) is PURE
  additive data through the generic loop → ZERO `registry_check.rs` change; drift guard
  `committed_model_surface_checks_ok` (pinned hashes == real `schemas/` + `registry/prompts/` bytes)
  absorbs it. Prompt CONTENT is ungated (existence + hash + shape only) — first-draft wording, refined at
  run-m2.2's live recording. GOTCHA: roadmap's schemas-export.2 spec carries a STALE .2-era grammar
  hash/size → read the live hash from schemas.yaml/emit.rs, never that spec. Path safety =
  `is_safe_relative_path` (pub ckc-core), ONE predicate reused by the pure validator (`UnsafePath`) + the
  CLI read-guard — LEXICAL only (rejects absolute + `.`/`..`), so a committed repo-local SYMLINK pointing
  outside the tree passes and `std::fs::read` follows it → a real fix is an I/O-layer
  symlink/canonicalize guard across BOTH read loops = its OWN scoped security unit (DEFERRED: low,
  pre-existing, local repo-committed inputs only, not remotely exploitable). Core fixtures (SCHEMAS
  included) use SYNTHETIC hashes; editing SCHEMAS also breaks `strict_loading_rejects_bad_documents`.
- Experiment pipeline-set binding (§14, M2.6): `ExperimentEntry` carries TWO mutually-exclusive
  forms — legacy `pipeline: Option<Id>` (M1) and the set `pipelines: Vec<Id>` + `baseline_pipeline:
  Option<Id>` (the §7.3 delta baseline), all `#[serde(default, skip_serializing_if=…)]` so the M1
  `pipeline:` key stays valid AND each form round-trips back to its own shape (omit-empty); a value round-trip alone can't catch a `skip_serializing_if` regression → a test pins the serialized KEY SET per form (legacy: only `pipeline`; set: only `pipelines`+`baseline_pipeline`). Read the
  binding through the accessors, never the raw field: `baseline()` is SHAPE-AWARE (mirrors the validator: legacy single, or in-set `baseline_pipeline`; any malformed shape → `None`, so `run` rejects EXACTLY what `registry check` does — a plain `.or()` would silently run a both-forms legacy `pipeline`, or a stray/out-of-set baseline, since `run` does targeted resolution NOT whole-set validation)
  and `resolved_pipelines()` (the set, or the single normalized to a one-element vec). `validate_registries`
  is form-aware (`match (&pipeline, pipelines.as_slice())`): legacy `(Some,[])` w/ no baseline → per-pipeline
  Dangling; set `(None,[_,..])` → per-member Dangling + no dups (new experiment-scoped `DuplicatePipeline`; generic `note_duplicates`/`Duplicate{pool:"pipelines"}` would collide w/ the GLOBAL candidates pipelines-pool dup check, which has no experiment to scope it) + baseline must be Some (`Empty{field:"baseline_pipeline"}`)
  and ∈ set (new `BaselineNotInSet`); anything else (neither, both, or legacy + stray baseline) → new
  `PipelineBinding`. The CLI consumes findings via `to_string()`/Display (no exhaustive match) → new
  RegistryFinding variants need ZERO `registry_check.rs` change. `run.rs` deliberately executes ONLY
  `baseline()` and records `pipelines: vec![baseline]` (behavior-locked to M1) — run-m2.1 completes the
  multi-route loop (`resolved_pipelines()` + recording the full set) AND seeds the real set-form
  `exp.m2_multihop` (still unseeded — its route pipelines don't exist yet, would dangle `check`). SPEC: §8.4
  stays M1-singular (faithful history); the M2 generalization went into §14's registry-evolution ledger
  (no §14 byte-pin → free prose).
- Test/example producer IDs: `pipe.<qual>` (`pipeline_id`) + `processing_stage.<qual>.<step>` (`pipeline_step_id`); shared `<qual>` links a pipeline to its steps. Generic unit fixtures use `qual=test`; scenario fixtures keep their own (`m1`/`t`/`base`). Never `cand.*`/`comp.*` — those echo the pre-rename `candidate`/`component` field names the terminology cleanup removed.
- Component vs pipeline-step terminology: reserved now in identifiers AND comments (`b6e1177` + follow-up sweep) — `component` = the §5 IR `ComponentRecord`/`DocIR`/structural concept only; a registry `processing_stage` entry = a pipeline step. OPEN + deliberate (not a missed rename): SPEC §8.4 prose + `registry/candidates.yaml` still read "processing stage component(s)"; resolving it = a SPEC-level vocabulary call (route through the user), so skip auto-"fixing" it on a grep sweep.
- "Oracle" naming: the epistemic-overclaim sense was renamed `runtime-oracle`→`runtime reference`
  (results = locked measurements, not a real-world-truth authority); the TEST-ORACLE sense (pass/fail
  vs the reference) deliberately PERSISTS in `run_oracle.rs` + `rules.rs`. A global retirement
  (`run_oracle.rs`→`run_reference_check.rs`) is an OPEN user/style call → don't auto-rename on a sweep.
- ckc-smt's `serde` dep reads as unused (no `serde::`/`Serialize`/`Deserialize` in ckc-smt/src
  beyond the `fieldless_enum!` invocations) but is REQUIRED: that ckc-core macro expands to
  `::serde::Serialize`/`Deserialize` impls *in the caller's crate*, so every fieldless_enum! user
  must depend on serde — dropping it breaks the build (`E0433` unresolved `::serde`). Holds for any
  crate adopting the macro. Those serde impls go unused there (the canonical path is
  Canonical/CanonRead), an accepted KISS cost of one shared macro over per-call serde gating; don't
  "tidy" the dep away.
- M1 reviewed (gates green, zero code defects in the milestone body — git/roadmap hold the detail). §4.4-vs-§8.3 tension RESOLVED
  by SPEC amendment: a processing stage's total operation result IS its §4.6 EventRecord (§8.3 has no
  per-stage total artifact); only commands materialize a standalone TotalOperationResult (value/
  residual/ambiguity/incoherence buckets stay empty until typed placeholders exist). GUARDRAIL: do NOT
  add per-stage TotalOperationResults — inert + redundant with EventRecords until then (M2+ may
  revisit). OPEN enhancement (unscheduled, AGENTS.md-preferred): tests are example/byte-pin only →
  property-based/fuzzing for the canon layer (round-trip identity, reject-any-mutation) + StringPolicy
  idempotence.
- Engine-agnostic DELIVERABLE (user directive): the committed SPEC/code/registry/roadmap/`schemas/`
  name NO specific LLM inference engine, grammar dialect, or model-file format. M2 elaboration picks the
  engine at build time behind the generic harness contract (greedy + fixed seed, grammar/JSON-Schema
  constraint fed by the exported `schemas/`, recorded subprocess, identity/quant/runtime-version in
  manifests); match §3's engine-neutral phrasing `the M2 local-model runtime`. The CONCRETE runtime/model
  actually used is a machine-specific environment detail recorded in `## Runtime (machine-specific)` below,
  NOT in the agnostic deliverable (the contract is the artifact; the pick is config). `docs/` research
  corpus (model-routes.md etc.) may name engines as landscape — out of scope. Fixtures/test
  values obey this too — use unmistakably-synthetic tokens (`model.baseline`/`fixture_quant`/`1.0.0`),
  since a realistic generic quant/format token still names a real scheme (M2.1 codex follow-up: a
  real bit-width token had slipped into a fixture whose comment asserted it named none; RECURRED M2.9 r3 — `q4` slipped into a model-adapter test fixture, the dialect-only de-leak grep missed it). Audit = word-boundary `git grep -niP` (names `\b`-bracketed) over EVERY forbidden category (engines + grammar dialects + quant/model-format tokens `q4`/`gguf`/…), not just one, and not a bare substring grep (false-matches a Cargo.lock dependency name). Catch the bare quant token with a case-SENSITIVE lowercase `\bq[2-8](_[0-9km])?\b` (drop the global `-i`, wrap the engine/dialect/format names in `(?i:...)`): it matches `q4`/`q4_0` yet skips the uppercase `Q1`/`Q2` SMT labels (a case-INSENSITIVE `q[2-8]` false-hits them — the prior skip-bare-`q[2-8]` rule left bare `q4` UNAUDITED; codex M2.17). The repo-wide grep OVER-matches by design (AGENTS.md: a filtered finding beats a dropped bug) → triage each hit vs the standing exempt/false-positive set: `docs/` landscape (out of scope), this bullet's own rule-doc (the `q4`/`gguf` examples), the route.fixture cassette (real identity, exempt), and the lowercase `q2`/`q3` SMT-pair variables in `verdict.rs` (M1 query indices, not quants), the English word `guidance` in prose (false-match on the constrained-decoding library token), and dev/review-tool names (`Codex`, `Claude Code`) in process/provenance notes — the rule targets the route's INFERENCE engine (what fills the schema), not dev tooling, and the mandated `Codex-Review:` commit trailer sanctions `Codex`, so triage these EXEMPT, never remove (codex M2.18 re-flagged a roadmap `Codex follow-up` note; rejected — dev-tool, not a route-engine leak). So the per-UNIT close gate = the unit's TOUCHED files carry no token (scope the grep to them); the milestone review runs the full-repo triage. Hand codex-review prompts THIS full exempt set verbatim, not a condensed invariant — else codex re-flags the rule-doc's own quant/format examples (codex M2.20). Reconstruct the command from this bullet — no banked wip (consumed at .2b close). EXCEPTION (user decision, model-cassette.2): committed RECORDED cassettes (under `crates/ckc-cli/tests/fixtures/cassettes/` now, run-m2.2's experiment-cassette roots later) carry the runtime's REAL `model_identity` (model/quant/engine strings) — machine-specific MEASUREMENT data with honest provenance, NOT engine-neutral contract/fixture artifacts → EXEMPT from the synthetic-token rule + this audit. Audit FAIL-CLOSED: exclude only the SPECIFIC live-recorded path(s) whose cassettes carry the runtime's REAL `model_identity` (model/quant/engine strings) — today `route.fixture/` (its recorded `seed-42.json`): `git grep -niP … -- . ':(exclude)crates/ckc-cli/tests/fixtures/cassettes/route.fixture/'`, adding run-m2.2's experiment roots as recorded — NOT the whole `cassettes/` tree, so CRAFTED synthetic cassettes (route-single-ir golden + bad, SYNTHETIC identity) stay AUDITED + pass (a `cassettes/`-root exclude would wrongly free-pass them). Replay pins output/provenance/content-hash + the full recording envelope (producer + empty diagnostics/trace/runtime), never an identity VALUE BY NAME → the host runtime swaps with no test-code edit, but the identity rides the pinned content-hash (it changes only via a deliberate re-bless + re-pin).
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
  - DiagnosticRecord field convention (whole codebase): schema/parse codes (`schema_invalid`,
    `ai_schema_violation`) carry the failure reason in `payload`, leaving `region_ids` +
    `artifact_hashes` EMPTY; `artifact_hashes` is populated ONLY by hash-identity diagnostics
    (`replay_mismatch` = the diverging content hashes). So `ai_schema_violation` MIRRORS
    `schema_invalid` (empty) + diagnostic→source lineage rides the §4.6 event/trace layer (run-m2.1),
    NOT the diagnostic's own hashes — keep `ai_schema_violation` `artifact_hashes` empty.
    `ai_hallucinated_source` likewise leaves both EMPTY — a hallucinated id resolves to NO real §4.5
    span, so the sorted+deduped absent ids ride `payload` (`absent_source_ids`), never `region_ids`.
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
    JSON-Schema (standard) for ClinicalIR, BNF grammar (ABNF-style `;` comments) for the SMT surface (no engine
    constraint-dialect name); the env wrapper compiles them to the runtime's constraint format.
  - "test all layer configurations" (user directive) → deferred to M3 as the §10 route-axis gradient
    seed: every meaningful single_ir IR layer + the DMN-style alt. The user chose keeping M2 the §9
    minimal pair over widening §9; the gradient is the experiment §10 ("vary and layer existing IR
    forms") was written to be.
  - Recon mechanics that right-size the units: a processing-stage `kind` is a free-form Id (no enum)
    → adding `model_fill` is registry data, not an enum change; the middle-layer derive fns live in
    ckc-cli (`segment.rs`, `normalize.rs`, `rules.rs` `derive_norm_ir`), only `DocIr::from_graph` +
    `FormalIr::derive`/`FormalConstraint::from_rule` sit on the ckc-core types → `run-refactor`
    extracted ONLY the per-group compile→verify back end (per-doc derive fns already pub → route units
    compose them directly; full spec in respec commit `93953c4`). PLAN LESSON (this respec recovered an
    overflow): a unit framed "extract a tail/chain X→Y→Z" must share ONE iteration granularity —
    `derive_norm_ir`/`assemble` are per-document (N×), `compile`/`verify` per-group (1× fan-in), so they
    cannot be one linear fn; conflating granularities forced a full-session design re-derivation. Check
    stage granularity at plan time. Route→tail wiring (agent-confirmed): a route feeds the M1
    `compile_verify_group` back end by HAND-BUILDING a minimal `Resolved` (that fn reads only
    `pipeline_id` + `pipeline_step_ids[4=compile]`/`[5=verify]` + `toolchain_manifest_hash` +
    `budget_ms`; `documents`/`groups`/`plan` are unread stubs); `resolve()` is NOT reusable (hard-requires
    all 8 stage KINDS + `[Id; 8]`, returns None for the 6-stage single_ir pipeline); the route fn lives in
    `run.rs` (`Resolved` + `compile_verify_group` private to `mod run`). The single_ir route's
    accept-closure (`single_ir_accept`) + per-doc fill (`single_ir_fill`: extract→segment→`model_fill`
    Replay→deterministic tail mirroring `assemble_bundle`) + golden-cassette wiring LANDED in `run.rs`
    (route-single-ir.2/.2b); run-m2.1 reuses this minimal-`Resolved` pattern (or generalizes `resolve()`
    to N stages) for the in-`execute` route loop. route-single-ir.3 added the verdict-half scoring test
    (`single_ir_route_scores_m1_groups`): a route-scoring test mirrors
    `run_oracle.rs::assert_group_matches_reference` IN FULL (both branches, incl. the no-conflict
    `expected_no_conflict_result` Q1-unsat/Q2-skipped closure + panic-on-unknown-outcome; a partial
    mirror passes vacuously) and resolves groups + reference from `exp.m1_scaffold` (doc-id→bundle map,
    iterate `test_source_groups`, assert `reference.len()==test_source_groups.len()`), never a hardcoded
    membership (drifts silently vs the registry) — codex M2.18 caught both. route-direct-smt +
    metrics/report-m2 score the same M1 groups → reuse this shape. Ceiling = smoke test (`.2b` pins
    payload-equality to M1, run_oracle pins M1 verdicts vs reference); the load-bearing route-execution
    wiring is run-m2.1's. route-single-ir.4 closed the route's §7.4 rejection coverage
    (`single_ir_route_rejection_codes`): pin the accept-closure→§7.4-code mapping by calling
    `model_fill(store, key, Replay, repair_limit, single_ir_accept(regions, segs))` DIRECTLY (inspect
    `ModelFill.target`/`diagnostics`/`repairs`/`recorded_calls`); `single_ir_fill` is asserted via
    `shell.ledger()` for the ROUTE-level §7.4 surfacing (recovery → 1 `ai_schema_violation` + a bundle;
    hallucinated → `ai_hallucinated_source` + `None`). Committed bad cassettes under
    `route.single_ir/test_source.m1_guideline_a` (synthetic-identity → audited), seeds 99/98/97
    (+derived) = hallucinated-terminal / schema→recover / exhaust-through-re-prompt; mechanics +
    derived-seed constants live in the tests (`single_ir_route_rejection_codes`,
    `bless_single_ir_rejection_cassettes`). LESSON (codex M2.19): `repair_limit=0` hits
    model_fill's terminal branch at attempt 0 BEFORE the re-prompt loop → it proves only the zero-budget
    boundary (already model_fill.rs's coverage), NOT multi-attempt exhaustion; faithful route-level
    exhaustion needs malformed cassettes at the base AND each derived seed through the budget. Pin the
    `ai_schema_violation` payload SHAPE too (key `reason`, non-empty, empty refs), symmetric to the
    hallucinated/exceeded pins.
  - route.direct_smt seam (M2.20 respec → route-direct-smt.1-.5; user-confirmed both forks): the direct
    route emits SMT, but `verdict::verify` consumes a `CompiledArtifact` whose `validate()` demands
    non-empty `region_ids` per assertion — a no-IR route CANNOT honestly supply them, so do NOT reuse
    `m1.verify` / build a fabricated artifact. Seam = BYPASS: extract verify()'s Q1→Q2 gate into a shared
    `verify_pair` (private; borrowed `(&Id,&str)` overlap/deontic tuples), add `pub
    verify_query_pairs(adapter, &[MintedQueryPair], budget)` (LANDED .2, both re-exported from ckc-smt lib;
    `MintedQueryPair=((Id,String),(Id,String))` owned pairs — a `pub type` alias dodges clippy
    `type_complexity` on the nested-tuple slice while preserving the pinned type; caller-minted ids + model
    bodies → `invoke` + `assemble_result`, no artifact); registry `processing_stage.m2.verify_smt` consumes
    `smt_query` DIRECTLY (`kind: verify`, no `compiled`), so the 3-stage "model_fill→syntactic-validity→
    verify" chain the roadmap first sketched collapses — "syntactic-validity" folds into that solver run
    (`target_syntax_failure`/`TargetParseError`, .5's direct-unique terminal, NO repair). Per-query emission
    (grammar `<query>` = ONE query, unchanged + hash-pinned): a pair = 2 `model_fill` replays keyed by
    ROLE-namespaced source at the base seed — `source: <gid>.overlap` + `<gid>.deontic` (NOT a shared `<gid>`
    source with Q2 at `derive_seed(base,1)`: `model_fill` reads repair attempt `i` under `derive_seed(base,i)`
    on the SAME source [model_fill.rs L132], so a shared-source Q2 would ALIAS Q1's first repair — a real
    collision, caught in codex review); accept =
    shallow well-formedness (utf8 + `(set-logic`/`(check-sat)`) → `Schema` only, NO grounding (the solver is
    the syntactic authority). GOLDEN cassette bytes = the group's M1 emitted `query_bodies[2k]`/`[2k+1]`
    VERBATIM → the `:named a.<rule_id>` labels == reference `expected_unsat_core` → scoring reuses
    `single_ir_route_scores_m1_groups`'s shape. These golden cassettes = harness/determinism
    fixtures (both routes verify identical SMT → identical verdicts BY CONSTRUCTION): they prove
    plumbing + scoring, NOT a §9 route-quality difference. The real route contrast is measured ONLY by
    run-m2.2's LIVE weak-model run (the weak baseline degenerates on the direct route) + the degraded
    rejection cassettes (route-single-ir.4 + route-direct-smt.5) → report/run units attribute
    happy-path-replay parity to the harness, never to §9 evidence (codex M2.20 .3a). 5 units, not ≤4 (the bypass ADDS the `verify_pair` refactor
    + `verify_query_pairs`; the roadmap's "likely fewer" guess assumed reusing m1.verify, which the region-id
    wall rules out). direct tail is its OWN fn (`compile_verify_group` inlines `compile()` + hardcodes
    COMPILE=4/VERIFY=5; the 4-stage direct pipeline has `verify_smt` at slot 3, no `compiled`). Two slot-3
    consequences (codex review): (1) `finish_processing_stage(idx)` derives the kind from
    `PROCESSING_STAGE_KINDS[idx]` AND gates the solver-budget counter on `idx == VERIFY(5)` [run.rs
    L1310-1316] → idx-3 reuse mis-stamps `"assemble"` + drops the budget (idx-5 over-runs the 4-entry
    step-id array) → emit the verify event directly with kind `"verify"` + `SOLVER_BUDGET_KEY`; (2) verify
    cites the UPSTREAM artifact hash (single_ir: `compiled.content_hash`), so direct wraps each fill body as an
    `smt_query` `ArtifactWrapper<QueryBody>` (Canonical+CanonRead) and `verifier_results` cite those two
    `content_hash`es — the CASSETTE hash is fill-wrapper provenance deferred to run-m2.1 (both routes).
    LANDED (.3a golden cassettes + .3b fill) in run.rs beside single_ir's — `direct_smt_accept` +
    per-group `direct_smt_fill` exactly per the seam above. NOVEL raw-AI provenance (distinct from single_ir's ir_bundle
    `DeterministicCompiler`/`MechanicalEvidenceStatus`): the smt_query IS the raw model body, only
    shallow-accepted (no deterministic transform, no source linkage) → `Origin::AiGenerated` +
    `EvidenceStatus::AcceptedEvidenceStatus` + EMPTY external effects. Why consistent: `wrapper()` forces
    effects=`[]`, and `validate()` enforces ONLY the effects↔status rule (non-empty effects ⇒
    `EvidenceDiscoveryOnly`), so empty-effects PERMITS an Accepted status; it does NOT check origin/status/kind
    → the `direct_smt_fill_reproduces_m1_query_bodies` test pins origin/status/kind/schema/producer explicitly.
    Keep this shape (do not "fix" Accepted→DiscoveryOnly or add effects). .4 verdict tail
    `direct_smt_verify_group` LANDED beside `compile_verify_group` (its OWN fn per the slot-3 rule above;
    `DIRECT_VERIFY=3` const, M1 `VERIFY`=5 = inert padding in the direct `[Id;8]`; run-m2.1 wires
    `direct_smt_fill`→it). Scoring test `direct_smt_route_scores_m1_groups` keys the no-conflict closure off
    minted `<gid>.overlap`/`.deontic` ids (no `solver_query_plan`) — metrics/report-m2 reuse this shape.
    GOTCHA (codex .4): `input_hashes` canonicalize as a §4.3 SET — the landed wrapper sorts them by hash, the
    in-memory event keeps insertion order → multi-input provenance assertions compare as SETS, never pin
    emitted order (bit the .4 event/wrapper test; will bite metrics/report-m2).
  - Runtime-gate findings (gate MET, confirmed functionally on a real test source; byte-stability +
    seed-inertness + degeneration mechanics live in `## Runtime`, machine specifics in
    `.agent/runtime.local.md`). Metrics/report conclusions: constrained decoding forces schema-VALID output
    that can be semantically WRONG, or INCOMPLETE/INVALID when the constraint format permits unbounded
    whitespace + the model is weak (greedy loops on free whitespace, truncates at the token budget) → the
    M2 report scores BOTH acceptance-rate (schema-validity, counting truncation/parse-incompleteness as a
    failure mode) AND verdict-accuracy, never validity alone; a tight-grammar route (explicit newlines, no
    free whitespace) sidesteps the truncation mode. The baseline deliberately pins a weak model whose
    free-form/direct-route output degenerates → exercises §9's "direct-route failures common" path (pin the
    exact identity in the run config; alternatives ok). Greedy is SEED-INERT so k per-sample seeds draw
    identically (convergence trivially 1.0) → MEANINGFUL k-sample convergence (metrics-m2.2) needs a
    sampling config (temperature > 0, seed fixing each draw), a downstream decision NOT the adapter's
    (`invoke_samples` stays config-agnostic: derive seeds, invoke, record).
- Metrics assembly (§7.3/§9, metrics-m2.1/.2, `ckc-cli/src/metrics.rs`; run-m2.1 wires, report-m2
  embeds). `route_metrics(pipeline_id, fills, groups, samples, reference)` — the `samples` channel
  (`&[Vec<GroupObservation>]`, one battery per draw) feeds `k_sample_convergence` = mean over the
  UNION group universe of pairwise verdict-fingerprint agreement (Σ agree_pairs / (|G|·C(k,2)));
  k<2 or empty universe → zero denominator → `not_applicable`, so the row is ALWAYS emitted and a
  single-draw recorded run honestly reads NA (the .1b run.rs pinned vectors carry it; greedy is
  seed-inert → meaningful convergence waits on a sampling config, see runtime-gate bullet).
  Fingerprint = verdict CONTENT projection: query_pairs + per-result (query_id, category,
  verdict-or-`-`, unsat_core-or-`-`) in §4.3 order; EXCLUDES solver_identity (environment),
  diagnostics (telemetry), model (sat witness bytes); None==None agrees (consistent absence =
  stable); sensitivity to model-minted qids/pair-ids = by-design translation-instability signal.
  `experiment_metrics(routes, baseline_pipeline_id)` fail-closed panics on dup routes / missing
  baseline; per-route delta rows = union of metric ids (sorted), `Rational::sub` (ckc-core id.rs —
  signed exact, added .2) on Value×Value else `not_applicable`; baseline gets NO self-delta row.
  `ExperimentMetrics::emission_order()` IS the §9 raw-rows-before-ranking contract (all RawRows
  sections strictly before all DeltaTable sections) — UNCONSUMED until run-m2.1/report-m2 land;
  those units must render through it, never the fields ad hoc (codex M2.22: the §9 guarantee
  reaches artifacts only once an emitter walks it). REPORT-m2.1 TRAP (DISCHARGED in the .1a
  salvage, byte-position-tested): §4.3 sorts member keys, so raw-before-delta in BYTES needs the
  raw-rows key below the delta key → keys `raw_rows` < `route_deltas`, Rust fields stay
  routes/deltas; the same key-sort trap applies to ANY future §-ordered canonical member pair.

## Runtime

Concrete M2 runtime specifics (engine, model id/quant, install paths, API symbols, measured timings,
observed outputs/degeneration) are machine-specific + drift → recorded in gitignored
`.agent/runtime.local.md`, keeping the committed deliverable engine-agnostic (Policy). Engine-agnostic
conclusions the committed code + units rely on:
- The §9 runtime command installs on PATH under the adapter's default bare name → `ModelAdapter::
  new()` resolves it live — CONFIRMED end-to-end (`.2b` `model_live` test); install/invocation specifics
  → `runtime.local.md`.
- Greedy decoding = deterministic argmax → output BYTE-STABLE within + across processes on a fixed
  host/runtime build, and SEED-INERT (argmax ignores the seed) → `invoke_samples`' k per-seed draws
  coincide under greedy (sample diversity needs a sampling config → downstream, see M2-plan).
  Cross-ENVIRONMENT determinism NOT guaranteed → the recorded-bytes cassette, not a live re-run, is the
  correctness mechanism.
- Constraint mechanism: OBSERVED to honor a bounded schema (complete + valid terminating instance) on the
  local runtime; a permissive full schema lets a weak greedy model degenerate (whitespace loop →
  truncated/invalid) = expected weak-baseline failure, not a mechanism fault. A LOCAL OBSERVATION, NOT an
  engine-general guarantee → live-CHECKED by the `.2b` `model_live` test: constrained output parses +
  schema-validates against the committed bounded fixture — conformance CONSISTENT WITH `--constraint` being
  honored, necessary but not alone sufficient (a fixed conforming object would also pass; the bounded
  schema makes accidental conformance unlikely for a free-running weak model).
- `derive_seed` exact splitmix64 draws (engine-agnostic, replay-load-bearing) are pinned in the
  `derive_seed_is_deterministic_and_distinct` test (model.rs, `.2a`) — read the test, not a memory copy;
  the `.2b` `model_live` test re-asserts them live through `invoke_samples`.
- `.2b` DONE — `crates/ckc-cli/tests/model_live.rs` (`#[ignore]`d; `cargo test -p ckc-cli --test
  model_live -- --ignored`) is the standing live confirmation of the §9 runtime properties — read the
  test, not a copy. Non-obvious: it does NOT assert cross-seed equality (greedy seed-inertness is
  environment-specific → `runtime.local.md`); its bounded enum+bool constraint fixture lives in
  `tests/fixtures/` NOT `schemas/` (test artifact, not a production route constraint; the plan-line
  'schemas/ constraint' was shorthand — don't relocate).
