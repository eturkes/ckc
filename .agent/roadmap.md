# CKC roadmap

Build plan for /session-prompt — the session protocol, bookkeeping format, and stamp
semantics live in that command; SPEC.md is the design authority, its §2 the milestone
sequence. One milestone at a time: header `## <name> — plan <hash> — review <hash>` over an
ordered unit checklist; unchecked lines carry the full unit spec; checked items collapse to
one-line stubs `- [x] <id>: <gist>. NN% NNNK/200K <hash>`; a reviewed milestone keeps its
stubs until the next milestone is planned (that planning reads them to right-size units),
then persists as a bare header; git history retains all removed text. The active milestone's
`plan <hash>` shows `PENDING` until its first unit's closing commit fills it — the planning
commit is then known (M1's `89c4cba` was filled retroactively too). The trailing Backlog
section is NOT a milestone — it holds triggered, unscheduled items a future planning session
may pull in.

## M1 scaffold — plan 89c4cba — accept m1 — review f6d68a0

## M2 multi-hop PoC — plan 2a4f03d — accept m2 — review 5ec33f7

## M3 route comparison (ClinicalCNL slice) — plan PENDING

Scope = SPEC §10 (2026-07-12 architecture reset = design authority; the pre-reset §10/M3
plan is git-resident at `ecc19d3` — SPEC §14 retrieval note). Deliverable: `route.single_cnl`
= ClinicalCNL v1 slice (§10.4 — JA-only, closed lexicon, no escape, parse → bridge →
ClinicalIR, no from-IR rendering) compared against the landed §9 pair in `exp.m3_cnl` over
the locked M1 inputs + reference under the §10.1–§10.2 neutral contract, with faithfulness /
round-trip / resource instruments and the run explorer. Milestone gate (model runtime) binds
ONLY cnl-grammar-smoke + record-cnl (recorded MET at pre-reset planning — identity probe
clean, contract-conformant; re-confirm functionally at each gated unit). M1/M2 pins stay
byte-identical except two scheduled re-blesses: lexicon_hash-carrying value pins
(lexicon-cnl-fields) and report/metrics pins (metrics-faithful). Engine-agnostic rule applies
to every committed byte.

Cross-unit decisions (SPEC §10.4 carries the semantics; these are the implementation pins):
- Module homes: normalize.rs takes ONLY mechanical optional serde fields (2166L file — keep
  edits tiny); ALL new logic in fresh ckc-cli modules `cnl_lexicon.rs` / `cnl.rs` /
  `cnl_grammar.rs` / `cnl_parse.rs` / `cnl_render.rs` / `cnl_bridge.rs`; ckc-core untouched
  except the DiagnosticCode fieldless_enum append (`cnl_round_trip_mismatch`) and the two
  reset-adjusted ir.rs doc comments (already landed with the reset commit).
- Typed role view (`cnl_lexicon.rs`): the single source every CNL module reads; construction
  hard-errors on any integrity/lint finding, so a dirty lexicon produces no CNL anywhere.
  Roles pin test on committed data: pop.* → population, cond.* → condition, drug.abx_a →
  action_target, q.age_years → population (keeps the frozen normalize.rs prefix partition
  and the role-driven bridge in agreement on the locked corpus by data).
- Grammar: bnf 0.6 (existing workspace pin; add as ckc-cli dev-dep for oracle tests) — no
  postfix repetition, so document = the right-recursive `(rule <nl>)+` lowering with
  literal-LF `<nl>` (smt_query.grammar pattern); ckc-smt emit.rs holds the two bnf API
  pitfalls — copy its working pattern; the Earley oracle proves language MEMBERSHIP only
  (superset), so lexer determinism is guarded by the prefix-freedom lint, and single-parse
  asserts use `parse_input().take(2)`. Basis-id production = the pinned Id-exact shape
  (`<basis-id> ::= <lower> <basis-id-rest>`; rest chars lower|digit|`_`|`.`|`:`|`-`) — a
  bare one-or-more-chars production over-admits `1r`/`.r`/`:r`/`-r`/`_r`. ONE declared
  over-approximation class: numerals beyond i64 (parser-bounded; leading-zero-free register
  grammar-side, `0` bare).
- Interval validity battery (shared table, AST + parser sides): 16 bound-presence masks ×
  per-bound values {-1,0,1}, valid iff exactly one bound present ∧ value ≥ 0.
- Bridge worked oracle (2-disjunct rule × 2 exception sentences + trailing 1-disjunct rule):
  stmt.0 owns exc.0/exc.1, stmt.1 owns exc.2/exc.3, clone content + basis duplicated per
  statement, counters document-continuous (the trailing rule catches per-rule resets);
  enumerated bind.<k> → concept oracle beside it.
- Findings owner (first run with two bundle-bearing routes; landed §7.1 mint is
  single-view): owner SELECTION over id qualification — the findings view consumes the
  first bundle-bearing pipeline in experiment binding order (single_ir); non-owner compiled
  routes' groups/bundles land route-namespaced in artifacts + trace (shared-out collision:
  both routes otherwise write `groups/<gid>/verifier_results.json`); payload ids stay
  unprefixed; report field `findings_owner_pipeline_id` (§7.2). Discriminating fixtures per
  memory's selector-semantics lesson.
- Authored JA lexicon data (transcribe at lexicon-cnl-fields; chosen against the §10.4
  decree + prefix law): concepts as adnominal_ja / negated_ja — pop.adult 成人 / 非成人;
  pop.child 小児 / 非小児 (both interval-carrying: negated forms parse-only under the
  negative-occurrence bar); cond.sepsis 敗血症のある / 敗血症のない; cond.renal_severe
  重度腎機能障害のある / 重度腎機能障害のない; cond.pregnancy 妊娠中である / 妊娠中でない
  (copula pair — trailing-の bar rejects 妊娠中の, and bare 妊娠中+でない would
  prefix-collide); drug.abx_a action_target-only → no adnominal/negated, citation =
  surfaces[0] 抗菌薬A. Quantity row q.age_years {role population, surface_ja 年齢, unit_ja
  歳}. act.administer noun_ja 投与. Modality tail_ja: (for,strong) を強く推奨する;
  (for,weak) を提案する; (permit,weak) を考慮してもよい; (against,strong) を推奨しない;
  (against,weak) を提案しない; (contraindicate,strong) tail on the 禁忌 row は禁忌である
  while 投与しないこと stays deliberately tail-less (its surface embeds the action verb;
  first tail-bearing row = 禁忌 = canonical). Certainty renders committed surfaces[0], no
  new field. Prefix audit banked (re-verify mechanically at lexicon-cnl-view over the
  composed inventory): the JA token set above + fixed terminals + digits is pairwise
  proper-prefix-free — load-bearing: 非-prefix negations, ある/ない + である/でない
  divergence inside clause forms, tails all を/は-led.

- [ ] lexicon-cnl-fields: mechanical loader extension + authored data. normalize.rs Lexicon
  row structs gain optional fields (deny_unknown_fields holds; JA surfaces via
  StringPolicy::SemanticJa): concept `adnominal_ja`/`negated_ja`/`roles`, action `noun_ja`,
  modality `tail_ja`, NEW quantity table `{var_id, role, surface_ja, unit_ja}`; ckc-core's
  independent strict YAML mirror (test_sources_m1.rs Lexicon structs) gains the same
  optional fields + table (reddens otherwise). ja_core.yaml authored per the plan-header
  data table. Tests: loader fixtures, round-trip, unknown-field rejection, committed-lexicon
  loads green. Gate: full suites + lexicon_hash-carrying value pins re-blessed (grep the
  observed-bless literals) — the ONLY scheduled re-bless here.
- [ ] lexicon-cnl-view: fresh cnl_lexicon.rs — typed role view (per-slot concept surface
  sets: context slots serve adnominal_ja/negated_ja, exception slot adnominal-only, target
  slot the surfaces[0] citation; quantity-role lookup; canonical tail/certainty rows) +
  integrity hard-errors (refs resolve; role sets nonempty/deduped/legal with
  population/condition exclusive; quantity var set == used interval vars, exactly one row
  per var, orphan rows error; ≥1 tail-bearing row per present (direction,strength) pair,
  first = canonical, pin vs the authored tails; noun_ja present where act.* referenced;
  per-language duplicate-literal rejection by SEMANTIC TOKEN — Concept(row) /
  NegatedConcept(row) / ActionNoun(row) / Tail(dir,strength) / Certainty(value) /
  QuantityVar(var) / var-free Unit(literal) / Fixed(terminal) / Digit(char); same-token
  occurrences dedup, cross-token collisions reject naming both) + lint (reserved-terminal
  collisions — connectives かつ/、または, punctuation 、。「」 brackets parens, backtick;
  trailing-の on adnominal/negated (fixture 妊娠中の); pairwise proper-prefix-freedom
  across ALL lexer-visible tokens incl. Fixed + Digit) + the typed fixed-terminal/token
  inventory module the grammar emitter, parser, and renderer all consume (single source).
  View construction hard-errors on ANY finding. Tests: per-rule rejection battery over
  bad-lexicon fixtures + positive controls (multi-role surface across its slots, shared
  unit literal on two quantity rows, same-pair duplicate tail) + committed-data
  zero-findings + roles pin. Committed bytes untouched.
- [ ] cnl-ast: fresh cnl.rs — CnlAtom {Concept|ConceptNegated|Interval} / CnlContext /
  CnlException / CnlRule / CnlDocument (§5 row members: document_id, grammar id+hash,
  per-rule AST + canonical JA text, text hash) + Canonical emit/read (sorted-key slots,
  omit-None; all-None + populated byte pins per memory's extension pattern) + two-layer
  validate: STRUCTURAL lexicon-free (nonempty rules/DNF/brackets; brackets
  sorted+deduplicated; Id grammar; interval register via the 16×3 truth table; per-rule
  texts line-break-free — LF and CR; text hash recomputes == hash_bytes(concat(rule_text +
  LF)) — frame executable) then lexicon-scoped vs the role view (refs resolve, slot roles
  admit positions, tail-backed pair, interval vars resolve + dangling-var case,
  negated/exception refs interval-free) + rejection battery + one mixed
  structural-and-lexicon case pinning layer precedence (structural reports) + round-trip
  tests. No run.rs contact. Seam if oversized: type family + Canonical | validate + battery.
- [ ] cnl-grammar: cnl_grammar.rs emitter `clinical_cnl_grammar(&view) -> Vec<u8>` (BNF,
  smt_query.grammar dialect + `;` comments): document/rule productions per §10.4 slot
  shapes; DNF connectives with precedence by production shape; mid vs patient-adjacent atom
  alternations threading の before 患者 (interval atoms); per-quantity-row interval
  productions (each var's surface paired with its own unit terminal — the sole unit↔var
  binding); numeral register; basis-id production per the plan header; terminals =
  whole-surface literals in slot-specific alternations from the role view + the inventory
  module's fixed terminals; hard-error on a lint-dirty lexicon (view gate). In-crate bnf
  oracle battery (dev-dep): worked §8.6-shape sentences full-match, trailing-garbage
  reject, wrong-slot-surface reject, swapped-unit reject, numeral membership (`0` accepts,
  `05` rejects), basis-id membership over a shared accept/reject corpus (one-letter id +
  every-rest-char composite; reject `1r`/`.r`/`:r`/`-r`/`_r`/uppercase/slash), の
  present/missing/stray cases, multi-rule + exception coverage, take(2) single-parse spot
  asserts. Then committed schemas/clinical_cnl_ja.grammar + schemas.yaml entry
  `schema.clinical_cnl` + never-writes drift guard + `#[ignore]`d bless + hash-pin const +
  `.gitattributes` eol=lf + registry check green + committed_model_surface_checks_ok
  extension. Seam if oversized: emitter + oracle battery | committed bytes + registry.
- [ ] cnl-grammar-smoke (gated: model runtime): feasibility stop-the-line BEFORE parser
  work — env wrapper compiles the emitted JA grammar (scratch dump, uncommitted) as a
  decoding constraint + bounded constrained emissions with ASSERTED coverage: one output
  containing a chosen multibyte lexicon terminal verbatim; textual checks only (parser
  lands later). Proves multibyte whole-surface terminals survive the constraint mechanism.
  Machine specifics → runtime.local.md; committed bytes engine-agnostic. A failure stops
  the line: grammar/dialect redesign reaches the user before further CNL units.
- [ ] cnl-parse.1: cnl_parse.rs token layer + context parser — longest-match lexer over the
  inventory module + role-view surfaces (JA table); atoms concept/negated/interval with
  slot legality from the role view; かつ binds tighter than 、または. Rejection battery:
  bare off-lexicon surface, wrong-slot registered surface, malformed interval bounds,
  の-position violations, hand-oracled four-bound fixture (以上/以下/未満/超 ↔ ge/le/lt/gt
  — expected values hand-written, never derived from the mapping under test), wrong-unit-
  for-var, numeral overflow boundary (i64::MAX parses, +1 = repairable error), leading-zero
  numerals reject (`05`/`00`; bare `0` parses), connective misuse, mid-token truncation.
- [ ] cnl-parse.2: document parser — full slot order per §10.4, multi-rule documents under
  the document frame (exactly one LF per rule, last included), single deterministic pass;
  parser normalizes per-bracket basis sorted+deduplicated at AST build (+ normalization
  pin: unsorted/duplicate surface parses to the sorted-deduplicated AST); malformed battery
  (duplicate/missing slots, unterminated/empty bracket, out-of-grammar basis id + the
  shared basis-id accept corpus embedded in well-formed sentences, exception sentence
  missing its bracket, connective/negated/interval inside an exception, empty document,
  missing terminal LF, blank line, CRLF + lone-CR, stray whitespace — all repairable);
  differential accept/reject agreement vs the bnf Earley oracle over the full battery +
  bounded generated corpus.
- [ ] cnl-render: cnl_render.rs — canonical JA text per AST (stored ContextExpr order;
  bracket ids sorted; document frame assembly) asserting STRUCTURAL validity fail-closed;
  round-trip properties via bounded deterministic AST enumeration (no new deps):
  parse∘render == id over valid ASTs, render∘parse == id over canonical documents; frame
  pins (LF terminator, line-break-free rule texts).
- [ ] cnl-bridge: cnl_bridge.rs — `to_ir(&CnlDocument, &segments) -> Result<ClinicalIr,
  FillReject>` per §10.4 (disjunct split; role partition; exception cloning + statement-
  major exc.<k>; bind.<k> first-reference mint in post-split emission order; binding
  region_ids = sorted union of citing statements' cited regions; statement source refs =
  its rule's brackets' union; source_segment_ids via segments reverse map; preconditions:
  cited region resolves — Grounding reject — and anchors in exactly one segment — Schema
  reject naming the region) + `single_cnl_accept` closure (UTF-8 → parse → validate →
  re-render/re-parse self-check → to_ir → grounding; Schema rejects repairable,
  cnl_round_trip_mismatch terminal) + the worked 2×2 oracle + per-reject battery + bundle
  validity of emitted IR (IrBundle::validate green over the golden path). Seam if
  oversized: to_ir + oracle | accept closure + battery.
- [ ] route-single-cnl.1: registry + stage wiring — candidates.yaml entries
  processing_stage.m3.model_fill_cnl (nondeterministic, [source_document_graph, segments] →
  [cnl_document]) + processing_stage.m3.bridge (deterministic, [cnl_document, segments] →
  [clinical_ir]) + pipe.m3_single_cnl chain per §10.4; experiments.yaml exp.m3_cnl
  (pipelines [pipe.m2_direct_smt, pipe.m2_single_ir, pipe.m3_single_cnl], baseline
  direct_smt, M1 groups/seed/budgets/reference); prompts.yaml CNL prompt entry (template
  composes source + segments + region ids + CNL vocabulary — run-m2.2a grounding-scaffold
  pattern); RouteShape::SingleCnl fingerprint + resolve battery; fill fn (model_fill core +
  single_cnl_accept) + bridge stage fn + per-doc landing under the route namespace +
  §4.6 events + §4.4 wrappers per the §10.4 provenance chain. registry check green.
- [ ] route-single-cnl.2: run-loop integration — execute_routes dispatches the SingleCnl
  view through fill → bridge → m2.assemble → m1.compile → m1.verify; TWO bundle-bearing
  routes land without collision (route-namespaced group dirs + bundles; memory's
  cross-route landing lesson — single-route tests never cover it, so the gate here is a
  two-bundle-route fixture run); findings owner = first bundle-bearing pipeline in binding
  order + `findings_owner_pipeline_id` populated in report.json (§7.2) + misalignment-
  discriminating fixture (non-owner sorts first / carries different same-numbered rule);
  trace/census over the 3-route plan; event/landing pin battery.
- [ ] metrics-faithful: metrics.rs additions per §7.3 — `ir_match` faithfulness (both
  IR-landing routes; projection per §10.4: provenance-grain fields excluded, everything
  else exact on canonical bytes; applicability ATOMIC — present ⇔ accepted IR-landing fill,
  violation = fail-closed instrument error, `not_applicable` for direct_smt),
  round_trip_identity_rate (accepted CNL docs re-parse to stored AST + re-render
  byte-identical), accepted-emission byte-size rows (all routes); report.json/md rows +
  M1/M2 report byte-pin re-bless (scheduled) + rendered-body consts re-bless.
- [ ] explorer: `ckc explore runs/<run-id> --out <path>.html` per §10.5 — one deterministic
  self-contained HTML (in-crate template consts, embedded canonical JSON, no network/JS
  deps beyond inline vanilla): per document × route the chain source spans → route-native
  accepted artifacts (CNL text / IR JSON / SMT) → verdicts; findings + metrics raw rows +
  delta table; view-only posture (renderer identity embedded; writes only the named output
  file, never under the run's attested set). Tests: byte-stability (two builds identical),
  chain-content asserts over an in-test M1 fixture run + a replayed recorded run; dispatch
  battery (missing run dir, malformed report).
- [ ] record-cnl (gated: model runtime, LIVE): re-confirm gate (identity probe), then record
  exp.m3_cnl into a scratch root with a newly created EMPTY cassette store (persist
  overwrites same-key without compare); census route.single_cnl/** vs the run's
  model-attempt ledger BOTH directions; identity-agreement vs existing cassettes — agree ⇒
  M2 cassettes stand, drift ⇒ full re-record + M2 recorded_run re-bless (documented
  fallback); replay matched; copy into committed /cassettes REPLACING the
  route.single_cnl subtree, never merging; do-not-read sync verified (census via runtime
  indirection — /cassettes is deny-Read). Prompt/constraint refinement from observation
  lands here (tokenizer/truncation lessons per memory's §9 bullets).
- [ ] recorded-cnl-battery: exp.m3_cnl recorded-run battery — census / §9 manifest fields /
  replay-matched / audit-file + explorer pins over the committed cassettes (recorded_run.rs
  pattern, own test file — M2 battery untouched).
- [ ] acceptance-m3: §10.5 acceptance themes against the recorded run via the
  acceptance-driver pattern (memory) — 3-route raw-before-delta, determinism laws green,
  round-trip 1.0 on accepted docs, faithfulness rows for both IR-landing routes (measured,
  never gated), expected outcomes per reference, replay byte-stability, drift guards,
  explorer renders every route's chain, §0 vocabulary; promotion-evidence summary (§11.1
  criteria rows over the run, cited by run id) appended to the report or a standalone md;
  tag accept/m3.

## Backlog — NOT a milestone (unscheduled; schedule by trigger; full pre-reset unit specs at `git show ecc19d3:.agent/roadmap.md`)

- Hardening — trigger: before any run whose evidence leaves the operator-controlled tree, or
  the first milestone reworking model.rs/verify.rs: spawn-retry (fs-portable ETXTBSY retry
  tests via injectable spawn op + clock; suite GREEN on the dev fs 2026-07-12, 4/4 pass —
  portability only); path-confine (registry-path containment resolver; review-reproduced
  absolute-path escape — operator-owned working copy, so evidence-quality impact local);
  input-snapshot.1–.3 + constraint-snapshot (read-once input layer; manifest attests
  resolution-time bytes; staged frozen constraints); subproc-runner.1/.2 (shared subprocess
  runner + drain-cap/EOF-verdict hardening — memory's model-runtime bullet carries the
  rulings).
- canon-props: canon-layer generated-case harness (AGENTS.md-preferred property hardening) —
  schedule after M3's last canonical-shape change.
- Deferred CNL capabilities: SPEC §11.3 (EN mirror, escape, from-IR rendering, findings CNL
  quoting, lexicon accretion) — promotion-gated, never scheduled without §11 evidence.
