# Agent Memory

Entries add value beyond spec / AGENTS.md / code / git / runtime env — project-independent tooling
pitfalls (RTK, Headroom, Serena, Claude Code, web) live in each agent's global guidance, not here.
Exception: high-value reminders derivable but easily forgotten under token pressure. Consolidated
aggressively; full pre-consolidation text in git history.

## Policy

- Context hygiene (user directive; bg `git show 531f586`): keep every session lean + phrased in project
  vocabulary (processing stages, units, gates, artifacts) — plain operational words over research jargon
  in memory/roadmap/commits/code. Consult `docs/` via read-only subagents so its vocabulary stays out of
  the main window. Root `.rgignore` keeps ripgrep sweeps (subagent Grep, `rtk proxy rg`) out of `docs/`;
  Bash `grep -r` still enters it → scope Bash greps by path; deliberate docs searches use `git grep <pat>
  -- docs/`, `rg --no-ignore`, or explicit paths. Implement sessions match patterns from the latest
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
  pinned bytes is a deliverable, not a preamble); crate foundations pair only with a small type surface (one payload module each); deterministic code + a
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
  secondary hint only (drifts under edits above it); cite only untagged checked roadmap lines as anchors
  (`[S]` = salvage-assisted, usage understated). At plan/re-scope time audit any spec a unit must
  byte-reproduce — readability listings (alignment padding, inline result comments, illustrative
  declaration/conjunct order) contradict deterministic-emission rules and need a scheduled re-pin
  deliverable (smt-emit.3a: §8.6 smt2 vs §6 sorted-declaration). SALVAGE RETIRED (user directive,
  2026-07-02): banking applyable wip artifacts (`.agent/wip-*`
  patches / byte-exact code copies / transcription blueprints a redo line points at) cheats the unit — the
  redo's recorded context-usage measures artifact application, not the unit as specced, so sizing
  anchors come from untagged stubs only. Overflow recovery is LAND-OR-REVERT: either the proven half closes
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
  unchanged-expected-bytes pin tests = the regression guard, never edit their literals). Emitter sorts on
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
  warnings' cargo doc -p <crate> --no-deps` whenever a unit touches doc comments. Two failure shapes:
  a public item's ``[`priv`]`` intra-doc link to a PRIVATE item (`private_intra_doc_links`) → plain
  ticks `` `priv` ``; a link to a type not `use`d in the module (unresolved) → qualified-path
  `` [`T`](crate::T) `` (a docs-only `use` trips `unused_imports`). Counting gotcha: `grep -c "^error"` on the doc output includes the trailing ``error: could not
  document `ckc-cli` `` summary line → real standing count = matches − 1 (17 link errors read as 18);
  diff the error LIST against the standing set (model.rs/replay.rs/trace.rs/cassette.rs), not counts.
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
  test mirroring `tests/model_live.rs`, content-hash-pinned. LANDED run-m2.1f1+f2: the `--record`
  surface = f1 pure selectors/composers + f2 dispatch flag (`take_bool_flag` pre-partitions `--record`
  out of `rest` BEFORE `take_flags`)→validate→execute→execute_routes threading; `execute_routes` builds
  `RecordSetup` (adapter probe + `validate_model_registry`) ONCE `if record` else `None`, then per-view
  `build_route_record` (f1 select→template/constraint/`RecordContext`) feeds both fills' new
  `Option<&RouteRecord>` → `FillSource::Record` (type-enforced, NO live call); §9 manifest fields landed
  e-B. DEFERRED to run-m2.2 (LIVE, via `FillSource`): live record exercise + replay.rs model-artifact
  coverage. f2 RULING: the respec's test-(6a) `set_var(CKC_MODEL_COMMAND, bogus)` no-probe assert is
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
- M1 reviewed (git/roadmap hold the detail). §4.4-vs-§8.3 tension RESOLVED by SPEC amendment: a
  processing stage's total operation result IS its §4.6 EventRecord (§8.3 has no
  per-stage total artifact); only commands materialize a standalone TotalOperationResult (value/
  residual/ambiguity/incoherence buckets stay empty until typed placeholders exist). GUARDRAIL: do NOT
  add per-stage TotalOperationResults — inert + redundant with EventRecords until then (M2+ may
  revisit). OPEN enhancement (unscheduled, AGENTS.md-preferred): tests are example/byte-pin only →
  property-based/fuzzing for the canon layer (round-trip identity, reject-any-mutation) + StringPolicy
  idempotence.
- §4.6 event IS the stage's total result (above) → a stage that LANDS artifacts inside a loop must emit
  its one event on EVERY path once anything has landed; an infra-error EARLY-RETURN (copied from a
  single-artifact fill's event-less `CassetteError` abort — safe there, it lands nothing pre-event)
  orphans the already-landed artifact + drops its counters. Event-less abort is safe ONLY before the
  first land; after, ride the event like the wrap/land-error break paths (`direct_smt_fill` deontic
  cassette-fails-after-overlap-lands; codex .1d4a).
- run-m2.1d5a-1 LANDED the model-route loop in `execute()` + the cross-route group-namespacing
  collision fix (the sizing bullet's CROSS-ROUTE LANDING problem). `execute()` dispatch is 3-way: a
  lone `[M1Layered]` view runs the M1 body inline verbatim; any set MIXING M1Layered + model routes →
  ONE command diagnostic + zero artifacts; all-model → `execute_routes(root, &views, shell)`. BOTH
  routes' group artifacts now land under `out/routes/{pid}/groups/{gid}/` (mirrors head namespacing
  `routes/{pid}/artifacts/{doc}`): `direct_smt_fill` mints that dir internally (it holds `resolved`);
  the M1-shared `compile_verify_group`/`direct_smt_verify_group` stay param-driven, so `execute_routes`
  passes the namespaced `dir` while the M1 layered path keeps bare `groups/{gid}`. OBSERVED landed
  layout (the .1d5a-1 gate's literals — pin .1d5a-2/.1e from these, don't recompute): `out/routes/` =
  `[pipe.m2_direct_smt, pipe.m2_single_ir]`; single_ir artifacts `[ir_bundle.json, segments.json,
  source_document_graph.json]` + group `[compiled.json, smt, verifier_results.json]` (compile lands
  SMT bodies under an `smt/` subdir); direct artifacts `[segments.json, source_document_graph.json]`
  (NO ir_bundle — mints no IR) + group `[deontic.smt_query.json, overlap.smt_query.json,
  verifier_results.json]`; NO bare `out/groups/`. `RouteRun{pipeline_id, ledger slice
  (shell.ledger()[start..]), fills, groups, samples: vec![groups.clone()]}` collected
  `#[allow(dead_code)]` (`let _ = &route_runs`) → .1e metrics consume (the .1d5a-2 tails use the
  separate `all_*` vecs, NOT RouteRun). Identity
  agreement folds each Some ModelIdentity into `agreed`; a differing Some → one command diagnostic +
  fail-closed return (goldens agree → the clean gate never trips it). Codex .1d5a-1 follow-up:
  single_ir's group loop emits the COMPILE partial-group diagnostic+event (mirrors `group_pipeline`,
  the module header's documented partial-group rule) on a member-short group; direct's loop keeps a
  bare skip — it mints no compiled artifact, so that compile-stage rule does not apply, and the member
  head already failed+diagnosed upstream. Neither skip is ever fully silent (the short member's own
  head/fill stage always diagnosed first) → the dedicated error-path TESTS defer to .1d5a-2b (single_ir
  partial-group event; mixed-shape→command-diagnostic; identity-disagreement→fail-closed). Helper
  `route_group_dir(resolved,gid)` now centralizes the `routes/{pid}/groups/{gid}` dir (was 6 duplicated
  format strings across both loops + `direct_smt_fill` + 3 direct-verify test sites — a split-dir class
  Codex flagged where the fill landed route-namespaced but the paired verify landed bare).
- run-m2.1d5a-2 LANDED the unified run-level tails. `execute_routes` collects
  `all_docs`/`all_graphs`/`all_group_traces` across the view loop, then runs
  `trace_processing_stage`/`report_processing_stage` ONCE over both routes at the BARE run root
  (trace_bundle.json/lineage_index.json/report.json/report_en.md/manifest.json/replay_manifest.json
  beside `routes/`+`logs/`). NEW `emit_event: bool` on both tail fns gates the §4.6 census EVENT
  only: M1 execute() callers pass `true` (M1 byte-identical); the M2 tails pass `false` (route
  pipelines declare no trace/report step, run under the baseline route's padded `UNUSED_STAGE`
  slots) → M2 tails emit NO event yet stay fail-CLOSED by raising the failure diagnostic DIRECTLY
  (`shell.diagnostic(diagnostic.clone())` on the Err arm when `!emit_event`; the diagnostic
  otherwise reaches the shell ONLY through the un-emitted event). Cross-route correctness (in-code,
  commented): all_graphs deduped keep-FIRST-per-`payload.document.document_id` via a seen-set
  `retain` — NOT `dedup_by_key`, since all_graphs = direct's BTreeMap-sorted heads ++ single_ir's
  first-appearance graphs → dupes are non-adjacent and consecutive dedup misses them; all_docs
  STABLE-sorted bundle-bearing-first (`sort_by_key(|d| d.bundle.is_none())`) so assemble_trace's
  first-by-id lineage lookup resolves the single_ir bundle-doc, not direct's bundle-less head (views
  run direct-first per `pipelines:[direct,single_ir]`), else assemble_report rejects `MissingLineage`.
  The trace keeps BOTH routes' parallel chains (route-prefixed artifact_ids → distinct chain nodes;
  the shared source node dedups whole-node) → `TraceBundle.claims.len()==3` (single_ir's 2 compiled
  groups mint claims; direct mints none). RESPEC banked-fact ERROR corrected (verify-against-code
  rule): the gate test DID carry `assert_eq!(listing(&out), ["logs","routes"])` — the respec's
  do-not-read note claimed it did NOT — so it now pins the 8-entry root set + a
  `strict_at::<TraceBundle>` claims assertion. RECURRED M2.1e-C1 (codex): C1 claimed no M1 success test pins an exhaustive root listing — FALSE (tests/cli_shell.rs + tests/run_oracle.rs both do); the respec swept run.rs's in-module M1 tests (`.exists()`/`strict_at`, non-exhaustive) but not the tests/*.rs integration files → verify-against-code must grep the WHOLE crate test surface (`sorted_entries`/`files_under`/`expected_files`/`read_dir` over tests/ AND src), not just the edited module. Additive-safety split: a rendered view landed via `shell.write_under` (report_en/ja.md) is NOT lineage/manifest-tracked (the manifest derives from the report wrapper, not a dir scan) → byte-safe for report.json/manifest/lineage pins, perturbs only exhaustive file-SET listings; `land`/`wrapper`/`land_record` DO track. Error-path diagnostics LANDED .1d5a-2b (3 run-binary
  tests over `write_m2_root` variants: drop guideline_b's single_ir cassette → fill + partial-group
  compile diagnostics co-occur, diag order fill<compile since the fill loop precedes the group loop;
  swap direct→layered in experiments.yaml `pipelines` + re-point baseline in-set → mixed-shape command
  diagnostic + `assert_only_logs`; re-bless one single_ir cassette with a divergent synthetic identity
  → fail-closed command diagnostic). Reusable: divergent-identity re-bless = `store.replay(key).payload`
  clone + swap `model_identity` + `build_wrapper`/`persist` (CassettePayload all-public → no head
  re-run); a fully-bundle-less doc does NOT break the tails (`assemble_trace` Option-guards bundle
  nodes, mints claims/lineage only from compiled groups, and a member-short group `continue`s before
  its GroupTrace push → untraced). Two-run determinism + event census (tails contribute 0 events) →
  .1d5b; §9 report sections (passed `None` here) → .1e.
- run-m2.1d5a-2b codex-review (xhigh): 4 findings, all under-constrained-assertion gaps (production
  sound, tests passed). Strengthened: identity-disagreement now pins fail-closed via ABSENT run-level
  tails — `read_dir(out).sorted()==["logs","routes"]` (the `agree_model_identity` false→`return`
  precedes the post-loop tails, so no `trace_bundle`/`report`/manifest lands) + direct-first order via
  `routes/pipe.m2_direct_smt/groups/group.m1_no_conflict/verifier_results.json` exists (empirically
  confirms the `pipelines:[direct,single_ir]` resolve order the comment claimed, else direct never
  runs); member-short now pins tails-COMPLETE via `trace_bundle.json`+`report.json` exist, and
  exactly-one-short via Invalid-`compile`-event `.count()==1`. CORRECTED event-scope model (I had it
  wrong first pass): command-scope diagnostics (`shell.diagnostic`) ride the CLOSING command event =
  `events.last()` (`processing_stage`==operation_id `"m1"`), NOT "no event"; stage-scope diagnostics
  ride their processing_stage event; BOTH land in `logs/diagnostics.jsonl` (=`self.ledger`, append
  order) = the `executed()`-returned `diagnostics` vec (all scopes) → pin command-scope by
  `events.last().diagnostics` carrying it, not by absence from events.
- run-m2.1e-A LANDED run-level landing completeness + honest producer, closing the .1d5a-2 replay
  hole. NEW `GroupTrace.smt_queries: Vec<ArtifactWrapper<QueryBody>>` (single_ir empty; direct = a
  `[overlap,deontic]` PREFIX — `[]`/`[overlap]`/full, roles land sequentially + break on fail);
  single_ir pushes GroupTrace on `Some(compiled)`, direct on `df.smt_queries` NONEMPTY (new
  `DirectFill.smt_queries` retains every landed role; verify only on the complete `pair`), NOT
  verify-success — only GroupObservation (report group-row) gates on `results`; a landed-but-
  unverified compiled/lone-role/pair stays replay-covered; `manifest_inputs`
  walks `group.smt_queries` into output_hashes. DURABLE: replay coverage (replay.rs diffs manifest
  output_hashes) ≠ trace-DAG membership — the pair hashes live ONLY in the manifest (`TraceNodeKind`
  has no smt_query variant → `assemble_trace` UNCHANGED), so the manifest hash is DAG-orphaned unless
  `direct_smt_verify_group`'s `verifier_results.input_hashes=[overlap,deontic]` provenance-links it
  (clean path). Honest producer: `tail_producer(resolved, idx, emit_event)` — false (M2 run-level
  tail) → `static_id("processing_stage.run.{trace,report}")` (consts RUN_TRACE_STEP/RUN_REPORT_STEP),
  true → early-returns `producer(resolved, idx)` (M1 byte-identical); fixes M2 tails formerly minting
  the route's inert `UNUSED_STAGE` step-id. Pins (m2_route_loop_lands_both_routes_namespaced): TYPED
  `RunManifest.output_hashes.contains` (not substring) over direct overlap+deontic content_hashes, +
  full synthetic-producer pins (tails' `producer.pipeline_id` == baseline `pipe.m2_direct_smt` AND
  `producer.pipeline_step_id` == RUN_TRACE_STEP/RUN_REPORT_STEP). NEW
  full-run pin `m2_direct_partial_landing_is_replay_covered`: delete the deontic cassette → §4.4
  valid-remainder (CassetteError rides its §4.6 event, group skips remaining stages) still lands a
  manifest covering the lone overlap. Codex (xhigh) follow-up, DURABLE: (1) the initial .1e-A carried
  only the complete `pair`, discarding a PARTIAL `[overlap]` landing (already pinned by
  `..._still_emits_event`) pre-execute_routes → replay-uncovered; fixed via the `DirectFill.smt_queries`
  prefix vec, ADDITIVE (kept `pair` as the type-safe complete-verify input) to avoid churning ~8
  `.pair` test sites. (2) content_hash-EXCLUDED provenance (producer.pipeline_step_id) needs an
  EXPLICIT regression pin — hash/layout/census/determinism pins ALL pass through a producer
  regression, so a revert to `UNUSED_STAGE` stays green without one. (3) the run-level producer is
  SYNTHETIC non-registry: `pipeline_id` stays baseline `pipe.m2_direct_smt` + the step id is undeclared
  in candidates.yaml — documented as synthetic, not registered.
- run-m2.1e-C1 LANDED the report ModelRunSections wiring + report_ja.md landing (B2a compute-then-green
  half; C2 pins the values). `report_processing_stage` gained a `route_runs: &[RouteRun]` param and now
  builds `route_diagnostics` (per-route ledger slices) + calls new private `model_route_metrics(root,
  resolved, route_runs, model_routes) -> Result<Option<Vec<RouteMetrics>>, String>` (empty
  `model_routes` → `Ok(None)`; else mirrors `manifest_inputs`' experiment lookup but parses the reference
  via `parse_reference` since `route_metrics` needs `&[ReferenceEntry]`, not raw bytes). The chain head
  swapped the hardcoded `assemble_report(…, None)` for a `match (route_metrics, agreed.as_ref())` →
  `Some(ModelRunSections{route_diagnostics, route_metrics, baseline_pipeline_id, model_identity})` ONLY
  when metrics present AND a `Some` agreed identity; either absent → `None`, no panic. CODEX-C1 CORRECTION:
  a degraded model route (`model_routes` non-empty, `agreed`=None) does NOT mirror the manifest — the §9
  manifest still emits that run's setup hashes (test_source/reference/schema/prompt) with `model_identity`=None
  (gates only on `model_routes.is_empty()`), while the report drops ALL sections (`ModelRunSections` requires
  a non-optional identity) → report vs manifest DIFFER on a degraded route BY DESIGN, a §7.x view declining to
  attribute results to an unknown evaluator (flag for M2 review: is an identity-less report representation
  wanted?). Fix: gate `model_route_metrics` on `agreed.is_some()` at the call site — a degraded route now
  SKIPS the reference parse whose result the `None` arm discards (pre-fix a malformed-but-readable
  `expected_outcomes` sank a degraded run that `assemble_report(None)` completes; `manifest_inputs` only
  raw-reads+hashes that file, never parses). LOW-SEV DEFERRED: `model_route_metrics` + `manifest_inputs`
  independently re-read experiments+reference on an identity route (benign TOCTOU on static committed files
  in one synchronous stage; threading one read couples the two fns → future factoring). report_ja.md
  lands beside report_en.md in the SHARED stage (BOTH M1 + M2) via `shell.write_under` → lineage/manifest-
  UNtracked, so every M1 byte-pin (report.json/manifest.json/replay_manifest.json/lineage_index.json/
  report_en.md) stays byte-identical; only the exhaustive file-SET listings moved (3 updated: run.rs M2
  `m2_route_loop_...` + tests/cli_shell.rs M1 + tests/run_oracle.rs M1 `expected_files`). GREEN with M2
  values UNasserted BECAUSE the m2 loop test `strict_at`-reads report.json (typed-validity, not exact-byte)
  → wiring the sections in (M2 report.json now section-bearing) exercises the path proving it computes
  without erroring, leaving the new VALUES for C2's observed-output pin battery. No new doc-lint error
  (private→private intra-doc link to `manifest_inputs` doesn't trip `private_intra_doc_links`).
- run-m2.1e-C2 LANDED the model-route report VALUE pins (extended
  `m2_route_loop_lands_both_routes_namespaced`, no new test fn → suite count unchanged). DURABLE
  (recurs run-m2.2 live pins + acceptance-m2): a full-body `report_en.md`/`report_ja.md` const-pin
  over a REAL run must NORMALIZE the solver version — z3's `--version` is live-parsed at Z3Adapter
  construction, so it's env-dependent (WHY no LIVE-run body is const-pinned — report.rs const-pins SYNTHETIC bodies freely,
  their version a hand-set fixture; a live body cannot: run_oracle.rs
  re-renders the landed report.json and compares instead of pinning bytes). Pattern: read
  `solver_identity.version`, assert it appears EXACTLY ONCE per body (a future version colliding with
  another rendered token — the fixture model `runtime_version` `1.0.0`, an integer fraction — then
  fails LOUD, never silently rewrites the collided token), `.replace(version, "Z3_VERSION")`, pin the
  normalized body against a const (blessed by dumping the landed bytes from a scratch `write_m2_root`
  run, `4.13.3`→`Z3_VERSION`, embedded via a Python splice so the Japanese stays byte-exact). Other
  observed facts on a CLEAN model run: each route's `failure_taxonomy` code map is EMPTY but the route
  is still NAMED (Report::validate rule 5 — a present empty-map route = a clean route); `metrics`
  baseline = the direct route, `emission_order` = raw rows for both routes (direct, single_ir) before
  the lone single_ir baseline-delta table, `k_sample_convergence` NA on every route (single k=1
  draw). Pin split: part (a) asserts report.json STRUCTURE (emission_order + k_sample NA + agreed
  identity + taxonomy shape) via the typed `strict_at` Report; the raw metric VALUES
  (`recorded_call_count` 4/1 direct, 3/1 single_ir; delta -1/1; all others 1/1 or 0/1) ride the
  rendered-body const pins. The halves OVERLAP at the render surface (route names, empty
  taxonomy, baseline id, k_sample NA, model identity appear in both); (a) uniquely pins the
  TYPED shape (enum order, integer map counts, typed identity fields), the bodies uniquely pin
  the numeric metric cells + full prose — neither half is dead.
- run-m2.1d5a-2 codex-review (xhigh): core soundness CONFIRMED — fail-closed (each tail = one
  `landed` Result-funnel, every failure → the single Err arm that raises `shell.diagnostic` when
  `!emit_event`), M1 byte-identical, `.expect(baseline)` unreachable (`baseline()` set-form filters
  `pipelines.contains` → exactly one is_baseline view). DURABLE analysis: trace determinism does NOT
  rest on the all_docs/all_graphs merge order — `assemble_trace` `sort_canonical`s nodes/edges/
  claims/rows before returning, so merge order is MOOT for the trace bytes; the bundle-first sort
  matters ONLY for the pre-sort first-by-id lineage lookup (`find(document_id).and_then(bundle)`
  skips a bundle-less doc that sorts first). `producer.pipeline_step_id` is write-only provenance
  (lives only in wrapper.rs emit/read, never consumed by replay/manifest, payload-excluded from
  `content_hash`) → the `UNUSED_STAGE` tail producer is inert + deterministic. DEFERRED to .1e
  (accepted, roadmap broadened): GroupTrace omits accepted landed artifacts on non-full-success
  paths (direct smt_query pair always; single_ir compiled + direct pair when verify fails — both
  arms push only on `(Some,Some)`) + the run-level producer. Fixed this follow-up: stale module doc
  ("later unit" → tails land here); gate now asserts both routes' route-prefixed nodes reach the run
  trace (`claims.len()==3` alone is single_ir-satisfiable, so it did not prove the tail ran over the
  direct route).
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
    `budget_ms` + `shape` (via `route_id_prefix`, .1d3a); `documents`/`groups`/`plan` are unread stubs); `resolve()` NOW resolves route
    pipelines too (run-m2.1a: per-route views, `[Id; 8]` = declared ids padded with `UNUSED_STAGE`,
    `shape: RouteShape`); the route fn lives in
    `run.rs` (`Resolved` + `compile_verify_group` private to `mod run`). The single_ir route's
    accept-closure (`single_ir_accept`) + per-doc fill + golden-cassette wiring LANDED in `run.rs`
    (route-single-ir.2/.2b; reshaped .1d3a: `route_document_head` lands the extract→segment head as
    a `DocHead`, `single_ir_fill(head, …)` replays + direct-emits the model_fill §4.6 event —
    diagnostics ride the event ONLY, `processing_stage_event` ledgers them — and returns a
    `RouteDoc{trace, graph, fill, identity}`; tail lands via slot-3 `close_processing_stage`);
    run-m2.1d5a consumes `resolve()`'s per-route views for the in-`execute` loop; the hand-built minimal-`Resolved` stays a test-fixture pattern (both route fixtures carry
    `shape` + `UNUSED_STAGE` padding now). Scoring/rejection test shapes (route-single-ir.3/.4,
    consumers DONE; the tests hold mechanics + derived-seed constants) — durable lessons only: a
    route-scoring test mirrors `run_oracle.rs::assert_group_matches_reference` IN FULL (a partial
    mirror passes vacuously) and resolves groups + reference from the registry, never hardcoded
    membership; `repair_limit=0` proves only the zero-budget boundary, NOT
    multi-attempt exhaustion — faithful route-level exhaustion needs malformed cassettes at the base
    AND each derived seed through the budget; pin rejection payload SHAPE (key
    `reason`, non-empty, empty refs) symmetric across codes. Rejection cassettes: single_ir seeds
    99/98/97 (+derived) = hallucinated / recover / exhaust, under
    `route.single_ir/test_source.m1_guideline_a`.
  - route.direct_smt residue (units .1-.5 DONE; git + run.rs hold the build story — only
    forward-load-bearing facts here): the route verifies raw model SMT via pub ckc-smt
    `verify_query_pairs(adapter, &[MintedQueryPair], budget)`; a no-IR route CANNOT honestly mint
    `CompiledArtifact.region_ids`, so always verify via that bypass, never `m1.verify` or a fabricated
    artifact (the wall that forced the seam). Cassette keying: ROLE-namespaced sources
    `<gid>.overlap`/`<gid>.deontic` at the base seed — a shared `<gid>` source would ALIAS Q2 with Q1's
    first repair (`model_fill` reads attempt `i` under `derive_seed(base, i)` on the SAME source); keep
    the namespacing. Slot-3 consequences for run-m2.1's route loop: the direct 4-stage pipeline has
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
- Record-mode prompt composition (run-m2.1f1, run.rs above `manifest_inputs`, `#[allow(dead_code)]`
  until f2 consumes): `select_record_{schema,prompt}` key by `id.as_str()`
  (SingleIr→clinical_ir/single_ir, DirectSmt→smt_query/direct_smt, M1Layered→None — DEFENSIVE vs
  `manifest_inputs`' Err on M1Layered). Prompt FORMAT (f2 threads verbatim, run-m2.2 refines
  wording): `single_ir_prompt` = template ++ `document: <doc_id>` ++ spans; `direct_smt_prompt` =
  template ++ `group: <gid>` ++ `role: <role>` ++ per-member(`document: <doc_id>` ++ spans); spans
  sorted by `reading_order` (shared `reading_order_text` helper), `\n`-joined.
- RESPEC-COMPLETENESS: when a unit must CONSTRUCT a type, bank its CONSTRUCTOR + a mirror call site,
  not just a field list — the f-respec banked `SourceTextSpan`'s fields but not `::derive` /
  report.rs `graph` helper → cost a targeted source_linkage.rs read at f1 impl. Fixtures build
  array-order ≠ `reading_order` to prove the sort — verify EVERY parameterized case exercises the
  property: codex caught f1's direct_smt fixture accidentally pre-sorted (an identity no-op sort
  would've passed the pin; single_ir alone proved it) → a test that only half-proves its claim is a
  fake success criterion.

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
