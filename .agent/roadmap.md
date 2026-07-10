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

## M2 multi-hop PoC — plan 2a4f03d — accept m2 — review 5ec33f7

## M3 ClinicalCNL v1 — plan PENDING

Scope = SPEC §10 (2026-07-07 elaboration = design authority; its worked text illustrative
until the committed grammars land): bilingual ClinicalCNL — JA primary, EN mirror — two
concrete syntaxes over ClinicalIR statement content, parse/render mutual inverses;
`route.single_cnl` third route; audit views rendered from accepted IR on every route (M1's
included); `exp.m3_cnl` binds [pipe.m2_direct_smt (baseline), pipe.m2_single_ir,
pipe.m3_single_cnl] over the locked M1 inputs, scored by the same reference. Milestone gate
(model runtime) MET at planning — identity probe clean, contract-conformant; no §15 gate
(locked measurements stand alone). Engine-agnostic rule applies to every committed byte.
Cross-unit decisions (durable copy in memory's M3-plan bullet):
- Module home = ckc-cli fresh modules (`lexicon.rs` — lexicon-extract moves the Lexicon
  family out of ckc-cli::normalize (2166L) as the CNL modules' neutral dependency point —
  plus `cnl.rs`, `cnl_grammar.rs`, `cnl_parse.rs`, `cnl_render.rs`, `cnl_bridge.rs`):
  Canonical impls outside core proven (report.rs), consumers all CLI-side. ckc-core IR shapes + committed
  clinical_ir.schema.json UNTOUCHED (no IR field change — ClinicalStatement already carries
  certainty/exceptions/source refs); sole core touch = DiagnosticCode fieldless_enum append
  (codes-cnl).
- CNL AST ≠ ClinicalIr: CnlAtom {Concept|ConceptNegated|Interval|Unregistered(surface)}
  (escape = own variant), CnlConceptRef {Registered(Id)|Unregistered(surface)} — §10 admits
  the escape in EVERY concept slot incl. action target — CnlContext {any: Vec<Vec<CnlAtom>>}
  flat two-level DNF, CnlException (DNF), CnlRule {context, action kind + target:
  CnlConceptRef, direction+strength, certainty?, exceptions, basis region refs}, CnlDocument
  = the landed model_fill artifact payload per §5's table: document_id + grammar id/hash refs
  + rules (AST + per-rule canonical text ja/en) + per-language text hashes — accept re-renders
  and hash-locks canonical bytes beside the AST (§10); report.json cites the artifact's
  hashes. Parser mints NO ids; bridge derives them.
- Bridge determinism: one ClinicalStatement per context-disjunct (ids `stmt.<k>`/`exc.<k>`/
  `bind.<k>` document-order counters mirroring normalize.rs's mints EXACTLY — §8.6 reserves
  `<doc>.rule.<k>` for norm-layer rule ids; disjunct split appends statements in document
  order); population-vs-condition partition by lexicon
  id namespace (`pop.*` → population, else condition); each exception disjunct → one
  ExceptionClause; bindings = one Exact-status TerminologyBinding per distinct referenced
  concept, minted in first-reference document order (matches M1's first-mention scan order on
  the locked corpus; divergence = measured ir_match miss, never asserted), system =
  lexicon.system, code = concept id, region_ids = the citing rules' basis regions — a
  KNOWINGLY lossy reconstruction of M1's mention-based regions, hence metrics-faithful
  compares under the §10 projection excluding binding region_ids (Action::new derives
  key itself); basis refs = region ids; source_segment_ids derived region→segment via the
  segments artifact (ClinicalSegment.region_ids reverse map — m3.bridge stage inputs therefore
  [cnl_document, segments]). Round-trip laws, precise: from_ir = one single-disjunct CNL rule
  per statement (projection, no regrouping) ⇒ from_ir∘to_ir == disjunct-split normal form
  (== id exactly on already-split docs); to_ir∘from_ir == id on bridge-image IR.
- Grammar terminals = whole-surface string literals (ASCII digits + basis-id chars as literal
  alternation) — portable to LLM constraint mechanisms + atomic in bnf — with EXACTLY ONE open
  lexical production per language: the escape's free quoted surface (§10) is inexpressible as
  finite literals → dialect open-content notation decided at cnl-grammar.1, portability risk
  contained there; emitter takes an escape mode — Committed(open) vs
  OracleBound(enumerated test surfaces) — since bnf parses literals only; cnl-grammar.1b
  probes the open-production portability risk, record-cnl.1 audits the full terminal set. bnf 0.6
  (existing workspace pin) verified unicode-capable, byte-offset whole-terminal matching;
  its Earley oracle proves language MEMBERSHIP (superset — explores all segmentations), so
  lexer segmentation determinism is guarded by the lexicon prefix-overlap lint instead;
  single-parse asserts use `parse_input().take(2)`, never full counts.
- Record strategy: record exp.m3_cnl into a scratch root, copy `route.single_cnl/**` into the
  committed /cassettes (keys disjoint from M2's); identity-agreement vs the existing cassettes
  decides — agree ⇒ done, drift ⇒ full re-record + M2 recorded_run re-bless (documented
  fallback).
- Known deliberate re-bless costs, scheduled in their units: ja_core.yaml growth →
  lexicon_hash-carrying value pins (lexicon-cnl.1); report CNL population → M1/M2 report
  byte-pins + rendered-body consts (report-cnl.2/.3).
- 2026-07-10 external pre-M3 review absorbed, then codex follow-up hardened it; §10 (amended
  in the same two commits) is the authority for: text↔AST inverse laws quantified over the
  parser-accepted language + AST↔ClinicalIR bridge (ids stmt/exc/bind mirroring normalize.rs;
  faithfulness = projection match, binding region_ids excluded), escape payload contract,
  accept-side renderability closure, lexicon tail fields + integrity set, audit views keyed
  (pipeline, document). Units inserted: path-confine + spawn-retry open the milestone,
  cnl-grammar.1b = early runtime probe, accept-total after cnl-render, subproc-runner.1/.2
  before route-single-cnl.3 (live wiring).
- 2026-07-10 second external review (validation pass on fadc674) absorbed: reproduced the
  input multi-read attestation defect — resolve, corpus/lexicon reads, record setup,
  model_route_metrics, and manifest_inputs each reopen registry-named paths, so mutating
  registry/corpora.yaml between resolution and manifest assembly yields an `ok` run whose
  manifest.json attests bytes the execution never used → units input-snapshot.1–.3 directly
  after path-confine. Named stage handles replace the positional [Id; 8]→[Id; 9] widening
  plan → unit route-stage-handles before route-single-cnl.1. lexicon-extract inserted before
  lexicon-cnl.1 (the CNL modules' neutral dependency point); spawn-retry gains the shared
  retry-policy-helper option; memory's stale bridge-id + ETXTBSY texts corrected.

- [ ] path-confine: pre-M3 hardening (review-reproduced: absolute corpus + expected_outcomes
  paths under /tmp pass `registry check` AND a full run — `Path::join` swallows the root on an
  absolute rhs and keeps `..`). Core: validate_registries extends the schemas/prompts
  is_safe_relative_path finding (UnsafePath) to corpus.path + experiment.expected_outcomes.
  CLI: ONE I/O resolver (lexical check → canonicalize root + existing candidate → candidate
  strictly under root → regular file; a failed resolve lands the diagnostic and the read is
  skipped) applied to every registry-data-controlled read — registry_check.rs
  expected_outcomes ref loads + schema/prompt byte reads, run.rs corpus entry.path (×2 call
  sites), expected_outcomes (×2), record-path template/constraint reads; fixed-name reads
  (CORPORA_FILE, LEXICON_FILE, …) are code constants, out of scope. Tests: absolute / `..` /
  in-repo symlink→outside (`#[cfg(unix)]`) / valid nested file accepted / non-regular
  rejected — across registry check + run. Gate: full gates + registry check green on the
  committed tree.
- [ ] input-snapshot.1: read-once input layer, fresh module
  crates/ckc-cli/src/input_snapshot.rs. ResolvedFile<T> {canonical path, bytes, hash =
  hash_bytes(bytes), value: T parsed FROM those bytes} + run-level InputSnapshot {corpora,
  candidates, experiments, lockfile + toolchain (raw), lexicon, expected_outcomes, per-entry
  corpus bytes, schemas? + prompts? (populated iff the resolved set contains model routes —
  the §9 manifest gate signal, so the M1 read surface is unchanged)} + a builder in
  resolution order (registries → experiment selection → selected slots; immutable once
  resolution completes). Registry-data-controlled paths load through path-confine's
  resolver; fixed-name files join as today; parse fns reused (ckc-core registry parse_*,
  normalize load_lexicon). Per-doc corpus slots Option — a failed read carries today's
  file+reason payload and the doc skips downstream. Unit tests: loader errors, hash =
  the file bytes' sha256, confinement rejection through the resolver. No run.rs contact.
- [ ] input-snapshot.2: resolution + execution consume the snapshot — resolve() takes the
  built snapshot instead of load()ing CORPORA/candidates/EXPERIMENTS + TOOLCHAIN itself; the
  two corpus entry.path head-fn reads, the two LEXICON_FILE reads, and build_record_setup's
  SCHEMAS/PROMPTS loads become snapshot lookups (root stays for outputs;
  build_record_parts' template/constraint payload reads stay put — record-gated, pre-write
  byte-verified, path-confined). Same bytes on a stable tree ⇒ gate: full suites green with
  M1/M2 byte-pins UNCHANGED; test edits confined to input-error paths whose diagnostics move
  to snapshot-build time (file+reason shape preserved).
- [ ] input-snapshot.3: metrics + manifest attest the snapshot — model_route_metrics (today
  rereads EXPERIMENTS + expected_outcomes) and manifest_inputs (today rereads LOCKFILE/
  CORPORA/EXPERIMENTS/expected_outcomes/SCHEMAS/PROMPTS) consume snapshot fields; manifest
  hashes = the snapshot's byte hashes (stable tree ⇒ identical bytes, pins green).
  Regression test: an explicit test-only barrier hook reachable from run.rs's in-crate tests
  (NO sleeps) mutates registry/corpora.yaml between resolution and manifest assembly — the
  run stays `ok` AND manifest.json attests the resolution-time bytes (today it attests the
  mutated file while the artifacts used the original mapping — the review-reproduced flip).
- [ ] spawn-retry: pre-M3 hardening — both spawn_piped_surfaces_persistent_etxtbsy tests
  (model.rs, verify.rs) assert the FILESYSTEM produces ETXTBSY (fs-dependent, BOTH outcomes
  observed: some filesystems yield ETXTBSY, others — overlayfs among them — let the spawn
  succeed), and the retries_through twins pass vacuously
  wherever the first spawn succeeds. Rework BOTH mirrored copies (full machinery extraction
  stays subproc-runner.1): retry loop parameterized over an injectable spawn-attempt op
  (prod path = the real Command spawn); a small shared retry-policy helper — home ckc-smt,
  ckc-cli already depends on it — MAY host the loop once so the injected seam isn't
  implemented twice; deterministic tests drive ETXTBSY,ETXTBSY,success /
  persistent-through-grace / immediate non-ETXTBSY error / immediate success; happy-path
  process integration tests stay; ≤1 fs
  integration test per crate, capability-probing the mount and skipping cleanly where the fs
  cannot produce ETXTBSY. Gate: workspace strict suite green with zero
  environment-dependent outcomes.
- [ ] lexicon-extract: behavior-locked move — the Lexicon family (types, YAML row structs,
  load_lexicon, validation + their tests) leaves normalize.rs (2166L) for fresh
  crates/ckc-cli/src/lexicon.rs, normalize re-imports; the neutral dependency point
  lexicon-cnl.*, cnl_grammar, cnl_parse, cnl_render, cnl_bridge all consume. Zero public
  behavior change: existing suites the gate, assertion surface untouched (import edits only).
- [ ] lexicon-cnl.1: CNL surface fields — LexiconConcept +adnominal_ja/negated_ja/gloss_en,
  LexiconAction +noun_ja/noun_en, LexiconModality +tail_ja/tail_en (canonical deontic tails ≠
  source-match surfaces per §10 — optional per row, parse-accepted synonyms when present),
  LexiconCertainty +surface_en,
  NEW LexiconQuantity {var_id, surface_ja, unit_ja, surface_en, unit_en} table; load_lexicon
  strict extension (deny_unknown_fields holds; JA surfaces through StringPolicy::SemanticJa,
  EN surfaces through SemanticEn — §10 EN canonical text is ASCII-lowercase; §10 integrity
  hard-errors NEW: implies_action resolves to an action entry, quantity var_ids unique,
  exactly one quantity row per interval var any concept uses, nonempty normalized
  surfaces+units both languages, per-language exact-duplicate parse terminals rejected across
  ALL CNL surface fields + lexer categories, every (direction,strength) pair present carries
  ≥1 tail-bearing row — first tail-bearing row per pair = canonical render row, a test pins it
  against §10's worked tails (を強く推奨する / は禁忌である) — and concept intervals
  CNL-representable, v1 one unsigned bound); ja_core.yaml
  authored for the full M1 set (6 concepts,
  act.administer, 7 modality rows, certainty rows, q.age_years). Gate: load/normalize tests
  green + lexicon_hash-carrying value pins re-blessed (grep the observed-bless literals) +
  full gates.
- [ ] lexicon-cnl.2: CNL lexicon lint — reserved-token collisions (a surface containing a
  connective/punctuation grammar terminal), missing-CNL-surface findings, same-category
  proper-prefix overlap (segmentation determinism — the Earley-superset caveat's guard) +
  per-variant rejection battery over bad-lexicon fixtures. Pure findings layer beside
  load_lexicon's existing checks.
- [ ] cnl-ast: cnl.rs type family — CnlAtom/CnlConceptRef/CnlContext/CnlException/CnlRule/
  CnlDocument (grammar refs + per-rule text + text-hash members per the plan header) +
  Canonical emit/read (sorted-key slots, optional members omit-None) + validate (nonempty
  rules, Id grammar, interval bound coherence mirroring ir.rs, §10 escape payload contract —
  nonempty ≤80 scalars, single line, control/quote-delimiter chars excluded,
  SemanticJa-normal fixpoint; + lexicon-scoped validity vs a passed pair/id view: modality
  pair tail-backed, concept/action refs resolved — makes §10's lexicon-valid-AST quantifier
  well-defined) + all-None/populated byte pins
  + round-trip tests. Fresh module, no run.rs contact.
- [ ] cnl-grammar.1: cnl_grammar.rs emitter — clinical_cnl_grammar(lexicon, lang) -> Vec<u8>
  BNF (smt_query.grammar dialect, `;` comments): document = rule+; rule = context 患者には、
  action deontic-tail / optional certainty paren / ただし-exceptions / [根拠 …] basis; DNF
  connectives かつ / 、または with precedence by production shape; atoms
  concept|negated|interval|escape (未登録概念「…」 / unregistered concept "…"; admitted in
  action-target position too — §10); EN mirror productions; terminals = lexicon whole-surface
  literals; interval numerals = ASCII-digit literal alternation; escape quoted-surface content
  = the single open production (plan header — emitter escape mode Committed|OracleBound;
  payload contract per §10, parser-enforced — the production stays open).
  Oracle tests in-crate (bnf workspace dev-dep added to ckc-cli, OracleBound grammars): §10
  worked examples full-match both languages, trailing-garbage reject, per-production coverage
  incl. escape/interval/multi-rule, take(2) single-parse spot asserts. ckc-smt emit.rs's two
  bnf API pitfalls apply — copy its working pattern (a fresh derivation from bnf docs re-hits them).
- [ ] cnl-grammar.1b (gated: model runtime): runtime grammar feasibility smoke — env wrapper
  compiles the emitter's JA grammar (scratch dump, uncommitted) + bounded constrained
  emissions with ASSERTED coverage — one output containing a chosen multibyte lexicon terminal
  verbatim, one forced-escape output (minimal sub-grammar or steering prompt) shaped
  未登録概念「…」 with an in-contract payload; textual checks, the parser lands later — proving
  multibyte whole-surface terminals and the open escape production survive the constraint
  mechanism (§10 validation-program probe). Feasibility only — full tokenizer
  audit, repetition/degeneration stress, template refinement stay record-cnl.1. Machine
  specifics → runtime.local.md; committed bytes engine-agnostic. A failure stops the line:
  grammar/dialect redesign reaches the user BEFORE cnl-grammar.2 commits grammar bytes.
- [ ] cnl-grammar.2: committed schemas/clinical_cnl_{ja,en}.grammar (ignored bless +
  never-writes drift guard + hash-pin consts — schema.rs pattern) + schemas.yaml entry
  schema.clinical_cnl = the JA grammar, §10's singular registry id (the route's decoding
  constraint; EN grammar stays committed + drift-guarded + hash-pinned, registry entry only
  if the coverage check demands one) + .gitattributes eol=lf + registry check green +
  committed_model_surface_checks_ok drift-guard extension.
- [ ] cnl-parse.1: cnl_parse.rs token layer + context-DNF parser — longest-match lexer over
  per-language token tables (lexicon surfaces + fixed terminals + digits; tables differ,
  parser shared), atoms concept/negated/interval/escape, かつ binds tighter than 、または;
  rejection battery: bare off-lexicon surface = parse error (≠ escaped accept), malformed
  interval bounds, connective misuse, mid-token truncation, escape-payload contract
  violations (empty / over-80-scalars / control or quote-delimiter chars — plain parse
  errors, repairable).
- [ ] cnl-parse.2: document parser — full slot order (context 患者には、 action deontic tail /
  certainty paren / ただし exceptions / basis bracket), multi-rule documents, single
  deterministic pass (no backtracking); malformed battery (duplicate/missing slots,
  unterminated bracket, empty document); differential accept/reject agreement vs the Earley
  oracle over this unit's corpus.
- [ ] cnl-render: cnl_render.rs — render_ja/render_en canonical text (modality pair → the
  pair's canonical tail — the first tail-bearing row per pair — per language, basis sorted,
  certainty optional
  paren, stored DNF order preserved — canonicalization never reorders semantics; missing-pair
  lookup = Err, fail-closed — §10 totality + accept-total make it unreachable from accepted
  IR) +
  canonical-fixpoint spot tests (bounded-variation inputs re-render canonical) + 3 M1-document
  byte pins from hand-built ASTs (§10 worked example, guideline_b contraindication tail,
  control shape).
- [ ] accept-total: single_ir_accept rejects CNL-inexpressible accepted-IR shapes as
  repairable FillReject::Schema naming the offense (mirrors off_lexicon_ids, empty-refs
  payload convention): (direction, strength) pair without a tail-bearing lexicon row; EMPTY
  statements array (run.rs's accept battery currently pins empty ClinicalIr = accepted);
  signed or two-sided quantity intervals (v1 register — the committed IR schema's
  IntervalBound pattern admits negatives, IrBundle::validate admits two-sided) — closing §10
  render-totality for the one IR-landing route without a grammar/derivation guard (M1 derives
  from lexicon rows + integrity; single_cnl's grammar admits only lexicon tails). Tests: each
  reject class + boundary accepts + repair recovery; M2 recorded-run battery green proves no
  retroactive census flip (a flip ⇒ stop, user decision). Read scope: run.rs accept-closure
  region + the lexicon modality table (lexicon.rs post-extract) only.
- [ ] cnl-bridge: cnl_bridge.rs — to_ir + from_ir per the plan-header determinism rules +
  both round-trip laws as pinned there (from_ir∘to_ir == disjunct-split normal form;
  to_ir∘from_ir == id on bridge-image IR) + worked-example content test
  (parse(§10 JA) bridges to the §8.6 rule content). Read scope: ir.rs shapes + rules.rs
  derive_norm_ir contract only — run.rs stays closed.
- [ ] cnl-laws: depth-bounded AST enumeration harness (all atom kinds × ≤2 disjuncts × ≤2
  conjuncts × all tail-backed modality pairs × certainty on/off × ≤2 exceptions × 1–2 basis
  refs; + one unbacked-pair render-Err assertion) →
  render→parse identity both languages + cross-language agreement + canonical fixpoint +
  single-parse (take(2) Earley differential over a bounded sample, OracleBound escape) + the
  two bridge round-trip laws (plan-header form — split normal form, ≠ naive identity on
  multi-disjunct inputs). Codeco method; bound sizes to CI-sane runtime.
- [ ] codes-cnl: DiagnosticCode +CnlParseError/CnlRoundTripMismatch/CnlUnregisteredConcept
  (fieldless_enum append) + FillReject +Parse(String) (repairable → cnl_parse_error) /
  +Unregistered{surface, position} (terminal → cnl_unregistered_concept; payload = the
  lexicon-entry proposal) / +Instrument(String) (terminal fail-closed →
  cnl_round_trip_mismatch, spends no repair) + model_fill.rs mapping + stage tests (repair
  recovery / both terminals / instrument path). Codes carry payload, empty refs
  (DiagnosticRecord convention).
- [ ] route-stage-handles: behavior-locked rework (2nd 2026-07-10 review: widening the
  positional array for the bridge stage compounds off-by-one + provenance risk) — retire
  run.rs's positional stage plumbing (Resolved.pipeline_step_ids [Id; 8] UNUSED_STAGE-padded;
  index consts MODEL_FILL=2/DIRECT_VERIFY=3/COMPILE=4/VERIFY=5/TRACE=6/REPORT=7 over
  PROCESSING_STAGE_KINDS) for validated named handles: StageHandle {kind, step_id} +
  per-shape RouteStages enum with named stage fields (M1Layered/SingleIr/DirectSmt;
  resolve_route validates the declared kind sequence and constructs it — no sentinel, no
  resize); finish_processing_stage + emission/provenance sites take a handle (kind travels
  WITH step_id, never re-derived from an index); compile_verify_group + producers
  parameterized by handles. Zero behavior change: suites green, assertion surface untouched
  (fixtures rebuild to named construction), M1/M2 pins unchanged.
- [ ] route-single-cnl.1: registry data — processing_stage.m3.model_fill_cnl (model_fill,
  [sdg,segments]→[cnl_document]) + m3.bridge (bridge, [cnl_document,segments]→[clinical_ir]) +
  pipe.m3_single_cnl (7 stages; m2.assemble + m1.compile/m1.verify reused — a
  RouteStages::SingleCnl named-handle variant per route-stage-handles' representation, the
  bridge stage a named field, never an index shift; resolve fixtures follow) + exp.m3_cnl set
  binding (direct baseline) + prompt.single_cnl entry + first-draft template +
  single_cnl_prompt composer (single_ir_prompt mirror + CNL vocabulary blocks: adnominal/
  negated surfaces, action nouns, modality tails, quantity surfaces+units, basis region ids;
  fixture proves ordering non-trivially — the f1 pre-sorted-fixture lesson); RouteShape::
  SingleCnl fingerprint (7-kind sequence + model_fill out [cnl_document]) + resolve +
  select_record_{schema,prompt} + manifest_inputs want-set arms (schema.clinical_cnl = the JA
  decoding constraint) + resolve-rejection additions + registry check green. run.rs read
  scope: resolve/consts + manifest want-set + prompt-composer regions only.
- [ ] route-single-cnl.2: single_cnl_accept closure — parse (Parse reject, repairable) →
  escape scan over context + exception + action-target slots (Unregistered terminal) →
  grounding (basis regions ⊆ regions, derived segments
  ⊆ segments; Grounding terminal) → re-render + re-parse round-trip (Instrument on mismatch)
  → Ok(CnlDocument); battery mirrors single_ir_accept's (valid / parse-repair-recover / both
  terminals / instrument / empty-grounding panic). run.rs read scope: accept-closure region +
  cnl modules.
- [ ] subproc-runner.1: behavior-locked extraction — ONE shared subprocess runner (home
  ckc-smt, ckc-cli already depends on it — confirm at impl) absorbing the mirrored
  spawn/timeout/kill/drain machinery of model.rs + verify.rs behind spawn-retry's injectable
  seam; zero behavior change, existing suites the gate (test edits = imports only).
- [ ] subproc-runner.2: runner hardening (the M2-deferred codex-rejected fixes + drain cap,
  once, shared): bounded stdout/stderr capture with an explicit truncation state, checked
  deadline arithmetic (Instant+budget overflow), post-grace detached-drain cap/reap;
  deterministic tests through the injectable seam; model + solver behavior stays consistent.
- [ ] route-single-cnl.3: single_cnl_fill + execute_routes SingleCnl arm (head reuse → fill →
  bridge + assemble tail into the per-group compile/verify loop via the shape's named stage
  handles — route-stage-handles' representation) + landing
  (cnl_document + clinical_ir + bundle wrappers route-namespaced; bundle input_hashes cite
  cnl_document + accepted cassette; check TraceNodeKind coverage at impl) + §4.6 events (fill
  + bridge; clock discipline — compose prompts outside timed intervals) + landing-gate test.
  The .1d3a-analog run.rs unit — event/landing PIN battery stays OUT (next unit).
- [ ] route-single-cnl.4: golden CNL cassettes ×3 (hand-authored canonical JA CNL bodies for
  the M1 docs) + reproduce-M1 gate (single_cnl verdicts == reference through the locked tail,
  run_oracle mirror).
- [ ] route-single-cnl.5: event/landing pin battery over the golden-cassette run (census,
  layout, event input_hashes pinned independently — the .1d4b lesson: pins split from wiring
  AND from cassette authoring).
- [ ] route-single-cnl.6: rejection cassettes (parse-error → derived-seed repair → recover /
  unregistered-concept terminal / hallucinated-basis terminal / exhaustion) + §7.4 ledger pins
  + RouteTaxonomy cnl-code wiring + taxonomy pins.
- [ ] metrics-cnl: FillObservation optional CNL fields (round_trip_ok, surface_tokens,
  accepted_rules) + round_trip_identity_rate + surface_tokens_per_accepted_rule rows (§7.3
  surface-quality family; rows emitted only when observations carry the fields — prove M2
  replay rows byte-unchanged) + delta/NA wiring + tests.
- [ ] metrics-faithful: FillObservation optional ir_match bool + ir_faithfulness_rate row
  (§7.3 translation-faithfulness family, §10) — run.rs fill-tail computes it for IR-landing
  model routes: landed ClinicalIR == the deterministic
  reference derivation, the M1 normalize+derive chain recomputed in-run over the route's own
  head values already in hand, compared under the §10 faithfulness projection — binding
  region_ids excluded (CNL carries rule-basis, never mention-region, provenance), all else
  exact incl. stmt/bind/exc ids (single_ir: accepted fill; single_cnl: bridged IR; direct_smt
  lands no IR → field None, row not_applicable). Golden path pins projection-match 1.0. Rows gate on observations carrying the field (M2 replay rows
  byte-unchanged, omit-None); deltas ride the existing route-delta loop. Tests: match /
  mismatch / None-NA + golden-path 1.0 + M2-replay byte-pin. Read scope: normalize/derive fn
  signatures, run.rs fill-tail region, metrics.rs row assembly — the §7.3 family text names
  the rationale.
- [ ] report-cnl.1: Report shape — cnl_documents keyed (pipeline, document) per §10 ({ja,en}
  text hashes) + cnl_rules ({ja,en} strings, same key) omit-empty members + validate rules (sorted ids, line-break-free
  strings, code-span-inert) + populated fixture + byte pins; M1 bytes byte-identical (plumbing
  half).
- [ ] report-cnl.2: population + audit views — assemble_report CNL inputs (single_cnl route:
  the accepted CnlDocument's own text/hashes — audit honesty; other routes incl. M1: from_ir
  + render over accepted ClinicalIr) + run.rs report-tail lands
  audit/<pipeline-id>/<doc-id>.cnl.{ja,en}.txt (§10 keying — a multi-route experiment accepts
  the same document several times; write_under + byte read-back, report_en.md pattern; text
  hashes into report.json) + M1/M2 report byte-pin re-bless sweep (deliberate,
  bless-from-observed).
- [ ] report-cnl.3: md renderers — findings quote rules as CNL beside quoted spans (JA body
  quotes JA CNL, EN body EN CNL; Labels) + rendered-body const re-bless (Z3_VERSION-normalize
  pattern) + emission-order/validate coupling tests.
- [ ] record-cnl.1 (gated: model runtime): constraint/tokenizer audit (cnl-grammar.1b already
  proved compile + one bounded emission — this unit is the full pass over the committed
  grammar): tokenizer audit of every grammar terminal (§9 truncation +
  UTF-8 boundary lessons); probe repetition points (DNF connectives / exception sentences /
  rule+) for degeneration loops under the token budget — archived-PoC prior: grammar-masked
  verbose forms loop at unbounded repetition and truncate mid-structure (§10 emission-posture
  paragraph carries the mitigation frame); template refinement from observation + RecordParts byte-verify
  green + model_ms_per_call budget entry. Machine specifics → runtime.local.md, committed
  bytes stay engine-agnostic.
- [ ] record-cnl.2 (gated: model runtime, LIVE): record exp.m3_cnl in a scratch root; copy
  route.single_cnl/** into committed /cassettes (base + derived-seed repairs, REAL identity,
  audit-exempt); identity-agreement vs existing cassettes — agree ⇒ done, drift ⇒ full
  re-record + M2 recorded_run re-bless fallback; replay matched; do-not-read sync verified
  (/cassettes already deny-Read — census via runtime indirection).
- [ ] record-cnl.3: recorded-run battery — exp.m3_cnl census/§9/re-render/replay-matched/
  audit-file pins over the committed cassettes (recorded_run.rs pattern, own test file — M2
  battery untouched).
- [ ] acceptance-m3: §10 acceptance themes against the recorded run (3-route raw-before-delta,
  determinism laws green, round-trip rate 1.0 on accepted docs, faithfulness rows emitted
  beside surface rows — measured never gated, golden path 1.0, audit views every route incl.
  M1, golden-cassette reproduce-M1 gate, replay byte-stability, grammar/lexicon drift guards,
  §0 vocabulary) via the acceptance-driver pattern; tag accept/m3.
