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
  D1/D3/D5/D8/D9 frame/keyword/interval/guard/exception tables) + SPEC §10.6 pointer +
  `clinical/goldens/` OBSERVED per-construct goldens (`surface_cases.pl` seeds + `surface_ulex/1`;
  `surface_expected.pl` generated bytes) + replayer `clinical/surface_goldens.pl` (`run_seam/3`
  canonical seam `noclex=on,guess=off,solo=drs` + `capture/0`). Gate `run_tests(surface_goldens)`
  GREEN 25/25 (fresh-process + idempotent capture): 16 parses (4 frames, 2 guard atoms, 4 v1 markers
  + eq/exactly non-v1, 3 §8.6 thread composites, exception body) + 9 rejected (3 frames, 3 intervals,
  3 OOV). Corrected banked probe claims (memory Runtime): full-ACE frames not telegraphic; conditions
  need determiner `has a <cond>`; capitalized OOV → text/xml reject at guess=off (named hole =
  guess=on-only). SURFACE.md supersedes §L·probe for surface/seam facts. 0 err/warn loads. 37% 372K/1M
- [ ] kb-contract: `clinical/KB.md` kernel contract (rules-as-data fact family per §L·lp —
  exact signatures/arities = this unit's call; ground-term grammar: ids `stmt.k`/`bind.k`/
  `exc.k` document-continuous, action key `<kind>:<target>`; interval atom shape (var, bound
  value, open|closed, dir); PROLEG NAF exception-guard shape; safety invariants (every stmt
  var bound by population/condition atoms); execution semantics = SLD+NAF derivability queries
  the conflict layer consumes) + `clinical/kb_kernel.pl` term validators + plunit validator
  accept/reject over hand-written NORMATIVE examples. Gate: plunit. Seam: [KB.md] |
  [validators]. Reads: §L·lp/§L·conflict + §L·spec §5-§6 anchors.
- [ ] kb-writer: canonical KB writer in `kb_kernel.pl` — define+commit the canonical clause
  order (byte-sorted, §6 "Emission is deterministic"-consistent) + a dedicated deterministic
  emitter (never bare write_term defaults) + byte-pinned writer tests (hand-written normative
  bytes over kb-contract's examples; input-order shuffle → identical bytes). Gate: byte
  plunit. Reads: kb-contract outputs only.
- [ ] ulex: `clinical/clinical_ulex.pl` mirroring §L·ids EXACTLY (cond.sepsis→noun_sg(sepsis,
  sepsis,neutr); cond.renal_severe→noun_sg('severe-renal-impairment',…); cond.pregnancy→
  noun_sg; drug.abx_a→pn_sg('Abx-A','Abx-A',neutr); act.administer→tv_finsg(takes,take)+
  tv_infpl(take,take); q.age_years→noun_sg(age)+noun_sg(year)+noun_pl(years,year); patient→
  noun_sg(patient,patient,human); have→tv_finsg(has,have)+tv_infpl(have,have); pop.adult/
  pop.child = interval atoms, NO lexical entry) + bidirectional id↔surface registry
  `clinical/registry.pl` (raw gate + mapper consume; holds the pn allowlist + D1 keyword
  table) + integrity checker (dup/malformed/unknown-id/uncovered-concept rejects) + one-golden
  APE parse smoke under ulextext = this file's bytes. Gate: plunit. Reads: SURFACE.md; ulex
  entry templates banked (ulex.pl `lexicon_template/1`).
- [ ] raw-gate: `clinical/raw_gate.pl` — DCG over raw document bytes: framing parse (blocks,
  ids, keywords, certainty, basis) + per-sentence registered-pattern token templates
  (registry-driven WHITELIST per the Fail-closed decision; capitalized token legal iff
  pattern-anchored or ∈ pn registry) + output = ordered {sentence idx, ACE sentence, block
  context} for per-sentence APE dispatch (D2); rejects name {sentence idx, token, construct}.
  + accept battery (thread docs + interval/exception/certainty variants). Gate: plunit accept
  + core rejects. Reads: SURFACE.md + registry.pl.
- [ ] raw-gate-battery: full mutation reject battery over raw_gate — one mutant class per
  banked hazard: capitalized-OOV, `n:`-prefix, pronoun, or-guard, every-surface, bare-then,
  does-not (D5), cross-sentence definite, no-antecedent definite, decimal, leading zero,
  spaced multiword, unregistered modality keyword, dup rule id, dangling exception ref,
  exactly/eq marker, missing header, ACE comment, quotation. Gate: plunit all-reject naming
  sentence+construct. Reads: raw_gate.pl + SURFACE.md.
- [ ] profile-drs: `clinical/profile_check.pl` post-APE DRS checker: zero-message law;
  frame-op↔keyword map (D1, both directions); recursive `named(_)` scan vs pn registry;
  guard-shape whitelist (conjuncts {concept-have, interval-of} + one-level sublist flatten;
  reject `-drs`/`v()`/unknown functors/extra ops); action shape
  predicate(take,GuardRef,named(RegisteredDrug)); canonical-DRS equality vs the registered
  pattern's golden (first-parse-wins ambiguity kill). + accept battery over the goldens. Gate:
  plunit. Reads: `prolog/utils/is_wellformed.pl` + `vendor/acerules/engine/drs_checker.pl`
  (bounded 2-file template read) + goldens.
- [ ] profile-battery: full DRS-side reject coverage for profile_check — p7 DRS shapes (`v()`,
  fresh-referent then-part, bare-then, in-guard `-drs`, unregistered named, warning-bearing
  parses, op/keyword mismatch per modality, malformed interval sublists, non-golden DRS
  variants). Gate: plunit all-reject. Reads: profile_check.pl + goldens.
- [ ] map-core: `clinical/drs_map.pl` exception-free DRS→KB terms: guard walker (concept atoms
  via registry; interval atoms from object CountOp + D9 sublist flatten; disjunct grouping =
  one sentence per disjunct under one raw rule id → stmt.k stmt-major, D4); action key via
  registry; direction/strength via D1 (keyword+op); certainty; provenance {doc id, raw
  sentence idx} per clause group. Output = kb_kernel-validated TERMS (bytes = map-emit). Gate:
  plunit hand-oracled terms over the thread rules + 4 interval markers + a 2-disjunct rule.
  Seam: [concepts/action/modality] | [intervals + disjunct grouping]. Reads: kb_kernel.pl +
  registry.pl + goldens.
- [ ] map-emit: whole-document canonical emission — map-core terms → kb-writer; referent
  canonicalization → `stmt.k`/`bind.k`; byte-pins from OBSERVED emitter output over the thread
  docs; determinism gates (re-run identical; guard-conjunct/DRS-input reorder identical;
  goldens re-emit == pinned). Gate: byte plunit.
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
  as negated concepts). Gate: plunit hand-oracled pair battery (overlap/disjoint/polarity/
  exception cases). Reads: intervals.pl + kb_kernel.pl + map outputs.
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
