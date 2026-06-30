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
  small type surface (one payload module per foundation unit); deterministic code paired with a SLOW or
  exploratory live confirm over an external runtime = two units (code stub-gated + mechanical; the live
  confirm its own unit — model-adapter.2 (~24s/call + a one-time weak-model degeneration
  discovery) then model-cassette (modules + a live bless) both overflowed pairing them → apply to
  EVERY live-runtime-gated unit at plan time, not only the obviously-slow), and recovering such an overflow discharges that one-time exploration into memory
  `## Runtime` + persists any session-scratchpad tool the live unit needs to a stable machine-local path
  (on PATH for a bare-name command) so the redo is a checklist. Measured anchors (checked
  roadmap stubs carry the `NN%` figures): canonical JSON = five units; a five-layer recursive
  type family = three units; a lexicon-driven derivation half (loader / binding / builder) =
  three units; statement builder over a prebuilt binding core = one unit; exception
  attachment + determinism tests = one unit. Practices: house new type families in fresh
  modules (extending a ~2K-line module costs a full-file read); scope each split unit's
  Reading slice to exclude files its half leaves untouched; land a compiling skeleton before
  the full test battery; salvage a reverted session's compiling half as a committed
  `.agent/wip-<unit>.patch` the redo line points at (apply, verify against the line, delete in
  the closing commit; a recovery with context to spare PROVES the salvage green before
  reverting — apply the full set, run the gate, fix what it catches, then revert — so the redo is
  reproduction-only with the gate pre-proven + its pass counts banked in the redo line (a SOURCE-DIFF salvage banks the whole proven change as a `git diff`
  `.agent/wip-<unit>.patch` + a thin procedure-only `.txt`, so the redo is `git apply` → bless → gate →
  commit, transcribing NOTHING — route-single-ir.2b M2.17), latent bugs
  caught in recovery not redo: M2.13 caught a missing `Debug` on a public result type; a codex-review
  of a salvage targets the wip as the real deliverable + folds accepted NEW TESTS into it pre-redo,
  since deferring them would re-derive in the "reproduction-only" redo → re-prove green, then re-bank
  the wip's sha AND pass counts in the redo line, e.g. M2.14 added a grounding-on-repair test 425→426)
  — an uncompiled draft salvages the same way flagged UNCOMPILED,
  transcription-with-verification still beating re-derivation (recovery also verifies the draft's external assumptions —
  referenced types/APIs/field names — against source, making the salvage an assumption-verified target
  not a blind preserve; a whole-NEW-file draft salvages as a byte-verified `.rs.txt` copy not a
  diff, dodging RTK diff-compression + LSP indexing (pre-format the salvage — rustfmt reflows an
  unformatted draft, so a byte-exact restore fails `cargo fmt --check` otherwise); a unit that overflows on DERIVATION not
  implementation (SOTA notation/tool selection + empirical external-crate validation, e.g.
  schemas-export.2's BNF + `bnf`-crate pick) salvages the same way — the redo line banks the LOCKED
  decision, the validated + hashed artifact (any committed file → byte-exact `.agent/wip-<file>`, not
  only `.rs`), AND the wiring APIs pre-transcribed from source (emitter fixtures, the .1b hash-pin
  form), so the redo reads nothing but the line and runs pure wire+gate; a salvage shared by a
  multi-unit split is
  deleted at the LAST consuming unit's close); pin expected shapes from
  observed output, never hand-computed; cite only checked roadmap lines as measured anchors.
  At plan/re-scope time, audit any spec listing a unit must byte-reproduce: listings written
  for readability (alignment padding, inline result comments, illustrative declaration or
  conjunct order) contradict deterministic-emission rules and need a scheduled re-pin
  deliverable (caught pre-session for smt-emit.3a: §8.6 smt2 vs §6 sorted-declaration rule).
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
- Model-runtime adapter (§9, M2.8 model-adapter.1, `ckc-cli/src/model.rs`, mirrors `ckc-smt`
  Z3Adapter). Non-obvious decisions beyond the code/docs: (1) `pub mod model` NOT private — a
  skeleton landed ahead of its in-crate consumer (the forthcoming model-fill stage) must be pub
  API or clippy `--lib -D warnings` fails dead_code (test-only use doesn't count in the no-cfg-test
  lib build); matches the exposed pipeline-mechanics modules, not the private CLI glue. (2) MIRRORS
  not reuses the Z3 subprocess machinery (`spawn_piped`/`drain`/`run_process` + the 4 budget/grace
  consts) — roadmap "helpers mirror" + the refactor-first rule; extracting a shared cross-crate
  subprocess runner is a deferred future unit, don't ad-hoc-dedup. (3) `ModelOutcome::Completed{bytes}`
  intentionally duplicates `ModelRun.stdout_bytes` on a clean exit (the Z3 raw-vs-interpreted split) —
  only stdout_bytes carries the PARTIAL capture on Timeout/ExitFailure/SpawnFailure, so they diverge
  there; stdout stays raw bytes (never lossy-decoded — byte-stability is the cassette determinism).
  Documented to pre-empt a redundancy flag. (4) NO process-fate→DiagnosticCode map here: the §7.4
  `ai_*` codes are OUTPUT-parse concerns, so process spawn/timeout/exit→diagnostic is
  stage-model-fill's job; the adapter returns raw outcome data. (5) `set_var` is forbidden
  (edition-2024 unsafe + crate `#![forbid(unsafe_code)]`) → env policy split into pure
  `resolve_command(Option<String>)` tested without env mutation; default = neutral role name
  `ckc-model-runtime`, `CKC_MODEL_COMMAND`-overridable. (6) successful bare-name PATH resolution is
  covered LIVE in .2 (no `set_var` to inject PATH); .1 proves only that an absent bare name fails at
  the probe spawn. The committed CLI contract (probe `--identity` → `key=value` lines
  model_id/quant/runtime_version, parse order-independent/first-wins/all-required-non-empty/model_id a
  grammatical Id; generation args `--constraint <path> --seed <u64>` + prompt on stdin → generated
  bytes on stdout) lives in the module consts + docs — model-adapter.2/model-cassette/stage-model-fill
  + the env-supplied wrapper all bind to it. Tests drive a committed in-source stub (the `COMMITTED_STUB`
  contract emulator, materialized per-test to a unique temp exec) covering every outcome.
- Codex-review (M2.8) — mirror divergences from Z3Adapter + deferrals: (a) argv `&[&OsStr]` not
  `&[&str]`: model.rs is the first to pass a PATH as an argv element, so the constraint path reaches
  the runtime verbatim (Z3 passes only ASCII flags → its `&str` is lossless; `to_string_lossy`
  corrupted non-UTF-8 paths → silent wrapper open-failure); 0xFF-path regression-tested. (b)
  identity-probe stdout strict `from_utf8`→`IdentityUnparsed`, not lossy (recorded identity = the
  runtime's true bytes; stderr stays lossy/diagnostic). (c) REJECTED — Z3-mirrored, non-realistic, fix-both-not-one —
  `Instant::now()+budget` overflow-panic (absurd Durations) + the ETXTBSY-recover test's sub-60ms
  vacuous-pass window; folded into the deferred shared-subprocess-runner refactor. The post-grace
  detached drain is unbounded-worst-case (a descendant holding stdout open + writing forever keeps the
  thread appending to its `Vec`) — accepted for the local trusted runtime under no-unsafe + no-extra-dep
  scope, capped or reaped in that same refactor (M2.9 r3 codex).
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
- Committed-artifact + hash-pin pattern (`schemas-export.1b` = first repo instance). EMITTER-BACKED
  variant (committed file regenerable from code, e.g. `.1b`'s ClinicalIR JSON-Schema): two tests beat
  one env-gated test —
  a `CKC_BLESS`-style write-in-test masks drift the moment its token leaks into CI (codex M2.3): a
  drift guard that NEVER writes (`assert_eq!` committed bytes vs emitter output, so no env state can
  mask it) + an `#[ignore]`d bless that regenerates the file (`create_dir_all` + write), run manually
  (`cargo test <bless_fn> -- --ignored`). Pin `const <X>_HASH = hash_bytes(bytes).as_str()` (plain
  sha256 → `sha256:<hex>`, byte-identical to `sha256sum`; re-pin from `sha256sum` after blessing; the
  assert_eq also cross-checks committed == emitted). Oracle = dev-only `jsonschema`,
  `default-features=false` (drops remote-$ref resolvers + TLS a self-contained schema never needs,
  keeps `validator_for`/`is_valid` + `pattern`). Pin the rejection REASON, not just `!is_valid`:
  assert each malformed case's `(instance_path, schema_path)` via `iter_errors()` (a failed `oneOf`
  reports at the parent's `.../oneOf`, not the nested keyword → prove the nested split, e.g. pattern
  vs type, by the baseline accepting the canonical value). HAND-AUTHORED variant (no emitter — e.g.
  schemas-export.2's BNF grammar, route prompt files): the file IS the source + its oracle is the
  `bnf` recognizer (not jsonschema), so skip bless + the cross-check; the lone `hash_bytes(file) ==
  <X>_HASH` pin IS the whole drift guard (edit → hash flips → fail). `bnf` 0.6 wiring (the working
  form is in `emit.rs`; these two facts are not): recognize through `g.build_parser()?` +
  `p.parse_input(s).next().is_some()` (full-match Earley) — `Grammar::parse_input` is DEPRECATED so
  `-D warnings` forbids it; `parse_input` binds `input: &'gram str` to the parser's grammar borrow,
  so rebuild the parser per call (or compute every input before `build_parser`) to free input
  lifetimes — a tiny grammar makes the rebuild free. Oracle scope = SOUND SUPERSET of the
  emitter image, NOT its exact shape: a CFG can't bind cross-field coupling
  (logic↔produce↔result), assertion cardinality, or declare-before-use → the §8.6 byte pins own
  those; keep the grammar the construct-surface union (downstream grammar-constrained decoding wants
  the union, not the 2-query image), don't tighten it to match emit. Cover every production incl. the
  easy-to-miss empty-context→`true` collapse. Reject-case honesty: full-match is proven ONLY by a
  trailing-garbage case (complete query + extra bytes); a missing-terminator rejects via its terminal
  production regardless of anchoring. Byte-pinned text file → `.gitattributes eol=lf` so the sha256 +
  the literal-LF `<nl>` survive any checkout.
- Schema↔canonical coupling (maintenance): the oracle validates `canonical_payload_bytes(ir)` parsed as
  JSON against the emitted schema, so any §4.3 canonical-encoding change (key rename, integer formatting,
  union shape, a new field) silently breaks good-instance validation unless `schema.rs` tracks it —
  `schema_accepts_canonical_clinical_ir` is that guard (M3 ClinicalStatement additions must extend both).
  Non-obvious anchor: canonical integers are STRING-quoted (`emit_int`→`emit_string`), so interval bounds
  are schema `string`+INT_PATTERN (a bare JSON number is rejected), not `number`.
- Registry model surface (§14, M2.5): `registry/schemas.yaml` (`SchemaEntry` =
  id/path/schema_hash/target_kind) + `registry/prompts.yaml` (`PromptEntry` = id / path-xor-inline /
  template_hash / route). Hash fields are `Hash`-typed → grammar-validated on load (Id/Hash use
  `#[serde(try_from="String")]`; a plain derived newtype Deserialize would NOT validate). Validation is
  SEPARATE: `validate_model_registry` (not folded into `validate_registries`) because the model surface
  has no §8.4 cross-refs yet — model-fill stages will bind schema/prompt ids in route units; fold into
  `validate_registries` only when a stage→schema/prompt dangling check is actually wanted (else 18
  call sites churn for nothing). Layer split: pure findings (id uniqueness, schema path nonempty,
  prompt path-xor-inline → `PromptSource`/`Empty`) live in core; schema FILE existence + `schema_hash`
  match are I/O → CLI `check_model_registry` emits them as sorted-key `actual`/`expected`/`schema` (or
  `reason`/`schema`) diagnostics mirroring `load`'s file/reason shape, NOT `RegistryFinding`s. Both
  files are OPTIONAL via `load_optional` (absent→empty, no diagnostic — additive surface, M1 ran
  without them; keeps existing tempdir CLI tests at their old counts). schemas.yaml seeded: ids
  `schema.clinical_ir`/`schema.smt_query`, `target_kind` = constrained output layer
  `clinical_ir`/`smt_query`. prompts.yaml seeded by route-single-ir.1: `prompt.single_ir` →
  `registry/prompts/single_ir.txt` (route `route.single_ir`, `eol=lf`-pinned) + the symmetric prompt
  file/hash loop in `check_model_registry` (path → file bytes, inline → text bytes, vs `template_hash`;
  mismatch payload sorted `actual`/`expected`/`prompt`; read-error sorted `prompt`/`reason`). Prompt
  CONTENT is NOT gated (only existence + hash + path-xor-inline shape) — first-draft wording, refined at
  run-m2.2's live recording; route.direct_smt seeds its own prompt later. (M2.15 codex-review: prompt
  prose reframed negatives→positives per AGENTS.md pink-elephant rule + hash re-pinned; positive framing
  is a standing style rule, distinct from the deferred perf tuning.) Drift guard =
  `committed_model_surface_checks_ok` (schemas.yaml + prompts.yaml pinned hashes must equal the real
  `schemas/` + `registry/prompts/` bytes).
  Roadmap's schemas-export.2 spec carries a STALE .2-era schema hash/size (codex `ecca074` tightened the
  grammar; collapses at M2 review) → read the live hash from schemas.yaml/emit.rs, never that spec.
  M2.5 codex-review: registry paths checked safe-relative via `is_safe_relative_path` (pub ckc-core) —
  ONE predicate, in the pure validator (`UnsafePath` finding, schema+prompt paths) AND reused at the
  CLI read-guard (skip reading an unsafe path); don't duplicate. LEXICAL only (rejects absolute + `.`/`..`
  components) → a committed repo-local SYMLINK pointing outside the tree still passes + the CLI
  `std::fs::read` follows it (only the target's hash, not content, reaches a diagnostic); the pure
  validator can't catch it (no I/O) → a real fix = an I/O-layer symlink/canonicalize guard across BOTH
  read loops = its own scoped security unit (M2.15 codex-review deferred: low, pre-existing, local
  repo-committed inputs only, not remotely exploitable). Core fixtures (SCHEMAS included) use
  SYNTHETIC hashes; editing SCHEMAS also breaks `strict_loading_rejects_bad_documents` (it replaces a
  SCHEMAS hash to forge a bad doc).
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
- "Oracle" cleanup (`b0e51b2`/`caefcbb`/`e4f983a`) renamed only the epistemic-overclaim sense
  (`runtime-oracle`→`runtime reference`; results are locked measurements, not a real-world-truth authority)
  across SPEC/Rust/registry/corpus/reference/IDs/config (`docs/` excluded). The TEST-ORACLE sense (deciding
  a test's pass/fail vs the reference) deliberately persists in `run_oracle.rs` + `rules.rs` (`// THE
  oracle`). Decision: NARROW — leave the test-sense; a global retirement (`run_oracle.rs`→
  `run_reference_check.rs`) is an OPEN user/style call.
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
  corpus (model-routes.md etc.) may name engines as landscape — out of scope. Fixtures/test
  values obey this too — use unmistakably-synthetic tokens (`model.baseline`/`fixture_quant`/`1.0.0`),
  since a realistic generic quant/format token still names a real scheme (M2.1 codex follow-up: a
  real bit-width token had slipped into a fixture whose comment asserted it named none; RECURRED M2.9 r3 — `q4` slipped into a model-adapter test fixture, the dialect-only de-leak grep missed it). Audit = word-boundary `git grep -niP` (names `\b`-bracketed) over EVERY forbidden category (engines + grammar dialects + quant/model-format tokens `q4`/`gguf`/…), not just one, and not a bare substring grep (false-matches a Cargo.lock dependency name). Catch the bare quant token with a case-SENSITIVE lowercase `\bq[2-8](_[0-9km])?\b` (drop the global `-i`, wrap the engine/dialect/format names in `(?i:...)`): it matches `q4`/`q4_0` yet skips the uppercase `Q1`/`Q2` SMT labels (a case-INSENSITIVE `q[2-8]` false-hits them — the prior skip-bare-`q[2-8]` rule left bare `q4` UNAUDITED; codex M2.17). The repo-wide grep OVER-matches by design (AGENTS.md: a filtered finding beats a dropped bug) → triage each hit vs the standing exempt/false-positive set: `docs/` landscape (out of scope), this bullet's own rule-doc (the `q4`/`gguf` examples), the route.fixture cassette (real identity, exempt), and the lowercase `q2`/`q3` SMT-pair variables in `verdict.rs` (M1 query indices, not quants). So the per-UNIT close gate = the unit's TOUCHED files carry no token (scope the grep to them); the milestone review runs the full-repo triage. Reconstruct the command from this bullet — no banked wip (consumed at .2b close). EXCEPTION (user decision, model-cassette.2): committed RECORDED cassettes (under `crates/ckc-cli/tests/fixtures/cassettes/` now, run-m2.2's experiment-cassette roots later) carry the runtime's REAL `model_identity` (model/quant/engine strings) — machine-specific MEASUREMENT data with honest provenance, NOT engine-neutral contract/fixture artifacts → EXEMPT from the synthetic-token rule + this audit. Audit FAIL-CLOSED: exclude only the SPECIFIC live-recorded path(s) whose cassettes carry the runtime's REAL `model_identity` (model/quant/engine strings) — today `route.fixture/` (its recorded `seed-42.json`): `git grep -niP … -- . ':(exclude)crates/ckc-cli/tests/fixtures/cassettes/route.fixture/'`, adding run-m2.2's experiment roots as recorded — NOT the whole `cassettes/` tree, so CRAFTED synthetic cassettes (route-single-ir golden + bad, SYNTHETIC identity) stay AUDITED + pass (a `cassettes/`-root exclude would wrongly free-pass them). Replay pins output/provenance/content-hash + the full recording envelope (producer + empty diagnostics/trace/runtime), never an identity VALUE BY NAME → the host runtime swaps with no test-code edit, but the identity rides the pinned content-hash (it changes only via a deliberate re-bless + re-pin).
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
    wiring is run-m2.1's.
  - Runtime-gate findings (the "gate MET" above, confirmed functionally on a real test source; concrete
    runtime/model identity → gitignored `.agent/runtime.local.md`; agnostic conclusions in `## Runtime`): constrained decoding forces
    schema-VALID output
    that can be semantically WRONG (observed: a greedy run emitted a wrong enum) → the M2 report scores
    BOTH acceptance-rate (schema-validity) AND verdict-accuracy, never validity alone. The baseline
    deliberately pins a weak sub-4B model whose free-form/direct-route output degenerates → exercises §9's
    "direct-route failures common" path (pin the exact model identity in the run config; alternatives ok).
    Greedy output is byte-stable within + across processes on one host/device/quant but NOT across
    environments → the recorded-bytes cassette (engine-agnostic boundary above), not a live re-run, is the
    correctness mechanism; replay needs no model runtime present. Two M2.9-respec refinements: (a)
    constrained output can be INCOMPLETE/INVALID (not just semantically wrong) when the constraint format
    permits unbounded whitespace + the model is weak (greedy loops on free whitespace, truncates at the
    token budget) → the acceptance-rate metric counts truncation/parse-incompleteness as a failure mode,
    and the tight-grammar route (explicit newlines, no free whitespace) sidesteps it. (b) greedy is
    SEED-INERT → the k per-sample seeds yield identical draws (convergence trivially 1.0); MEANINGFUL
    k-sample convergence (metrics-m2.2) needs a sampling config (temperature > 0, the seed fixing
    each draw) — a downstream config decision, NOT the adapter's (invoke_samples stays config-agnostic:
    derive seeds, invoke, record).

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
