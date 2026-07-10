# Agent Memory

Entries add value beyond spec / AGENTS.md / code / git / runtime env — project-independent tooling
pitfalls (RTK, Headroom, Serena, Claude Code, web) live in each agent's global guidance, not here.
Exception: high-value reminders derivable but easily forgotten under token pressure. Consolidated
aggressively; full pre-consolidation text in git history.

## Policy

- Context hygiene (user directive; bg `git show 531f586`): keep every session lean + phrased in project
  vocabulary (processing stages, units, gates, artifacts) — plain operational words over research jargon
  in memory/roadmap/commits/code. `docs/` (research compendium) is git-history-resident — SPEC §14
  holds the retrieval recipe (`git show e8b5cf6:docs/<file>`); consult via read-only subagents so its
  vocabulary stays out of the main window. Implement sessions match patterns from the latest
  unit-scoped commit (`git log --oneline`), not bare HEAD, when HEAD is hygiene/memory work.
- AI-written specs may carry mistakes (user, 2026-07-03): apparent incorrectness is likely unintended —
  verify against SPEC.md + code, rule with best judgment, record the ruling where its implementer reads it
  (first applied .1d5: findings body = single_ir structurally; "BASELINE only" was a phantom-collision fix
  — direct lands no compiled, mints no claims).
- LSP coverage map (ckc): Serena-served = rust, bash, json, yaml, toml, markdown (Marksman), html,
  lean4 (`.serena/project.yml` `languages:`; lean4's server starts once `.lean` files exist);
  `global`-marketplace plugins = xml, smt2 (dolmen), alloy, egglog. Audited gaps: TLA+, ASP/Clingo,
  categorical-CQL have no standalone LSP; Isabelle = marketplace gap plugin at adoption; Python
  solidlsp-covered (add at adoption). Registry-YAML-only compendium families carry no LSP.

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
  §-semantics + unit tests) + its recorded-run integration test = 2; canonical-emit layer over an
  existing type family (one module) + a byte-pinned record-shape extension consuming it (a second module) = 2, split at the module seam; a record-shape extension's PLUMBING (fields + assembly wiring + fixtures/byte-pins in the record module + a trivial None-stub at each cross-module construction site, no signature change) vs its cross-module COMPUTATION+THREADING (populate the fields from a gated source + thread a new param through the caller chain + a run-binary integration test, a second module) = 2 at the module seam EVEN WITH THE DESIGN LOCKED (run-m2.1e-B overflowed read+write with the full design locked in-session, ZERO code — a locked design removes REASONING but not the read-to-place-edits over a large caller + byte-pin-test authoring, which alone overflow; the plumbing half leaves the crate green fields-plumbed-but-None so omit-None keeps bytes byte-identical)
  — RECURSES: that COMPUTATION+THREADING+run-binary-test half (B2) overflowed TOO (wrote all the
  code, then an un-banked debug loop discovering the fixture gap tipped it at ~99%), so split ONCE
  MORE at the SAME seam — cross-module COMPUTATION+THREADING+FIXTURE (leaves the crate GREEN on
  existing tests, which exercise the new path → prove it computes without erroring, the new VALUES
  unasserted) vs VALUE-PIN-TEST authoring. FIXTURE-PROVISION COROLLARY: a fixture-replayed
  producer (`manifest_inputs`) that gains a registry-file READ needs its fixture BUILDER
  (`copy_committed_registry`) to copy that file too — an un-banked provision gap surfaces only at
  test-run time as a debug loop reading the builder + its callers + the whole write-fixture chain,
  and THAT is what overflowed B2; bank the provision (which builder, which files, callers-
  harmless) at respec time; record-shape
  extension + fresh-designed member type + validator + per-variant rejections vs its populated fixture +
  byte-pin capture = 2; derivation fn + its test-source-pinned battery + an attachment sub-feature = 2;
  type family + assembly + validation = 3;
  assembly fn + its live-pipeline pin battery = 2; a live-pin battery over the run binary is its OWN unit
  (never paired with assembly or stage wiring); orchestrator wiring over N pre-built route stages +
  per-stage landing/eventing + a determinism gate ≥ N+2 units — per-route stage-rework units first, the
  orchestrator+gate last, cross-cutting type/trace plumbing its own opener; the orchestrator+gate unit
  ITSELF splits at the loop/tails seam when its tails do cross-route work (dedup, per-route→node
  assembly) — the per-view LOOP (lands per-route artifacts, own landing gate) vs the UNIFIED TAILS-ONCE
  (run-level trace/report over all routes, own trace-parse gate); the loop's CALL SEQUENCE is bankable
  off the per-route *_scores tests but its cross-route LANDING is NOT — those tests each run ONE route
  into its own out, so they never exercise the shared-out collision (both routes write bare
  `groups/{gid}/verifier_results.json` → clash unless the group dir is route-namespaced like the heads).
  Banking a route-namespaced dir as "confirmed from the scores tests" hid it (Codex .1d5a caught it):
  a banked "CONFIRMED from test X" literal must be byte-diffed against X's actual literal — a divergent
  value is a DESIGN choice not a confirmation, and single-route tests never cover multi-route landing.
  Beyond banked VALUES, banked DESIGNS hide CORRECTNESS bugs codex catches even in a fully-LOCKED spec
  (.1e-B2, 2 blockers): a gate keyed on an `Option<T>`'s presence aliases two run-modes when T is
  legitimately absent (a failed model route ≡ M1's all-None) → gate on the MODE signal + fill the optional
  field honestly; a provenance/measurement hash must cover the run's ACTUAL inputs not the whole registry
  even when equal today, else later registry-growth silently rewrites an unrelated run's golden + breaks
  the SPEC per-run locked-measurement semantic → adversarial-verify a banked DESIGN against SPEC intent +
  reachability, not only its apply-anchors. Selecting those actual inputs via a one-directional
  `filter(want.contains(id))` is asymmetric — it drops unwanted registry entries but never checks every
  WANTED id resolved; a drifted hardcoded route→id map (typo/rename independent of the fill path) then
  silently locks `aggregate([])`'s empty-set hash into an attestation record under an `ok` run (.1e-B2a
  codex) → coverage-check want⊆found, fail loud naming the gap; a normally-unreachable non-model shape
  in the model-route set is a caller-contract Err, not a silent skip (would zero the want-set).
  The tails hold further cross-route uncertainty
  (source-node dedup vs route-prefixed ids, GroupTrace-from-route) → the read-cost that overflows a
  combined unit lives in the tails, so land the loop first (run-m2.1d5a respec: overflowed the combined
  unit at 51% on READING alone, zero code); a route-stage rework
  (landing+eventing rewiring of an existing fill fn + mechanical call-site updates) and its
  event/landing PIN battery = 2 — behavior lands one unit, observed-output pins the next (and an
  error-path pin battery testing a PRIOR unit's ALREADY-landed branches is independent of the current
  unit's new wiring → its OWN unit, not folded in: run-m2.1d5a-2 split unified-tails wiring from the
  partial-group/mixed-shape/identity-disagreement tests pinning .1d5a-1's branches); spec-byte
  amendment (re-pin + reference/test mirror sweep) + new feature code = 2 (an open decision that amends
  pinned bytes is a deliverable, not a preamble); a prompt-TEMPLATE refinement must enumerate the supply
  mechanism for every input the template promises at plan time — a template promising
  instrument-supplied inputs (ids/vocabulary) the composer never composes hides a composer redesign, and
  scaffold-completion / live-record / pin-battery = 3 units (run-m2.2 respec); crate foundations pair only with a small type surface (one payload module each); deterministic code + a
  SLOW/exploratory live confirm over an external runtime = 2 (code stub-gated + mechanical; the live
  confirm its own unit) → apply to EVERY live-runtime-gated unit at plan time, not only the obviously-slow,
  and on recovery discharge the one-time exploration into memory `## Runtime` + persist any
  session-scratchpad tool the live unit needs to a stable machine-local path (on PATH for a bare-name
  command) so the redo is a checklist. MEASURED ANCHORS (checked stubs carry `NN%`): canonical JSON = 5;
  five-layer recursive type family = 3; lexicon-driven derivation half (loader/binding/builder) = 3;
  statement builder over a prebuilt binding core = 1; exception attachment + determinism tests = 1.
  PRACTICES: house new type families in fresh modules (extending a ~2K-line module costs a full-file
  read); on a big file gather EVERY region the session's edits touch BEFORE the first edit — post-edit
  reads re-orient against shifted lines and can return stale; scope each split's Reading slice to exclude
  files its half leaves untouched; land a compiling skeleton before the full test battery — `cargo check`
  after the production edits, an end-loaded uncompiled battery leaves nothing landable; pin expected
  shapes from observed output, never hand-computed; spec code references = fn/test NAMES, ≈line =
  secondary hint only (drifts under edits above it). At plan/re-scope time audit any spec a unit must
  byte-reproduce — readability listings (alignment padding, inline result comments, illustrative
  declaration/conjunct order) contradict deterministic-emission rules and need a scheduled re-pin
  deliverable (smt-emit.3a: §8.6 smt2 vs §6 sorted-declaration). SALVAGE RETIRED (user directive,
  2026-07-02): banking applyable wip artifacts (`.agent/wip-*`
  patches / byte-exact code copies / transcription blueprints a redo line points at) cheats the unit — the
  redo's recorded context-usage measures artifact application, not the unit as
  specced. Overflow recovery is LAND-OR-REVERT: either the proven half closes
  as its OWN completed unit (own gate, own honest usage figure, artifacts committed at their final paths)
  within the session's remaining margin, or the tree reverts CLEAN and the recovery respec-splits into
  fresh SELF-CONTAINED units. A respec line may resolve decisions, confirmed facts, and reading pointers
  in prose (that is planning); its banked content is prose only — the redo session itself writes every
  line of implementation code. Retired wip artifacts remain in git history as provenance only. Any wip
  scratch file a session does create gets deleted before that session's closing commit. RESPEC-SESSION
  CLOSE (run-m2.1 respec 3b1066a): a respec whose seam confirmation reads span multiple modules has
  already spent the implementation margin → commit the respec, re-score the first half against the window
  REMAINING, and
  implement only on a clear fit; the default close is the respec commit itself (the session-prompt
  clause mirrors this). A banked respec line pre-pays the next session's derivation ONLY if it carries
  the confirmed facts (caller counts, helper signatures, fixture slots, exact reasons) — bank those at
  respec time while they are in-window, AND cap the READ list to the minimal COMPLETE apply-anchor set —
  EVERY edit site listed, the enumerated SOURCES (the mirror fn, the type modules) EXCLUDED: a respec that
  ENUMERATES shapes (event/destructure fields, signatures) must forbid re-reading those sources, else the
  implementer re-incurs the very derivation-read the respec prepaid; but the set must still name every EDIT
  target, or an unlisted-but-required edit silently drops (esp. one no test pins). run-m2.1d4a overflowed
  its first implement attempt DESPITE a fully-pinned respec — its READ-FIRST relisted the mirror + shape
  modules whose every field the respec already enumerated → reverted, re-scoped to the edit set: the
  replace span, the adjacent verify-tail edits, and the call-site regions incl. their docs (sources out).
- Read-cost is a unit-sizing axis distinct from deliverable count (route-single-ir.2 overflowed a 200K
  window during READING, ZERO code written → nothing to salvage). A 'one deliverable + one gate' unit
  still overflows when its test/bless/fixture scaffolding needs byte-exact shapes — signatures,
  sorted-field orders, enum variants, harness helpers, `Resolved`-style stamp structs — assembled across
  many modules; a deterministic-REPRODUCTION gate reads the WHOLE upstream type + helper set. Detect at
  PLAN time: count the modules a unit's gate/bless scaffolding must read for exact shapes, not just its
  conceptual pieces. Nothing-written overflow recovers FORWARD: (a) SPLIT the production fn from its
  golden-fixture + gate when separable (route-single-ir.2 = accept closure; .2b = fill+bless+gate);
  (b) pre-resolve the blocking FACTS — confirmed signatures, verified equality premises (e.g. clinical_ir
  diagnostics empty for the 3 docs), insertion anchors — into the respec'd roadmap LINE as prose
  (facts/decisions = planning; verbatim code or a pointed-at wip artifact = retired salvage); a fact set
  too large for a line ⇒ still oversized, split further. A self-checking gate (`content_hash ==
  reference`) bounds reproduction-error risk on the PAYLOAD path ONLY: a content-hash-affecting line fails
  loudly; off-payload lines don't (wrong signature → compile error; producer/wrapper/input_hash fields
  compile AND pass silently → still targeted-read those). Mark gate-IRRELEVANT fields (producer stamps /
  step-ids / wrapper-level fields under a payload-only `content_hash`) explicit so the session skips
  pinning them.
- Renaming canonical (§4.3) JSON member keys = silent test-breaker. The object emitter buffers members
  then sorts by key bytes on `finish`; the reader (`canon.rs` `member`/`optional`) is positional — peeks
  the next key, demands keys requested in ascending byte order. So a rename moves the sort slot: code
  still compiles, but round-trip reads fail `MissingField` at runtime + pinned byte-string literals
  mismatch. Fix = re-sort each Canonical read+emit member sequence AND every pinned byte-string to the new
  key order (`printf '%s\n' k1 k2 … | LC_ALL=C sort`). Related: a `#[serde(rename_all="snake_case")]` enum
  serializes by variant name → a snake wire-key rename must also rename the CamelCase variant (e.g.
  ViewText→RenderedText) — caught by name-pin asserts, never the compiler. Hyphenated scope-IDs
  (`stage-extract.1`, `core-grounding`, `fixtures-m1`) in roadmap+comments = git-commit-traceability keys:
  keep them historical on a terminology rename (rename only dotted runtime IDs `processing_stage.m1.*` +
  living prose).
- Backward-compatible canonical-record extension (proven M2.1 model-types, inverse of the rename break):
  add fields to a byte-pinned §4.3 record without disturbing pins = make them `Option<T>`, emit
  `obj.optional(name, self.f.as_ref(), |b,v| v.emit_canonical(b))`, read `obj.optional(name, T::read)?`,
  each in the field's sorted-key slot. Omit-None emits nothing → prior pins stay byte-identical (the M1
  unchanged-expected-bytes pin tests = the regression guard; their literals stay frozen). Emitter sorts on
  `finish` (emit-call order cosmetic) but the positional reader REQUIRES the `obj.optional` call in
  ascending-key position (peek next key: `<name`→UnknownField, `==`→consume, `>name`/absent→None) → a
  misplaced optional misreads. Pin BOTH an all-None fixture (locks old bytes) AND a fully-populated one
  (locks new members' slots) — once per extended record type, not one exemplar per family (a populated
  round-trip proves read/write inverse but only a byte-pin locks canonical order/content → each record,
  RunManifest AND ReplayManifest, needs its own populated pin; codex caught the missing replay pin). Pin
  the all-None case in the RECORD module's OWN test too — a downstream run/integration byte-pin is NOT a
  substitute (doesn't self-guard the assembly seam's omit-None; B1 shipped only the populated pin → codex
  re-caught). Populated-fixture seeds = GLOBALLY unique across the WHOLE fixture, not merely distinct
  within the new field group: a new field mis-wired to an existing same-seeded field emits identical
  bytes → slips the pin (B1 §9 seeds a–f collided toolchain/lockfile/corpus/lexicon → fixed to unused
  0,3,4,5,6,8).
  `content_hash` = the generic `content_hash<T: Canonical>` free fn → every Canonical type gets it with
  zero per-type code (a roadmap "content_hash for the new types" clause needs no impl).
- Behavior-locked extraction past a timed interval (M2.7 run-refactor): a `ProcessingStageClock` opens in
  the CALLER before the extracted call → pure setup (`format!`/alloc) left in the callee body runs INSIDE
  the timed interval, falsifying a timing-identity claim byte pins CAN'T catch (`duration_ms` normalized →
  tests stay green while the guarantee breaks). Audit clock boundaries when extracting: hoist pre-clock
  setup to the caller, pass it in — `compile_verify_group` takes `dir: &str` so its
  `format!("groups/{gid}")` stays outside COMPILE timing (route.single_ir supplies its own dir + clock
  likewise). Call-boundary overhead is inherent + below ms/normalization resolution — only named setup is
  worth hoisting.
- Doc-lint gate (Rust): the per-unit test+fmt+clippy gate MISSES rustdoc → run `RUSTDOCFLAGS='-D
  warnings' cargo doc --workspace --no-deps` whenever a unit touches doc comments; baseline = 0
  errors (the M2-review fix cleared the standing 17+17 debt). Fix shapes: a public item's doc link
  to a PRIVATE item (`private_intra_doc_links`) → plain ticks; a link to a type not `use`d in the
  module (unresolved) → qualified-path `` [`T`](crate::T) `` (a docs-only `use` trips
  `unused_imports`); a doc comment INSIDE an exported macro body cannot assume any link resolves at
  its expansion sites → plain ticks only (`fieldless_enum!`'s `ValidationError::Enum`).
- Contract-tense docs (codex flagged twice): a doc claim about pending wiring must be unit-attributed —
  "report-m2.1b embeds X in `report.json`" holds before + after the unit lands; present-state phrasing
  ("carriers today: report.json bytes agree") overreaches until the wiring commit. House pattern:
  "run-m2.1 wires the observations". Apply at write time — each violation costs a codex follow-up commit.
- Selector-semantics fields need discriminating fixtures: a contract picking ONE candidate among several
  (`model_identity` = LAST attempt's cassette; `accepted_cassette_hash` = ACCEPTED attempt's, never the
  base recording) is pinned only when the fixture makes candidates DIFFER — uniform fixtures satisfy the
  assert under an any-candidate regression. Pattern: `later_identity()` seeded at the last attempt via
  `seed_cassette_as` (model_fill tests); the hash side already discriminated (recovery pins derived-seed
  cassette cited + base NOT). Apply at test-write time whenever a field's doc says "the last/accepted/
  first X". Design-side corollary: content-hash selection collapses where reproduction pins make byte-equal
  candidates the NORM (route bundle hash == M1's) → select by identity ids, keep the hash as a conjunct
  check (`GroupTrace.member_bundles` ∧ `input_hashes`); fixture axis = candidates differing in id while
  EQUAL in content.
- Model-runtime adapter (§9, `ckc-cli/src/model.rs`, mirrors `ckc-smt` Z3Adapter; DONE .1/.2a/.2b).
  Live facts beyond code/git: `pub mod model` — a pre-consumer skeleton must be pub or clippy `--lib
  -D warnings` flags dead_code (no-cfg-test lib build; recurs for cassette/route fns). MIRRORS not
  reuses Z3's subprocess machinery — the shared cross-crate runner is SCHEDULED (M3
  subproc-runner.1 behavior-locked extraction + .2 hardening; the mirrors stay duplicated until it
  lands): .2 absorbs the codex-REJECTED `Instant+budget` overflow-panic fix (rejected Z3-mirrored,
  non-realistic, fix-both-not-one) AND the cap/reap of the
  STILL-unbounded post-grace detached drain (a descendant holding stdout open appends to its Vec forever;
  accepted meanwhile for the local trusted runtime, no-unsafe/no-extra-dep); the ETXTBSY
  vacuous-window half is discharged earlier by M3 spawn-retry (fs-dependent tests → injectable
  deterministic retry tests). `Completed{bytes}`
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
  test mirroring `tests/model_live.rs`, content-hash-pinned. LANDED run-m2.1f1+f2: the `--record`
  surface = f1 pure selectors/composers + f2 dispatch flag (`take_bool_flag` pre-partitions `--record`
  out of `rest` BEFORE `take_flags`)→validate→execute→execute_routes threading; `execute_routes` builds
  `RecordSetup` (adapter probe + `validate_model_registry`) ONCE `if record` else `None`, then per-view
  `build_route_record` (f1 select→template/constraint/`RecordContext`) feeds both fills' new
  `Option<&RouteRecord>` → `FillSource::Record` (type-enforced; Record arm live-capable via `CassetteStore::record`→`adapter.invoke`, not live-exercised in f2); §9 manifest fields landed
  e-B. Pre-write BYTE-verify LANDED 2a — `build_record_parts` hashes the SELECTED template/schema bytes
  against the registry-declared `template_hash`/`schema_hash` before any record (the record path no
  longer trusts declared hashes; `registry check` remains the replay-path verifier). Both deferrals
  CLOSED: live record exercise → 2b, replay.rs model-artifact coverage → 2c
  (`tests/recorded_run.rs` `replay::execute` → `matched()` over the recorded experiment run). f2 RULING: the respec's test-(6a) `set_var(CKC_MODEL_COMMAND, bogus)` no-probe assert is
  void under `#![forbid(unsafe_code)]` (set_var forbidden, above) → the no-probe property is STRUCTURAL
  (record_setup built only `if record`, else `None` → a replay run never constructs `ModelAdapter`),
  proven by threading `false` through the green `m2_route_loop_lands_both_routes_namespaced`; the flag
  itself is pinned by a direct-`parse()` `run_record_flag_parses` (no-flag→false / bare→true, flags
  intact / dup→reject / `--record=x`→reject). RULE: a respec step prescribing `set_var` for
  env-dependent test setup is unimplementable here → substitute the structural guard + a green
  regression + a direct parse-level unit test.
- Model-fill stage core (§7.4/§9, stage-model-fill.1 core + .2 repair/grounding,
  `ckc-cli/src/model_fill.rs`). DECOUPLED core
  `model_fill<T>(store, key, source: FillSource, repair_limit, accept) -> Result<ModelFill<T>, CassetteError>` →
  `ModelFill<T>{target: Option<T>, accepted_cassette_hash: Option<Hash> (accepted attempt's cassette
  wrapper content_hash, Some iff target — run-m2.1c), model_identity: Option<ModelIdentity> (last
  attempt's, always Some on Ok — .1d checks cross-route identity agreement against it), diagnostics,
  recorded_calls, repairs}` — a plain value, NOT a §4.6
  event/`ArtifactWrapper`. `FillSource::Replay` (default, runtime-absent) / `Record{adapter,prompt,
  constraint,ctx}` (gated) gets each attempt's cassette via `CassetteStore`, decodes `output_bytes()`, runs
  the route's `accept: impl Fn(&[u8])->Result<T, FillReject>` = the §4 acceptance check (route supplies the
  ClinicalIR/SMT parser+grounding; target + acceptance stay route-side). The `FillReject` variant picks the
  §7.4 code AND repair-vs-terminal: `Schema(reason)` → `ai_schema_violation` and RE-PROMPTS under
  `derive_seed(base, attempt)` (each attempt its own derived-seed cassette) up to `repair_limit`, then
  terminal `repair_limit_exceeded`; `Grounding(absent)` → terminal `ai_hallucinated_source`, spends NO
  repair. The stage ASSERTS the closure's `Grounding` carries ≥1 absent id (empty = a deterministic route
  bug → fail-closed panic, house `expect`/`unreachable` style, not a silent empty-`absent_source_ids`
  diagnostic); route-single-ir still enforces route-side too (defense-in-depth). A
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
  its oracle is the format's own recognizer (working `bnf` Earley form + its two API pitfalls live in
  `emit.rs` — copy it; deriving fresh from `bnf` docs re-hits them) → skip bless + cross-check; the
  lone `hash_bytes(file) == <X>_HASH` pin IS the whole drift guard. DESIGN LESSON
  (any grammar/schema oracle): oracle = SOUND SUPERSET of the emitter image, NOT its exact shape — a
  CFG can't bind cross-field coupling / assertion cardinality / declare-before-use, so §8.6 byte pins
  own those; keep the grammar the construct-surface union (grammar-constrained decoding wants the
  union), cover every production incl. the empty-context→`true` collapse, and prove full-match ONLY via
  a trailing-garbage case. Byte-pinned text file → `.gitattributes eol=lf` (sha256 + literal-LF `<nl>`
  survive checkout).
- Schema↔canonical coupling (maintenance): the oracle validates `canonical_payload_bytes(ir)` parsed as
  JSON against the emitted schema, so any §4.3 canonical-encoding change (key rename, integer formatting,
  union shape, a new field) silently breaks good-instance validation unless `schema.rs` tracks it —
  `schema_accepts_canonical_clinical_ir` is that guard (later-milestone ClinicalStatement additions — M3 CNL bridge fields, M4 distinguishing/temporal — must extend both).
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
  outside the tree passes and `std::fs::read` follows it — and the 2026-07-10 external review
  found corpus.path + expected_outcomes lack even the LEXICAL check (absolute /tmp paths pass
  `registry check` AND a full run) → fix SCHEDULED as M3 unit path-confine: lexical findings in
  core + ONE canonicalize/containment I/O resolver across every registry-data-controlled read. Core fixtures (SCHEMAS
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
  RegistryFinding variants need ZERO `registry_check.rs` change. run-m2.1a landed two-route
  resolution: `resolve()` → `Option<Vec<Resolved>>`, one view per `resolved_pipelines()` member in set
  order, each fingerprinted by `resolve_route` (declared kind sequence + model_fill
  `output_artifact_kinds` → `RouteShape::{M1Layered,SingleIr,DirectSmt}`, else "unsupported
  processing-stage sequence" naming the kinds; undefined stage / undefined pipeline / malformed binding
  each carry their own reason — .1b pins the rejection battery); ONE shared plan carries the full set
  (M1 = `[baseline]`, plan bytes unchanged); `execute()` runs exactly one M1Layered view, any
  model-route set → one command diagnostic + zero artifacts (run-m2.1d wires the loop). Set-form
  `exp.m2_multihop` SEEDED (registry check green). SPEC: §8.4
  stays M1-singular (faithful history); the M2 generalization went into §14's registry-evolution ledger
  (no §14 byte-pin → free prose).
- Test/example producer IDs: `pipe.<qual>` (`pipeline_id`) + `processing_stage.<qual>.<step>` (`pipeline_step_id`); shared `<qual>` links a pipeline to its steps. Generic unit fixtures use `qual=test`; scenario fixtures keep their own (`m1`/`t`/`base`). `cand.*`/`comp.*` stay retired — they echo the pre-rename `candidate`/`component` field names the terminology cleanup removed.
- Component vs pipeline-step terminology: reserved now in identifiers AND comments (`b6e1177` + follow-up sweep) — `component` = the §5 IR `ComponentRecord`/`DocIR`/structural concept only; a registry `processing_stage` entry = a pipeline step. OPEN + deliberate (not a missed rename): SPEC §8.4 prose + `registry/candidates.yaml` still read "processing stage component(s)"; resolving it = a SPEC-level vocabulary call (route through the user), so skip auto-"fixing" it on a grep sweep.
- "Oracle" naming: the epistemic-overclaim sense was renamed `runtime-oracle`→`runtime reference`
  (results = locked measurements, not a real-world-truth authority); the TEST-ORACLE sense (pass/fail
  vs the reference) deliberately PERSISTS in `run_oracle.rs` + `rules.rs`. A global retirement
  (`run_oracle.rs`→`run_reference_check.rs`) is an OPEN user/style call → sweeps leave it as-is.
- ckc-smt's `serde` dep reads as unused (no `serde::`/`Serialize`/`Deserialize` in ckc-smt/src
  beyond the `fieldless_enum!` invocations) but is REQUIRED: that ckc-core macro expands to
  `::serde::Serialize`/`Deserialize` impls *in the caller's crate*, so every fieldless_enum! user
  must depend on serde — dropping it breaks the build (`E0433` unresolved `::serde`). Holds for any
  crate adopting the macro. Those serde impls go unused there (the canonical path is
  Canonical/CanonRead), an accepted KISS cost of one shared macro over per-call serde gating; the dep stays.
- M1 reviewed (git/roadmap hold the detail). §4.4-vs-§8.3 tension RESOLVED by SPEC amendment: a
  processing stage's total operation result IS its §4.6 EventRecord (§8.3 has no
  per-stage total artifact); only commands materialize a standalone TotalOperationResult (value/
  residual/ambiguity/incoherence buckets stay empty until typed placeholders exist). GUARDRAIL: per-stage totals stay EventRecords
  alone — a standalone TotalOperationResult there is inert + redundant until then (M2+ may
  revisit). Enhancement (AGENTS.md-preferred; since scheduled into M3: canon-props): tests are example/byte-pin only →
  property-based/fuzzing for the canon layer (round-trip identity, reject-any-mutation) + StringPolicy
  idempotence.
- M2 reviewed (plan 2a4f03d .. accept/m2 b2e010b, 201 commits; fixes in 5ec33f7). Durable: the six
  §9 theme verdicts rest on acceptance-m2's LOCAL driver run (evidence-runs-local design,
  independently codex-re-verified). OPEN user items: SPEC §8.4
  "processing stage component(s)" prose + candidates.yaml wording (SPEC-level vocabulary call);
  `run_oracle.rs` test-oracle naming; property-based/fuzzing for the canon layer (M1 review
  enhancement, since scheduled into M3: canon-props); shared cross-crate subprocess runner + registry symlink guard
  (both since scheduled into M3: subproc-runner.1/.2, path-confine).
- CNL-first architecture (user directive 2026-07-07, set in the Codex-continued cnl-ir-research
  session — rollout `~/../debian/.codex/sessions/2026/07/07/rollout-2026-07-07T11-09-50-*.jsonl`;
  SPEC amended same day = design authority, read SPEC not this bullet for semantics): clinician-
  auditable CNL = the product's knowledge surface; probabilistic step confined to source→emission
  surface; accepted+reviewed CNL doc = locked KB; audit views render from accepted IR only, any
  route. MILESTONE RENUMBER M3↔M4: §10 = M3 ClinicalCNL v1 (full design, elaboration-depth —
  next session plans units from it), §11 = M4 route field + comparison (absorbs old-§10 content +
  invented ablations + §6 LP profile lane); every old-M3 feature (M4 conflict kinds, FactualRule,
  distinguishing fields, temporal atoms, component store, coverage, methods.yaml) renumbered M4 →
  trust SPEC's current numbering over pre-2026-07-07 prose/git text. Research corpus:
  `docs/cnl-{attempto,multilingual-ja,landscape,design-codex}.md` (Codex design = second opinion;
  its dual-surface split — compact DSL emission + generation-only prose — was REJECTED for the
  single-surface CNL bet, kept as §11 ablations). Name SETTLED (user, 2026-07-07): **ClinicalCNL**
  (mirrors ClinicalIR); id forms follow the ClinicalIR↔clinical_ir precedent — grammar files
  `schemas/clinical_cnl_{ja,en}.grammar`, registry schema id `clinical_cnl`; `route.single_cnl`
  stays (parallel to single_ir). GF adoption deferred until JA parse of
  non-CKC text or >2 languages (docs/cnl-multilingual-ja.md §5 verdict).
- M3 plan (ClinicalCNL v1; gate MET at planning — runtime identity probe clean, contract-
  conformant). Durable decisions beyond the roadmap lines (which collapse at M3 review):
  module home = ckc-cli FRESH modules (lexicon.rs — lexicon-extract's behavior-locked move
  of the Lexicon family out of normalize.rs, the CNL modules' neutral dependency point —
  plus cnl.rs/cnl_grammar.rs/cnl_parse.rs/cnl_render.rs/cnl_bridge.rs) —
  Canonical-outside-core proven by report.rs; ckc-core IR shapes + committed clinical_ir.schema.json untouched (ClinicalStatement
  already carries certainty/exceptions/source refs; sole core touch = DiagnosticCode
  fieldless_enum append). CNL AST = own family, NOT ClinicalIr (CnlAtom
  Concept|ConceptNegated|Interval|Unregistered — escape is a variant, and via CnlConceptRef
  {Registered|Unregistered} it reaches EVERY concept slot incl. action target, §10; CnlContext
  flat two-level DNF; CnlDocument payload per §5 = document_id + grammar id/hash refs + rules
  (AST + per-rule canonical text ja/en) + text hashes — accept re-renders + hash-locks
  canonical bytes beside the AST, report.json cites those hashes); parser mints NO ids —
  bridge derives them (ids `stmt.<k>`/`exc.<k>`/`bind.<k>`, document-order counters
  mirroring normalize.rs's mints EXACTLY — §8.6 reserves `<doc>.rule.<k>` for norm-layer
  rule ids, rules.rs's mint; one ClinicalStatement per
  context-disjunct; population partition by `pop.*` id namespace; exception disjunct → one
  ExceptionClause; one Exact TerminologyBinding per referenced concept, system =
  lexicon.system, region_ids = citing rule's basis regions; Action::new derives key; basis = region ids,
  source_segment_ids derived region→segment via the SEGMENTS artifact — m3.bridge stage
  inputs [cnl_document, segments]; round-trip laws: from_ir = single-disjunct projection ⇒
  from_ir∘to_ir == disjunct-split normal form, to_ir∘from_ir == id on bridge-image IR).
  Registry schema id = singular schema.clinical_cnl (JA grammar = the decoding constraint;
  EN grammar committed + drift-guarded, no route binding). run.rs's positional stage
  plumbing (pipeline_step_ids [Id; 8], MODEL_FILL/DIRECT_VERIFY/COMPILE=4/VERIFY=5/TRACE/
  REPORT index consts, UNUSED_STAGE padding) is retired by route-stage-handles BEFORE route
  wiring — per-shape named StageHandle {kind, step_id} fields validated at resolve — so the
  7-stage pipe adds a RouteStages::SingleCnl variant, never an [Id; 9] widening (off-by-one +
  provenance risk; 2nd 2026-07-10 review). Lexicon EN surfaces normalize under StringPolicy::SemanticEn (ASCII-lowercase —
  §10 EN canon), JA under SemanticJa. Grammar terminals = whole-surface string literals
  (ASCII-digit + basis-id-char alternation) + EXACTLY ONE open lexical production per
  language — the escape's free quoted surface, inexpressible as finite literals; emitter
  escape mode Committed(open) vs OracleBound(enumerated) since bnf parses literals only —
  LLM-constraint-portable + bnf-atomic; bnf 0.6
  verified unicode-capable (byte-offset whole-terminal matching); its Earley oracle proves
  language MEMBERSHIP (superset — explores all segmentations) → lexer segmentation determinism
  guarded by the lexicon same-category proper-prefix lint instead; single-parse asserts =
  `parse_input().take(2)`, never full counts (ambiguous blowup). FillReject grows Parse
  (repairable → cnl_parse_error) / Unregistered (terminal → cnl_unregistered_concept, payload
  = lexicon-entry proposal) / Instrument (terminal fail-closed → cnl_round_trip_mismatch,
  spends no repair). Record strategy: scratch-root record, copy route.single_cnl/** into
  committed /cassettes (keys disjoint from M2's); identity drift ⇒ full re-record + M2
  recorded_run re-bless fallback. Deliberate re-bless costs scheduled in units: ja_core.yaml
  growth → lexicon_hash value pins (lexicon-cnl-data); report CNL population → M1/M2 report +
  rendered-body pins (report-cnl.2/.3). Surface-quality metric rows gate on observations
  carrying the new FillObservation fields → M2 replay rows byte-unchanged (metrics-cnl proves).
  Archive-mined additions (user-directed 1M dig, 2026-07-07; measured priors distilled into
  docs/poc-archive.md — SPEC §10/§11/§7.3 cite it, sessions read the SPEC text not the doc):
  §7.3 gains the translation-faithfulness family + M3 unit metrics-faithful
  (ir_faithfulness_rate = accepted IR content-hash == in-run deterministic reference
  recompute; verdict metrics saturate + round-trip certifies only the surface, so an
  accepted-but-mistranslated doc is otherwise invisible until M4); record-cnl.1 probes grammar
  repetition points for degeneration loops (archived prior: verbose grammar-masked forms loop
  + truncate); §11 seeded with route.reason_ir (free-reasoning stage → constrained commit —
  the constraint-placement axis; sole archived form to beat single-IR faithfulness) +
  indirect-rendering test-source variants (semantic indirection dents faithfulness where
  surface-metamorphic variants leave verdicts intact). Census lens for record-cnl.3/
  acceptance-m3: the archived conflict-killer signature = stable same-direction misses on
  conflict groups (well-formed wrong deontic token) — check the M3 recorded census against it
  before attributing verdict deltas to the CNL surface. Archived lineages live IN-REPO
  (history refs, zero working-tree presence): branch archive/poc-m2-3-4 + tag
  accept/m2-3-4-poc (the PoC), branches archive/spec01..03 (pre-restart spec lineages, mined
  empty for CNL/DSL), and at `e8b5cf6` docs/charters/ (genesis prompt + its three charter
  executions) + docs/poc-archive/*.json (M5 oblique canonical reports, the never-doc-synced
  evidence) — scratch copies outside the repo are dispensable.
  2026-07-10 external pre-M3 review absorbed, then a same-day codex follow-up corrected it —
  §10 + the unit lines hold the normative text; memory keeps the evidence behind the calls:
  bridge ids MUST mint stmt/exc/bind counters mirroring normalize.rs (§8.6's <doc>.rule.<k> =
  norm-layer rule ids only); faithfulness = §10 projection excluding binding region_ids
  because CNL carries rule-basis, never mention-region, provenance (M1 binds adult→r.2 only
  while guideline A's statement basis = r.2 r.3 — exact-hash ir_match would zero on its own
  goldens); modality CNL tails ≠ source-match surfaces (を強く推奨する = particle + strength
  adverb; the worked contraindication tail 禁忌である is the SECOND ja_core row for its pair,
  so canonical selection needed explicit tail_ja/tail_en fields over first-listed file order);
  run.rs accept battery pins EMPTY ClinicalIr = accepted, schema IntervalBound admits
  negatives, IrBundle::validate admits two-sided intervals → accept-total closes every
  CNL-inexpressible shape, census-flip-gated; §10 laws quantify over the parser-accepted
  language (runtime grammar = repairable superset — the open escape production admits payloads
  the parser rejects); audit views need (pipeline, document) keying because exp.m3_cnl accepts
  the same document on two routes. ETXTBSY fact behind spawn-retry: BOTH outcomes observed
  across filesystems — the spawn_piped ETXTBSY tests pass where the fs yields ETXTBSY and
  fail where it doesn't (overlayfs among the latter) — fs-dependence is the defect, the
  retry impl is not. A second same-day external review (2026-07-10, validation pass on fadc674)
  reproduced the input multi-read attestation defect → input-snapshot.1–.3 +
  route-stage-handles + lexicon-extract inserted (roadmap holds the specs). A third
  (validation pass on c30f295): spawn-retry moved first (fs-dependent red tests must not
  gate the earlier units' full-suite conditions); constraint-snapshot inserted after
  input-snapshot.2 — the child reopens the constraint pathname, transient A→B→A defeats
  any reread → pass a frozen per-run staged copy (committed constraints verified
  single-file self-contained: sole $ref fragment-only, grammars literal-only);
  lexicon-cnl.1 split → lexicon-cnl-shape/-data/-integrity, data BEFORE integrity (the
  hard-errors bind the committed lexicon on landing); canon-props scheduled after cnl-laws
  (roadmap holds the specs).
- §4.6 event IS the stage's total result (above) → a stage that LANDS artifacts inside a loop must emit
  its one event on EVERY path once anything has landed; an infra-error EARLY-RETURN (copied from a
  single-artifact fill's event-less `CassetteError` abort — safe there, it lands nothing pre-event)
  orphans the already-landed artifact + drops its counters. Event-less abort is safe ONLY before the
  first land; after, ride the event like the wrap/land-error break paths (`direct_smt_fill` deontic
  cassette-fails-after-overlap-lands; codex .1d4a).
- M2 route loop (run-m2.1d5a-1, code+tests hold the detail): `execute()` dispatches 3-way (lone
  M1Layered inline / mixed set fail-closed / all-model → `execute_routes`); both routes land
  route-namespaced under `routes/{pid}/{artifacts,groups}/` (`route_group_dir` centralizes the dir).
  Partial-group asymmetry is principled: single_ir emits the COMPILE partial-group diagnostic+event
  on a member-short group (module partial-group rule); direct keeps a bare skip — it mints no
  compiled artifact and the short member's own head/fill stage already diagnosed upstream.
- M2 run-level tails (run-m2.1d5a-2/.1e, code holds the mechanics: emit_event gating, all_graphs
  seen-set dedup, all_docs bundle-first sort). Durable beyond code: trace DETERMINISM rests on
  `assemble_trace`'s `sort_canonical`, so doc/graph merge order is moot for trace bytes — the
  bundle-first sort matters only for the pre-sort first-by-id lineage lookup;
  `producer.pipeline_step_id` is write-only provenance (never read back for logic). RECURRED rule:
  verify-against-code must grep the WHOLE crate test surface (`sorted_entries`/`files_under`/
  `expected_files`/`read_dir` over tests/ AND src), not just the edited module. Additive-safety
  split: `shell.write_under` output (report_en/ja.md) is lineage/manifest-UNtracked (manifest
  derives from the report wrapper, not a dir scan) → byte-safe for report/manifest/lineage pins,
  perturbs only exhaustive file-SET listings; `land`/`wrapper`/`land_record` DO track. Reusable
  fixture recipe: divergent-identity re-bless = `store.replay(key).payload` clone + swap
  `model_identity` + `build_wrapper`/`persist`. A fully-bundle-less doc never breaks the tails
  (`assemble_trace` Option-guards bundle nodes; a member-short group `continue`s before its
  GroupTrace push).
- Event-scope model (corrected twice — pin it): command-scope diagnostics (`shell.diagnostic`) ride
  the CLOSING command event = `events.last()`, NOT "no event"; stage-scope diagnostics ride their
  processing_stage event; BOTH land in `logs/diagnostics.jsonl` (append order) = `executed()`'s
  diagnostics vec. Fail-closed pinning patterns: identity-disagreement pins the ABSENT run-level
  tail set (`read_dir(out)==[logs,routes]`); member-short pins tails-COMPLETE (trace_bundle.json +
  report.json exist) + exactly-one-Invalid-compile-event.
- Replay coverage ≠ trace-DAG membership (run-m2.1e-A): replay.rs diffs manifest `output_hashes`;
  the direct pair hashes live ONLY in the manifest (no smt_query `TraceNodeKind`), DAG-orphaned
  unless `verifier_results.input_hashes` provenance-links them. Both routes push GroupTrace on
  artifact LANDING (direct: `DirectFill.smt_queries` = a `[overlap, deontic]` prefix), not
  verify-success — only GroupObservation gates on `results`. A content_hash-EXCLUDED field
  (producer step ids) needs an EXPLICIT regression pin — hash/layout/census/determinism pins all
  pass through a producer regression. The run-level tail producer is SYNTHETIC non-registry:
  baseline `pipeline_id` + `processing_stage.run.{trace,report}` step ids, documented not registered.
- Report-vs-manifest degraded-route asymmetry (run-m2.1e-C1; M2-review RULING: keep by design). A
  model route that attests no agreed identity yields a section-less report (`ModelRunSections`
  requires a non-optional identity — a §7.x view declines to attribute results to an unknown
  evaluator) while the §9 manifest still records the run's setup hashes with `model_identity`=None
  (gates only on `model_routes` non-empty); the ledger + §4.4 outcome still document the
  degradation. `model_route_metrics` gates on `agreed.is_some()` so a degraded route skips the
  reference parse. The re-read TOCTOU (`model_route_metrics` + `manifest_inputs`
  independently re-read experiments+reference; resolve/corpus/record-setup reads
  reopen paths too) was LOW-SEV-deferred as benign on static committed files — the 2nd
  2026-07-10 external review reproduced the attestation flip (corpora.yaml mutated between
  resolution and manifest assembly ⇒ `ok` run whose manifest attests bytes the execution
  never used) → SCHEDULED as M3 input-snapshot.1–.3 (read-once ResolvedFile/InputSnapshot,
  every phase consumes the snapshot). The 3rd review demonstrated the CONSTRAINT variant:
  the runtime child reopens the pathname mid-call, so a transient A→B→A rewrite defeats ANY
  parent-side reread (cassette.rs's ConstraintDrift included) → M3 constraint-snapshot
  passes the child a frozen per-run staged copy of the snapshot bytes; the sealed-hash
  claim narrows to "the exact constraint bytes supplied to the runtime".
- Live-body const-pin pattern (run-m2.1e-C2; recurs at every future live pin): a full-body
  `report_en/ja.md` const pin over a REAL run must NORMALIZE the solver version (z3 `--version` is
  live-parsed, env-dependent — report.rs const-pins SYNTHETIC bodies freely; run_oracle.rs
  re-renders instead of pinning). Pattern: read `solver_identity.version`, assert it appears
  EXACTLY ONCE per body (a colliding future version fails loud), `.replace(version, "Z3_VERSION")`,
  pin the normalized body (bless by dumping observed bytes; splice so Japanese stays byte-exact).
  Typed-shape asserts and rendered-body pins OVERLAP at the render surface by design — (a) uniquely
  pins typed structure, the bodies uniquely pin numeric cells + prose; neither half is dead.
- Engine-agnostic DELIVERABLE (user directive): the committed SPEC/code/registry/roadmap/`schemas/`
  name NO specific LLM inference engine, grammar dialect, or model-file format. M2 elaboration picks the
  engine at build time behind the generic harness contract (greedy + fixed seed, grammar/JSON-Schema
  constraint fed by the exported `schemas/`, recorded subprocess, identity/quant/runtime-version in
  manifests); match §3's engine-neutral phrasing `the M2 local-model runtime`. The CONCRETE runtime/model
  actually used is a machine-specific environment detail recorded in `## Runtime` below,
  NOT in the agnostic deliverable (the contract is the artifact; the pick is config). The
  history-resident `docs/` corpus names engines as landscape — outside the tree audit. Fixtures/test
  values obey this too — use unmistakably-synthetic tokens (`model.baseline`/`fixture_quant`/`1.0.0`),
  since a realistic generic quant/format token still names a real scheme (M2.1 codex follow-up: a
  real bit-width token had slipped into a fixture whose comment asserted it named none; RECURRED M2.9 r3 — `q4` slipped into a model-adapter test fixture, the dialect-only de-leak grep missed it). Audit = word-boundary `git grep -niP` (names `\b`-bracketed) over EVERY forbidden category (engines + grammar dialects + quant/model-format tokens `q4`/`gguf`/…), not just one, and not a bare substring grep (false-matches a Cargo.lock dependency name). Catch the bare quant token with a case-SENSITIVE lowercase `\bq[2-8](_[0-9km])?\b` (drop the global `-i`, wrap the engine/dialect/format names in `(?i:...)`): it matches `q4`/`q4_0` yet skips the uppercase `Q1`/`Q2` SMT labels (a case-INSENSITIVE `q[2-8]` false-hits them — the prior skip-bare-`q[2-8]` rule left bare `q4` UNAUDITED; codex M2.17). The repo-wide grep OVER-matches by design (AGENTS.md: a filtered finding beats a dropped bug) → triage each hit vs the standing exempt/false-positive set: this bullet's own rule-doc (the `q4`/`gguf` examples), the route.fixture cassette (real identity, exempt), and the lowercase `q2`/`q3` SMT-pair variables in `verdict.rs` (M1 query indices, not quants), the English word `guidance` in prose (false-match on the constrained-decoding library token), and dev/review-tool names (`Codex`, `Claude Code`) in process/provenance notes — the rule targets the route's INFERENCE engine (what fills the schema), not dev tooling, and the mandated `Codex-Review:` commit trailer sanctions `Codex`, so triage these EXEMPT, never remove (codex M2.18 re-flagged a roadmap `Codex follow-up` note; rejected — dev-tool, not a route-engine leak). So the per-UNIT close gate = the unit's TOUCHED files carry no token (scope the grep to them) — respec/planning closes INCLUDED, their roadmap/memory edits are committed files (the M2.2 respec banked dialect/device/env-var tokens into the roadmap; codex caught it — bank machine specifics in `runtime.local.md`, point the roadmap at them); the milestone review runs the full-repo triage. Hand codex-review prompts THIS full exempt set verbatim, not a condensed invariant — else codex re-flags the rule-doc's own quant/format examples (codex M2.20). Reconstruct the command from this bullet — no banked wip (consumed at .2b close). EXCEPTION (user decision, model-cassette.2): committed RECORDED cassettes (under `crates/ckc-cli/tests/fixtures/cassettes/` now, run-m2.2's experiment-cassette roots later) carry the runtime's REAL `model_identity` (model/quant/engine strings) — machine-specific MEASUREMENT data with honest provenance, NOT engine-neutral contract/fixture artifacts → EXEMPT from the synthetic-token rule + this audit. Audit FAIL-CLOSED: exclude only the SPECIFIC live-recorded path(s) whose cassettes carry the runtime's REAL `model_identity` (model/quant/engine strings) — today `route.fixture/` (its recorded `seed-42.json`) + the run-m2.2b live-recorded experiment roots = repo-root `/cassettes/**` (whole tree recorded, real identity): `git grep -niP … -- . ':(exclude)crates/ckc-cli/tests/fixtures/cassettes/route.fixture/' ':(exclude)cassettes/'` — the fixture-tree exclude stays NARROW (`route.fixture/` only), so CRAFTED synthetic cassettes (route-single-ir golden + bad, SYNTHETIC identity) stay AUDITED + pass; the repo-root `cassettes/` tree is all-recorded, hence excluded whole (it is also deny-Read + Serena-ignored — census via runtime indirection only). Replay pins output/provenance/content-hash + the full recording envelope (producer + empty diagnostics/trace/runtime), never an identity VALUE BY NAME → the host runtime swaps with no test-code edit, but the identity rides the pinned content-hash (it changes only via a deliberate re-bless + re-pin).
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
    (`exp.m2_multihop`)"; §10's `exp.m3_cnl` reuses this set-form shape (three pipelines, direct
    baseline); §11's `exp.m4_*` ids stay separate experiments.
  - Manifest identity (§9 vs code, finder-confirmed): §9 SEPARATES model identity from prompt hashes
    → `ModelIdentity` = `{model_id, quant, runtime_version}` ONLY (mirrors `SolverIdentity`'s
    identity-only shape; no prompt hash inside). Both manifests carry the §9 set (model_identity +
    test-source/reference/schema/prompt-template/model/runtime hashes) as OMITTABLE fields: B1 plumbed
    all-None (M1 bytes + pins byte-identical via omit-None), B2a computes them in `manifest_inputs`
    (run.rs) gated on the run MODE — new `model_routes: &[RouteShape]` empty (M1) → 7×None, non-empty
    (model route) → `model_identity = agreed.cloned()` (honest None if no fill identity, ≠ M1 all-None)
    + the 4 hashes over THIS run's ACTUAL inputs (not registry-wide): `aggregate_hashes`
    (sort+dedup+join`\n`+hash_bytes) over the run's docs (`test_source_hash`) / the ROUTE-RELEVANT
    Schema+Prompt entries (`schema_hash`/`prompt_template_hash`, want-sets by RouteShape→static_id,
    single_ir↔clinical_ir/single_ir, direct_smt↔smt_query/direct_smt), raw-byte hash of the
    experiment's `expected_outcomes` (`reference_hash`); `model_hash`/`runtime_hash` STAY None (env
    bare-name commands, no committed bytes; identity rides `model_identity`). Fixture:
    `copy_committed_registry` gained schemas.yaml + prompts.yaml → `write_m2_root` (calls it)
    provisions the model-route test; reference file already copied. B2b DONE — value-pinned
    (bless-from-observed) the RunManifest §9 record + ReplayManifest parity in
    `m2_route_loop_lands_both_routes_namespaced`. GUARDS (durable, recur at run-m2.2's live
    value-pins) — (1) same-typed observed-bless: slot→field is fixed BY CONSTRUCTION (each slot
    built from its named accessor in the assert diff; a verbatim copy can't swap them), and a
    single independently-verifiable field (`reference_hash` == `sha256sum` of expected_outcomes)
    proves THAT slot ALONE — to anchor another same-typed field, recompute it from its input
    bytes (codex caught "one cross-check anchors all 4" as overreach); (2) a cross-record
    "parity" assert over two records that clone one `manifest_inputs` = disk-round-trip + MIRROR
    parity, NOT independent recompute; (3) an assembly-level all-None byte-pin proves only the
    encoder — the pipeline mode-gate (empty `model_routes`→None) needs a live-run assert to
    cover end-to-end (codex: was waved off as "already held", now asserted live in run_oracle).
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
  - run-m2.2 settled decisions: direct `:named` labels are a NAMING SCHEME in template text,
    ROLE-SENSITIVE — overlap `ctx.<document-id>.rule.<n>`, deontic `a.<document-id>.rule.<n>`
    (per-document 0-based rule order; §8.6 reference queries + trace.rs split the prefix by role,
    and label LISTS would import IR-layer rule counts into the no-IR route). `model_ms_per_call` =
    the model-invocation cap (registry budget key, `Resolved.model_ms_per_call: Option<u64>` read
    unconditionally, REQUIRED only at record time in `build_record_parts`).
  - "test all layer configurations" (user directive) → deferred to M4 as the §11 route-axis gradient
    seed (was §10/M3 pre-renumber): every meaningful single_ir IR layer + the DMN-style alt. The user
    chose keeping M2 the §9 minimal pair over widening §9; the gradient is the experiment §11 ("vary
    and layer existing IR forms") was written to be.
  - Route mechanics (M3 single_cnl + M4 routes reuse when adding routes): a processing-stage `kind` is a free-form Id →
    a new stage kind is registry data, not an enum change. A route feeds the M1
    `compile_verify_group` back end by hand-building a minimal `Resolved` (reads only pipeline_id
    + stage handles + toolchain hash + budget_ms + shape; documents/groups/plan unread stubs) — the
    test-fixture pattern (M2 built these positionally: `[Id; 8]` padded with `UNUSED_STAGE` +
    `shape: RouteShape`; route-stage-handles RETIRES that form — post-refactor, hand-build the
    named RouteStages variant and derive shape from it). PLAN LESSON: a unit framed
    "extract a tail/chain" must share ONE iteration granularity (per-document derive vs per-group
    compile/verify cannot be one linear fn) — check stage granularity at plan time. Scoring-test
    shape: a route-scoring test mirrors `run_oracle.rs::assert_group_matches_reference` IN FULL (a
    partial mirror passes vacuously) and resolves groups + reference from the registry, never
    hardcoded membership; `repair_limit=0` proves only the zero-budget boundary — faithful
    exhaustion needs malformed cassettes at the base AND each derived seed; pin rejection payload
    SHAPE (key `reason`, non-empty, empty refs) symmetric across codes. Rejection cassettes:
    single_ir seeds 99/98/97 (+derived) = hallucinated / recover / exhaust under
    `route.single_ir/test_source.m1_guideline_a`.
  - route.direct_smt residue (units .1-.5 DONE; git + run.rs hold the build story — only
    forward-load-bearing facts here): the route verifies raw model SMT via pub ckc-smt
    `verify_query_pairs(adapter, &[MintedQueryPair], budget)`; a no-IR route CANNOT honestly mint
    `CompiledArtifact.region_ids`, so always verify via that bypass, never `m1.verify` or a fabricated
    artifact (the wall that forced the seam). Cassette keying: ROLE-namespaced sources
    `<gid>.overlap`/`<gid>.deontic` at the base seed — a shared `<gid>` source would ALIAS Q2 with Q1's
    first repair (`model_fill` reads attempt `i` under `derive_seed(base, i)` on the SAME source); keep
    the namespacing. Slot-3 consequences for run-m2.1's route loop (the index mechanics below are
    the positional form route-stage-handles RETIRES — the durable fact is the direct-emission
    pattern, not the indices): the direct 4-stage pipeline has
    `verify_smt` at slot 3, but `finish_processing_stage(idx)` stamps kind from
    `PROCESSING_STAGE_KINDS[idx]` + gates the solver-budget counter on `idx == VERIFY(5)` → the direct
    tail (`direct_smt_verify_group`, `DIRECT_VERIFY=3`, M1 `VERIFY`=5 inert padding in its `[Id; 8]`)
    emits its verify event DIRECTLY with kind `"verify"` + `SOLVER_BUDGET_KEY`; its `verifier_results`
    cite the two `smt_query` wrapper `content_hash`es (cassette-hash provenance LANDED run-m2.1c:
    single_ir bundle cites source+segments+accepted cassette hash, direct per-role wrapper cites
    member-order source+segments+its OWN accepted cassette hash; input_hashes = §4.3 set on emit →
    tests compare as sets, and the recovery pin asserts the ACCEPTED attempt's hash, not the base's;
    both
    routes). Raw-AI provenance GUARDRAIL (keep the shape, never "fix"): smt_query = raw model body →
    `Origin::AiGenerated` + `AcceptedEvidenceStatus` + EMPTY effects (validate() enforces only
    non-empty-effects ⇒ DiscoveryOnly; the .3b test pins origin/status/kind/schema/producer
    explicitly). Golden cassettes (M1 query bodies verbatim; `:named a.<rule_id>` ==
    `expected_unsat_core`) = harness/determinism fixtures — both routes verify identical SMT →
    identical verdicts BY CONSTRUCTION: report/run/acceptance units attribute happy-path-replay parity
    to the harness, never to §9 evidence; the real contrast = run-m2.2's LIVE weak-model run + the
    degraded rejection cassettes (route-single-ir.4 + route-direct-smt.5). Scoring-test shape for
    metrics/report-m2 reuse: `direct_smt_route_scores_m1_groups` keys the no-conflict closure off the
    minted `<gid>.overlap`/`.deontic` ids (no `solver_query_plan`). GOTCHA (bites report-m2.2 tests):
    `input_hashes` canonicalize as a §4.3 SET — the wrapper sorts by hash, the in-memory event keeps
    insertion order → multi-input provenance assertions compare as SETS, never pin emitted order. The
    EventRecord emits BOTH `input_hashes` AND `output_hashes` via `emit_set` (wrapper.rs) → a read-back
    multi-output event (the direct model_fill event's two smt_query bodies, run-m2.1d4b) is hash-sorted
    too, so compare event `output_hashes` as a SET whenever >1 (single-output pins stay order-free); the
    remaining .1d5/.1e event census pins inherit this. COMPLETENESS (codex-caught .1d4b): a
    directly-emitted event needs its `input_hashes` pinned INDEPENDENTLY — the payload `content_hash`
    equality and the role WRAPPER's own input pin do not cover the §4.6 EVENT's `input_hashes` field, so
    an empty/wrong event-input regression slips a body-only battery (mirror single_ir's `events[2]` input
    pin: the direct model_fill event carries the pair's member source+segments, no cassette hashes).
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
  sections strictly before all DeltaTable sections) — both md renderers walk it (.3a/.3b);
  run-m2.1e renders through the renderers, never the fields ad hoc (the §9 guarantee reaches
  artifacts only once an emitter walks it). Rendering landed (.3a/.3b; pins + Labels
  mechanics live in report.rs tests): `render_markdown`/`render_markdown_ja` share one walk;
  `one_canonical_report_renders_both_language_bodies` proves both pinned bodies render from the
  pinned canonical report.json bytes alone; run-m2.1e writes them as report_en.md/report_ja.md. JA
  mapping: §0 labels + ids/hashes/codes + `not_applicable` verbatim ENGLISH inside JA prose,
  structural chrome JA (`、` joiner, `なし。` empty slot), delta heading's ` - ` language-invariant.
  RENDERED ⇒ VALIDATED: every member a renderer walks must sit under a
  `Report::validate` rule (rule 6 canonical sets; rule 7 code-span-inert identity text → renderers
  interpolate bare, no escaping layer; rule 3 rejects line breaks in quoted span text — else valid
  text injects block structure into both bodies). REPORT-m2.1 TRAP (DISCHARGED in landed
  .1a code, byte-position-tested): §4.3 sorts member keys, so raw-before-delta in BYTES needs the
  raw-rows key below the delta key → keys `raw_rows` < `route_deltas`, Rust fields stay
  routes/deltas; the same key-sort trap applies to ANY future §-ordered canonical member pair.
  CONSUMPTION SEAM (report-m2.2, landed): `assemble_report` grew an 8th param
  `Option<ModelRunSections>` — per-route §7.4 ledgers (`&[(Id, &[DiagnosticRecord])]`, ALL the
  route's records, clean route = empty slice, dup → `DuplicateRouteLedger`, route set must equal the
  RouteMetrics set → else `SectionRouteMismatch`) + `route_metrics()` outputs + baseline id +
  ModelIdentity. `experiment_metrics` runs IN-assembly (its fail-closed panics = caller bugs there);
  run-m2.1 must NOT call it in run.rs — hand over raw RouteMetrics. Taxonomy derives by counting
  ledger codes per route (§4.3-sorted both levels); the M2-member expected values in tests tie to
  the .1c byte-pinned populated_report fixture (`m2_route_metrics()`/`baseline_model_identity()`
  shared helpers).
- Record-mode prompt composition (run-m2.1f1 selectors; run-m2.2a completed the grounding scaffold —
  format byte-pinned in `single_ir_prompt_composes_grounding_scaffold_and_orders_spans`, read the test
  not a copy): `select_record_{schema,prompt}` key by `id.as_str()` (SingleIr→clinical_ir/single_ir,
  DirectSmt→smt_query/direct_smt, M1Layered→None — DEFENSIVE vs `manifest_inputs`' Err on M1Layered).
  `single_ir_prompt(template, doc_id, graph, segments, lexicon)` emits document / per-segment
  `segment: <id> kind=<k> regions=<r,..>` / `regions:` / `system:` / `concept: <id>[ var=<v>
  ge=/gt=/le=/lt= present canonical bounds]` / `action:` scaffold lines in artifact/file order
  (deterministic upstream §4.3 — no re-sort), then
  reading_order-sorted spans (shared `reading_order_text`); regions read from `graph.regions`, NOT a
  separate param — the exact set the accept closure grounds against (single source of truth).
  `direct_smt_prompt` composes NO label list — the role-sensitive `:named` scheme is TEMPLATE TEXT
  (M2-plan respec sub-bullet holds scheme + rationale). Vocabulary consistency = THREE layers (2a +
  its codex follow-up): committed `clinical_ir.schema.json` bakes the lexicon-derived enums (concept
  codes / action kinds / interval vars / terminology system + closed enums) → scaffold INFORMS (incl.
  canonical bounds), schema ENFORCES at generation (constrained decoding), and `off_lexicon_ids`
  behind `single_ir_accept` enforces membership at REPLAY — binding system/code/alternatives,
  concept/negated/interval-var atoms, action kind/target, exception atoms → repairable
  `FillReject::Schema` naming the off ids, ahead of grounding; interval BOUNDS stay free
  (schema-mirroring — the text may state its own numbers). A lexicon edit re-exports the schema via
  the existing bless+drift guard. Record assembly: adapter-FREE
  `build_record_parts(root, schemas, prompts, resolved) → RecordParts{template, constraint, ctx}` =
  select → require `model_ms_per_call` (record-only) → load template (inline|path) → pre-write
  byte-verify → ctx; `build_route_record` = thin parts+`&setup.adapter` zip (`ModelAdapter::new()`
  probes a subprocess → keep it OUT of the testable fn). Template⇄composer CONTRACT: a template's
  given-list enumerates exactly what its composer supplies → ANY template edit re-pins prompts.yaml
  AND the §9 blessed `prompt_template_hash` literal (the m2 loop test catches it; verify the other 3
  §9 hashes stay put as the cross-consistency check).
- RESPEC-COMPLETENESS: when a unit must CONSTRUCT a type, bank its CONSTRUCTOR + a mirror call site,
  not just a field list — the f-respec banked `SourceTextSpan`'s fields but not `::derive` /
  report.rs `graph` helper → cost a targeted source_linkage.rs read at f1 impl. Fixtures build
  array-order ≠ `reading_order` to prove the sort — verify EVERY parameterized case exercises the
  property: codex caught f1's direct_smt fixture accidentally pre-sorted (an identity no-op sort
  would've passed the pin; single_ir alone proved it) → a test that only half-proves its claim is a
  fake success criterion.
- Recorded-run battery (run-m2.2c, `crates/ckc-cli/tests/recorded_run.rs`) = the standing
  runtime-absent proof over repo `/cassettes/**` (weak-baseline census + §9 manifests + re-render +
  `replay::execute` matched); acceptance-m2 reads it. §4.3 attestation-census facts it pinned:
  manifest/replay `output_hashes` is a payload-content SET → route-independent heads dedup
  ACROSS routes AND `SegmentIr` dedups across structurally-identical DOCS (control ≡ guideline_b
  segments payload — SegmentIr carries no doc id/text; text distinctions ride the graph, all 3
  graph hashes distinct) → 12 landed head wrappers attest as 5 hashes + 3 run-level tail wrappers
  (trace/lineage/report) = 8. Census pins must derive from landed wrappers (set equality), never
  doc-count arithmetic. Identity EQUALITY pattern: anchor = one committed cassette strict-read
  from the copied root at runtime (deny-Read honored; identity-agreement gate makes any one
  representative); report/manifest identities assert equal to it — no literal. Payload files
  (`schemas/*`, `registry/prompts/*.txt`) are opened by `registry check` ONLY — replay-mode
  `ckc run` reads just their DECLARED registry hashes (§9 slots) → the battery copies them in +
  gates on a `registry check` invocation over its root, the payload-byte drift pin (codex M2.2c).
- Acceptance-driver pattern (acceptance-m2; reuse at M3+ acceptance): one scratchpad Python driver
  runs the evidence runs + every deny-Read artifact check (paths baked into the script, never on
  the command line). Codex-hardened instrument rules: EVERY independent recompute is a failing
  check, never an INFO print (a claimed MATCH needs a gate that can fail); the driver itself
  builds the release binary + asserts `manifest.git_commit == git rev-parse HEAD` on a clean tree
  (else "release build of HEAD" rides an unproven stale binary); pin the EXACT A/B diff SET, not
  an upper-bound `<=` allowance (the pass hides which files actually differed — b2e010b's message
  overstated "manifests differ by --out" when manifest.json is byte-EQUAL). Facts (re-confirmed
  post-hardening): two runs differing only in `--out` differ EXACTLY in {replay_manifest.json: the
  `--out` token in its `command` member; events.jsonl: timing fields + `run_id` = the `--out`
  basename}; manifest.json + all other landed bytes byte-equal — that A/B twin + `ckc replay`
  (exit 0, `matched`) = the byte-stability theme; running one twin under a bogus
  `CKC_MODEL_COMMAND` override functionally proves replay never invokes the runtime (completes
  identically). A §0-vocabulary check keys on closed-set conformance + §15-gated-claim ABSENCE +
  the prose metrics lead, never on label presence: `wording` is row-derived (findings/no-conflict
  rows), so a zero-accepted-translation run renders `wording: none.` faithfully — asserting the
  label there would demand fabricated findings.

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
- Grammar constraint (the direct route's BNF SMT surface) verified LIVE (run-m2.2b): the env wrapper
  translates the committed grammar to the engine's constraint dialect at invocation time (machine-local
  transform → `runtime.local.md`) → the committed repo stays dialect-free; the constraint compiled and
  a separate PROBE prompt produced a complete grammar-conforming output on the live device (the
  recorded run's route outputs still fail the acceptor — the honest weak-baseline census).
- Instrument-truncation diagnostic (run-m2.2b lesson, engine-agnostic): a runtime holding a FIXED total
  sequence budget can silently truncate generation mid-token → surfaces as §7.4 fill failures that
  poison the weak-baseline census. Tell apart: output length INVERSELY tracks prompt length + outputs
  end mid-token ⇒ instrument fault (fix env, wipe cassettes, re-record); genuine weak-model
  degeneration loops then truncates at the LOGICAL token cap (length ≈ cap, prompt-independent). Apply
  before trusting any failure census from a new runtime/env.
- Re-record after ANY env/wrapper change starts from an EMPTY cassette store: record mode OVERWRITES a
  same-key cassette without comparing (the 2a "record byte-verify" = the pre-write template/schema-hash
  drift guard + constraint-drift re-read, NOT an existing-cassette compare) → stale files from the
  superseded env survive as ORPHAN keys (attempts the fixed run no longer makes, e.g. derived-seed
  repairs) = wrong committed census. Wipe, re-record, commit the fresh tree.
- `derive_seed` exact splitmix64 draws (engine-agnostic, replay-load-bearing) are pinned in the
  `derive_seed_is_deterministic_and_distinct` test (model.rs, `.2a`) — read the test, not a memory copy;
  the `.2b` `model_live` test re-asserts them live through `invoke_samples`.
- `.2b` DONE — `crates/ckc-cli/tests/model_live.rs` (`#[ignore]`d; `cargo test -p ckc-cli --test
  model_live -- --ignored`) is the standing live confirmation of the §9 runtime properties — read the
  test, not a copy. Non-obvious: it does NOT assert cross-seed equality (greedy seed-inertness is
  environment-specific → `runtime.local.md`); its bounded enum+bool constraint fixture lives in
  `tests/fixtures/` NOT `schemas/` (test artifact, not a production route constraint; the plan-line
  'schemas/ constraint' was shorthand — the fixture stays put).
