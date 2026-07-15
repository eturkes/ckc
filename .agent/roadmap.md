# CKC roadmap

Build plan for /session-prompt — the session protocol, bookkeeping format, and stamp
semantics live in that command; SPEC.md is the design authority, its §2 the milestone
sequence. One milestone at a time: header `## <name> — plan <hash> — review <hash>` over an
ordered unit checklist; unchecked lines carry the full unit spec; checked items collapse to
one-line stubs `- [x] <id>: <gist>. NN% NNNK/1M <hash>`; a reviewed milestone keeps its
stubs until the next milestone is planned (that planning reads them to right-size units),
then persists as a bare header; git history retains all removed text. The active milestone's
`plan <hash>` shows `PENDING` until its first unit's closing commit fills it — the planning
commit is then known (M1's `89c4cba` was filled retroactively too). The trailing Backlog
section is NOT a milestone — it holds triggered, unscheduled items a future planning session
may pull in.

## M1 scaffold — plan 89c4cba — accept m1 — review f6d68a0

## M2 multi-hop PoC — plan 2a4f03d — accept m2 — review 5ec33f7

## M3 ClinicalCNL product line (APE fork) + loop framework — scope f76e1fa — plan 3bb4a38 — replan ac8ce88

Scope = SPEC §10.6 (2026-07-13 product push = design authority; §0 honesty note). Deliverable:
vendored APE fork building green under SWI-Prolog (9.2.9; environment gate — every
Prolog-running unit re-confirms functionally via its own consult/run; smoke =
`sh clinicalcnl/clinical/ape_build_smoke.sh`), ClinicalCNL v1 as a fail-closed ACE profile
(EN surface) over a demand-authored clinical ulex, a DIRECT DRS → clinical-Prolog-KB mapping
(AceRules = bounded differential only; labeled exception overrides, per-sentence provenance,
byte-identical re-emission), Prolog-side conflict/no-conflict queries re-deriving the M1
docA×docB thread in-lane, a locked synthetic conformance corpus behind ONE runner command, and
the /cnl-optimize + /loop round framework. Rust tree untouched this milestone; engine-agnostic
rule unchanged (SWI-Prolog/APE are nameable solver/host tooling like Z3). No unit needs the
model runtime. History: the 13-unit downstream design was superseded 2026-07-13 (codex-accepted;
text at `git show 3bb4a38:.agent/roadmap.md`); the REPLAN unit executed 2026-07-14 probed the
in-tree `ape.exe` substrate directly — decomposition below rests on §L·probe + decisions D1-D10.

Cross-unit decisions:
- Layout: three zones keep fork-vs-vendored-vs-ours auditable — `clinicalcnl/` root = the APE
  fork (upstream layout preserved: diffability + GPL corresponding-source clarity);
  `clinicalcnl/vendor/acerules/` = the AceRules adaptation-source subset (see ape-vendor);
  `clinicalcnl/clinical/` = CKC clinical additions ONLY (gate, profile, mapping, conflict,
  ulex, corpus, goldens, runner). Upstream files edited only where wiring demands, each edit
  commented `% CKC (<date>):`. Fork provenance = `clinicalcnl/CKC_FORK.md` (distinct name — APE's
  own `README.md` sits at the root; the two upstream roots also collide on `.gitignore`/`LICENSE.txt`).
- Licensing (SPEC §11.5): APE + AceRules evidence rows land in the ape-vendor commit as §11.5
  PROSE (extend the "Standing verdicts" sentence — no registry file); ape-vendor owns the
  acquisition-record schema + first-hand header verification. Clex: VENDORED in-tree (full
  lexicon, GPL-3.0-or-later, `vendor/clex/`, §11.5 row) — ape-build wired it drop-in; the
  upstream suite uses the in-tree copy (no live download); also a clinical-term mining seed.
- Fail-closed (design FINAL, probe-grounded): TWO layers. (1) RAW gate BEFORE APE = WHITELIST —
  framing grammar + per-sentence registered-pattern token templates; any non-matching document
  or sentence rejects. The whitelist subsumes the token bans: inline `n:`/`v:`/`a:`/`p:`
  prefixes, unregistered capitalized tokens (else silent `named(_)` + mere warning, p6),
  pronouns beyond the pattern-anchored frame "It", `or`/`every`/quantifier surfaces, decimals,
  leading zeros, spaced multiword terms, comments, quotations. APE irreversibly erases surface
  facts (prefix tokens, numeral spellings, comments, silent anaphora merges) → the raw gate is
  the ONLY enforcement point for surface bans. (2) DRS profile AFTER APE = defense-in-depth:
  zero-message law (every registered pattern parses warning-free → ANY message rejects),
  frame-op↔keyword consistency (D1), recursive `named(_)`-vs-registry scan, guard/action shape
  whitelist, canonical-DRS equality vs the pattern's golden. Guessing stays off
  (noclex+ulextext); lowercase OOV hard-errors (p6); a clean parse alone never proves lexical
  closure — the two gates do.
- Determinism: same accepted document → byte-identical KB text (canonical clause order +
  dedicated emitter); provenance = {document id, sentence index} on every emitted clause group.
  Per-sentence APE invocation (D2): DRS SID is always 1; the sentence index comes from the raw
  layer's block counter; TID stays APE-native.
- Conformance runner: ONE command (plunit driver under `clinicalcnl/clinical/`) = surface
  goldens + gate/profile batteries + mapping/oracle batteries + conflict + corpus round-trip +
  attack-surface negative + upstream-suite leg; THE /loop round gate. Milestone acceptance =
  runner green over the locked corpus + a ledgered manual R1 round (loop-framework).
- Replan-resolved design decisions (2026-07-14, each grounded in §L·probe):
  D1 modality/strength: direction lives in the ACE frame; strength lives in the raw-layer rule
  header's modality keyword. Frames (surface → DRS op): "it is recommended that S" →
  `should` (for); "it is admissible that S" → `may` (permit); "it is not recommended that S" →
  `-[should]` (against); "it is not possible that S" → `-[can]` (contraindicate). Keyword →
  (required op, direction, strength): recommend→(should,for,strong), suggest→(should,for,weak),
  may-consider→(may,permit,weak), not-recommend→(-should,against,strong),
  not-suggest→(-should,against,weak), contraindicate→(-can,contraindicate,strong). Aux forms
  (should/may/cannot) parse but stay UNREGISTERED → gate-rejected (one canonical surface per
  direction). 1:1 decode = keyword→pair total+injective + op-mismatch reject battery.
  D2 per-sentence APE invocation — kills cross-sentence silent referent merging structurally (p7).
  D3 lexical surfaces: conditions = countable nouns, guard conjunct "the patient has a <cond>";
  drugs = registered `pn_sg` proper names, action "the patient takes <Drug>"; v1 action verb =
  takes → `act.administer`; population = "a patient" introduced by the first guard conjunct,
  every later mention = definite "the patient" (within-sentence anaphora with antecedent =
  warning-free); mass nouns excluded v1 (ACE demands a determiner on mass, p6).
  D4 DNF: or-guards REJECTED (p7: `v()` + broken then-part anaphora + warning); disjunction =
  one rule sentence per disjunct, grouped under one rule id by the raw layer → stmt.k per
  disjunct (bridge-oracle numbering preserved).
  D5 in-guard negated-concept DEFERRED from the v1 surface ("does not have" parses clean, p7 →
  reject-battery member); all negative context via labeled exceptions (NAF); §5 IR keeps
  negated-concept — a profile-scope restriction, no SPEC change.
  D6 exceptions: raw-layer labeled blocks bound to rule ids; ACE body = self-contained
  'A patient has a <cond>.' (single concept, interval-free); mapper compiles the PROLEG NAF guard.
  D7 certainty: optional raw-header field {high|moderate|low|very_low} → KB fact; never in ACE.
  D8 back-reference: the no-reference arm — per-sentence isolation; within-sentence definites
  require an in-sentence antecedent (anaphor warning = reject via the zero-message law).
  D9 interval surface: "the patient has an age of <marker> <INT> years"; markers at least→geq /
  at most→leq / less than→less / more than→greater; exactly/bare-eq = reject members (single-bound
  law, 16-mask battery); geq/greater land top-level, leq/less/exactly land in NESTED sublists
  (guard walker flattens one level).
  D10 conflict arithmetic: exact-rational bound algebra (SWI native rationals), open/closed
  bounds distinct (geq 18 vs greater 18) — integer FD INSUFFICIENT (18<X<19 empty over FD,
  nonempty over Q), no clp library dependency; DNF disjunct-pair enumeration + concept polarity
  + exception expansion.
- §L·probe (2026-07-14 empirical ledger; substrate = in-tree APE under swipl 9.2.9. RECIPE —
  raw DRS: from `clinicalcnl/` run `swipl -g "consult(get_ape_results),
  ace_to_drs:acetext_to_drs('<ACE>',_,_,Drs,Msgs), …" -t halt` (root `get_ape_results.pl` sets
  the search path; guess=off default); fileless noclex+ulex: prepend `clex:set_clex_switch(off),
  ulex:add_lexicon_entries([<entries>])`; product seam: `get_ape_results([text='…', noclex=on,
  ulextext='<entries>', solo=drs], CT, C)` → success CT=text/plain + serialized numbervar'd DRS
  (THE golden byte format), failure CT=text/xml `<messages>` (fail-closed discriminator). Probe
  ulex: noun_sg patient(human)/sepsis/pregnancy/'severe-renal-impairment'/age/year +
  noun_pl(years,year) + pn_sg('Abx-A') + tv_finsg takes/has + tv_infpl take/have):
  p1 frames: recommended→should, admissible→may, possible→can, necessary→must; "not <same>" →
  `-drs([],[op(…)])` for all four; "It is false that it is recommended" ≡ "It is not
  recommended". REJECTED as not-ACE: suggested / permitted / obligatory / prohibited /
  "strongly recommended" / "must not".
  p2 aux: should/may/must + should-not/may-not parse with subject HOISTED outside the op box;
  inside If-then aux ≡ frame (identical DRS modulo TIDs) — frames registered, aux rejected (D1).
  p3 rule shape: If-then → `=>(drs(guard), drs([],[OP(drs(action))]))`; then-part definites
  resolve INTO guard referents (take(A,…) reuses guard A).
  p4 thread parses CLEAN, zero messages: docA-core 'If a patient has a sepsis and the patient
  has an age of at least 18 years then it is recommended that the patient takes Abx-A.' →
  =>(…,[should(take(A,named('Abx-A')))]); docB (+ pregnancy conjunct, "it is not possible
  that") → -[can]; control ("less than 18") → -[can] with nested [relation(of),object(year,less,18)].
  p5 intervals: "has an age of at least 18 years" clean; markers→CountOp per D9; bare "18
  years"→eq, "exactly"→exactly (nested); bare number without unit noun + "years old" REJECT;
  "the age of a patient is …" parses but anaphor-WARNS → rejected surface (D8/D9 form wins).
  p6 lexicon: lowercase OOV under noclex → hard error message(error,word,…,'Use the prefix n:,
  v:, a: or p:.'); capitalized OOV → `named('X')` + warning 'Undefined word. Interpreted as a
  singular proper name.' = THE hole (raw gate + registry close it); registered pn_sg →
  `named()` ZERO warnings (registry membership = the authoritative discriminator); bare mass
  rejects, "some <mass>" parses (→ D3 countable-only); hyphenated lemmas = single tokens.
  p7 hazards (reject-battery evidence): or-guard → `v(drs,drs)` + then-part anaphora BREAKS
  (fresh referent + warning); two-sentence text: 'The patient' silently merges to the
  sentence-1 referent, ZERO warning (→ D2); pronoun 'he' silently resolves, ZERO warning;
  'Every patient takes Abx-A.' → the SAME => DRS shape as If-then (surface restrictions are
  enforceable only raw-side); bare-then (no frame) parses; in-guard "does not have" → clean
  `-drs` inside the guard (→ D5).
  Every row reproduces from the RECIPE + the quoted sentence; goldens re-capture them in-tree.
- Reading legend (banked-once shared pins; units cite by tag — read these, skip rediscovery).
  §L·spec (design authority; section-anchor + distinctive-string, ≈line drifts): §5 domain IR
  (`NormativeRule`/`Action`; `ContextExpr` = DNF over concept | negated-concept | "quantity
  interval" atoms, ≈L456); §6 conflict + LP profile ("rules-as-data" ≈L597; byte-emission
  "Emission is deterministic" ≈L578; lane-separation "LP verdicts never replace" ≈L609); §8.6
  worked thread (≈L826); §10.4 sentence model = shape reference (the APE profile REPLACES the
  bespoke grammar; full EN slot table `ecc19d3:SPEC.md` L923-944); §10.6 product line ("Compile"
  ≈L1291, determinism ≈L1296, "IR bridge" deferral ≈L1299). §L·ids (concept inventory — mirror
  `corpus/lexicon/ja_core.yaml` EXACTLY; EN surfaces = D3/D9): `pop.adult`/`pop.child` are
  INTERVAL-CARRYING → collapse to their interval atom (`age>=18`/`age<18`), NOT a concept atom;
  `cond.sepsis`, `cond.renal_severe`, `cond.pregnancy`; `drug.abx_a`; `act.administer`;
  `q.age_years` (var; surface "age"/"years"). Modality → (direction,strength) table lives in D1;
  certainty {high|moderate|low|very_low} = proof-visible annotation, NOT consumed by conflict
  logic. §L·drs (APE parse reality — §L·probe-confirmed 2026-07-14): DRS = `drs(Referents,
  Conditions)`, each condition `Cond-SID/TID`; rule + ops + interval shapes = p1-p5;
  `object(Ref,Lemma,_,Unit,CountOp,N)`, CountOp ∈ {geq,leq,less,greater,exactly,eq};
  `prolog/utils/is_wellformed.pl` = profile allowlist base; first-parse-wins determinism ≠
  unique reading → goldens pin ONE canonical DRS per registered pattern; API =
  `get_ape_results/2,3` (module `ape`); fail-closed signal = text/xml `<messages>` / non-empty
  Msgs / `drs([],[])`, never exit code. §L·acerules (AceRules reality, corrected 2026-07-13 —
  `court` is pure SWI-Prolog, ASP severable: only `stable_interpreter` shells to smodels
  (stable-mode-only, unused) → `dependencies/` excluded; engine `ape_location` rewired in
  ape-build): `generate_rules/3` (`engine/acerules_processor.pl`) takes `InputCodes` = RAW TEXT
  it reparses via APE, dropping SID/TID provenance — NOT a DRS→rule seam → the clinical DRS→KB
  mapper is DIRECT; AceRules native rule = triple `(Label,Head,Body)`, no deontic force
  (direction/strength purpose-built). Defeat = `court` (`Label: <ACE>.` + `L1 overrides L2.` →
  `parser/priority_handler.pl`). `court` RESOLVES, never REPORTS conflicts → conflict detection
  is built fresh (the clinical KB never contains court priorities); platypus = 4 rules + 2
  priorities where naive-NAF genuinely DIVERGES from `court` → NOT a 1:1 oracle, use the
  purpose-built isomorphic pair + truth table (court-differential unit). §L·lp (Prolog KB
  shape, §6 LP profile): rules-as-data facts `rule/population/condition/action/direction/
  strength/certainty/exception/source` over a fixed kernel; exceptions = NAF-guarded labeled
  predicates (PROLEG); action key `<kind>:<target>`. §L·conflict (§6 verdict machinery):
  direction groups positive{for,require,permit} / against{against,avoid} /
  contraindicating{contraindicate,avoid}; eligible = same normalized action ∧ one direction
  positive while the other against/contraindicating; M1 kind `deontic_direction_conflict`;
  verdicts `semantic_contradiction` / `semantic_no_conflict` (+ `documented_no_conflict_result`);
  LP evidence labels = `participating_rules`, lane=lp, solver_status=not_run — SMT vocabulary
  (`unsat_core` etc.) stays out of the LP lane (§6 separation). §L·thread (§8.6 docA×docB — the
  standing conformance thread): docA `test_source.m1_guideline_a.rule.0` = {action
  `act.administer:drug.abx_a`, context cond.sepsis ∧ ¬cond.renal_severe ∧ age>=18, for/strong,
  exc.0}; docB `…guideline_b.rule.0` = {cond.sepsis ∧ age>=18 ∧ cond.pregnancy, contraindicate,
  same action-key → eligible}; control `…m1_control` = {cond.sepsis ∧ age<18, contraindicate →
  age disjoint → no-conflict}. Verdict: overlap sat, deontic unsat, core
  `[a.…guideline_a.rule.0, a.…guideline_b.rule.0]`, `deontic_direction_conflict`; control
  documents no-conflict. §8.2 groups + `corpus/reference/m1_expected.yaml`. Confirmed EN
  renderings = p4 (docA's ¬renal_severe enters via exc.0, not an in-guard negation — D5/D6).
  §L·pins (transplant sources): bridge oracle `6406066:.agent/roadmap.md` L56-59
  (+ `ecc19d3:.agent/roadmap.md` L850-861); interval 16-mask battery `6406066:.agent/roadmap.md`
  L54-55 (fixture `ecc19d3:.agent/roadmap.md` L652-656); harvested APE/AceRules upstream report
  `git show e8b5cf6:docs/cnl-attempto.md`.

- [x] ape-vendor: APE @5f4d535 → `clinicalcnl/` (132) + AceRules engine subset @5b7afb7 → `clinicalcnl/vendor/acerules/` (158) + full Clex @20960a5 → `clinicalcnl/vendor/clex/` (3) (`.git`-stripped, byte-identical to upstream; trees `ac239d2`/`1cebf98`(full-root)/`210d7ea`); grants verified first-hand (APE+AceRules LGPL-3.0-or-later, Clex GPL-3.0-or-later); §11.5 permissive regime + per-resource rows + `CKC_FORK.md`; swipl 9.2.9. 44% 436K/1M — `a400dd1` + codex-review remediation (permissive §11.5; corrected holders/claims per H1/H2/M1/M2; Clex pulled in)
- [x] ape-build: `make install` → full-vocab `ape.exe` (1.3M) under swipl 9.2.9, 0 err/warn; `get_ape_results` (module `ape`, `prolog/ape.pl`) loads + `ace_to_drs:acetext_to_drs/5` returns `drs(Refs,Conds)` should()-DRS on the clinical frame `It is recommended that a patient takes a drug.`. Full Clex wired DRY via `prolog/lexicon/clex.pl` `clex_file/1` → source-relative `../../vendor/clex/clex_lexicon.pl` (loader redirect — NO 3.2M blob copy, vendored blobs stay pristine); ape.exe rebuilt full-vocab. AceRules `ape_location` rewired to the nested layout (`% CKC (2026-07-14):` in `vendor/acerules/engine/parameters.pl` `../../../prolog/` + `acerules_processor.pl` source-relative resolve), engine loads clean + court nixon courteous-override smoke (guess=on — vendored Attempto Clex lacks `quaker`, republican/pacifist present, so guess=off cannot byte-match the older-Clex `output/nixon`; assert the `It is false that Nixon is a` override). Reproducible fail-closed gate `clinicalcnl/clinical/ape_build_smoke.sh` (5 checks). ACCEPTANCE (a)(b)(d) met; (c) upstream drace regression DEFERRED (optional, finder-confirmed in ape-vendor, not the real gate) — its remaining wiring (upstream-suite `consult(clex:clex_lexicon)` loads `tests/clex_lexicon.pl`, the ABSENT downloaded full Clex, bypasses `clex_file/1`) + `testruns/` baseline reproduction belong to the upstream-suite unit. 76% 757K/1M — real total (context.sh sums real API input, NOT inflated), over the 200K soft aim (permitted): ~270K stored conversation + redacted extended-thinking (Opus max-effort — 64 blocks, 0 chars persisted in the `.jsonl`) + ~50K fixed overhead = genuine 1M-wall occupancy absent from the transcript. The earlier session-prompt-CLAUDE.md-re-injection guess was falsified 2026-07-14 (`.jsonl` forensics — see memory Lessons)
- [x] surface-goldens: `clinical/SURFACE.md` (AUTHORITY: product seam + framing grammar +
  D1/D3/D5/D6/D8/D9 frame/keyword/interval/guard/exception tables + §Anaphora) + SPEC §10.6
  pointer + `clinical/goldens/` OBSERVED goldens (`surface_cases.pl` seeds + frozen
  `surface_ulex/1`; `surface_expected.pl` generated {CT, DRS, messages}) + replayer
  `clinical/surface_goldens.pl` (`run_seam/4` canonical seam `noclex=on,guess=off,solo=drs` +
  `capture/0`). Gate `run_tests(surface_goldens)` GREEN 24/24 (fresh-process + idempotent
  capture), 3 kinds: 12 v1 (text/plain ∧ zero msgs — 4 frames, 4 v1 markers, 3 §8.6 threads,
  exception body), 5 nonv1 (text/plain, msgs pinned — in-guard neg, eq/exactly, anaphor surface,
  named hole), 7 reject (text/xml — 2 frames, 3 intervals, 2 OOV). Codex-review (a633313)
  remediation: reverted the false "guess=off closes the named hole / SURFACE.md supersedes
  §L·probe" regression — §L·probe p6 stands (cap OOV → text/plain named()+warning; seam drops
  warnings; fail-closure = raw-gate + registry + profile zero-message law, now a tested v1
  invariant); frames CONSEQUENT-MODAL `=>(guard,op(action))` (D1/p3/p4); interval surface `has an
  age of <marker> <INT> years` (D9; the-age-of-is = non-v1 anaphor); exception body = bare
  condition (D6); threads corrected (docA renal via exc.0, docB/control `-can`). 0 err/warn loads.
  37% 372K/1M
- [x] kb-contract: `clinical/KB.md` (AUTHORITY: KB term family + grammar + invariants + execution
  semantics — read it, skip re-deriving) + `clinical/kb_kernel.pl` validators/OWNED-vocabulary/
  `derivable/3` + `clinical/goldens/kb_examples.pl` normative examples + `clinical/kb_kernel_tests.pl`
  plunit gate. Rules-as-data 9-fact family over doc-qualified `<doc>.<kind>.<k>` ids; the mechanical
  detail (fact signatures, atom grammar, CountOp→(open,dir) map, closed vocabulary, direction groups,
  safety invariants) lives in KB.md. Design calls downstream honors: population = the subject
  (`pop.patient` v1; adult/child = an age `interval`, NOT a `pop.adult` concept); exceptions = LP-lane
  NAF guards NOT the SMT lane's in-context negated conjuncts (docA renal via exc.0 — §6 separation);
  ids doc-qualify stmt/bind/exc too (not just §8.6's rule_id) so a flat multi-doc conflict KB never
  collides; `source` completeness = a map-emit obligation (kernel validates shape+ref when present);
  `derivable/3` = PROLEG NAF fixture reference, §15 G-RULE-EVAL-gated (conflict-core builds symbolic
  overlap on the same atom/exception structure, never patient-evaluates). Gate `run_tests(kb_kernel)`
  GREEN, 0 warn/err — pure Prolog, no APE dep: accept the valid set (doc_a/doc_b/control §8.6 thread +
  `multi` synthetic) + one isolated-defect reject per validator rule (each pins its SOLE violation
  functor) + direct id-grammar / action-key / context-atom / direction-group / derivability-boundary
  tests. `kb-writer` byte-pins the `valid` set. 40% 402K/1M
- [x] kb-writer: canonical KB emitter `kb_bytes/2`+`write_kb/2` in `kb_kernel.pl` (AUTHORITY:
  `clinical/KB.md` §Canonical emission) — one QUOTED re-readable term per line + `.`, lines BYTE-sorted
  over the emitted line strings (functor bytes then args — DISTINCT from `sort/2`-over-terms' arity-first
  order, which the hand-pins encode to catch), single trailing newline, exact bounds `NrD` not float;
  side-effect-free (caller validates). Gate `clinical/kb_writer_tests.pl` (`run_tests(kb_writer)`) GREEN
  10 tests: hand-authored byte-sorted normative bytes over all 4 valid examples (docA/docB/control/multi)
  + byte-sorted / round-trip / order-invariant / single-newline properties + rational/empty/singleton/
  non-list. 0 warn/err, no kb_kernel regression. 17% 166K/1M
- [x] ulex: `clinical/registry.pl` = id↔surface authority (raw-gate + map-core consume — read it,
  skip re-deriving surfaces): bidirectional reg_concept/reg_drug/reg_action/reg_quantity/
  reg_population/reg_guard_verb facts + pn_allow/1 (drug proper-name allowlist = the named()
  discriminator) + D1 reg_keyword/4 (keyword→op/direction/strength) + reg_frame/2 (op→ACE frame
  phrase) + reg_op/1 + reg_v1_direction/1 vocab; integrity checker registry_errors/2 +
  valid_registry/0 (pure over a fact list; 4 classes uncovered/unknown_id/duplicate/malformed
  cross-checked vs kb_kernel's closed vocab, data-driven lexeme_family/4). `clinical/clinical_ulex.pl`
  = 12 explicit APE ulex entries (frozen role order) + ulex_text/1 (ulextext text-identical, ASCII→byte,
  to the frozen surface_cases:surface_ulex/1 oracle). pop.adult/pop.child carry NO lexical entry (age
  interval). Gate `clinical/ulex_tests.pl` run_tests([registry,ulex]) GREEN 27/27 (accept + per-rule
  sole-violation rejects + D1 keyword/frame content pins + cardinality tripwire + ulextext text-identity
  + registry↔ulex set cross-check + one APE smoke: frame_recommend parses clean under the file's own
  bytes), 0 warn/err loads. codex-review hardened registry_errors/2 sound over nonground/improper
  input. 23% 227K/1M
- [x] raw-gate: `clinical/raw_gate.pl` = the pre-APE WHITELIST (raw-gate-battery + profile-drs +
  map-core consume): a two-stage DCG over raw v1 document bytes — (1) framing (blank-separated
  paragraphs → `document <id>` + rule/exception blocks; a header DCG parses id + modality keyword +
  certainty/basis fields under strict LF/single-space discipline), (2) per-sentence registry-driven
  templates (the registered-pattern grammar admits ONLY registry surfaces + closed frame/function/
  marker words, so every unregistered/`n:`-prefix/capital-OOV/`or`/`every`/decimal/malformed token
  fails) + the D1 frame-op↔keyword cross-check. Accept output ok(doc(DocId, [sentence(Idx, Ace, Ctx)
  …])) — Idx = 0-based raw sentence ordinal (provenance sentence_index), Ace = the verbatim ACE
  (byte-identical to the surface_cases oracle), Ctx = rule(K,Keyword,DisjIdx,Cert,Basis) |
  exception(K,RuleK,Cert,Basis); reject output reject([reject(Idx,Token,Construct)…]); fail-closed +
  total, NEVER runs APE (pure Prolog, fast). Gate `clinical/raw_gate_tests.pl` run_tests(raw_gate)
  GREEN 37/37 — 8 accept ALL cross-checked byte-identical vs the surface_cases oracle, 27 core rejects each
  the sole diagnosis (raw-gate-battery review added tab/whitespace + exact empty_document), 2 totality; 0 warn/err loads, sibling gates unregressed. Codex-review hardening: number
  agreement (APE `1 year`/`18 years`; gate rejects `1 years`/`2 year`; year(s)/age/patient/has/takes registry-
  DERIVED), total over any term (reject(bad_input), no throw/wildcard), Basis = string|none (KB.md), doc-id
  validated to kb_kernel `<doc>` prefix, document-level id integrity (duplicate_rule_id/duplicate_exception_id/
  dangling_exception), Idx advances per content line on every block path. 17% 166K/1M
- [x] raw-gate-battery: `clinical/raw_gate_battery_tests.pl` = the systematic per-hazard mutation
  matrix over raw_gate:gate_document/2 (mutation DEPTH; raw_gate_tests.pl carries the core suite = accept
  cross-checks + one sole-diagnosis reject per reachable Construct). `banked_hazard/1` closes the hazard set
  = the roadmap's 23 + op_mismatch (SURFACE.md §Modality assigns the op-mismatch reject battery HERE; the
  unit line omitted it) = 24 classes over the whitelist/sentence/id surface; each mutant is a single-locus
  edit of a PROVEN-VALID base asserting the exact
  reject(Idx, Token, Construct). Constructs OBSERVED by running each (never assumed): capital-OOV →
  unregistered_capital; prefix/pronoun/or/every/does-not/spaced-multiword/exactly/`#`-comment/quotation →
  unregistered_token; bare-then/no-antecedent-definite/cross-sentence-definite/number-agreement/bare-eq →
  malformed_sentence; decimal/leading-zero-interval → bad_number; leading-zero-block-id/missing-header →
  bad_header; unregistered-keyword → bad_keyword; op-mismatch → op_mismatch; bad-doc-id → bad_document_header;
  bad-input → bad_input; dup/dangling → duplicate_rule_id/duplicate_exception_id/dangling_exception. Gate
  run_tests(raw_gate_battery) GREEN 117/117 = 5 accept controls (canonical shapes + atom/string/codes
  encodings) + 55 bases_accept (per-mutant anti-vacuity) + 55 mutants_reject (55 mutants: mono 21 + raw 16 +
  op_mismatch Cartesian 18) + covers_every_banked_hazard / no_unbanked_class; 0 warn/err loads, raw_gate suite
  unregressed (154 combined). Pure test-authoring — no raw_gate.pl change. Codex-review: anti-vacuity made
  PER-MUTANT (each mutant carries its exact accepted base ⇒ bases_accept catches a base that silently stops
  accepting — a year_noun(1)/code_list regression); op-mismatch = the full registry keyword×frame-op Cartesian
  (18 mismatch + 6 match bases, was 6 samples); mutants_reject wraps the stored reject/3 tuple; "exhaustive
  over gate_document/2" → "systematic over the banked hazards" (the framing/field Constructs are the core
  suite's); raw_gate_tests.pl gained tab/whitespace + exact empty_document (the real untested-anywhere gaps).
  13% 133K/1M
- [x] profile-drs: `clinical/profile_check.pl` post-APE DRS whitelist (2nd fail-closed layer, pure
  Prolog no APE dep) + `clinical/profile_check_tests.pl` gate. profile_check(+Ctx,+Drs,+Msgs,-Result)
  → ok | reject(Reason) single-cause; Ctx = raw-gate rule(_,Keyword,_,_,_) | exception(_,_,_,_) (kind
  + Keyword read). Ordered checks: zero-message law → is_wellformed/1 canonical hygiene (APE util;
  backstop, passes on all text/plain incl. nonv1, NOT the discriminator) → recursive named scan vs pn
  registry (THE drug-registration authority) → rule shape drs([],[=>(guard,drs([],[op]))]) / exception
  bare concept-have → guard whitelist (eq-1 pop/concept/age objects + year unit object v1_countop ∈
  {geq,greater,leq,less} + relation(of) + predicate(have), one-level interval sublist flatten; reject
  -drs/v/alien=guard_shape, exactly/bare-eq=interval_countop, deeper=nested_sublist) → consequent
  op↔keyword D1 (op_mismatch) → action predicate(_,take,<subject==population>,named(_)). Gate GREEN
  36/36 via the read-back-golden pattern: DRS TERMS reconstructed from the byte-pinned surface goldens
  via read_term_from_atom (serialized DRS reads back to the SAME term; golden_roundtrip proves fidelity
  byte-exact ×17) → pure/fast, no live APE — accept 12 v1 + covers-all-v1 + 5 nonv1 non-vacuity
  (guard_shape/interval_countop×2/nonempty_messages×2) + covers-all-nonv1 + roundtrip ×17. Dedup: drug
  registration = the ONE top named scan (action checks named(_) shape only). Left to profile-battery /
  map-exc: exhaustive reject coverage, exception single-concept cardinality, the op_mismatch /
  bad_action / bad_top_shape reject paths. 0 warn/err loads, sibling gates unregressed. 24% 241K/1M
- [x] profile-battery: `clinical/profile_check_battery_tests.pl` = systematic hand-mutant DRS reject matrix
  over `profile_check/4` (pure Prolog, hand-built DRS terms with real referent vars — no live APE). One
  mutant CLASS per reject Construct the checker emits (17), each a single-locus edit of a proven-accepted
  base; the p7 hazards become concrete terms (v()/-drs/alien guard = guard_shape, fresh-referent subject =
  action_subject_mismatch, bare-then top = bad_top_shape, off-allowlist named, warning parse =
  nonempty_messages, FULL registry keyword×op Cartesian = 18 op_mismatch + 6 matched bases, nested interval
  sublist = nested_sublist, int(5) target = bad_action_target). Per-mutant anti-vacuity (bases_accept over
  every base). HONEST COVERAGE: profile_construct/1 closed set cross-checked against profile_check.pl's own
  reject(...) sites (source PARSED AS TERMS = independent authority, not the self-referential banked list) +
  every_construct_has_mutant → exhaustive over the gate's rejects; NO dead branch (bad_action_referent
  reachable when the used event arg is a guard referent). Gate run_tests(profile_battery) GREEN 79 = 3 meta
  + 38 bases_accept + 38 mutants_reject, each reject OBSERVED (dumped + eyeballed, never assumed); 0 warn/err,
  8 clinical gates unregressed. Pure test-authoring, no profile_check.pl change. CODEX-REVIEW OVERTURNED the
  map-core deferral (below): per SPEC §10.6 profile_check must reject anything outside the registered patterns,
  so the accept-gaps (top-level leq/less placement, >1 interval, non-interval-in-sublist, unconstrained
  of/have + patient-only guard + patient-only/multi-concept exception) are profile_check's to reject → new
  profile-structure unit; F3/F4 battery fixes fold there. 24% 245K/1M
- [x] profile-structure: `profile_check/4` expanded SHAPE-per-atom → full STRUCTURAL whitelist (SPEC §10.6;
  codex-overturned the map-core deferral — the profile rejects anything outside the registered patterns). Guard =
  two passes: normalize/3 (interval marker + D9 placement, flatten the one-level interval sublist) + wire_guard/3
  (sole population ref, then component-by-component consume anchored on concept/age OBJECTs + have/of wiring,
  ==-matched post-is_wellformed; ≥1 component, no leftover). New rejects: no_guard_component (patient-only),
  guard_wiring(Obj) (missing/mis-wired have/of, orphan), interval_placement(CO) (leq/less top | geq/greater
  nested), interval_sublist(E) (non-{relation,year} sublist element incl. a deeper list),
  bad_exception(population|concept_count|wiring); guard_shape / interval_countop / bad_population kept. Ordering
  keeps goldens green (profile_check_tests.pl UNCHANGED): leftover-reject BEFORE no_guard_component (guard_neg =
  patient+negation stays guard_shape), interval_countop (exactly/eq) BEFORE placement (iv_exactly/iv_bare).
  Exception (D6) = exactly 1 pop + 1 concept + have. USER DECISION (2026-07-15, asked at unit start): a bounded
  age range (≥1 well-wired interval component, ANY count) is VALID v1 → ACCEPTED, verified against the real
  warning-free seam parse (geq top + less nested, 0 msgs); NO >1-interval reject → conflict-core / interval-algebra
  must intersect same-quantity (age) interval atoms WITHIN a guard before cross-guard overlap. Battery reworked
  (profile_check_battery_tests.pl): patient_rule base retired → patient_only mutant; disjunction/negation re-based
  to recommend_rule; +mutants per new reject; F3 (mutants_reject `=@=` vs fully-spelled reject terms, echo
  subterms pinned); F4 wording; honest coverage now 21 constructs == source scan; accept_bounded_range test on the
  real probed bytes. Gate run_tests(profile_check) 40 + run_tests(profile_battery) 102 GREEN; all 8 clinical gates
  unregressed, 0 warn/err. Found+fixed an SWI clause-compile quirk (memory Lessons: `Sub=Term, Mut=drs([…,Sub])`
  last-goal head-arg fold DROPS the Sub binding → route via a predicate call mk_exc). 33% 333K/1M — +
  codex-review remediation: is_wellformed proves DECLARATION not object-ROLE uniqueness → 5 verified over-accepts
  of role-aliased / non-canonical DRS closed — object-referent-uniqueness (shared_object_referent /
  bad_exception(aliased)), EXACT [of-relation, countable/na year] sublist, EXACT exception body (exc_wire =
  sole-leftover have vs extra/duplicate), year countable/na pinned everywhere, provenance SID=1 (bad_provenance);
  +ground_rejects_exercised coverage past functor granularity + honest doc; battery → 23 constructs / 115 GREEN,
  all 8 gates unregressed; +Prolog term-walk-var false-positive lesson (nonvar+integer guard).
- [x] map-core: `clinical/drs_map.pl` exception-free DRS→KB terms (AUTHORITY: interface +
  extraction rules + provenance policy — read it, skip re-deriving). `map_rule(DocId,
  rule_header(RuleOrd,Keyword,Cert,Basis), Disjuncts=[disj(SentIdx,Drs)], base(StmtIdx,BindIdx)0,
  Facts, Base1)` maps ONE rule block; ASSUMES profile-validity + EXTRACTS (no reject path). Guard
  walk: flatten_intervals lifts the leq/less one-level sublist (D9) → anchor-object surface order →
  concept/age objects emit `condition(bind.k,…)`; interval companions matched by referent IDENTITY
  (`==`, not a unify filter → bounded-range safe). CountOp→(openness,dir) `countop_bound`; D1
  direction/strength off the keyword (`reg_keyword`; DRS op IGNORED); action key via
  reg_action/reg_drug+`action_key/3`. Disjuncts stmt-major `stmt.0..n` under `rule.RuleOrd` (D4);
  stmt/bind counters DOCUMENT-CONTINUOUS threaded via `base/2` (map-emit advances across blocks);
  rule counter = raw block ordinal. Provenance = ONE source per rule (clause group):
  `source(rule.k, DocId, sorted-sentence-Regions, Basis string|none)` — NO stmt-level source
  (kb_examples's varying density = kernel-test fixture, not the map oracle); certainty optional
  (`none`=absent). Output kb_kernel-valid by construction; emit order free (kb_bytes sorts). Gate
  run_tests(drs_map) 8 GREEN via read-back goldens (read_term_from_atom, profile-drs pattern, no
  live APE): 4 markers (CountOp→bound) + 3 threads (docA exc-free/docB/control full sets, valid_kb)
  + 2-disjunct rule (grouping + Base==base(2,3) + certainty). All pure gates unregressed, 0 warn/err
  loads; additive (2 new files, no sibling edits). 20% 199K/1M
- [ ] map-emit: whole-document driver + canonical emission — group raw-gate sentences by rule
  ordinal, thread `base(StmtIdx,BindIdx)` across rule-blocks (`drs_map:map_rule/6`, base-threaded —
  it already assigns `stmt.k`/`bind.k`) + exception-blocks (map-exc), collect facts → kb-writer
  (`kb_bytes/2`/`write_kb/2`); byte-pins from OBSERVED emitter output over the thread docs;
  determinism gates (re-run identical; guard-conjunct/DRS-input reorder identical; goldens re-emit
  == pinned). Gate: byte plunit. Reads: drs_map.pl (map_rule/6) + kb_kernel (kb_bytes) + goldens.
- [ ] map-exc: labeled exception compilation — exception blocks → NAF-guarded PROLEG overrides
  on their rule's statements (exc.k stmt-major, document-continuous counters; D6
  self-contained bodies; clause region_ids = own block's basis verbatim) + bridge-oracle
  transplant (§L·pins: 2-disjunct rule × 2 exceptions → stmt.0{exc.0,exc.1}/stmt.1{exc.2,
  exc.3} + trailing 1-disjunct rule → rule.2/stmt.2/exc.4 catching per-rule counter resets;
  enumerated bind.k→concept map). Gate: oracle plunit + emission re-pin. Reads: map-emit
  modules + §L·pins blobs.
- [ ] court-differential: purpose-built ISOMORPHIC differential vs AceRules `court` — clinical
  pair (recommend rule + contraindicate exception) ↔ republican/pacifist structure; exhaustive
  fact-presence truth table (all subsets of participating facts); per row assert clinical-KB
  derivability == court verdict where semantics coincide + DOCUMENT divergences (naive-NAF vs
  courteous — §L·acerules; platypus divergence = known). Gate: differential plunit. Reads:
  `vendor/acerules/engine/testcases/court/*` + `parser/priority_handler.pl` (bounded; court
  smoke recipe in memory Runtime).
- [ ] interval-algebra: `clinical/intervals.pl` exact-rational bound algebra (D10): bound =
  (value, open|closed, dir); intersection/emptiness over SWI native rationals; 16-mask
  validity battery transplant (§L·pins: 16 bound-presence masks × per-bound values {-1,0,1},
  valid iff exactly one bound present ∧ value ≥ 0 — expectations hand-written) + open/closed
  boundary properties (geq 18 vs greater 18 vs less 19 adjacency). Gate: plunit hand-oracled.
  Reads: kb-contract interval shape only.
- [ ] conflict-core: `clinical/conflict.pl` — eligibility (same action key ∧ §L·conflict
  direction groups) + context overlap: DNF disjunct-pair enumeration × concept polarity ×
  interval intersection (intervals.pl) × exception expansion (exceptions join their statement
  as negated concepts). Bounded-range guards are v1 (user 2026-07-15, profile-structure) → a
  guard may carry >1 same-quantity (age) interval atom; intersect them WITHIN the guard to its
  effective bound before cross-guard overlap. Gate: plunit hand-oracled pair battery
  (overlap/disjoint/polarity/exception + a bounded-range pair). Reads: intervals.pl +
  kb_kernel.pl + map outputs.
- [ ] conflict-verdict: verdict layer — records {category semantic_contradiction |
  semantic_no_conflict (+ documented_no_conflict_result), kind deontic_direction_conflict,
  participating_rules set `a.<source>.rule.k`, evidence {document_id, sentence_index},
  lane=lp, solver_status=not_run}; thread battery: docA×docB → contradiction w/ core set;
  docA×control → no-conflict (age-disjoint); field shapes mirror
  `corpus/reference/m1_expected.yaml`. Gate: plunit. Reads: conflict.pl + m1_expected.yaml.
- [ ] attack-surface: vendored entry-point inventory (APE webservice/server modules, AceRules
  stable-mode smodels shell, any HTTP surface) → prove non-reachability from the clinical
  pipeline (negative plunit: those modules absent from the gate/profile/map/conflict consult
  closure) + `CKC_FORK.md` security section (AGENTS.md bullet: vendored network surfaces stay
  dark, never invoked). Gate: negative plunit. Small.
- [ ] upstream-suite: wire the fork's OWN suite to in-tree Clex — point the `tests/`
  `consult(clex:clex_lexicon)` resolution at `vendor/clex/` (source-relative `% CKC:` shim;
  banked GAP: the suite expects a downloaded `tests/clex_lexicon.pl`, bypassing `clex_file/1`)
  + reproduce the `testruns/` baseline (3733 cases, 0 NEW mismatches — finder-confirmed in
  ape-vendor). Gate: suite runs + baseline comparison clean (git-diff-gated, not exit code).
  Seam: [consult shim] | [baseline run+compare]. Reads: `tests/` harness entry + `testruns/`
  layout (bounded).
- [ ] corpus-lock: `clinical/corpus/` v1 — docA/docB/control as raw ClinicalCNL documents (ids
  `test_source.m1_guideline_a`/`.b`/`m1_control`, groups `group.m1_conflict`/
  `group.m1_no_conflict`) + ≥2 fresh docs (an interval-boundary adjacency pair; the
  2×2+trailing multi-exception doc hosting the bridge oracle) + case MANIFEST (ordered
  document pairs; expected category/kind/participating_rules/evidence HAND-AUTHORED at
  authoring time, never route-derived — §0 honesty; oracle scope = KB-level ONLY: facts,
  exception ids, sentence indices, bytes — ClinicalIR ids stay behind the backlog IR bridge)
  + committed expected KB bytes + drift guard (gate→APE→profile→map→emit == committed bytes
  per doc). Gate: round-trip + manifest plunit. Seam: [thread docs + manifest] | [fresh docs].
- [ ] runner: ONE command `clinical/run_conformance.sh` → plunit driver aggregating surface
  goldens + raw-gate(+battery) + profile(+battery) + map batteries/oracles +
  court-differential + intervals + conflict + verdicts + corpus round-trip&manifest +
  attack-surface negative + upstream-suite leg. Runner = milestone acceptance core + the
  /loop round gate (cnl-optimize.md points at it). Gate: runner green over the locked corpus.
  Reads: prior units' plunit entry points only.
- [ ] loop-framework: bootstrap fix + protocol validation — edit
  `.claude/commands/cnl-optimize.md` (explicit one-time R1 bootstrap state; ledger line staged
  INSIDE the round commit; queue-bank rules re-checked; round gate = run_conformance.sh), then
  execute ONE manual dry-run round R1 (pick a small increment, land `cnl-opt (R1)` green,
  ledger line) → milestone IMPLEMENTED. Gate: R1 commit green + ledgered; user then enables
  skillOverrides.loop.

## Backlog — NOT a milestone (unscheduled; schedule by trigger; full pre-reset unit specs at `git show ecc19d3:.agent/roadmap.md`)

- Hardening — trigger: before any run whose evidence leaves the operator-controlled tree,
  before M4-scale locked measurements, or the first milestone reworking model.rs/verify.rs
  (M3 mitigations: clean-committed-tree + post-run attestation verify in record-cnl/
  acceptance-m3; verify-eof landed the solver-capture slice): spawn-retry (fs-portable
  ETXTBSY retry tests via injectable spawn op + clock; suite GREEN on the dev fs
  2026-07-12, 4/4 pass — portability only); path-confine (registry-path containment
  resolver; review-reproduced absolute-path escape — operator-owned working copy, so
  evidence-quality impact local; FIRST SLICE when scheduled: lexical `is_safe_relative_path`
  check on corpus.path + expected_outcomes in validate_registries); input-snapshot.1–.3 +
  constraint-snapshot (read-once input layer; manifest attests resolution-time bytes;
  staged frozen constraints); subproc-runner.1/.2 remainder (shared cross-crate subprocess
  runner extraction + model.rs-side drain-cap parity — the model-runtime rulings live in
  the archived `## Runtime` (`git show e388ee4:.agent/memory.md`)).
- Replay-attestation deepening (codex 2026-07-12): replay compares output payload hashes
  only — extend to the manifest's deterministic identity/input projection + cassette
  prompt/constraint-hash cross-check against the current route; wrapper provenance bytes
  attested by own hashes. Trigger: with input-snapshot, or before external evidence.
- Model/runtime byte-fingerprinting (codex 2026-07-12): §9 identity = self-reported probe
  strings; add executable/model-file/config fingerprints to the identity record + enforce
  one identity across every attempt (not last-attempt-only). Trigger: first multi-machine
  or re-recorded comparison; M3 mitigates via record-cnl's byte-reproduction spot-check.
- k>1 sampling support (SPEC §11.4): runner multi-draw + pairwise convergence metric — the
  landed runner enforces model_sample_count=1; reliability evidence beyond
  acceptance/repair/taxonomy waits on it.
- canon-props: canon-layer generated-case harness (AGENTS.md-preferred property hardening) —
  schedule after M3's last canonical-shape change.
- Deferred CNL capabilities: SPEC §11.3 (EN mirror, escape, from-IR rendering, findings CNL
  quoting, lexicon accretion) — promotion-gated, never scheduled without §11 evidence.
- Rust CNL lane + route comparison (deferred 2026-07-13 product push, SPEC §0/§10.6): the
  ENTIRE pre-push M3 unit set (lexicon-cnl-fields … acceptance-m3, incl. route-stage-handles,
  verify-eof, metrics-faithful.1/.2, explorer, record-cnl, recorded-cnl-battery) — full specs
  + cross-unit pins at `git show 9b23c93:.agent/roadmap.md`; trigger: JA surface scheduled,
  SMT cross-check / IR bridge demanded, or comparison evidence wanted before any
  promotion/selection claim.
- IR bridge (APE line → ClinicalIR): deterministic mapping from the §10.6 clinical KB (or its
  DRS) into ClinicalIR so APE-line documents verify under Z3 + land harness
  metrics/provenance; trigger: first report citing APE-line results beyond §10.6 conformance,
  or M4 opening.
- ClinicalCNL JA surface: mission-primary surface on the product line; mine
  `git show ecc19d3:SPEC.md` §10 + the banked JA lexicon/prefix-audit pins (9b23c93 roadmap
  header); trigger: user schedules it, or first real-JA-corpus work.
