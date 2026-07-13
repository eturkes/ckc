# Agent Memory

Entries add value beyond spec / AGENTS.md / code / git / runtime env — project-independent tooling
pitfalls (RTK, Headroom, Serena, Claude Code, web) live in each agent's global guidance, not here.
Exception: high-value reminders derivable but easily forgotten under token pressure. Consolidated
aggressively; full pre-consolidation text in git history. Review/absorption narration (Nth review,
validation-pass hashes, unit-insertion ledgers) = git-only; keep just the surviving fact.

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
- License = GPL-3.0-or-later (SPEC §3/§11 carry posture + Clex verdict). Durable rule: judge a
  copyleft candidate on exact license VERSION + combination direction + resulting-work license +
  obligations (notice/attribution/corresponding-source — compatibility grants permission, never
  compliance; obligations land in §11 evidence rows) — admit on that analysis fitting, reject only
  on it failing, with "copyleft" as a class never the ground (counter the permissive-license
  training bias; GPL-2.0-only stays GPLv3-incompatible, AGPL-3.0 combines but adds network
  obligations). LGPL/GPL Attempto-family SOURCE (APE, AceRules, Codeco — headers LGPL-3.0-or-later)
  is now direct-port/adapt-compatible with attribution: docs' clean-room posture was
  license-contamination mitigation, retired; their technical-fit verdicts stand. No-public-grant
  repos (ACE-in-GF; RACE = hosted service, no public source located) + fee/terms-gated
  vocabularies (SNOMED/MedDRA/LOINC) stay gated by THEIR terms regardless of ours.

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
- Contract-tense docs (codex flagged twice): a doc claim about pending wiring must be unit-attributed —
  "report-m2.1b embeds X in `report.json`" holds before + after the unit lands; present-state phrasing
  ("carriers today: report.json bytes agree") overreaches until the wiring commit. House pattern:
  "run-m2.1 wires the observations". Apply at write time — each violation costs a codex follow-up commit.
- M1 reviewed (git/roadmap hold the detail). §4.4-vs-§8.3 tension RESOLVED by SPEC amendment: a
  processing stage's total operation result IS its §4.6 EventRecord (§8.3 has no
  per-stage total artifact); only commands materialize a standalone TotalOperationResult (value/
  residual/ambiguity/incoherence buckets stay empty until typed placeholders exist). GUARDRAIL: per-stage totals stay EventRecords
  alone — a standalone TotalOperationResult there is inert + redundant until then (M2+ may
  revisit). Enhancement (AGENTS.md-preferred; backlogged as canon-props): tests are example/byte-pin only →
  property-based/fuzzing for the canon layer (round-trip identity, reject noncanonical
  mutations) + StringPolicy idempotence.
- M2 reviewed (plan 2a4f03d .. accept/m2 b2e010b, 201 commits; fixes in 5ec33f7). Durable: the six
  §9 theme verdicts rest on acceptance-m2's LOCAL driver run (evidence-runs-local design,
  independently codex-re-verified). OPEN user items: SPEC §8.4
  "processing stage component(s)" prose + candidates.yaml wording (SPEC-level vocabulary call);
  `run_oracle.rs` test-oracle naming; property-based/fuzzing for the canon layer (backlog
  canon-props); shared cross-crate subprocess runner + registry symlink guard (backlog
  subproc-runner.1/.2, path-confine — all backlogged at the 2026-07-12 reset).
- Architecture reset (user directive 2026-07-12; supersedes the 2026-07-07 CNL-first
  directive; SPEC rewritten same day = design authority — read SPEC §0/§3/§10/§11, never this
  bullet, for semantics): CKC = representation-neutral research harness FIRST; ClinicalCNL =
  high-priority CANDIDATE under the same §11 promotion bar as every route. Its former
  product-surface commitments (audit views on every route, from-IR rendering, EN mirror,
  escape, findings CNL quoting, lexicon accretion) are §11.3 promotion-gated scope, no longer
  committed. Surviving decisions: name ClinicalCNL + id forms (clinical_cnl_ja.grammar,
  schema.clinical_cnl, route.single_cnl); GF adoption deferral (until JA parse of non-CKC
  text or >2 languages); expected outcomes = intended semantics fixed at corpus
  authoring (acceptance-reviewed), never route-derived; faithfulness-vs-M1-reference = diagnostic only (agreement-with-instrument);
  probabilistic-step-at-one-boundary invariant. The pre-reset §10 elaboration, 40-unit M3
  plan, and this file's former M3-plan bullet are git-resident at `ecc19d3` (SPEC §14
  retrieval note) — mine them whenever a pre-reset design is consumed (deferred-capability
  promotion, restored units like route-stage-handles/verify-eof), never re-derive; the
  reset commit's roadmap carries forward the still-live implementation pins (authored JA
  lexicon table, prefix audit, bnf facts, bridge oracle, findings-owner ruling).
- Product push (user directive 2026-07-13; third direction change 2026-07-07 CNL-first →
  2026-07-12 reset → this; conservation intact): ClinicalCNL + compiler = committed build
  scope NOW on in-tree APE fork (`clinicalcnl/`, Prolog under SWI-Prolog — 9.2.9 confirmed on
  dev machine; EN ACE profile first, JA stays mission-primary, mined from ecc19d3); target =
  clinical Prolog KB (AceRules-adapted, PROLEG-style labeled exceptions); conflict queries
  in-lane; Z3 cross-check deferred behind IR-bridge backlog item; doc supply SYNTHETIC-ONLY
  (user pick); comparison machinery + Rust CNL lane → backlog, full specs
  `git show 9b23c93:.agent/roadmap.md`; SPEC §0 honesty rule: APE line = user-directed bet,
  never described "evidence-selected". Open-ended improvement = Claude Code /loop (NOT /goal
  — /goal is end-state+judge shaped) driving /cnl-optimize rounds (protocol authority
  `.claude/commands/cnl-optimize.md`; SINGLE FILE by user consolidation 2026-07-13:
  `.claude/loop.md` deleted, loop-mode rules live in cnl-optimize.md's Running-under-/loop
  section — start as `/loop /cnl-optimize`, NEVER bare `/loop` (no loop.md ⇒ bare /loop falls
  back to Claude Code's built-in generic maintenance prompt, not this protocol); ROUNDS-ONLY
  (milestone units ALWAYS via normal /session-prompt sessions, never loop iterations): each round ONE increment → green commit `cnl-opt (R<n>)` or queue
  bank (`.agent/cnl-queue.md`), tree never dirty, continuous integration ruled over shelving,
  review batches via /codex-review never per-round; user-side enable =
  skillOverrides.loop:"on" (user edits settings themselves — agent read attempt was denied);
  loop sessions = 1M window + autoCompact ON (user-managed at loop time); round state lives
  in git+queue so between-round compaction is safe by design; 80% stop rule = fallback for
  autoCompact-off sessions. Engine-agnostic rule unchanged (LLM engines only; SWI-Prolog/APE
  nameable like Z3).
- RESPEC-COMPLETENESS: when a unit must CONSTRUCT a type, bank its CONSTRUCTOR + a mirror call site,
  not just a field list — the f-respec banked `SourceTextSpan`'s fields but not `::derive` /
  report.rs `graph` helper → cost a targeted source_linkage.rs read at f1 impl. Fixtures build
  array-order ≠ `reading_order` to prove the sort — verify EVERY parameterized case exercises the
  property: codex caught f1's direct_smt fixture accidentally pre-sorted (an identity no-op sort
  would've passed the pin; single_ir alone proved it) → a test that only half-proves its claim is a
  fake success criterion.

- Vendoring/fetch units (ape-vendor; ape-build's Clex/acetexts): verify an LGPL grant via the per-file source HEADER (`ape.pl`/`*_processor.pl` first ~25 L = the operative grant) + a `LICENSE.txt` TITLE spot-check (first ~15 L). Bank in the unit spec so execution needs ~zero rediscovery: exact pins + `HEAD^{tree}` hashes (confirm-not-rediscover), narrow-read ranges, placement via `git archive HEAD <paths>|tar -x`. Whole-file `LICENSE.txt` (41.8K ea) / big-source reads = the ~100K overflow that sank ape-vendor's first attempt.

## Archived — deep M1/M2 Rust lessons (git-resident)

M3 = Prolog under `clinicalcnl/`, Rust tree untouched → the deep M1/M2 Rust lessons are DORMANT,
verbatim at `git show e388ee4:.agent/memory.md`. Retrieve (never re-derive) the relevant block when
its surface reopens: canonical JSON (key-sort/omit-None/rename traps) + schema↔canonical + committed-
artifact hash-pin + ckc-smt serde-macro dep; model.rs adapter/cassette/model_fill; registry model
surface + experiment pipeline-set binding; run-loop route/tails/event-scope/replay/report; metrics +
record-mode prompt composition; engine-agnostic synthetic-token audit; recorded-run + acceptance-
driver patterns; the whole `## Runtime` (greedy/constraint/truncation; machine specifics in gitignored
`.agent/runtime.local.md`); retired vocab rulings (component-vs-pipeline-step, "oracle" naming — their
OPEN user items survive in the M1/M2-reviewed bullets above). Triggers: Rust tree reopens (backlog IR
bridge, Rust CNL lane, hardening, canon-props); a report cites APE-line results beyond §10.6; or an M3
acceptance/review session needs the audit / acceptance-driver / recorded-run patterns.
