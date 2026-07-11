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
  certainty/exceptions/source refs); CNL's sole core touch = DiagnosticCode fieldless_enum
  append (codes-cnl; path-confine separately extends core's validate_registries).
- CNL AST ≠ ClinicalIr: CnlAtom {Concept|ConceptNegated|Interval|Unregistered(surface)}
  (escape = own variant; Interval = ir.rs QuantityInterval's SHAPE — var + four optional
  signed bounds), CnlConceptRef {Registered(Id)|Unregistered(surface)} — §10 admits
  the escape in EVERY concept slot incl. action target — CnlContext {any: Vec<Vec<CnlAtom>>}
  flat two-level DNF. AST validity two-layered, STRUCTURAL first (lexicon-free grammar-image
  shapes up to parse normalization — §10's law quantifier; round-trip equality over the
  semantic AST, frame members recomputed both sides): outer DNF + every conjunction
  nonempty + per-bracket basis sorted+deduplicated (set semantics — parse normalizes
  surface order, from_ir emits sorted) + interval atoms
  exactly one unsigned bound among ge|gt|le|lt, value nonnegative — §5 coherence
  (IrBundle::validate/bundle.rs; ir.rs owns the shape only)
  admits the signed + two-sided shapes the grammar cannot write, so coherence-mirrored
  validity would bless unrenderable ASTs; cnl-ast enforces ahead of the lexicon-scoped
  layer (SHAPE vs FRAME sublayers — frame members = render's own output, checked on
  stored documents), cnl-render asserts the SHAPE sublayer fail-closed; acceptance runs
  the lexicon-scoped layer post-parse (negative-occurrence bar its sole parse-unenforced
  clause). CnlException {concept: CnlConceptRef, basis: nonempty region refs —
  the sentence's own bracket} (§10 single-concept register — a sentence list, disjunctive
  across entries; no DNF/negation/interval inside an entry; per-sentence basis brackets keep
  per-clause provenance reconstructible — a rule-global bracket cannot say which exception
  clause owns which region),
  CnlRule {context, action kind + target:
  CnlConceptRef, direction+strength, certainty?, exceptions, basis region refs (the rule's
  own bracket, nonempty)}, CnlDocument
  = the landed model_fill artifact payload per §5's table: document_id + grammar id/hash refs
  + rules (AST + per-rule canonical text ja/en) + per-language text hashes — accept re-renders
  and hash-locks canonical bytes beside the AST (§10); report.json cites the artifact's
  hashes. Parser mints NO ids; bridge derives them.
- Bridge determinism: one ClinicalStatement per context-disjunct (ids `stmt.<k>`/`exc.<k>`/
  `bind.<k>` document-order counters in normalize.rs's id forms + document-local
  counter scope (bind ORDER = the pinned traversal below; normalize scans mentions instead) —
  §8.6 reserves
  `<doc>.rule.<k>` for norm-layer rule ids; disjunct split appends statements in document
  order); population-vs-condition partition by the lexicon's typed slot roles (§10: every
  concept row a validated nonempty role set over population|condition|action_target —
  population/condition mutually exclusive per row, action_target free to combine
  (multi-role deliberate); every quantity row exactly one context role — concept atoms
  land by row role, interval atoms by quantity role; ONE typed role view is the single
  source every CNL consumer reads — grammar slot alternations, cnl-ast validate, parser
  slot legality, bridge partition (from_ir Err on wrong-slot IR — CNL-inexpressible,
  re-parses into a different partition), accept wrong-slot rejects — no prefix tests in
  CNL modules; normalize.rs's frozen M1 prefix partition stays untouched, locked-corpus
  agreement pinned by lexicon-cnl-integrity's M1 role data test; ruling: explicit roles
  FIELD over a prefix-derived index — multi-role is data and a future namespace never
  silently falls through to condition); each exception sentence → one
  single-atom ExceptionClause PER SPLIT STATEMENT — a multi-disjunct rule clones its
  exception list into every emitted statement ((D1∨D2)∧¬E = (D1∧¬E)∨(D2∧¬E); bundle
  validation demands globally unique exception ids), `exc.<k>` counting emitted clauses
  statement-major then sentence order (worked 2×2: two disjuncts × two exception sentences →
  stmt.0 owns exc.0/exc.1, stmt.1 owns exc.2/exc.3 — sentence order within statement, clone
  content + basis duplicated per statement), clause region_ids = its own sentence's basis bracket
  verbatim (per-sentence brackets; clones share their sentence's basis) — (positive
  interval-free `Concept` — the §10 register + negative-occurrence bar; the locked rules.rs
  lowering negates ONLY positive Concept atoms into the rule's single conjunct and NEVER
  interval-lowers a negative occurrence, context ConceptNegated included, so an
  interval-carrying entry in any negative slot splits Bool/Real unlinked in emit — sound for
  single-atom interval-free clauses via ¬(E1∨…)=¬E1∧… and for nothing wider — a
  conjunctive exception needs De Morgan ¬A∨¬B, and it ignores negated/interval exception
  atoms); bindings = one Exact-status TerminologyBinding per distinct referenced
  concept, minted at first reference in POST-SPLIT emission order — statement-major; per statement
  population atoms, condition atoms, action target, then exception clauses, each in emitted
  order; a concept exclusive to a later disjunct mints after the earlier disjunct's
  atoms (matches M1's first-mention scan order on
  the locked corpus; divergence = measured ir_match miss, never asserted), system =
  lexicon.system, code = concept id, region_ids = the union of the citing emitted
  statements' segment CLOSURES (each citing statement's source segments' full region sets —
  the closure its rendered brackets jointly cover, rule bracket = closure minus
  exception-owned = the Recommendation-kind closure under the basis-ownership laws below;
  invariant under bracket normalization, keeping to_ir∘from_ir exact — an
  authored-bracket union breaks the law whenever the citing statements' brackets jointly
  under-cover the closure union, minimal case an exception-free rule citing one region of a
  two-region Recommendation segment re-bridging WIDER) — a
  KNOWINGLY lossy reconstruction of M1's mention-based regions (coarser: statement-grain
  closures; §5 region refs producer-graded),
  hence metrics-faithful
  compares under the §10 projection excluding binding region_ids (Action::new derives
  key itself); basis refs = region ids per sentence (rule bracket + one per exception
  sentence); source_segment_ids derived region→segment over their UNION via the
  segments artifact (ClinicalSegment.region_ids reverse map — m3.bridge stage inputs therefore
  [cnl_document, segments]; bridge preconditions, acceptance-enforced both sides: cited
  regions anchored in exactly one segment, derived segments' region sets unshared
  (closure-functional), basis ownership kind-aware PER STATEMENT — writing R = cited Recommendation
  segments' region union, E = cited Exception segments', X = exception brackets'/clauses'
  union: every cited segment Recommendation|Exception-kind, R nonempty (⇔ the nonempty
  remainder once the laws hold — alias them on accepted inputs only), and the primitives
  X ⊆ closure ∧ X ∩ R == ∅ ∧ E ⊆ X deriving X == E — so full closure − X == R and every basis region is handled exactly
  as the locked rules.rs tail handles it (Recommendation walk + per-clause append;
  kind-blind citation would silently vanish/duplicate/drop provenance), exception regions
  closure-contained — CNL-side
  by construction (derived segments span the bracket union), IR-side the predicate's
  containment class). Origin map: rule_origins(&CnlDocument) →
  `<doc>.rule.<k>` → originating rule index, pure fn (rule k = the k-th post-split
  statement, mirrors rules.rs derive_norm_ir's statement-enumerate mint) — non-core, ONE
  helper over accepted docs AND from_ir output (single-disjunct → identity); the report's
  per-rule CNL text consumes it; several rule ids legitimately share one rule's text.
  Round-trip laws, precise (ACCEPTED escape-free
  ASTs — single_cnl_accept's closure; to_ir = Err on any
  escape occurrence): from_ir = one single-disjunct CNL rule per statement (projection, no
  regrouping; each clause's region_ids render verbatim on its exception sentence, the rule
  bracket renders the segment-closed remainder — every cited segment's FULL region set minus
  the exception-owned regions, == the Recommendation-kind closure under the basis-ownership
  laws; from_ir's sole Err source = check_cnl_expressible at entry
  (the shared-predicate bullet below), projection Err-free past a passing check —
  CNL-inexpressible, accept-total-rejected by the same fn) ⇒ from_ir∘to_ir == bridge
  normal form — disjunct split + per-statement atom canonicalization (population before
  condition, §4.3 set order, byte-identical duplicates collapsed; the partition + set
  emission are lossy exactly there) + exception-owned segment-closed basis split (a labeled
  cover, not a partition — clauses may share a region;
  exception-owned regions render only on exception sentences; closure pulls each cited
  segment's remaining regions into the rule bracket) — (== id exactly on bridge-normal
  docs); to_ir∘from_ir == id on bridge-image IR (the image of accepted ASTs; exact incl.
  closure-derived binding region_ids).
- Findings owner — M3 = the first run with TWO compiled routes, and the landed §7.1 identity
  scheme is structurally single-view: compile mints bare `q.<gsuf>.pair<n>.<kind>` query ids,
  trace mints `finding.<gid>.<seq>` from every compiled+results group, assemble_report errors
  on a duplicate query id (M2's direct route dodges all three only via compiled: None +
  run-minted `<gid>.overlap` ids) — single_cnl reusing m1.compile/m1.verify would mint ids
  byte-equal single_ir's ⇒ trace canonical-set + DuplicateResult failures. Ruling: owner
  SELECTION over id qualification — the §7.1 view (trace finding mint + report results input,
  no-conflict results included) consumes ONE compiled view, the first bundle-bearing pipeline
  in experiment binding order (single_ir; = the landed lineage/claims rule, the all_docs
  bundle sort); payload ids stay unprefixed, M1/M2 finding/report bytes + §7.2 id forms
  untouched. Non-owner compiled routes' groups land route-namespaced in the trace DAG
  (census/replay/audit; per-route metrics via RouteRun) and stay OUT of the finding mint +
  report results (GroupTrace owner mark, is_baseline pattern; wired at route-single-cnl.3).
  cnl_rules stays all-route (audit surface) but finding/no-conflict md quotes the OWNER
  pipeline's entry alone, keyed by the canonical optional report field
  findings_owner_pipeline_id (report-cnl.1 shape, .2 population, .3 lookup): normative
  rule ids are route-local POSITIONAL identities, never cross-route alignment keys — a
  non-owner route that inserts/omits/differently splits an earlier statement makes its
  same-numbered id different content, and quoting it beside an owner finding fabricates
  support; cross-route side-by-side comparison needs an explicit alignment map (M4
  ablation scope), never positional-id equality.
  Route-qualified payload ids REJECTED: rewrites the §7.2 finding-id form + re-blesses every
  M1/M2 report/trace pin for a per-route findings matrix no M3 consumer reads (M4 ablation
  scope).
- CNL expressibility = ONE executable predicate, never a hand-maintained rejection list per
  consumer: check_cnl_expressible(clinical, lexicon (role + tail view), segments
  (segment_id → (kind, region_ids) keyed view — id uniqueness by construction; the §10
  basis-ownership classes read the kind)) -> Result<(),
  CnlExpressibilityError>, home cnl_bridge.rs
  (unit cnl-expressible seeds the module; the segment-closure helper it lands is the one
  from_ir's rule bracket takes its exception-owned remainder from AND to_ir's binding
  region_ids consume whole), defined over grounded lexicon-valid ClinicalIR
  (lexicon-valid = vocabulary MEMBERSHIP only, off_lexicon_ids' check — role/tail legality
  stays predicate-owned, every variant in-domain-reachable; membership + grounding run
  ahead of it), one error variant per
  CNL-inexpressible class (taxonomy enumerated at cnl-expressible). Consumers:
  single_ir_accept (accept-total wires each variant → repairable FillReject::Schema naming
  the offense) + from_ir (entry check = its SOLE Err source; projection past it constructs
  no Err — a residual failure is a fail-closed panic, instrument bug). §10 law,
  property-tested at expressible-law: over the domain — acceptance judged on the value's
  canonical bytes — acceptance succeeds ⇔ from_ir succeeds; the two domains one function
  apart, definitional drift structurally excluded, behavioral agreement law-tested.
- Grammar terminals = whole-surface string literals (ASCII digits as literal alternation;
  basis-id refs = Id's exact §4.1 grammar `[a-z][a-z0-9_.:-]*`, pinned to the
  smt_query.grammar `<identifier>` production: `<basis-id> ::= <lower> <basis-id-rest>`,
  `<basis-id-rest> ::= "" | <basis-id-char> <basis-id-rest>`,
  `<basis-id-char> ::= <lower> | <digit> | "_" | "." | ":" | "-"` — NEVER a bare
  one-or-more-basis-chars production, which
  additionally admits `1r`/`.r`/`:r`/`-r`/`_r` that Id::new rejects (an undeclared THIRD
  grammar-over-approximation class beside §10's declared two), and NEVER the broader
  identifier_ascii policy, which admits `/`) — portable to LLM constraint mechanisms +
  atomic in bnf — with EXACTLY ONE open
  lexical production per language: the escape's free quoted surface (§10) is inexpressible as
  finite literals → dialect open-content notation decided at cnl-grammar.1, portability risk
  contained there; emitter takes an escape mode — Committed(open) vs
  OracleBound(enumerated test surfaces) — since bnf parses literals only; cnl-grammar.1b
  probes the open-production portability risk, record-cnl.1 audits the full terminal set. bnf 0.6
  (existing workspace pin) verified unicode-capable, byte-offset whole-terminal matching;
  its Earley oracle proves language MEMBERSHIP (superset — explores all segmentations), so
  lexer segmentation determinism is guarded by the lexicon prefix-overlap lint instead;
  single-parse asserts use `parse_input().take(2)`, never full counts. Document frame pinned
  past bare `rule+` cardinality (§10 canonical-text bullet): canonical document bytes = one
  LF-terminated line per rule — LF the uniform rule terminator, last rule included, no other
  inter-rule bytes; grammar production document = (rule <nl>)+ in BOTH languages (the
  smt_query.grammar literal-LF `<nl>` convention; bnf 0.6 lacks postfix repetition →
  right-recursive lowering, <assertions> pattern); render assembles exactly those bytes;
  parse demands the exact frame (missing terminal LF = repairable parse error) and
  whitespace variation is NONE — parser language = grammar language, stray whitespace a
  repairable parse error; stored
  per-rule texts line-break-free — LF and CR (lexicon surfaces whitespace-folded §4.2,
  fixed terminals carry none, escape payload bars control chars); CnlDocument text hashes +
  report-cnl.2's audit `.txt` views cover exactly the assembled bytes, and cnl-ast validate
  RECOMPUTES the hashes from the stored texts (executable invariant).
- Record strategy: record exp.m3_cnl into a scratch root whose cassette store starts newly
  created + EMPTY; census `route.single_cnl/**` vs the run's model-attempt ledger both
  directions; identity-agreement vs the existing cassettes decides — agree ⇒ M2 cassettes
  stand (no re-bless), drift ⇒ full re-record + M2 recorded_run re-bless (documented
  fallback); after agreement + replay verify, copy into the committed /cassettes REPLACING
  the route.single_cnl subtree, never merging (keys disjoint from M2's); this hygiene =
  record-cnl.2 acceptance.
- Known deliberate re-bless costs, scheduled in their units: ja_core.yaml growth →
  lexicon_hash-carrying value pins (lexicon-cnl-data); report CNL population → M1/M2 report
  byte-pins + rendered-body consts (report-cnl.2/.3).

- [ ] spawn-retry: pre-M3 hardening, opens the milestone (red wherever the fs cannot
  yield ETXTBSY → every later unit's full-suite gate binds only once this lands) —
  both spawn_piped_surfaces_persistent_etxtbsy tests
  (model.rs, verify.rs) assert the FILESYSTEM produces ETXTBSY (fs-dependent, BOTH outcomes
  observed: some filesystems yield ETXTBSY, others — overlayfs among them — let the spawn
  succeed), and the retries_through twins pass vacuously
  wherever the first spawn succeeds. Rework BOTH mirrored copies (full machinery extraction
  stays subproc-runner.1): retry loop parameterized over an injectable spawn-attempt op
  (prod path = the real Command spawn) AND an injectable clock/sleeper — deadline reads +
  inter-attempt sleeps go through it (prod path = Instant::now/thread::sleep), so every
  deterministic test is wall-clock-free: persistent-through-grace advances the injected
  clock past SPAWN_BUSY_GRACE instead of consuming the real 250 ms; a small shared
  retry-policy helper — home ckc-smt,
  ckc-cli already depends on it — MAY host the loop + both seams once so they aren't
  implemented twice; deterministic tests drive ETXTBSY,ETXTBSY,success /
  persistent-through-grace / immediate non-ETXTBSY error / immediate success; happy-path
  process integration tests stay; ≤1 fs
  integration test per crate, capability-probing the mount and skipping cleanly where the fs
  cannot produce ETXTBSY. Gate: workspace strict suite green with zero
  environment-dependent outcomes.
- [ ] path-confine: pre-M3 hardening (review-reproduced: absolute corpus + expected_outcomes
  paths under /tmp pass `registry check` AND a full run — `Path::join` swallows the root on an
  absolute rhs and keeps `..`). Core: validate_registries extends the schemas/prompts
  is_safe_relative_path finding (UnsafePath) to corpus.path + experiment.expected_outcomes.
  CLI: ONE I/O resolver RETURNING CAPTURED BYTES (lexical check → canonicalize root +
  existing candidate → candidate strictly under root → pre-open stat filter (regular files
  only; keeps a contained FIFO from blocking the open) → open ONCE (`O_NONBLOCK` unix) →
  authoritative regular-file re-check on the opened handle's metadata → read from that
  handle; success = one typed {canonical path + bytes} value — input-snapshot.1's
  ResolvedFile shape — so RESOLVER consumers never reopen a checked pathname; the constraint
  pathname reread + runtime child reopen persist BY DESIGN until constraint-staging +
  constraint-snapshot land, cassette.rs's drift-check covering them meanwhile; a failed
  resolve lands the diagnostic and yields no bytes; residual canonicalize→open window =
  concurrent repo-tree path replacement mid-resolve — any rename-capable principal, not
  merely same-UID — ruled outside the threat model: the registry tree is the operator's own
  working copy, a weaker invariant than constraint-staging's owned dir, stated as such)
  applied to every registry-data-controlled read — registry_check.rs
  expected_outcomes ref loads + schema/prompt byte reads, run.rs corpus entry.path (×2 call
  sites), expected_outcomes (×2), record-path template/constraint reads; fixed-name reads
  (CORPORA_FILE, LEXICON_FILE, …) are code constants, out of scope. Tests: absolute / `..` /
  in-repo symlink→outside (`#[cfg(unix)]`) / valid nested file accepted / non-regular
  rejected via the opened handle's metadata, FIFO included without blocking (`#[cfg(unix)]`)
  — across registry check + run. Gate: full gates
  + registry check green on the
  committed tree.
- [ ] input-snapshot.1: read-once input layer, fresh module
  crates/ckc-cli/src/input_snapshot.rs. ResolvedFile {canonical path, bytes, hash =
  hash_bytes(bytes), value} — parse-at-capture ONLY where the run parses unconditionally
  (corpora/candidates/experiments; parse fns reused: ckc-core registry parse_*, normalize
  load_lexicon); expected_outcomes captures RAW bytes+hash, consumers parse on demand FROM
  snapshot bytes (degraded identity-less routes skip reference parsing today — a
  readable-but-malformed reference stays non-fatal). The run-level InputSnapshot builds in
  two phases around resolve, then freezes: phase A (pre-resolve) = registry files + toolchain + lockfile + lexicon;
  phase B (end of resolution, shapes fingerprinted) = selected per-entry corpus bytes, plus
  — iff the resolved set contains model routes — expected_outcomes + schemas + prompts +
  the resolved routes' record-path template/constraint byte files (knowable once routes
  resolve; capturing them later at record setup would breach the freeze) (M1
  reads none of those in-run; its stable test fixture ships NO reference file —
  ungated capture breaks the M1 suite). Registry-data-controlled paths load through
  path-confine's byte-returning resolver (its typed {canonical path + bytes} success IS the
  snapshot row's source — no reopen between check and capture); fixed-name files join as
  today (the two LEXICON_FILE
  sites are
  exclusive dispatch arms — one read per run already; snapshotting it single-sources, not a
  reread fix). Per-entry corpus slots hold bytes OR the read-failure payload;
  duplicate-path entries capture per entry (slot-consistent attestation). Unit tests:
  loader errors, hash = file bytes' sha256, confinement rejection, phase + model-route
  gating. No run.rs contact.
- [ ] input-snapshot.2: resolution + execution consume the snapshot — resolve() consumes
  phase-A parsed registries + toolchain and triggers the phase-B capture at its tail; the
  two corpus entry.path head-fn reads, both LEXICON_FILE sites, and build_record_setup's
  SCHEMAS/PROMPTS loads become snapshot lookups (root stays for outputs). Missing-doc
  behavior preserved: routes consume the slot's recorded failure and land TODAY's
  diagnostic per-route inside their own ledger slice (only the READ moves — multiplicity,
  attribution, and each site's payload shape unchanged). Record-path template/constraint
  reads become snapshot lookups (bytes = the phase-B capture); cassette.rs's existing
  post-call ConstraintDrift re-read stays untouched this unit — constraint-snapshot (next)
  retires the pathname-reopen scheme wholesale (a per-fill reread would still miss a
  transient A→B→A rewrite). Gate: full suites green, M1/M2 byte-pins UNCHANGED; test edits =
  mechanical plumbing where tests call resolve()/head fns directly + input-error paths;
  semantic assertions unchanged.
- [ ] constraint-snapshot: frozen constraint staging (review-demonstrated: the child
  reopens the constraint PATHNAME mid-call, so
  cassette.rs's post-call ConstraintDrift re-read — and any per-fill reread — accepts a
  transient A→B→A rewrite while the sealed hash names A). Record setup materializes the
  snapshot's constraint bytes into a private per-run staging dir under the run's out root:
  atomic publication (temp + rename), never rewritten after, dir cleaned at run end and
  excluded from outputs/manifest (same-UID tampering out of threat scope — that actor owns
  the whole run root). Typed StagedConstraint (constructible only by the staging step)
  replaces the &Path through FillSource::Record + CassetteStore::record, so the compiler
  migrates EVERY caller (ignored live-bless test included) — staged-only recording is
  structural, not cooperative; adapter.invoke receives only the staged path;
  constraint_hash = the staged bytes' hash (== snapshot hash); ConstraintDrift re-read +
  error variant retired. Docs narrow to what is proved: the sealed hash attests the
  snapshot bytes CKC PUBLISHED at the pathname the runtime was given (the child opens it
  independently; byte-USE proof needs wrapper-side digest attestation — out of scope).
  Staging is sound because committed constraints are single-file self-contained —
  clinical_ir.schema.json's $ref values are ALL fragment-only (31 occurrences), grammars
  are literal-only — overturning cassette.rs's "snapshot copy would break relative $refs"
  comment; a registry-driven per-format guard test walks committed schema entries and
  rejects non-fragment $ref values (future entries covered automatically). Tests:
  staged-path invocation observed (override wrapper records argv); mutate + transient
  modify/restore of the ORIGINAL registry path mid-run — record stays accepted, sealed
  hash == staged bytes; suites green with the drift variant gone.
- [ ] input-snapshot.3: metrics + manifest attest the snapshot — model_route_metrics parses
  experiments/reference from snapshot bytes (identity gating preserved); manifest_inputs
  (today rereads LOCKFILE/CORPORA/EXPERIMENTS/expected_outcomes/SCHEMAS/PROMPTS) consumes
  snapshot fields with every §9 field keeping its CURRENT derivation: raw-byte hashes
  (lockfile/corpora/reference/test sources) hash snapshot bytes; schema_hash +
  prompt_template_hash stay aggregates over the want-set entries' DECLARED hashes read from
  the snapshot's parsed schemas/prompts (raw registry-file hashes would break the pinned
  manifests). Stable tree ⇒ identical bytes ⇒ pins green. Regression test: an explicit
  test-only barrier hook reachable from run.rs's in-crate tests (NO sleeps) mutates
  registry/corpora.yaml AND one selected corpus doc between resolution and manifest
  assembly — the run stays `ok`, artifacts + manifest.json attest the resolution-time bytes
  (today the manifest attests the mutated registry while the artifacts used the original
  mapping — the review-reproduced flip).
- [ ] lexicon-extract: behavior-locked move — the Lexicon family (types, YAML row structs,
  load_lexicon, validation + their tests) leaves normalize.rs (2166L) for fresh
  crates/ckc-cli/src/lexicon.rs; normalize keeps the existing PUBLIC paths via pub use
  re-exports (Lexicon/LexiconError/load_lexicon are pub through ckc_cli::normalize and no
  suite would catch a silent path break); the neutral dependency point input_snapshot,
  the lexicon-cnl units, cnl_grammar, cnl_parse, cnl_render, cnl_bridge all consume. Zero public
  behavior change: existing suites the gate, assertion surface untouched (import edits only).
- [ ] lexicon-cnl-shape: CNL surface fields, shape only (split of lexicon-cnl.1) —
  LexiconConcept +adnominal_ja/negated_ja/gloss_en+roles (§10 slot-role set:
  population|condition|action_target; target citation form = existing surfaces[0] (JA) +
  gloss_en (EN) — no new field; EN negation likewise field-less, a fixed-negator
  composition over gloss_en at the grammar (JA negation lexical: negated_ja); §10
  composition-decree authoring contracts — gloss_en = lowercase article-free noun phrase
  reading after with/without/of, adnominal = direct-prenominal (before 患者 AND before
  かつ), interval-carrying context concepts author negated forms too (tokens parse;
  acceptance rejects use with the complement-interval repair); synonym
  surfaces[1..] stay source-match-only, never CNL
  terminals), LexiconAction
  +noun_ja/noun_en, LexiconModality +tail_ja/tail_en (canonical deontic tails ≠
  source-match surfaces per §10 — optional per row, parse-accepted synonyms when present),
  LexiconCertainty +surface_en, NEW LexiconQuantity {var_id, role (context slot:
  population|condition), surface_ja, unit_ja,
  surface_en, unit_en} table, + the typed role VIEW accessors every CNL module consumes
  (per-slot concept SURFACE sets — context slots serve adnominal_ja/negated_ja + gloss_en,
  the exception slot adnominal-only (§10 positive-only register), the target slot the
  surfaces[0]/gloss_en citation pair — + quantity-role lookup — §10 single source, no
  prefix tests downstream; view construction gains lexicon-cnl.2's zero-finding
  precondition when the lint lands); load_lexicon strict extension (deny_unknown_fields holds; JA
  surfaces through StringPolicy::SemanticJa, EN surfaces through SemanticEn — §10 EN
  canonical text is ASCII-lowercase); every new field/table optional at load so the
  committed lexicon stays green. Loader fixtures + round-trip + unknown-field rejection
  tests; ckc-core's independent strict YAML mirror (test_sources_m1.rs Lexicon structs,
  deny_unknown_fields, parses the committed lexicon) gains the same optional fields +
  quantity table — it reddens at lexicon-cnl-data otherwise. Committed bytes untouched,
  no re-bless.
- [ ] lexicon-cnl-data: ja_core.yaml authored for the full M1 set (6 concepts with §10
  slot roles — pop.* population, cond.* condition, drug.abx_a action_target — + decree
  surfaces: adnominal/negated worked forms (成人 / 敗血症のある style), gloss_en noun
  phrases (sepsis / adult status / antibiotic-a), certainty surface_en mirroring the
  committed JA label form (certainty: moderate);
  act.administer +noun_ja/noun_en (§10 worked 投与 / administration), 7 modality rows incl.
  §10's worked tails を強く推奨する / は禁忌である,
  certainty rows +surface_en each, q.age_years role=population) — written against lexicon-cnl-integrity's full rule list
  (data lands FIRST: those hard-errors bind the committed lexicon the moment they land —
  tail-availability + quantity-per-interval-var + action-noun/certainty-surface totality
  demand these rows). Gate: load/normalize
  tests green + lexicon_hash-carrying value pins re-blessed (grep the observed-bless
  literals) + full gates; shape/integrity leave committed bytes untouched (a later
  lexicon-cnl.2 lint finding against the committed data = fix + re-bless there,
  deliberate).
- [ ] lexicon-cnl-integrity: §10 integrity hard-errors NEW in load_lexicon —
  implies_action resolves to an action entry, quantity var_ids unique, quantity var set ==
  the set of interval vars concepts use (exactly one row per used var, orphan rows
  hard-error — an orphan row is grammar-visible interval vocabulary outside the committed
  schema's concept-derived enum + off_lexicon_ids' universe: parseable yet unacceptable),
  slot-role integrity (every concept row a nonempty
  deduped set of known roles, population/condition mutually exclusive per row; every
  quantity row exactly one context role agreeing with each interval-carrying concept using
  its var; a test pins the committed M1 roles — pop.*→population, cond.*→condition,
  drug.abx_a→action_target, q.age_years→population — so the frozen normalize.rs prefix
  partition and the role-driven bridge agree on the locked corpus by data),
  nonempty normalized surfaces+units both languages,
  render-surface totality (every action row paired nonempty noun_ja/noun_en + every
  certainty row nonempty surface_en — a membership-valid action/certainty value never
  reaches from_ir surface-less; action_target citation = surfaces[0] + gloss_en, gloss_en
  presence lint-owned — barred anyway at the view gate, lexicon-cnl.2),
  per-language duplicate-literal rejection by SEMANTIC TOKEN over lexer-visible surfaces
  (adnominal/negated + action_target-role surfaces[0] citation forms, action nouns, tails,
  certainty phrases, quantity surfaces/units): reject a literal exactly when its
  occurrences denote two DISTINCT tokens — Concept(row) (citation+adnominal of one row
  collapse; slot admissibility is grammar/parser business), NegatedConcept(row),
  ActionNoun(row), Tail(dir,strength), Certainty(value), QuantityVar(var), Unit(literal)
  var-free (the per-var interval production pairs each var's surface with its own row's
  unit terminal — shared unit literals unambiguous) — same-token occurrences deduplicate
  into one token-table entry (multi-role reuse, same-pair tail synonyms, shared units,
  same-value certainty synonyms accepted — REPLACES normalize.rs's value-blind
  certainty_pool reject; cross-row/-value/-category reject; Fixed(terminal) + Digit(char)
  join as own categories — a lexicon literal equal to either = cross-category reject,
  equality escapes the prefix rule; escape payloads + basis-bracket ids stay
  delimiter-scoped, outside the domain),
  tail_ja/tail_en present together or absent together (a row is
  tail-bearing iff BOTH — one-language tails would leave the other renderer partial),
  every (direction,strength) pair present carries ≥1 tail-bearing row —
  first tail-bearing row per pair = canonical render row, a test pins it against §10's
  worked tails — certainty-table render totality (every §5 Certainty value carries a row,
  first row per value = canonical render row — closed 4-value enum, a gap leaves from_ir's
  certainty parenthetical surface-less on in-domain IR; committed table already total) —
  and concept intervals CNL-representable (v1 one unsigned bound); per-rule
  rejection battery over bad-lexicon fixtures + a positive role-matrix control (a synthetic
  lexicon covering all five legal concept role sets — {population}, {condition},
  {action_target}, {population,action_target}, {condition,action_target} — plus both
  quantity roles loads clean; committed rows are all singleton-role with a population-role
  quantity, so without it an implementation rejecting legal multi-role rows or
  condition-role quantities passes every committed-data fixture) + semantic-token controls
  (accept: one multi-role surface across its slots, citation==adnominal on one row, one
  unit literal on two quantity rows, a same-pair duplicate tail literal, a same-value
  duplicate certainty literal; reject: one
  literal on two concept rows / two certainty values / a fixed terminal / a digit —
  errors name the colliding tokens) + render-totality rejects (noun-less + one-language-noun action row,
  surface_en-less certainty row). Gate: committed tree
  green under every rule
  (proves lexicon-cnl-data's authored rows satisfy them) + full gates.
- [ ] lexicon-cnl.2: CNL lexicon lint — reserved-token collisions (a surface containing a
  connective/punctuation grammar terminal or a backtick — §7.2 renders rule strings in
  single-backtick md code spans; escape payloads need no bar, terminal at accept + absent
  from accepted/report-rendered text (pre-accept escapes render + round-trip by design)), role-scoped missing-CNL-surface findings (a
  context-role concept lacking adnominal/negated/gloss forms, an action_target-role
  concept lacking gloss_en — citation = surfaces[0] + gloss_en, surfaces[0] row-required
  already; deliberately
  tail-less modality rows exempt — per-pair availability is integrity's rule), and
  proper-prefix overlap across ALL lexer-visible terminals, same- AND cross-category
  (maximal munch can steal across categories; segmentation determinism — the
  Earley-superset caveat's guard) + per-variant rejection battery over bad-lexicon
  fixtures. The reserved-terminal + lexer-category inventory lands here as ONE typed
  module that lint, grammar emitter, and parser all consume (single source, no drift).
  Wiring — the lint gates, never floats: committed-lexicon zero-findings test; the §10
  typed role VIEW constructor hard-errors on ANY finding and every CNL module reaches the
  lexicon only through the view (grammar emitter, parser, bridge, predicate, renderer,
  prompt composer) — a lint-dirty lexicon produces no CNL anywhere, so lint-owned
  role-scoped surfaces bar accepted artifacts + reports exactly like integrity hard
  errors (clinical_cnl_grammar's hard-error = this gate's special case).
  Pure findings layer beside load_lexicon's existing checks.
- [ ] cnl-ast: cnl.rs type family — CnlAtom/CnlConceptRef/CnlContext/CnlException/CnlRule/
  CnlDocument (grammar refs + per-rule text + text-hash members per the plan header) +
  Canonical emit/read (sorted-key slots, optional members omit-None) + STRUCTURAL validate,
  lexicon-free + checked BEFORE the lexicon-scoped layer (two sublayers — SHAPE,
  render-asserted: nonempty
  rules, nonempty basis per bracket — rule + each exception (§10 per-sentence provenance) —
  every bracket sorted + deduplicated (set semantics; parse/from_ir supply sorted),
  nonempty context DNF — outer any AND every conjunction,
  Id grammar, interval atoms the §10 v1 register — exactly one bound among ge|gt|le|lt,
  value nonnegative (plan-header ruling: NOT §5 coherence — IrBundle::validate/bundle.rs;
  ir.rs owns the shape only, disclaiming coherence — which admits the signed +
  two-sided shapes the grammar cannot write), §10 escape
  payload contract —
  nonempty ≤80 scalars, single line, control/quote-delimiter chars excluded,
  SemanticJa-normal fixpoint; FRAME, stored-document integrity — render's own
  output, checked on stored documents, never render-asserted: per-rule canonical-text
  members line-break-free — LF AND CR, matching
  report.rs's line-break validation (§10 document frame) — + per-language text-hash members
  RECOMPUTED equal from the stored per-rule texts under the frame assembly, hash ==
  hash_bytes(concat(rule_text + LF)) — the frame an executable invariant, never a
  convention; + lexicon-scoped validity vs a passed lexicon view
  (pairs/ids/roles): modality
  pair tail-backed, concept/action refs resolved, interval vars resolving to quantity rows
  (+ dangling-var rejection case), slot roles admit every Registered ref's position
  (context + exception concept refs context-role, action target action_target-role — §10
  wrong-slot bar, per-slot rejection cases; the Unregistered escape is roleless and
  admitted in every concept slot per §10 — per-slot escape-accept positives beside the
  rejections), negated/exception concept refs
  interval-free (§10 negative-occurrence bar) — makes §10's valid-AST quantifier
  well-defined) + structural rejection battery — empty outer DNF, empty conjunction,
  unsorted + duplicate-ref basis, and the interval predicate as an exhaustive truth
  table: 16 bound-presence masks × per-bound values {-1,0,1}, valid iff exactly one
  bound present ∧ value ≥ 0 (expectation table shared with cnl-expressible's IR-side
  battery — cross-side faithfulness by construction; subsumes signed / two-sided /
  boundless / same-side doubles) — + one mixed structurally-and-lexicon-invalid case
  pinning layer precedence (the structural class reports) + all-None/populated byte pins
  + round-trip tests. Fresh module, no run.rs contact.
- [ ] cnl-grammar.1: cnl_grammar.rs emitter — clinical_cnl_grammar(lexicon, lang) -> Vec<u8>
  BNF (smt_query.grammar dialect, `;` comments): document = (rule <nl>)+ — §10 document
  frame, <nl> the dialect's literal-LF terminal = the uniform rule terminator, last rule
  included (canonical separator + terminal-newline policy pinned, past bare cardinality);
  bnf 0.6 has no postfix repetition → committed form = the right-recursive lowering,
  smt_query.grammar's <assertions> pattern: <document> ::= <rule> <nl> | <rule> <nl>
  <document>;
  rule = context 患者には、
  action deontic-tail / optional certainty paren / [根拠 …] rule basis / ただし-exceptions
  (each sentence = ONE concept-or-escape slot + its OWN [根拠 …] bracket, §10 per-sentence
  provenance; no connectives/negation/interval
  inside 除く); DNF
  connectives かつ / 、または with precedence by production shape; atoms
  concept|negated|interval|escape (未登録概念「…」 / unregistered concept "…"; admitted in
  action-target position too — §10); EN mirror productions per the §10 composition decree — EN atoms
  position-invariant, one prepositional frame: positive/exception atoms `with <gloss_en>`,
  negated `without <gloss_en>` (fixed negator replaces `with`), interval `with <surface_en>
  <bound-words> <n> <unit_en>` (at least/at most/less than/more than ↔ ge/le/lt/gt, marker
  before numeral; JA 以上/以下/未満/超 after unit), escape-atom `with unregistered concept
  "…"`, bare gloss/escape after `of`; JA mid vs patient-adjacent atom alternations —
  interval + escape take the fixed の exactly before 患者 (two JA nonterminals;
  stray/missing の = parse error); certainty paren `(` + certainty-row surface + `)`
  between tail and terminator; spacing decree — JA spaceless, EN single-space separators
  owned by fixed terminals/joiners, bracket internals space-separated sorted ids;
  terminals = lexicon whole-surface
  literals in slot-specific alternations from the §10 role view (context + exception
  concept slots = context-role surfaces, action target = action_target-role surfaces —
  wrong-slot vocabulary unparseable)
  + fixed terminals from lexicon-cnl.2's inventory module (emitter hard-errors on
  a lint-dirty lexicon); interval productions PER QUANTITY ROW — each row's surface paired
  with its own unit terminal, both languages (the sole unit↔var binding, Unit tokens being
  var-free — a wrong-unit-for-var interval is unparseable);
  interval numerals = ASCII-digit literal alternation (unbounded
  repetition — value bound 0..=i64::MAX parser-enforced, the second grammar
  over-approximation beside the open escape); basis-bracket id refs = the plan-header
  basis-id production (leading `<lower>`, rest `<lower>|<digit>|"_"|"."|":"|"-"` —
  smt_query.grammar's `<identifier>` shape), the grammar's basis-id language == Id::new's
  exact `[a-z][a-z0-9_.:-]*` — no undeclared third over-approximation class (a bare
  one-or-more-basis-chars production would admit `1r`/`.r`/`:r`/`-r`/`_r`,
  Id::new-rejected); escape quoted-surface content
  = the single open production (plan header — emitter escape mode Committed|OracleBound;
  payload contract per §10, parser-enforced — the production stays open).
  Oracle tests in-crate (bnf workspace dev-dep added to ckc-cli, OracleBound grammars): §10
  worked examples full-match both languages, trailing-garbage reject, wrong-slot-surface
  reject, a multi-role surface parsing in EACH of its slots (synthetic
  {condition,action_target} lexicon — committed rows are singleton-role), swapped-unit
  reject + shared-unit-literal accept (two rows, one unit), basis-id production membership
  over the SHARED basis-id corpus (one fixture, this unit + cnl-parse.2 — both sides of
  language equality): accept one-letter id + composite ids covering every rest-char
  category (digit/`_`/`.`/`:`/`-`), reject leading digit `1r` / leading punctuation
  `.r`/`:r`/`-r`/`_r` / uppercase / slash-bearing ids — per-production coverage BOTH
  languages incl. negated/escape/interval/multi-rule + JA patient-adjacent の (final
  interval/escape accept; missing/stray の reject), take(2) single-parse spot asserts. ckc-smt emit.rs's two
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
  per-language token tables (lexicon surfaces + the inventory module's fixed terminals +
  digits; tables differ, parser shared), atoms concept/negated/interval/escape with slot
  legality from the §10 role view (context/exception concept slots accept context-role
  surfaces, action target action_target-role — mirroring the grammar's slot
  alternations), かつ binds tighter than 、または;
  rejection battery: bare off-lexicon surface = parse error (≠ escaped accept), wrong-slot
  registered surface (e.g. an action_target-only concept as a context atom), malformed
  interval bounds, JA の-position violations (missing patient-adjacent の / stray
  mid-chain の — §10 composition decree), wrong-unit-for-var interval (the QuantityVar row's pairing — mirrors
  the grammar's per-row productions), numeral overflow boundary (i64::MAX parses, i64::MAX+1 = repairable
  parse error — §10's second over-approximation class), connective misuse, mid-token
  truncation, escape-payload contract
  violations (empty / over-80-scalars / control or quote-delimiter chars — plain parse
  errors, repairable).
- [ ] cnl-parse.2: document parser — full slot order (context 患者には、 action deontic tail /
  certainty paren / rule basis bracket / ただし exceptions, each sentence a single
  concept-or-escape payload + its own basis bracket), multi-rule documents under the §10
  document frame (exactly one LF terminates each rule, the last included), single
  deterministic pass (no backtracking); malformed battery (duplicate/missing slots,
  unterminated bracket, empty bracket, out-of-grammar basis id — leading digit/punctuation
  `1r`/`.r`/`:r`/`-r`/`_r`, uppercase, slash-bearing — repairable parse errors, + the
  shared basis-id corpus's accept vectors (cnl-grammar.1's fixture: one-letter id +
  every-rest-char-category composite ids) embedded in well-formed sentences parse, ASTs
  carrying those ids — valid-id under-acceptance excluded (Id::new the
  id gate: parser basis-id language == the pinned production, differential agreement
  total), exception sentence missing its bracket, exception escape missing its
  patient-adjacent の (§10 decree), empty
  document, missing terminal LF / stray blank line between rules / CRLF + lone-CR line
  breaks / stray whitespace (§10: whitespace variation NONE — parser language = grammar
  language, so differential agreement stays total; stray whitespace = repairable parse
  error), connective/negated-concept/interval inside an
  exception sentence); parser normalizes per-bracket basis sorted+deduplicated at AST build
  (§10 basis-order surface variation collapses at parse — set semantics; + a normalization
  pin: unsorted/duplicate-ref surface parses to the sorted-deduplicated AST); differential
  accept/reject agreement vs the Earley
  oracle over this unit's corpus.
- [ ] cnl-render: cnl_render.rs — render_ja/render_en canonical text (modality pair → the
  pair's canonical tail — the first tail-bearing row per pair — per language, basis emitted
  as stored — sorted+deduplicated by SHAPE validity, parse/from_ir normalize; render
  asserts, never re-sorts (rule + per-exception brackets),
  certainty optional
  paren, stored DNF order preserved — canonicalization never reorders semantics; document
  assembly = each rule's rendered line + one LF, last included (§10 frame — the exact bytes
  the text hashes lock); missing-pair
  lookup = Err, fail-closed — §10 totality + accept-total make it unreachable from accepted
  IR; SHAPE breaches never leave parse/from_ir — the grammar writes no empty
  DNF/conjunction or non-register interval, parse normalizes basis order: render ASSERTS
  cnl-ast's structural SHAPE sublayer (frame members = render's own output, outside the
  assert), fail-closed house panic style
  (render consumes validated ASTs — an unvalidated hand-built AST = instrument bug, never
  silent bytes)) +
  canonical-fixpoint spot tests (bounded-variation inputs — synonym tails, unsorted basis
  surface (parse-side normalization) —
  re-render canonical) + one should_panic pin (render on a shape-invalid hand-built AST —
  the assert fires before any lexicon lookup) + 3 M1-document
  byte pins from hand-built ASTs (§10 worked example, guideline_b contraindication tail,
  control shape) + 1 patient-adjacent-の byte pin (interval-atom-final context + escape
  exception, both languages — §10 composition decree).
- [ ] cnl-expressible: cnl_bridge.rs seeded with the shared §10 expressibility layer —
  CnlExpressibilityError (one variant per class) + check_cnl_expressible(clinical, lexicon
  (role + tail view), segments (segment_id → (kind, region_ids) keyed view)) over grounded
  lexicon-valid
  ClinicalIr. Classes: (direction, strength) pair without a tail-bearing lexicon row;
  wrong-slot vocabulary (§10 role view — population atoms, concept or quantity var, not
  population-role; condition atoms not condition-role; action targets not
  action_target-role; exception concepts outside the context roles: lexicon-MEMBER ids in
  slots no role admits — off_lexicon_ids checks membership only, and the committed IR
  schema's enums stay role-agnostic — slot legality is this predicate's, a per-slot schema
  re-derivation would re-bless committed schema bytes + §9 pins); EMPTY
  statements array (run.rs's accept battery currently pins empty ClinicalIr = accepted);
  quantity intervals without exactly one unsigned bound — signed / two-sided / boundless /
  same-side ge+gt or le+lt doubles (v1 one-unsigned-bound register; the committed schema
  requires only var over four independent optional bound fields, IntervalBound admits
  negatives; validate's §5 coherence rejects boundless + doubled shapes only TERMINALLY at
  bundle time and admits two-sided + signed — the predicate rejects all four repairably at
  acceptance); exception
  clauses not exactly one positive Concept atom (structural class, §10 single-concept
  register — multi-atom / atomless / ConceptNegated / Interval exception shapes are
  CNL-inexpressible yet schema-valid, so model-reachable); negative occurrences of
  interval-carrying entries — context ConceptNegated or the sole exception concept of a
  structurally valid clause, disjoint from the structural class (§10 bar: the locked tail
  interval-lowers positive occurrences only, a negative one sits as an unlinked Bool beside
  the Real interval); statements with EMPTY population+condition (schema minItems-free +
  bundle-valid, CNL's DNF derives ≥1 atom); exception clauses with EMPTY region_ids
  (bundle-valid — validate only resolves cited regions); statements citing no
  Recommendation segment — R empty ⇔ the wholly-exception-owned empty rule bracket under
  the §10 basis-ownership laws (closure − X == R), covers EMPTY source_segment_ids (the
  segment-closure helper lands
  here; cnl-bridge's from_ir rule bracket takes its exception-owned remainder, to_ir
  binding region_ids the whole closure); cited segments of non-normative kind — neither
  Recommendation nor Exception (Evidence/Cq/Definition/TableRow/Metadata: the locked
  rules.rs tail walks cited Recommendation segments' full region sets then appends clause
  region_ids, so a non-normative segment's clause-uncited regions silently vanish from
  rule provenance — clause appends are kind-blind);
  exception regions Recommendation-owned — a clause citing a region a cited
  Recommendation segment owns (X ∩ R ≠ ∅: the tail would emit it at least twice, recommendation
  walk + each citing clause); cited Exception segments not clause-covered — a cited Exception
  segment carrying a region absent from every clause (E ⊄ X: the normal form would widen
  it into a rule bracket the tail never reads Exception segments into; clause set scoped
  per statement — R/E/X are per-statement unions, sharing statements never pool
  coverage); exception regions outside the statement's
  segment closure — a clause citing a grounded region of an UNCITED segment
  (ExceptionRegionOutsideStatementClosure — every clause's region_ids ⊆ the cited
  segments' closure, i.e. each exception region in exactly ONE cited segment under
  closure-functionality; minimal breach: statement cites seg.0 = {r.1}, exception cites
  seg.1's r.2 — membership + grounding + closure-functional + kind + R-nonempty + both
  ownership classes ALL
  pass (r.2's segment is uncited, so neither Recommendation-owned nor clause-uncovered
  sees it — containment stays non-redundant under the ownership laws), yet from_ir would
  render r.2 exception-sentence-only and the re-bridge would
  derive source segments WIDER — provenance-unfaithful, and bridge-image IR contains by
  construction so the identity law never holds the shape; jointly with the R-empty
  class = the exception-owned ⊊ closure precondition, the pair co-occurring on a
  blanketing union with uncited excess — R-empty checked FIRST, battery pins the
  overlap naming it on an all-Exception-cited fixture (a cited Recommendation segment
  would keep R nonempty yet still name containment while the uncited excess remains —
  Recommendation-owned needs the blanket trimmed to the closure first); pinned
  first-failing-check order over the topology classes: closure-functionality,
  non-normative kind, R-empty, containment, Recommendation-owned, clause-uncovered —
  containment ahead of the ownership pair, which checks the in-closure
  residue); statements whose cited segments
  carry no region or share a region with another segment — the shared region need not be
  statement-cited, the unshared law reads FULL region sets (closure-nonfunctional — breaks
  segment recovery from region-level basis — region→segment derivation needs
  functionality; the empty-region segment synthetic-only — segment.rs mints only from
  grounded spans, an all-ungrounded row leaves
  boundary residuals, and IrBundle::validate rejects empty support — the shared region
  bundle-valid, validate never checks cross-segment disjointness; the predicate owns both
  fail-closed over its raw view). Tests: per-class rejection battery naming the variant
  (interval class: the exhaustive 16-bound-presence-mask × per-bound {-1,0,1} truth table —
  valid iff exactly one bound present ∧ value ≥ 0; expectation table shared with cnl-ast's
  AST-side battery, cross-side faithfulness by construction; ownership classes
  discriminated: sole basis an Evidence segment → non-normative kind, a clause citing a
  Recommendation-owned region → Recommendation-owned, a cited two-region Exception
  segment (beside a disjoint Recommendation basis — R-empty would mask) with a one-region
  clause → clause-uncovered, two statements sharing a two-region Exception segment, each
  clause covering a different region → stmt.0 clause-uncovered (document-global X == E —
  pins the per-statement R/E/X scope a document-global checker misses)) + boundary
  accepts (incl. two
  clauses distributing a cited two-region Exception segment beside a Recommendation
  basis — X == E by distribution) + a
  locked-corpus positive control (the 3 M1-derived ClinicalIr + their segments pass — the
  report-cnl.2 audit-render domain — derived in-test from the committed corpus; land ONE
  shared derivation helper, expressible-law reuses it). Fresh-module seed, no run.rs
  contact. Read scope:
  ir.rs shapes + the lexicon modality table and role view (lexicon.rs post-extract) + the
  normalize→segment→rules corpus-derivation entrypoints (the positive-control helper).
- [ ] accept-total: single_ir_accept calls check_cnl_expressible after its vocabulary +
  grounding stages — closing §10 render-totality for the one IR-landing route without a
  grammar/derivation guard (M1 derives from lexicon rows + integrity; single_cnl's grammar
  admits only lexicon tails): the closure's segment id-set parameter widens to the segments
  artifact's segment_id → (kind, region_ids) keyed view (the same artifact single_ir_fill
  already grounds against, no new input; today's bare id sets cannot express the topology
  or basis-ownership classes)
  + it takes the lexicon role/tail view; every CnlExpressibilityError → repairable
  FillReject::Schema naming the offense (mirrors off_lexicon_ids' empty-refs payload
  convention). Tests: each predicate class rejected THROUGH the acceptance surface (payload
  names the class) + boundary accepts + repair recovery; BOTH existing positive fixtures
  rebuilt role-valid + CNL-expressible with their former shapes re-pinned as named rejects
  (classifies' cited output — empty context, empty exception atoms; vocabulary's base —
  q.age_years interval under condition, ConceptNegated exception atom; both tail-less —
  accept_lexicon() gains tail-bearing modality rows, roles, and a quantity row); M2
  recorded-run battery green proves no
  retroactive census flip (a flip ⇒ stop, user decision). Read scope: run.rs accept-closure
  region + cnl_bridge.rs's predicate surface only.
- [ ] cnl-bridge: cnl_bridge.rs — to_ir + from_ir per the plan-header determinism rules
  (to_ir partitions context atoms by the §10 role view;
  from_ir = check_cnl_expressible at entry — its SOLE Err source, one class list shared
  with acceptance (cnl-expressible's taxonomy; wrong-slot IR is CNL-inexpressible because
  any rendering re-parses into a different partition, silently moving the atom); past a
  passing check the projection constructs no Err — a residual failure is a fail-closed
  panic (instrument bug), never a fresh Err class
  — §10 render totality; predicate-Err unreachable
  from bridge-image (= to_ir over ACCEPTED ASTs — single_cnl_accept rejects the CNL-side
  mirrors: orphan/shared cited regions, non-normative cited kinds, Recommendation-owned
  exception-bracket regions, per-rule clause-uncovered Exception segments (coverage never
  pools across rules), blanketing exception
  brackets),
  accept-total-guarded, and locked-corpus IR; to_ir = Err on any escape
  occurrence, law-harness-pinned) +
  both round-trip laws as pinned there (from_ir∘to_ir == bridge normal form;
  to_ir∘from_ir == id on bridge-image IR) + a partial-segment-citation law case (an
  EXCEPTION-FREE rule whose basis cites ONE region of a two-region Recommendation
  segment — the uncited
  region in NO bracket: bridge mints closure-wide binding region_ids, from_ir widens the
  rule bracket to the closure, re-bridge == id — the exact case an
  authored-bracket-union binding fails) + worked-example content test
  (parse(§10 JA) bridges to the §8.6 rule content) + a synthetic-lexicon partition spot
  test (a condition-role quantity's interval lands under condition, a multi-role concept
  lands by its context role) + rule_origins (plan-header origin map) + the pinned 2×2
  expansion test (two disjuncts × two exception sentences, distinct per-sentence brackets,
  a second-disjunct-only concept — pins stmt/exc ids + clause ownership (both sentences
  cloned per statement, content + basis duplicated), clause region_ids verbatim per
  sentence, per-statement source_segment_ids over the bracket union, the COMPLETE enumerated
  bind.<k> → concept oracle (§10 traversal — population, condition, action target,
  exceptions; the second-disjunct-only concept mints after stmt.0's full walk; a bare order
  assertion admits divergent deterministic impls), rule_origins = both rule ids → index 0;
  the fixture appends a trailing single-disjunct rule (own exception sentence) pinning
  cumulative offsets — rule.2 → index 1, stmt.2/exc.4, bind counters document-continuous —
  a per-rule counter reset passes the bare 2×2). Read scope:
  ir.rs shapes + the §10 lexicon role view
  (lexicon.rs post-extract — BOTH directions consume it: to_ir partitions by it, from_ir
  validates placement against it) + rules.rs
  derive_norm_ir contract — run.rs stays closed.
- [ ] cnl-laws: depth-bounded AST enumeration harness (all atom kinds — interval atoms
  across all four bound kinds, register values — × 1–2 disjuncts × 1–2
  conjuncts × all tail-backed modality pairs × certainty on/off × ≤2 exceptions × 1–2 basis
  refs per bracket (rule + per-exception, emitted sorted) × 1–2 rules per document (§10 LF
  frame under the
  laws; documents built through the shared frame assembly, semantic-projection equality
  total); + one unbacked-pair render-Err assertion) →
  render→parse identity both languages + cross-language agreement + canonical fixpoint +
  single-parse (take(2) Earley differential over a bounded sample, OracleBound escape) + the
  two bridge round-trip laws over the escape-free slice under the segment-fixture axis
  (fixtures supply the §10 bridge preconditions — the SPEC's ACCEPTED-quantifier domain) +
  a to_ir-Err-on-escape pin + a
  from_ir-Err pin (exception brackets blanketing every cited segment's closure, cited
  segments Exception-kind → empty rule bracket, R empty) + a wrong-slot from_ir-Err pin
  (an atom bucket its role contradicts) + an
  uncited-segment from_ir-Err pin (an exception clause citing a grounded region outside
  the statement's cited segments — the closure-containment class; shape: stmt.0 cites
  seg.0 = {r.rule, r.in} with clause 1 = {r.in} contained + clause 2 = {r.out} owned by
  stmt.1's cited seg.1, r.out grounded AND document-cited so first-clause-only or
  document-global-closure checkers pass it, offending-clause position permuted) + a
  segment-fixture axis pinning each §10 bridge-precondition breach + edge (fixture
  segments carry kinds — the basis-ownership laws quantify over them):
  orphan cited region (reject), region shared across two segments — the share pinned
  OFF-bracket, a Recommendation and an Exception segment sharing a region absent from
  every authored bracket, each cited via its own distinct region (reject —
  closure-functionality: the unshared law reads full region sets, an
  authored-bracket-only checker passes it), region-less
  cited segment (reject), a cited non-normative-kind segment (reject), an exception
  bracket citing a Recommendation-owned region (reject), a cited multi-region Exception
  segment with exception brackets covering only part of it (reject — beside a disjoint
  Recommendation rule bracket, naming clause-uncovered not R-empty), two rules sharing a
  two-region Exception segment, each exception sentence covering a different region
  (reject per rule — document-global X == E, the per-statement scope pin), a multi-region
  Exception segment distributed across two exception sentences whose union covers it
  (accepted — X == E by distribution, each sentence keeps its own bracket, the rule
  bracket on a disjoint Recommendation region),
  rule∩exception bracket overlap on an Exception-owned region, remainder nonempty
  (accepted —
  normal form moves the region to the exception sentence), one region shared by two
  exception clauses (accepted — labeled cover, both render), a later-disjunct-only concept
  (bind.<k> mints in post-split emission order, not textual order), an exception-free
  rule's bracket citing part of a multi-region Recommendation segment — the rest
  bracket-absent (accepted
  — normal form widens the rule bracket to the closure, binding region_ids closure-stable
  across the re-bridge)
  (plan-header form — bridge normal form: split + atom canonicalization +
  exception-owned segment-closed
  basis split, ≠ naive identity on multi-disjunct / atom-disordered / partially-cited-segment
  inputs). Codeco method; bound
  sizes to CI-sane runtime.
- [ ] expressible-law: the §10 expressibility-agreement harness — bounded enumeration of
  the §10 law domain (canonical (ClinicalIr, lexicon, regions, segments) tuples passing
  membership + grounding — role/tail legality predicate-owned): positives = to_ir over
  enumerated accepted ASTs (bridge-image, cnl-laws' method) + the 3 locked-corpus derived
  ClinicalIr (cnl-expressible's derivation helper); negatives =
  per-class mutations landing in EVERY CnlExpressibilityError variant while staying
  in-domain (the predicate stays the deciding acceptance layer; the closure-containment
  mutation grounds a LATER clause's region in an UNCITED segment ANOTHER statement
  cites — membership + grounding stay green, first-clause-only + document-global-closure
  checkers caught; the basis-ownership mutations re-kind a cited segment Evidence,
  retarget a clause region to a Recommendation-owned one, drop an Exception segment's
  second region from every clause, and re-scope clause coverage across statements (two
  statements citing one two-region Exception segment, each covering a different region —
  document-global X == E holds yet each statement is clause-uncovered) — kind-blind and
  document-global checkers pass all four; positives inherit
  cnl-laws' distributed-Exception-cover axis case via
  to_ir (the partial-Recommendation-citation case collapses through to_ir to the
  full-citation IR — its discrimination lives in cnl-laws' normal-form law, not
  here)). Assert per
  case, three ways: single_ir_accept over canonical bytes ⇔ check_cnl_expressible ⇔
  from_ir — Ok together or rejecting the SAME class — and on the Ok side from_ir's AST
  renders both languages (§10 render totality end-to-end). Bounds CI-sane. Read scope:
  run.rs accept-closure call surface + cnl_bridge.rs + cnl-laws' enumeration harness +
  cnl_render.rs render entrypoints (the Ok-side bilingual assertion).
- [ ] codes-cnl: DiagnosticCode +CnlParseError/CnlRoundTripMismatch/CnlUnregisteredConcept/
  CnlInexpressibleIr (fieldless_enum append; CnlInexpressibleIr = report-stage only,
  report-cnl.2 lands it on a guard-less route's from_ir reject — record outcome
  unsupported (§4.4), canonical payload map {predicate_class, pipeline_id, document_id},
  predicate_class = the FIRST failing check under the predicate's pinned order (the order
  expressible-law already forces), stable spelling = the CnlExpressibilityError variant
  name; NO FillReject arm, EXCLUDED from RouteTaxonomy — route-single-cnl.6 wires the
  fill/accept cnl codes only) + FillReject +Parse(String) (repairable → cnl_parse_error) /
  +Unregistered{surface, position} (terminal → cnl_unregistered_concept; payload = the
  lexicon-entry proposal) / +Instrument(String) (terminal fail-closed →
  cnl_round_trip_mismatch, spends no repair) + model_fill.rs mapping + stage tests (repair
  recovery / both terminals / instrument path). Codes carry payload, empty refs
  (DiagnosticRecord convention).
- [ ] route-stage-handles: behavior-locked rework (widening the
  positional array for the bridge stage compounds off-by-one + provenance risk) — retire
  run.rs's positional stage plumbing (Resolved.pipeline_step_ids [Id; 8] UNUSED_STAGE-padded;
  index consts MODEL_FILL=2/DIRECT_VERIFY=3/COMPILE=4/VERIFY=5/TRACE=6/REPORT=7 over
  PROCESSING_STAGE_KINDS) for validated named handles: StageHandle {kind, step_id} +
  per-shape RouteStages enum with named stage fields (M1Layered/SingleIr/DirectSmt;
  resolve_route constructs it with its full fingerprinting kept verbatim, incl. the
  model-fill output_artifact_kinds validation — no sentinel, no resize); RouteShape DERIVES
  from the RouteStages variant (accessor, never stored independently — no shape/handle
  mismatch); finish_processing_stage + emission/provenance sites take a handle (kind
  travels WITH step_id, never re-derived from an index); run-scoped synthetic trace/report
  (processing_stage.run.*) become a named run-level handle pair replacing their
  index-derived kinds (producer pins stay byte-identical); diagnostic labels that
  intentionally differ from the registry kind (model_fill_smt payloads) stay literal —
  handles replace index-derived kinds only; compile_verify_group + producers parameterized
  by handles. Zero behavior change: suites green, semantic pins preserved (tests asserting
  the retired [Id; 8]/UNUSED_STAGE shapes rewrite mechanically to handle equality), M1/M2
  pins unchanged.
- [ ] route-single-cnl.1: registry data — processing_stage.m3.model_fill_cnl (model_fill,
  [sdg,segments]→[cnl_document]) + m3.bridge (bridge, [cnl_document,segments]→[clinical_ir]) +
  pipe.m3_single_cnl (7 stages; m2.assemble + m1.compile/m1.verify reused — a
  RouteStages::SingleCnl named-handle variant per route-stage-handles' representation, the
  bridge stage a named field, never an index shift; resolve fixtures follow) + exp.m3_cnl set
  binding (direct baseline) + prompt.single_cnl entry + first-draft template +
  single_cnl_prompt composer (single_ir_prompt mirror + per-slot CNL vocabulary blocks
  from the §10 role view: context adnominal/
  negated surfaces, action-target surfaces, action nouns, modality tails, quantity
  surfaces+units, basis region ids;
  fixture proves ordering non-trivially — the f1 pre-sorted-fixture lesson); RouteShape::
  SingleCnl fingerprint (7-kind sequence + model_fill out [cnl_document]) + resolve +
  select_record_{schema,prompt} + manifest_inputs want-set arms (schema.clinical_cnl = the JA
  decoding constraint) + resolve-rejection additions + registry check green. run.rs read
  scope: resolve/consts + manifest want-set + prompt-composer regions only.
- [ ] route-single-cnl.2: single_cnl_accept closure — parse (Parse reject, repairable) →
  AST validate (structural = debug-assert — parse output shape-valid by construction;
  lexicon-scoped ENFORCED, its sole parse-unenforced clause = the §10 negative-occurrence
  bar (token tables already guarantee tails/refs/vars/roles) → repairable reject carrying
  the §10 complement-interval repair) →
  escape scan over context + exception + action-target slots (Unregistered terminal) →
  grounding (every bracket's basis regions ⊆ regions — rule + per-exception — each cited
  region anchored in exactly ONE segment + derived segments' region sets unshared
  (closure-functional; orphan/shared reject) + the §10 basis-ownership laws over the
  derived segments (land HERE with the other bridge preconditions: every derived segment
  Recommendation|Exception-kind, ≥1 Recommendation-kind — R nonempty (⇔ the nonempty
  remainder once the laws hold) — and PER RULE the exception-bracket union == the
  Exception-kind closure, X == E from the primitive pair: an
  exception bracket citing a Recommendation-owned region rejects, a derived Exception
  segment's region absent from every exception bracket rejects; exception-owned ⊆ closure
  holds by construction, derived segments span the bracket union, the containment class
  bites IR-side only), derived
  segments
  ⊆ segments; Grounding terminal) → re-render + re-parse round-trip (Instrument on mismatch)
  → Ok(CnlDocument); battery mirrors single_ir_accept's (valid / parse-repair-recover / a
  negative-occurrence reject — interval-carrying concept in an exception slot, the reject
  naming the complement-interval repair / both
  terminals / instrument / empty-grounding panic) + the basis-ownership six (rule basis
  owned only by an Evidence segment → reject; an exception bracket citing a
  Recommendation-owned region → reject; a cited multi-region Exception segment (beside a
  Recommendation rule bracket) only
  partially exception-bracket-covered → reject; two rules sharing a two-region Exception
  segment, each exception bracket covering a different region → reject per rule
  (document-global X == E — the per-rule scope pin); a multi-region Exception segment
  (beside a Recommendation rule bracket)
  distributed across exception sentences whose union covers it → accept; partial citation
  of a multi-region Recommendation segment → accept, the bridge later normalizing to the
  full closure). run.rs read scope: accept-closure region +
  cnl modules.
- [ ] subproc-runner.1: behavior-locked extraction — ONE shared subprocess runner (home
  ckc-smt, ckc-cli already depends on it — confirm at impl) absorbing the mirrored
  spawn/timeout/kill/drain machinery of model.rs + verify.rs behind spawn-retry's injectable
  seam; the runner core carries model.rs's stdout-EOF tracking and the adapters PRESERVE
  today's asymmetry — model: clean exit gates Completed on the EOF flag (else
  CaptureIncomplete); solver: verify.rs has NO EOF state, mints Completed{verdict} from
  whatever stdout snapshot exists at drain-grace expiry — the solver adapter ignores the
  flag THIS unit (subproc-runner.2 closes it); zero behavior change, existing suites the
  gate (test edits = imports only).
- [ ] subproc-runner.2: runner hardening (the M2-deferred codex-rejected fixes + drain cap,
  once, shared): bounded stdout/stderr capture with an explicit truncation state, checked
  deadline arithmetic (Instant+budget overflow), post-grace detached-drain cap/reap; PLUS
  the ruled compatibility decision — the solver FAILS CLOSED on incomplete stdout capture,
  complete = stdout EOF within the drain grace AND un-truncated by the new cap (EOF alone
  is insufficient once a cap can drop bytes before EOF; the model keeps its documented
  whole-output Completed invariant under the same conjunction): the completeness gate
  covers EVERY verdict-parsed fate — Completed AND ExitFailure alike, since verdict.rs
  parses the leading verdict token over exit-1 replies too (z3's verdict-then-retrieval-
  error shape), so a truncated exit-1 prefix could otherwise mint a phantom verdict;
  incomplete capture mints a capture-incomplete RunOutcome — verdict None,
  SolverExecutionFailure-class diagnostic, NO Q2 — never a verdict over a partial
  snapshot, while complete exit-1 replies keep parsing as today — a deliberate behavior
  CHANGE closing subproc-runner.1's preserved asymmetry; deterministic tests through the
  injectable seam drive both runners' EOF-absent clean exits PLUS a consumer-level
  assemble_result test (capture-incomplete run → verdict None + execution-failure
  diagnostic + no Q2).
- [ ] route-single-cnl.3: single_cnl_fill + execute_routes SingleCnl arm (head reuse → fill →
  bridge + assemble tail into the per-group compile/verify loop via the shape's named stage
  handles — route-stage-handles' representation) + landing
  (cnl_document + clinical_ir + bundle wrappers route-namespaced; bundle input_hashes cite
  cnl_document + accepted cassette; check TraceNodeKind coverage at impl) + §4.6 events (fill
  + bridge; clock discipline — compose prompts outside timed intervals) + landing-gate test
  + findings-owner selection (plan-header bullet — this unit lands the second compiled+results
  route, the shared tails break without it): GroupTrace owner mark set by execute_routes,
  trace finding-mint + report-tail results filtered to owner groups (single_ir), single_cnl
  groups stay landed trace-DAG nodes; owner test = the two-compiled-route run lands trace +
  report green, findings/results byte-equal the owner-only view.
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
  accepted_rules; surface_tokens = the committed JA lexer's (cnl-parse.1 token layer)
  token count summed over the accepted CnlDocument's stored canonical JA per-rule texts —
  the hash-locked bytes, LF frame outside the count; deterministic + runtime-free per
  §7.3 — a model-runtime tokenizer count REJECTED, it would bind the metric to a
  versioned runtime-tokenizer replay dependency, identity+counts separately attested;
  accepted_rules = the document's rule count) +
  round_trip_identity_rate + surface_tokens_per_accepted_rule rows (§7.3
  surface-quality family; per-route Σ surface_tokens / Σ accepted_rules, exact fractions;
  applicability explicit: every ACCEPTED single_cnl fill carries the three fields
  ATOMICALLY — partial presence = fail-closed instrument error, rejected fills carry
  none — non-CNL routes never carry them → no rows, and an applicable route with zero
  accepted docs emits the §7.3 zero-denominator not_applicable rows — prove M2
  replay rows byte-unchanged) + delta/NA wiring + tests (incl. a worked-example
  token-count pin from observed lexer output, all-rejected NA, mixed accept/reject,
  partial-fields instrument error).
- [ ] metrics-faithful: FillObservation optional ir_match bool + ir_faithfulness_rate row
  (§7.3 translation-faithfulness family, §10) — run.rs fill-tail computes it for IR-landing
  model routes: landed ClinicalIR == the deterministic
  reference derivation, the M1 normalize+derive chain recomputed in-run over the route's own
  head values already in hand, compared under the §10 faithfulness projection — binding
  region_ids excluded (bridge bindings mint segment-closure regions off sentence-basis
  brackets — rule + per-exception — never M1's mention regions, provenance; the one
  by-construction divergence — binding status/cardinality diverge only OFF the locked
  corpus, canonical-label Exact per distinct concept vs M1's per-(segment, candidate set)
  surface-derived mints = measured misses), all else
  exact incl. stmt/bind/exc ids (single_ir: accepted fill; single_cnl: bridged IR; direct_smt
  lands no IR → field None, row not_applicable). Golden path pins projection-match 1.0
  (certifies the corpus miss-free). Rows gate on observations carrying the field (M2 replay rows
  byte-unchanged, omit-None); deltas ride the existing route-delta loop. Tests: match /
  mismatch / None-NA + golden-path 1.0 + M2-replay byte-pin. Read scope: normalize/derive fn
  signatures, run.rs fill-tail region, metrics.rs row assembly — the §7.3 family text names
  the rationale.
- [ ] report-cnl.1: Report shape — cnl_documents keyed (pipeline, document) per §10 ({ja,en}
  text hashes) + cnl_rules (same key, inner map normative rule id → {ja,en} strings — §10
  origin-map keying; a split legitimately duplicates one rule's text under several ids) +
  optional findings_owner_pipeline_id (§7.2 owner field — plan-header findings-owner
  bullet), omit-empty/omit-None members + validate rules (sorted ids, inner rule ids
  prefixed by their outer document key — `<doc>.rule.` agreement, line-break-free strings,
  code-span-inert (no literal backtick — §7.2's single-backtick spans), equal
  (pipeline, document) key sets across cnl_documents and cnl_rules; owner field when
  present a nonempty id-form pipeline key) + populated
  fixture + byte pins; M1 bytes byte-identical (plumbing half).
- [ ] report-cnl.2: population + audit views — assemble_report CNL inputs (single_cnl route:
  the accepted CnlDocument's own text/hashes — audit honesty; other routes incl. M1: from_ir
  + render over accepted ClinicalIr — a from_ir predicate reject (guard-less route,
  arbitrary M1 inputs) omits that (pipeline, document)'s cnl_documents/cnl_rules entries
  AND audit files, landing ONE cnl_inexpressible_ir diagnostic (codes-cnl payload map,
  empty refs, record outcome unsupported) computed at report population BEFORE the
  summary snapshot (run.rs's report tail today snapshots diagnostics pre-assembly +
  hardcodes an Ok/no-diagnostics event — reorder so the record lands in report.json's
  diagnostics summary AND the report-stage event, whose outcome then follows §4.4's
  severity order), assembly never fails ON IT — other validation/instrument failures
  stay fail-closed (locked corpus
  renders every IR-bearing route — cnl-expressible's positive control — so pins stay
  populated;
  test: a synthetic inexpressible-IR fixture whose pair IS the findings owner, with a
  finding row AND a documented no-conflict row — validate passes, owner FIELD present
  while the owner entry is absent, md omits the quotes per §7.2 fallback, audit-file
  census + every non-CNL member unchanged);
  cnl_rules per (pipeline, document) = rule_origins over
  that route's CnlDocument → per-rule-id {ja,en} text, ONE lookup path — accepted doc: the
  split duplicates text under each derived rule id; from_ir doc: identity origins) + run.rs
  report-tail lands
  audit/<pipeline-id>/<doc-id>.cnl.{ja,en}.txt (§10 keying — a multi-route experiment accepts
  the same document several times; body = the document's canonical bytes verbatim, §10
  LF-terminated frame; write_under + byte read-back whose re-hash must equal the
  stored/report text hash (independent frame check), report_en.md pattern; text
  hashes into report.json; §4.4 provenance widens — report wrapper/event input hashes add
  every consumed accepted artifact: each rendered route's accepted CnlDocument/ClinicalIr
  + segments beside the existing trace/lineage/graph/verifier inputs, with pins) +
  findings_owner_pipeline_id population = the §7.1 owner (the
  first bundle-bearing pipeline in experiment binding order — the selection the report
  tail already applies, every M1/M2/M3 experiment carries one) + validate tightened here
  (findings or documented no-conflict results present ⇒ owner field present) + M1/M2
  report byte-pin re-bless sweep (deliberate, bless-from-observed — the owner field rides
  the same sweep).
- [ ] report-cnl.3: md renderers — finding/no-conflict bodies quote rules as CNL beside
  quoted spans, single-backtick inline code spans = §7.2's normative delimiter (JA body
  quotes JA CNL, EN body EN CNL; Labels; lookup = finding rule_id → the
  findings_owner_pipeline_id pipeline's cnl_rules entry ALONE, owner-labeled —
  deterministic over report.json alone (the owner id is a report field), at most one quote
  per (row, rule_id, language) — result rows carry rule-id VECTORS, every carried id
  considered, document-prefixed inner ids make each per-rule lookup unique; non-owner
  entries NEVER render beside findings — positional rule ids don't align routes
  (plan-header findings-owner bullet), their views stay in audit surfaces; a rule id the
  owner's entry doesn't carry renders nothing, omit-empty md — the SPEC-stated fallback;
  an absent owner field beside nonempty findings/no-conflict rows is a validate failure
  (report-cnl.2's implication), never a renderer fallback) + rendered-body const re-bless
  (Z3_VERSION-normalize pattern) + emission-order/validate coupling tests + the
  misalignment discriminating test: a fixture whose non-owner route inserts a leading rule
  — owner and non-owner both carry `<doc>.rule.0` with DIFFERENT text — AND whose
  non-owner pipeline id sorts FIRST in cnl_rules key order (a scan-all/first-match
  renderer picks non-owner text and fails; only the owner-field lookup passes), unique
  EN+JA marker text per route, exercised in a finding row AND a no-conflict row: owner
  markers render in both md bodies, non-owner markers in neither, while staying present
  in report.json cnl_rules and the audit views.
- [ ] canon-props: canon-layer generated-case harness (standing AGENTS.md-preferred
  hardening, never an M3 entry gate; sits after M3's last canonical-shape change —
  codes-cnl, metrics, report) — bounded deterministic enumeration (cnl-laws' method, zero
  new deps) over an EXPLICIT hand-maintained generator registry of the public
  Canonical+CanonRead families, cnl.rs's included (completeness = a unit-time sweep of the
  impls, rechecked at milestone review): over canonical-storage values emit→strict-read =
  identity; over arbitrary generated values emit normalizes (sorts/dedups set-form vecs,
  policy-normalizes strings) and emit∘read∘emit = fixpoint; generated noncanonical byte
  mutations (key reorder, whitespace, duplicate keys, unsorted sets) rejected on strict
  read; every StringPolicy idempotent over its ACCEPTED inputs + stable rejection of the
  rest (IdentifierAscii is fallible). Bounds CI-sane; existing byte-pins stay the
  canonical-order authority.
- [ ] record-cnl.1 (gated: model runtime): constraint/tokenizer audit (cnl-grammar.1b already
  proved compile + one bounded emission — this unit is the full pass over the committed
  grammar): tokenizer audit of every grammar terminal (§9 truncation +
  UTF-8 boundary lessons); probe repetition points (DNF connectives / exception sentences /
  rule+) for degeneration loops under the token budget — archived-PoC prior: grammar-masked
  verbose forms loop at unbounded repetition and truncate mid-structure (§10 emission-posture
  paragraph carries the mitigation frame); template refinement from observation + RecordParts byte-verify
  green + model_ms_per_call budget entry. Machine specifics → runtime.local.md, committed
  bytes stay engine-agnostic.
- [ ] record-cnl.2 (gated: model runtime, LIVE): record exp.m3_cnl in a scratch root whose
  cassette store is newly created + EMPTY (persist overwrites same-key without compare →
  stale keys survive as orphan attempts; memory's empty-store rule bound here as
  acceptance); census route.single_cnl/** vs the run's actual model-attempt ledger in BOTH
  directions; identity-agreement vs existing cassettes — agree ⇒ M2 cassettes stand (no
  re-bless), drift ⇒ full re-record + M2 recorded_run re-bless fallback; replay matched;
  copy into committed /cassettes only after identity agreement + replay verification pass,
  REPLACING the committed route.single_cnl subtree, never merging (base + derived-seed
  repairs, REAL identity, audit-exempt); do-not-read sync verified (/cassettes already
  deny-Read — census via runtime indirection).
- [ ] record-cnl.3: recorded-run battery — exp.m3_cnl census/§9/re-render/replay-matched/
  audit-file pins over the committed cassettes (recorded_run.rs pattern, own test file — M2
  battery untouched).
- [ ] acceptance-m3: §10 acceptance themes against the recorded run (3-route raw-before-delta,
  determinism laws green, round-trip rate 1.0 on accepted docs, faithfulness rows emitted
  beside surface rows — measured never gated, golden path 1.0, audit views for every
  IR-bearing acceptance (single_ir + single_cnl; direct_smt no IR → none; M1's via its
  re-blessed golden run) + finding/no-conflict md quoting owner-route CNL only, golden-cassette reproduce-M1
  gate, replay byte-stability, grammar/lexicon drift guards,
  §0 vocabulary) via the acceptance-driver pattern; tag accept/m3.
