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

## M3 ClinicalCNL product line (APE fork) + loop framework — scope f76e1fa — plan 3bb4a38 — replan PENDING

Scope = SPEC §10.6 (2026-07-13 product push = design authority; §0 honesty note). Deliverable:
vendored APE fork building green under SWI-Prolog (9.2.9 confirmed on the dev machine
2026-07-13; environment gate — re-confirm functionally in ape-build + later Prolog-running
units; ape-vendor = `swipl --version` only, no Prolog execution), ClinicalCNL v1 as a
fail-closed ACE profile (EN surface) over a demand-authored clinical ulex, a DIRECT
DRS → clinical-Prolog-KB mapping (AceRules = bounded differential only; labeled exception
overrides, per-sentence provenance, byte-identical re-emission), Prolog-side conflict/no-conflict queries re-deriving the M1
docA×docB thread in-lane, a locked synthetic conformance corpus behind ONE runner command, and
the /cnl-optimize + /loop round framework. Rust tree untouched this milestone (no re-bless
anywhere); engine-agnostic rule unchanged (it targets LLM inference engines — SWI-Prolog/APE
are nameable solver/host tooling like Z3). No unit needs the model runtime (no gated units).

2026-07-13 codex verdict on 3bb4a38 (accepted in the review session; recording = user-delegated
decision, this follow-up commit): the 13-unit downstream design is SUPERSEDED — replan required
(REPLAN unit below); ape-vendor + ape-build STAND (the review itself rebuilt APE @ pin under
9.2.9 + reproduced the regression) and execute FIRST — the replan then designs the
surface/framing sub-language empirically against the in-tree built `ape.exe`. APE bet HELD
(user-directed; review validated build/licensing/runtime — it falsified only banked
surface+mapping design facts, corrected in place below). Superseded unit text =
`git show 3bb4a38:.agent/roadmap.md`; raw codex findings = review-session transcript a5ea24af
(2026-07-13; its /tmp scratch clones+`ape.exe` are volatile — rebuild in-tree, never depend).

Cross-unit decisions:
- Layout: three zones keep fork-vs-vendored-vs-ours auditable — `clinicalcnl/` root = the APE
  fork (upstream layout preserved: diffability + GPL corresponding-source clarity);
  `clinicalcnl/vendor/acerules/` = the AceRules adaptation-source subset (see ape-vendor);
  `clinicalcnl/clinical/` = CKC clinical additions ONLY (profile checker, mapping, queries,
  ulex, corpus, runner). Upstream files edited only where wiring demands, each edit commented
  `% CKC:`. Fork provenance = `clinicalcnl/CKC_FORK.md` (distinct name — APE's own `README.md`
  sits at the root; the two upstream roots also collide on `.gitignore`/`LICENSE.txt`).
- Licensing (SPEC §11.5): APE + AceRules evidence rows land in the ape-vendor commit as §11.5
  PROSE (extend the "Standing verdicts" sentence — no registry file); ape-vendor owns the
  acquisition-record schema + first-hand header verification. Clex: VENDORED in-tree (full
  lexicon, GPL-3.0-or-later, `vendor/clex/`, §11.5 row) — ape-build wires it drop-in + the
  upstream suite uses the in-tree copy (no live download); also a clinical-term mining seed.
- Fail-closed profile (CORRECTED 2026-07-13 — DRS-only checking is INSUFFICIENT: APE irreversibly
  erases surface facts — comments vanish, `n:Flarble` → ordinary `object(…,'Flarble',…)` NOT
  `named(_)`, `005`/`5` normalize identically, pronouns/definites resolve by silent referent
  merging, `named/1` can nest inside conditions): fail-closed = a RAW-TEXT gate BEFORE APE
  (canonical token/framing grammar; reject every noncanonical token class incl. inline
  `n:`/`v:`/`a:`/`p:` prefixes + capitalized content tokens) + APE message/syntax-output inspection
  + a RECURSIVE full-DRS scan (unregistered `named(_)` at any depth) + a mutation property battery
  (comments, prefixes, casing, variables, numeral spellings, quotations, contractions, pronouns,
  definites, ellipsis). Precise capitalization fact: an UNPREFIXED capitalized OOV token in
  proper-name position → `named(_)` + warning; a prefixed one bypasses `named(_)` entirely → the
  prefix ban is enforceable ONLY at the raw gate. Guessing OFF (`-noclex -ulexfile`; `-guess`
  default off) still rejects lowercase OOV; a clean parse alone never proves lexical closure. The
  profile checker still validates the DRS against registered sentence patterns, rejects naming the
  sentence + construct. EN interval markers at least / at most / less than / more than ↔
  ge/le/lt/gt (→ APE `object` CountOp geq/leq/less/greater) — hand-oracled battery, expected values
  hand-written, never derived from the mapping under test.
- Determinism: same accepted document → byte-identical KB text (canonical clause order +
  emission); provenance = {document id, sentence index} carried on every emitted clause group.
- Conformance runner: ONE command (plunit driver script under `clinicalcnl/clinical/`) =
  upstream fork suite + profile battery + mapping battery + conflict queries + corpus
  round-trip; THE loop round gate. Milestone acceptance = runner green over the locked corpus
  + a ledgered manual dry-run round (loop-framework). The upstream-suite leg's Clex gate is
  RESOLVED — full Clex vendored in-tree (`vendor/clex/`, §11.5 row); the runner uses the in-tree
  copy (no live download, no Clex-free respec). The corpus gains a
  case MANIFEST (ordered document pairs + expected category/kind/participating_rules/evidence)
  beside per-document round-trip.
- Reading legend (banked-once shared pins; the units below cite by tag — read these, skip
  rediscovery). §L·spec (design authority; section-anchor + distinctive-string, ≈line drifts): §5
  domain IR (`NormativeRule`/`Action`; `ContextExpr` = DNF over concept | negated-concept |
  "quantity interval" atoms, ≈L456); §6 conflict + LP profile ("rules-as-data" ≈L597; byte-emission
  "Emission is deterministic" ≈L578; lane-separation "LP verdicts never replace" ≈L609); §8.6 worked
  thread (≈L826); §10.4 sentence model / DNF / markers = shape reference (the APE profile REPLACES
  the bespoke grammar; full EN slot table `ecc19d3:SPEC.md` L923-944); §10.6 product line ("Compile"
  ≈L1291, determinism ≈L1296, "IR bridge" deferral ≈L1299). §L·ids (concept inventory — mirror
  `corpus/lexicon/ja_core.yaml` EXACTLY; EN surfaces demand-authored): `pop.adult`/`pop.child` are
  INTERVAL-CARRYING → in the norm layer they collapse to their interval atom (`age>=18`/`age<18`),
  NOT a concept atom (§8.6 docA carries the interval, no `pop.adult` atom) → rendered via the
  interval slot ("age at least 18 years"); `cond.sepsis`, `cond.renal_severe`, `cond.pregnancy`;
  `drug.abx_a`; `act.administer`; `q.age_years` (var; EN surface "age", unit "years"). Modality →
  (direction,strength): recommend→for/strong, suggest→for/weak, may-consider→permit/weak,
  not-recommend→against/strong, not-suggest→against/weak, contraindicate/禁忌→contraindicate/strong
  (禁忌 implies `act.administer`, no explicit verb). Certainty {high|moderate|low|very_low} =
  proof-visible annotation, NOT consumed by conflict logic. §L·drs (APE reality — a finder built +
  ran APE @ pin under SWI 9.2.9): DRS = `drs(Referents, Conditions)`, each condition `Cond-SID/TID`
  (native {sentence,token} provenance); rule = `=>(guard-drs, DEONTIC(action-drs))`; deontic ops
  `should`(recommend) / `must`(obligation) / `may`(permit) / `-drs([],[can(drs(…))])`(contraindicate
  — the negation WRAPS an embedded can-DRS; NO "must not"; 禁忌 = "cannot") / `-drs([],[should(…)])`
  (against-direction) / `~`(NAF); STRENGTH (strong/weak) is ABSENT from the operators → per-modality
  surface+DRS marker pairs w/ 1:1 decode tests are a replan design obligation; interval guards on `object(Ref,Lemma,_,Unit,CountOp,N)`,
  CountOp ∈ {geq,leq,less,greater,exactly,eq} (leq/less/exactly emit NESTED condition sublists);
  `is_wellformed.pl` = the profile allowlist base; first-parse-wins determinism ≠ unique reading →
  pin a canonical tree/DRS per registered pattern + reject multi-reading inputs; byte-identical KB
  must canonicalize referent var-names → `stmt.k`/`bind.k`. CAPITALIZED-OOV: see the corrected
  Fail-closed decision (unprefixed proper-name-position → `named(_)`; prefixed → ordinary `object`,
  raw gate owns the ban). API = `get_ape_results/2,3`
  (module `ape`). Fail-closed signal = non-empty `<message>` XML or `drs([],[])`, never exit code.
  §L·acerules (AceRules reality, CORRECTED 2026-07-13 — `court` is pure SWI-Prolog, ASP severable:
  only `stable_interpreter` shells to smodels (stable-mode-only, unused) → `dependencies/` excluded;
  engine load FIRST needs source-relative rewiring of its hardcoded `../ape/prolog/` APE path to the
  nested layout — ape-build owns it): `generate_rules/3` (`engine/acerules_processor.pl`) takes
  `InputCodes` = RAW TEXT it reparses via APE, dropping SID/TID provenance — NOT a DRS→rule seam →
  the clinical DRS→KB mapper is built DIRECT; AceRules native rule = triple `(Label,Head,Body)`, no
  deontic force (direction/strength purpose-built). Defeat = `court` (`Label: <ACE>.` +
  `L1 overrides L2.` → `priority_handler.pl`; retains `can`/`must` as modal terms). F1 (revised):
  emit SPEC's NAF-guarded PROLEG; `court` = a BOUNDED differential only, via a purpose-built
  ISOMORPHIC paired oracle + an exhaustive fact-presence truth table — platypus = 4 rules +
  2 priorities, and naive-NAF genuinely DIVERGES from `court` on it (both NAF guards fail → no
  `mammal`; `court` derives `mammal`) → NOT a 1:1 oracle. F3: `court` RESOLVES, never REPORTS
  conflicts → conflict-queries builds detection fresh, querying rule records/contexts directly
  (the clinical KB never contains court priorities). §L·lp (Prolog KB shape, §6 LP
  profile): rules-as-data facts `rule/population/condition/action/direction/strength/certainty/
  exception/source` over a fixed kernel; exceptions = NAF-guarded labeled predicates (PROLEG); action
  key `<kind>:<target>`. §L·conflict (§6 verdict machinery): direction groups
  positive{for,require,permit} / against{against,avoid} / contraindicating{contraindicate,avoid};
  eligible = same normalized action ∧ one direction positive while the other against/contraindicating;
  M1 kind `deontic_direction_conflict`; verdicts `semantic_contradiction` / `semantic_no_conflict`
  (+ `documented_no_conflict_result`); LP evidence labels = `participating_rules`, lane=lp,
  solver_status=not_run — SMT vocabulary (`unsat_core` etc.) stays out of the LP lane (§6
  separation). §L·thread (§8.6 docA×docB — the standing conformance thread):
  docA `test_source.m1_guideline_a.rule.0` = {action `act.administer:drug.abx_a`, context cond.sepsis
  ∧ ¬cond.renal_severe ∧ age>=18, for/strong, exc.0}; docB `…guideline_b.rule.0` = {cond.sepsis ∧
  age>=18 ∧ cond.pregnancy, contraindicate, same action-key → eligible}; control `…m1_control` =
  {cond.sepsis ∧ age<18, contraindicate → age disjoint → no-conflict}. Verdict: overlap sat, deontic
  unsat, core `[a.…guideline_a.rule.0, a.…guideline_b.rule.0]`, `deontic_direction_conflict`; control
  documents no-conflict. §8.2 groups + `corpus/reference/m1_expected.yaml`. EN renderings
  (CORRECTED 2026-07-13): the previously banked candidates are NOT valid ACE — probed rejections:
  `For patients …` openings, `[basis …]` brackets, `exception:` prefixes, the passive "…is strongly
  recommended", spaced multiword terms, capitalized content tokens → surface OPEN, the replan
  designs it empirically; confirmed seed frame: `It is recommended that <S>.` → `should(…)`.
  §L·pins (transplant sources): bridge oracle `6406066:.agent/roadmap.md`
  L56-59 (+ `ecc19d3:.agent/roadmap.md` L850-861); interval 16-mask battery `6406066:.agent/
  roadmap.md` L54-55 (fixture `ecc19d3:.agent/roadmap.md` L652-656); harvested APE/AceRules upstream
  report `git show e8b5cf6:docs/cnl-attempto.md`.

- [x] ape-vendor: APE @5f4d535 → `clinicalcnl/` (132) + AceRules engine subset @5b7afb7 → `clinicalcnl/vendor/acerules/` (158) + full Clex @20960a5 → `clinicalcnl/vendor/clex/` (3) (`.git`-stripped, byte-identical to upstream; trees `ac239d2`/`1cebf98`(full-root)/`210d7ea`); grants verified first-hand (APE+AceRules LGPL-3.0-or-later, Clex GPL-3.0-or-later); §11.5 permissive regime + per-resource rows + `CKC_FORK.md`; swipl 9.2.9. 44% 436K/1M — `a400dd1` + codex-review remediation (permissive §11.5; corrected holders/claims per H1/H2/M1/M2; Clex pulled in)
- [ ] ape-build: build the vendored APE + prove it runs under SWI-Prolog 9.2.9 (functional
  env-gate lands HERE). EMPIRICALLY DE-RISKED (a finder built + ran APE @ pin under 9.2.9): `make
  install` → `ape.exe`, 0 errors/0 warnings; the regenerated drace report matched the upstream
  baseline byte-for-byte INCLUDING its existing failures (3733 cases, 0 NEW mismatches; the
  baseline itself carries FAIL/ZERO/non-identical-equivalent rows — never cite it as a lossless
  DRS round-trip proof) — ZERO code changes needed. So this is a build-and-confirm unit, not
  a compat-debug gamble; patch only if the vendored (`.git`-stripped) tree diverges, each edit
  `% CKC:`. READ (post-vendor): `README.md` (SWI packs clib/sgml/http) + `Makefile` (2 steps —
  compile `prolog/parser/*.fit`→`.plp`, then `qsave_program('ape.exe')`; APE has NO `load.pl`); the
  programmatic driver seam = `get_ape_results/2,3` (module `ape`, `prolog/ape.pl`) — what
  cnl-profile/drs-map call, NOT the interactive `runape.pl`. FAIL-CLOSED signal (bank into memory
  `## Runtime` for downstream): a parse error returns non-empty `<message …>` XML or collapses the
  DRS to `drs([],[])`, never a nonzero exit. UPSTREAM-SUITE REALITY (corrected): NO Clex-free
  subset — every upstream test consults `tests/acetexts.pl` (committed) + the full Clex
  (now vendored in-tree at `vendor/clex/clex_lexicon.pl` — copy drop-in over
  `prolog/lexicon/clex_lexicon.pl` + recompile, NO live `ensure_clex` download; `download_acetexts`
  is dead-404 but acetexts is committed); the driver OVERWRITES version-controlled `testruns/` baselines → honest green =
  `git diff --quiet testruns/` + zero mismatch codes, NOT exit code. ACCEPTANCE (fail-closed,
  reproducible): (a) `make install` clean + `get_ape_results` loads under 9.2.9; (b) a smoke parse
  of a clinical-shape sentence (built-in lexicon or a stub ulex) returns a well-formed
  `drs(Referents, Conditions)` — asserted on DRS shape, not exit code; (c) OPTIONALLY the
  Clex-gated upstream regression as an "APE still parses" smoke — pin the EXACT driver command +
  the baseline files it rewrites, gate = `git diff` clean on those baselines + zero NEW mismatch
  codes (drivers differ; zero exit ≠ green; the full Clex is now vendored in-tree —
  GPL-3.0-or-later, §11.5 row — wired drop-in for full-vocabulary APE + a reproducible regression); (d) AceRules
  engine WIRING: rewire its hardcoded `../ape/prolog/` APE path to the nested layout
  (source-relative, `% CKC:` edits) → engine loads warning/error-free + a callable `court` smoke
  (nixon testcase). The real fail-closed corpus gate is conformance-seed's synthetic batteries
  (upstream has none Clex-free). Pre-declared respec seam if it overruns: `[make install +
  get_ape_results loads]` | `[smoke DRS + upstream-suite smoke + acerules wiring]`. Dep: ape-vendor.
- [ ] REPLAN — downstream respec (PLANNING-mode session; BLOCKED on ape-build, which delivers the
  in-tree `ape.exe` probe substrate). The superseded 13-unit expansion (cnl-ulex,
  cnl-profile.1/.2a/.2b, drs-map.1a/.1b/.2, conflict-queries, conformance-seed.a/.b,
  loop-framework) lives at `git show 3bb4a38:.agent/roadmap.md` — MINE it (much survives:
  interval battery, bridge oracle, dep shape, acceptance patterns), redo the decomposition on the
  corrected foundations; unit count unconstrained (13 was an underestimate — codex: expand around
  surface framing, raw-profile enforcement, KB contract/kernel, exception mapping, context solver,
  verdict adapter, runner packaging). METHOD: design the surface/framing sub-language EMPIRICALLY
  against the built `ape.exe` FIRST (byte-pinned raw-text→DRS goldens per construct), then
  re-derive units; every banked empirical fact carries its probe command; adversarially re-probe
  load-bearing facts before granting any FAST-PATH; every discovery-heavy unit gets a concrete
  forward seam + bounded read list. FOUNDATIONS (all precede any ulex/profile/map unit):
  (1) surface/framing grammar — exact raw document grammar (framing, metadata, exception labels,
  basis brackets) + genuinely-parseable ACE rule bodies + raw→DRS goldens (§L·thread rejections;
  seed frame `It is recommended that <S>.` → `should(…)`);
  (2) kb-contract/kernel — exact predicate signatures/arities, ground-term grammar, exception
  argument binding, safety invariants, execution semantics, canonical examples, plunit interface
  tests;
  (3) fail-closed raw-text gate + mutation property battery (per the corrected Fail-closed
  decision above).
  BOUND DESIGN DECISIONS (accepted codex findings 2026-07-13): direct DRS→KB mapper + isomorphic
  purpose-built court differential w/ exhaustive fact-presence truth table (§L·acerules);
  per-modality-pair surface+DRS markers w/ 1:1 decode tests (§L·drs); conflict = rational interval
  arithmetic (CLP(Q) or exact rationals — `clpfd` is INTEGER FD, `X#>18,X#<19` wrongly closes ⇒
  breaks §6 QF_LRA semantics) + DNF disjunct-pair enumeration + concept polarity + exception
  expansion + open/closed-bound properties, querying rule records/contexts directly; conformance
  case manifest + Clex ownership-or-respec (runner decision above); loop-framework bootstrap fix
  (explicit one-time bootstrap state OR split protocol-validation from R1; ledger line staged
  INSIDE the round commit — `.claude/commands/cnl-optimize.md` edit). OWNERS to assign or
  explicitly defer: negated-concept surface (canonical construction + fixtures, or excluded w/ §5
  coverage note), certainty (one exact optional surface or defer/remove), back-reference
  (parseable no-reference construction or a formal §10.6 respec — reaches user), ambiguity policy
  (canonical parse per pattern; reject multi-reading inputs), vendored attack surface (entry-point
  inventory; disable/exclude unused HTTP/webservice/stable-mode surfaces + negative invocation
  test; AGENTS.md security bullet), oracle scope (KB-level only — facts, exception ids, sentence
  indices, bytes; ClinicalIR ids stay behind the backlog IR bridge).

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
