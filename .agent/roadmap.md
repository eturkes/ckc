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

## M3 ClinicalCNL product line (APE fork) + loop framework — plan PENDING

Scope = SPEC §10.6 (2026-07-13 product push = design authority; §0 honesty note). Deliverable:
vendored APE fork building green under SWI-Prolog (9.2.9 confirmed on the dev machine
2026-07-13; environment gate — re-confirm functionally in ape-build + later Prolog-running
units; ape-vendor = `swipl --version` only, no Prolog execution), ClinicalCNL v1 as a
fail-closed ACE profile (EN surface) over a demand-authored clinical ulex, AceRules-adapted
DRS → clinical-Prolog-KB mapping (labeled exception overrides, per-sentence provenance,
byte-identical re-emission), Prolog-side conflict/no-conflict queries re-deriving the M1
docA×docB thread in-lane, a locked synthetic conformance corpus behind ONE runner command, and
the /cnl-optimize + /loop round framework. Rust tree untouched this milestone (no re-bless
anywhere); engine-agnostic rule unchanged (it targets LLM inference engines — SWI-Prolog/APE
are nameable solver/host tooling like Z3). No unit needs the model runtime (no gated units).

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
  acquisition-record schema + first-hand header verification. Clex: NO import (9997607 posture;
  candidate-mining seed) — its upstream-test-suite role is handled in ape-build.
- Fail-closed profile: guessing OFF (`-noclex -ulexfile`; `-guess` default off) rejects lowercase
  OOV, BUT APE forces every capitalized token to `named(_)` regardless (empirical) → fail-closed is
  enforced by the PROFILE CHECKER rejecting any unregistered `named(_)` referent + the grammar
  forbidding capitalized content tokens and inline `n:`/`v:`/`a:`/`p:` prefixes; a clean parse alone
  never proves lexical closure (§10.6's "guessing disabled = parse error" holds only for lowercase).
  The profile checker validates APE parse output against registered sentence patterns, rejects
  naming the sentence + construct. EN interval markers at least / at most / less than / more than ↔
  ge/le/lt/gt (→ APE `object` CountOp geq/leq/less/greater) — hand-oracled battery, expected values
  hand-written, never derived from the mapping under test.
- Determinism: same accepted document → byte-identical KB text (canonical clause order +
  emission); provenance = {document id, sentence index} carried on every emitted clause group.
- Conformance runner: ONE command (plunit driver script under `clinicalcnl/clinical/`) =
  upstream fork suite + profile battery + mapping battery + conflict queries + corpus
  round-trip; THE loop round gate. Milestone acceptance = runner green over the locked corpus
  + a ledgered manual dry-run round (loop-framework).
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
  `should`(recommend) / `must`(obligation) / `may`(permit) / `-(can(...))`(contraindicate — NO "must
  not"; 禁忌 = "cannot") / `~`(NAF); interval guards on `object(Ref,Lemma,_,Unit,CountOp,N)`,
  CountOp ∈ {geq,leq,less,greater,exactly,eq} (leq/less/exactly emit NESTED condition sublists);
  `is_wellformed.pl` = the profile allowlist base; first-parse-wins determinism; byte-identical KB
  must canonicalize referent var-names → `stmt.k`/`bind.k`. CAPITALIZED-OOV hole: a capitalized token
  ALWAYS → `named(_)` regardless of `-guess`/`-noclex` (see Fail-closed). API = `get_ape_results/2,3`
  (module `ape`). Fail-closed signal = non-empty `<message>` XML or `drs([],[])`, never exit code.
  §L·acerules (AceRules reality — `court` is pure SWI-Prolog, ASP severable: only `stable_interpreter`
  shells to smodels (stable-mode-only, unused) → `dependencies/` excluded, engine loads): DRS→rule
  seam = `generate_rules/3` (`engine/acerules_processor.pl`); native rule = triple `(Label,Head,Body)`
  → reshape to §6 vocabulary (F2: no deontic force → direction/strength purpose-built). Defeat =
  `court` (`Label: <ACE>.` + `L1 overrides L2.` → `priority_handler.pl`). F1 (decided): emit SPEC's
  NAF-guarded PROLEG; use `court` + oracles `engine/testcases/court/` (nixon; platypus =
  2-rule×2-exception ≅ clinical rec×contra) as a DIFFERENTIAL cross-check. F3: `court` RESOLVES,
  never REPORTS conflicts → conflict-queries builds detection fresh. §L·lp (Prolog KB shape, §6 LP
  profile): rules-as-data facts `rule/population/condition/action/direction/strength/certainty/
  exception/source` over a fixed kernel; exceptions = NAF-guarded labeled predicates (PROLEG); action
  key `<kind>:<target>`. §L·conflict (§6 verdict machinery): direction groups
  positive{for,require,permit} / against{against,avoid} / contraindicating{contraindicate,avoid};
  eligible = same normalized action ∧ one direction positive while the other against/contraindicating;
  M1 kind `deontic_direction_conflict`; verdicts `semantic_contradiction` / `semantic_no_conflict`
  (+ `documented_no_conflict_result`). §L·thread (§8.6 docA×docB — the standing conformance thread):
  docA `test_source.m1_guideline_a.rule.0` = {action `act.administer:drug.abx_a`, context cond.sepsis
  ∧ ¬cond.renal_severe ∧ age>=18, for/strong, exc.0}; docB `…guideline_b.rule.0` = {cond.sepsis ∧
  age>=18 ∧ cond.pregnancy, contraindicate, same action-key → eligible}; control `…m1_control` =
  {cond.sepsis ∧ age<18, contraindicate → age disjoint → no-conflict}. Verdict: overlap sat, deontic
  unsat, core `[a.…guideline_a.rule.0, a.…guideline_b.rule.0]`, `deontic_direction_conflict`; control
  documents no-conflict. §8.2 groups + `corpus/reference/m1_expected.yaml`. Candidate EN renderings
  (author + validate vs APE): docA "for patients with sepsis and age at least 18 years,
  administration of antibiotic A is strongly recommended. exception: patients with severe renal
  impairment."; docB "…and with pregnancy … is contraindicated"; control "…age less than 18 years …
  is contraindicated". §L·pins (transplant sources): bridge oracle `6406066:.agent/roadmap.md`
  L56-59 (+ `ecc19d3:.agent/roadmap.md` L850-861); interval 16-mask battery `6406066:.agent/
  roadmap.md` L54-55 (fixture `ecc19d3:.agent/roadmap.md` L652-656); harvested APE/AceRules upstream
  report `git show e8b5cf6:docs/cnl-attempto.md`.

- [ ] ape-vendor: fetch APE + an AceRules SUBSET into the fork tree (upstream layout, strip
  `.git`). Verified facts (probed + first-hand-checked 2026-07-13; re-probe at fetch): APE =
  `github.com/Attempto/APE`, HEAD `5f4d5354a45fb772763bf1a9543f508f15b28982` (default branch;
  core code 2008-2013, HEAD maintained through 2024) → `clinicalcnl/` ROOT
  (it is THE fork we build/patch); AceRules = `github.com/tkuhn/AceRules` →
  `clinicalcnl/vendor/acerules/`, engine-source subset ONLY (`engine/` = the DRS→rule mapping we
  adapt + `LICENSE.txt`/`README.md` for attribution), EXCLUDING `dependencies/` (bundled
  `lparse-1.1.2.tar.gz` + `smodels-2.34.tar.gz` = GPL-2.0-or-later ASP solvers (per-file headers
  "version 2 … or any later version" → GPLv3-COMPATIBLE, not GPLv2-only), excluded as UNNEEDED —
  our Prolog `court` conflict queries replace ASP solving; excluded ⇒ no obligation) + `docker/`/`webapp/`/`webclient/` (deployment/UI, outside the engine subset) + top-level `.gitignore` (repo metadata);
  record excluded paths. Files common to both roots that ARE placed (`LICENSE.txt`/`README.md`) land in different dirs →
  subdir placement + `CKC_FORK.md` naming keep them distinct (AceRules `.gitignore` excluded, no clash). License = LGPL-3.0-or-later per PER-FILE
  SOURCE HEADERS ("either version 3 of the License, or (at your option) any later version" —
  verified in `ape.pl` + `engine/acerules_processor.pl`); `LICENSE.txt` carries the LGPLv3 TEXT
  but the headers are the operative grant, so GitHub's NOASSERTION is non-authoritative detector metadata (it neither
  establishes nor contradicts the header grant). §11.5 SEQUENCING (reconciles "row recorded BEFORE acquisition" with post-fetch
  attestation): remote header preflight → draft row {rights holder, source URL, permissions,
  target commit} → fetch the pinned commit → verify headers first-hand + compute snapshot hash →
  finalize row + land the tree atomically in ONE commit. Deliver: (1) APE at root + AceRules
  subset at `vendor/acerules/`; (2) headers verified FIRST-HAND (`LICENSE.txt` +
  `ape.pl`/`engine/acerules_processor.pl` — never memory for grants); (3) §11.5 evidence rows =
  extend the "Standing verdicts" APE/AceRules prose (rows in §11.5 PROSE, not a registry file)
  per repo — repo, exact commit, snapshot hash = `git rev-parse HEAD` (commit) + `HEAD^{tree}`
  (tree SHA, recorded PRE-strip), as-of date, LGPL-3.0-or-later grant + operative-header note,
  obligations met (notices + LICENSE text retained, corresponding source = the vendored subtree,
  provenance file), AceRules excluded-paths note — and DROP the stale "no port planned" clause;
  (4) provenance `clinicalcnl/CKC_FORK.md` (per-repo: upstream repo, commit, license, what/why
  vendored, AceRules inclusion boundary, what CKC adds). Clex NOT vendored (9997607 posture;
  candidate-mining seed) — its test-suite role is ape-build's. NO Prolog execution — `swipl
  --version` only (functional confirmation is ape-build's).
  FAST-PATH (next session — recipe supersedes discovery; prose above = rationale; spec is self-contained, skip re-reading planning commits):
  - Pins (deterministic): APE `5f4d5354a45fb772763bf1a9543f508f15b28982`, AceRules `5b7afb7bdfbce56027997307f9b798af53551223`. Clone each + `git checkout <pin>` → `git rev-parse HEAD HEAD^{tree}` CONFIRMS commit+tree (APE tree `ac239d2…`, AceRules tree `1cebf98…`) → place via `git archive HEAD <paths> | (cd <dest> && tar -x)` (no `.git`): APE whole repo → `clinicalcnl/`; AceRules `engine LICENSE.txt README.md` → `clinicalcnl/vendor/acerules/`. Nothing-dropped = PER-SOURCE manifest match `find <dest> -type f | wc -l` == `git -C <clone> ls-tree -r --name-only <pin> -- <paths> | wc -l` (run the APE match BEFORE nesting acerules + `CKC_FORK.md` under `clinicalcnl/`, or path-scope it `-not -path '*/vendor/*' -not -name CKC_FORK.md`, else the post-placement `find clinicalcnl -type f` = 291 ≠ 132 gives a false drop) (an untracked dir COLLAPSES to a single `?? clinicalcnl/` line, so `git status --porcelain clinicalcnl | wc -l` = 1, NOT the file count); then a plain `git status --porcelain` clean outside the intended add.
  - VERIFY grants via NARROW reads ONLY — never whole-file (`LICENSE.txt` 41.8K ea, `ape.pl` 21.6K): `clinicalcnl/ape.pl` L1-25 + `…/acerules/engine/acerules_processor.pl` L1-13 = per-file grant ("…either version 3 … or (at your option) any later version") → LGPL-3.0-or-later; each `LICENSE.txt` first ~15 lines = "GNU LESSER GENERAL PUBLIC LICENSE / Version 3" spot-check.
  - CKC_FORK.md APE version = SWI-pack version `6.7.180715` (`pack.pl`) + release `6.7-180714` (`CHANGES.md`); APE HEAD 2024-04-21, AceRules HEAD 2024-11-01, © per repo (never conflate) — APE 2008-2013 Attempto Group / University of Zurich, AceRules 2008-2012 Tobias Kuhn. (`6.7-131003` = an OLD changelog/README-transcript entry, NOT the current version — never record it as core.)
  - SPEC §11.5 edit = narrow-read L1433-1456 (evidence-row SCHEMA at L1435-1446 + the "Standing verdicts:" sentence). Edit A: "…technical-fit verdicts stand, no port planned" → APE + the AceRules ENGINE SUBSET now vendored into `clinicalcnl/` (rows below), Codeco unvendored (no port planned). Edit B: insert 2 per-repo evidence bullets (facts above) before "Adopted ACE precedents:", each conforming to the row schema — rights holder, source URL, commit + `HEAD^{tree}`, as-of date, the 5 permission modes (acquire / process-mine / author-derivative / commit-derived / redistribute), operative LGPL-3.0-or-later HEADER grant, reuse-mode obligations met (notices + LICENSE-text + corresponding-source = the vendored subtree). Phrase the grant as the upstream-DECLARED project license with notices retained, NOT a per-blob rights audit.
  - Close (context + roadmap closure land INSIDE the one commit, NOT after): run `.agent/context.sh` → in roadmap.md record usage + collapse this unit to a `- [x]` stub + fill the M3 header `plan PENDING → f76e1fa` (the product-push `(M3 plan)` commit that set the current APE-fork scope; `git log --grep 'M3 plan): product push'` confirms — NOT `tail -1` of all `(M3 plan)`, which returns the pre-push series), milestone stays IN-PROGRESS → Marksman-clean SPEC.md + CKC_FORK.md → ONE atomic commit `clinicalcnl+spec (M3.ape-vendor): …` staging {vendor tree, `CKC_FORK.md`, SPEC §11.5 edit, roadmap closure}.
- [ ] ape-build: build the vendored APE + prove it runs under SWI-Prolog 9.2.9 (functional
  env-gate lands HERE). EMPIRICALLY DE-RISKED (a finder built + ran APE @ pin under 9.2.9): `make
  install` → `ape.exe`, 0 errors/0 warnings; full upstream regression 3733 cases / 0 mismatches,
  DRS→CoreACE byte-identical — ZERO code changes needed. So this is a build-and-confirm unit, not
  a compat-debug gamble; patch only if the vendored (`.git`-stripped) tree diverges, each edit
  `% CKC:`. READ (post-vendor): `README.md` (SWI packs clib/sgml/http) + `Makefile` (2 steps —
  compile `prolog/parser/*.fit`→`.plp`, then `qsave_program('ape.exe')`; APE has NO `load.pl`); the
  programmatic driver seam = `get_ape_results/2,3` (module `ape`, `prolog/ape.pl`) — what
  cnl-profile/drs-map call, NOT the interactive `runape.pl`. FAIL-CLOSED signal (bank into memory
  `## Runtime` for downstream): a parse error returns non-empty `<message …>` XML or collapses the
  DRS to `drs([],[])`, never a nonzero exit. UPSTREAM-SUITE REALITY (corrected): NO Clex-free
  subset — every upstream test consults `tests/acetexts.pl` (committed) + the full Clex
  (`tests/downloader.pl ensure_clex` ← github, live; `download_acetexts` is dead-404 but acetexts
  is committed); the driver OVERWRITES version-controlled `testruns/` baselines → honest green =
  `git diff --quiet testruns/` + zero mismatch codes, NOT exit code. ACCEPTANCE (fail-closed,
  reproducible): (a) `make install` clean + `get_ape_results` loads under 9.2.9; (b) a smoke parse
  of a clinical-shape sentence (built-in lexicon or a stub ulex) returns a well-formed
  `drs(Referents, Conditions)` — asserted on DRS shape, not exit code; (c) OPTIONALLY the
  Clex-gated upstream regression as an "APE still parses" smoke (git-diff-gated; Clex = GPL-3.0
  test-only, own §11.5 row + pin if run; 9997607 bars only content IMPORT into the product). The
  real fail-closed corpus gate is conformance-seed's synthetic batteries (upstream has none
  Clex-free). Pre-declared respec seam if it overruns: `[make install + get_ape_results loads]` |
  `[smoke DRS + upstream-suite smoke]`. Dep: ape-vendor.
- [ ] cnl-ulex: clinical ulex seed — the concept inventory (legend §L·ids) re-expressed EN as APE
  ulex Prolog facts, loaded fail-closed. Entry ids mirror the committed concept ids EXACTLY
  (IR-bridge alignment); EN surfaces demand-authored HERE (no EN corpus exists). READ
  (post-ape-build): `ulex.pl` `lexicon_template/1` = the authoritative fact functors/arities —
  `noun_sg(WF,Lemma,Gender∈{neutr,masc,fem,human})`/`noun_pl`/`noun_mass`, `tv_finsg/tv_infpl/tv_pp`,
  `iv_*`, `dv_*(…,Prep)`, `adj_itr(WF,Lemma)`/`adj_tr(…,Prep)` (+comp/sup), `adv`, `prep`,
  `pn_sg/pndef_sg(WF,Lemma,Gender)`, `mn_sg(WF,Unit)`; `prolog/lexicon/clex_lexicon.pl` = real
  examples; multiword = hyphen-joined single atom (`'beta-blocker'`), patient/doctor Gender
  `human`. FACTS: `q.age_years` → quantity surface "age" unit "years"; the contraindicate modality
  MUST surface as a "cannot"-form (APE has NO "must not"; `禁忌`/contraindicate → `-(can(action))`
  DRS, implies `act.administer`); modality→(direction,strength) per legend. Load fail-closed =
  `-noclex -ulexfile clinical.pl` (only clinical ulex + APE function words parse; clex gives ZERO
  clinical vocab — patient/drug/administer all OOV). Duplicate/redefine is a WARNING only in APE →
  this unit ENFORCES id-alignment + dedup itself. Acceptance: load-integrity battery (unknown id /
  duplicate id / malformed row rejected by our checker) + APE loads the ulex + parses a seed
  sentence (guessing off) returning a DRS over ONLY ulex + function-word terminals. Dep: ape-build.
- [ ] cnl-profile.1: profile checker v1 over APE parse output (`get_ape_results` DRS) for the
  RECOMMENDATION sentence pattern — context guard (population/condition concept + interval atoms,
  DNF) + action (target + `act.*`) + modality (direction+strength from the deontic operator) [+
  optional certainty, a proof-visible annotation NOT consumed by conflict logic]; fail-closed
  REJECTS out-of-profile, naming the sentence + construct. READ: DRS shape (legend §L·drs + memory
  `## Runtime` from ape-build) — rule = `=>(guard-drs, DEONTIC(action-drs))`, deontic ops
  `should`/`must`(for)/`may`(permit)/`-(can(…))`(contraindicate)/`~`(NAF); `is_wellformed.pl` = the
  authoritative legal-condition allowlist to base the checker on; AceRules `drs_checker.pl` (a
  3-level structural checker) = an ADAPTABLE template (its rules FORBID the DNF + surviving
  modality the clinical profile needs — reuse the level-framework, not the rules). CRITICAL
  fail-closed (legend §L·drs capitalized-OOV hole): REJECT every `named(_)` referent whose name ∉
  the registered proper-name allowlist, and reject inline `n:`/`v:`/`a:`/`p:` prefix tokens — a
  clean parse alone never proves closure. Acceptance: plunit ACCEPT battery (§8.6-shape EN
  recommendation sentences, legend candidate renderings) + REJECT battery (out-of-profile parse,
  unregistered `named(_)`, guess/prefix attempt, imperative, question, `must not` (a parse error)).
  Pre-declared seam if it overruns (first APE-DRS-shape read is the driver): `[checker-core + accept
  battery]` | `[reject battery]`. Dep: cnl-ulex.
- [ ] cnl-profile.2a: complete the profile's SENTENCE STRUCTURE — labeled EXCEPTION sentences
  (each a separate sentence with its OWN basis bracket = per-sentence provenance; one
  single-concept, interval-free ExceptionClause per entry; §5 exceptions = separate labeled
  payloads) + and/or DNF context (flat two-level; conjunction binds tighter than disjunction; each
  disjunct → one statement). READ: pre-reset sentence model + slot table `ecc19d3:SPEC.md`
  L923-944 (shape reference — the APE profile REPLACES the bespoke grammar); DNF precedence §10.4
  ("かつ binds tighter", ≈L1122); §6 (exceptions → negated context conjuncts, ≈L557). Completes the
  transplanted pre-reset SENTENCE MODEL (one rule = recommendation + basis + zero-or-more labeled
  exceptions). Acceptance: accept battery (rule + basis + ≥0 labeled exceptions; multi-disjunct DNF
  contexts) + reject battery (malformed exception, bad connective, three-level nesting). Dep:
  cnl-profile.1.
- [ ] cnl-profile.2b: the INTERVAL sub-language + the fresh anaphora-family reject probes.
  Interval atoms — four EN bound markers (at least/at most/less than/more than ↔ ge/le/lt/gt → APE
  `object` CountOp geq/leq/less/greater; APE emits NESTED condition sublists for leq/less/exactly),
  ASCII-digit leading-zero-free numerals ≥ 0; hand-oracled validity battery = the
  16-bound-presence-mask table (`6406066:.agent/roadmap.md` L54-55; fixture `ecc19d3:.agent/
  roadmap.md` L652-656): 16 masks × per-bound {-1,0,1}, valid iff exactly one bound present ∧ value
  ≥ 0 — expected values HAND-WRITTEN, never derived from the recognizer under test. Negative-
  occurrence bar: interval-carrying concepts barred from negated + exception slots (2a's slots);
  repair = complement-interval rewrite. Anaphora-family reject probes (FRESH — no transplant
  battery; APE natively RESOLVES anaphora/definite-ref, silently merging referents): detect + reject
  APE parses that used a pronoun, ellipsis, or a definite article beyond the intended in-rule
  back-reference. Acceptance: the 16-mask interval battery + negative-occurrence reject cases + the
  anaphora-family reject probes. Dep: cnl-profile.2a (its negated/exception slots).
- [ ] drs-map.1a: DRS → clinical Prolog KB CORE (AceRules-adapted), EXCEPTION-FREE. Map the
  accepted `=>(guard, DEONTIC(action))` DRS into the §6 LP-profile rules-as-data facts (legend
  §L·lp: rule/population/condition/action/direction/strength/certainty/source) — recommendation/
  contraindication predicates carrying {action, target, direction, strength}; population/condition
  GUARDS = §5 ContextExpr DNF over concept | negated-concept | INTERVAL atoms (intervals ARE DNF
  guard atoms per §5 "quantity interval" ≈L456, compiled from `object` CountOp — ALL guard
  compilation lands here, no separate interval unit); `{document_id, sentence_index}` provenance on
  every clause group (APE gives it native as `Cond-SID/TID`). READ (first AceRules-engine read):
  `acerules/engine/acerules_processor.pl` `generate_rules/3` = the DRS→rule seam we ADAPT (native
  rule = triple `(Label,Head,Body)`, rules-as-data but APE-DRS-shaped literals → reshape to §6
  vocabulary + build the {direction,strength} decode FRESH — AceRules gives NO deontic force, F2).
  DRS shape + deontic map = legend §L·drs. Action key `<kind>:<target>`
  (`act.administer:drug.abx_a`); `禁忌`/`-(can())` implies act.administer. Acceptance: maps the
  worked EXCEPTION-FREE rules (docB contraindication + control + a synthetic recommendation, all
  re-expressed EN) to the expected KB TERMS — asserted on term STRUCTURE + provenance, NOT bytes
  (byte-emission = drs-map.1b). Pre-declared seam if the AceRules read + guard compilation overruns:
  `[concept/negated guards + facts]` | `[interval-guard compilation]`. Dep: cnl-profile.2b, ape-build.
- [ ] drs-map.1b: canonical KB EMISSION + the byte-identical re-emission law, EXCEPTION-FREE. SPEC
  gives NO canonical clause order for the Prolog KB (a genuine gap — only §6 SMT byte discipline,
  "Emission is deterministic", ≈L578, as a template) → DEFINE + COMMIT one here: a canonical clause
  order (byte-sorted, SMT-consistent) + a dedicated deterministic clause writer over drs-map.1a's
  terms (do NOT lean on `write_term`; canonicalize referent var-names → `stmt.k`/`bind.k`, since APE
  var-names are serializer output). Law: an accepted document re-emits a byte-identical KB.
  Acceptance: byte-pin battery — an EXCEPTION-FREE document's KB text pinned from OBSERVED emitter
  output (never hand-computed); re-emit == the pinned bytes; canonical order stable under input
  clause reordering. (drs-map.2 extends this emitter to exception clauses + completes the law over
  exception-bearing docs.) Dep: drs-map.1a.
- [ ] drs-map.2: labeled EXCEPTION override compilation — the PROLEG pattern (a default rule +
  negation-as-failure-guarded labeled exception predicates: `head :- guards, \+ exception_k(…)`, §6
  LP profile ≈L603), extending drs-map.1b's emitter to exception clauses + COMPLETING the
  byte-identical law over exception-bearing docs. F1 (decided): emit SPEC's NAF-guarded shape; use
  AceRules `court` (strong-neg + runtime-priority defeat — a DIFFERENT encoding, same skeptical
  outcome) purely as a DIFFERENTIAL cross-check. READ: `acerules/engine/court_interpreter/` +
  `parser/priority_handler.pl` (defeat/priority machinery — understanding AceRules' semantics well
  enough to re-target is this unit's discovery); ready ORACLES `engine/testcases/court/` — nixon
  (2-rule+priority) + platypus (2-rule×2-exception, maps 1:1 onto clinical recommendation ×
  contraindication-exception). Acceptance: the worked 2-rule × 2-exception ORACLE (transplant the
  bridge-oracle shape: a 2-disjunct rule × 2 exceptions splits De Morgan
  `(D1∨D2)∧¬E=(D1∧¬E)∨(D2∧¬E)`, `exc.<k>` counted statement-major then sentence order — stmt.0 owns
  exc.0/exc.1, stmt.1 owns exc.2/exc.3, clone content + basis per statement, counters
  document-continuous; a trailing 1-disjunct rule → rule.2/stmt.2/exc.4 pins cumulative offsets,
  catching a per-rule counter RESET that passes the bare 2×2; enumerated `bind.<k>→concept` map;
  clause `region_ids` = its own sentence's basis bracket verbatim) — cross-checked against platypus
  under `court`. Full oracle: `6406066:.agent/roadmap.md` L56-59 + `ecc19d3:.agent/roadmap.md`
  L850-861. Dep: drs-map.1b.
- [ ] conflict-queries: Prolog-side conflict / no-conflict QUERY layer mirroring SPEC §6's verdict
  categories (legend §L·conflict), over the drs-map KB. F3: AceRules `court` RESOLVES conflicts into
  one model, it never REPORTS them → build the detection FRESH — for an eligible pair (same action,
  opposed direction groups) query derivability of the action literal AND its strong-negation BEFORE
  priority resolution → a contradiction naming the participating rules + provenance; context overlap
  / no-conflict via `library(clpfd)` (SWI-bundled, no fetch) over interval guards (`X#>=18 ∧ X#<18`
  fails → disjoint → documented no-conflict). Lane caveat (§6 ≈L609 / §10.6 ≈L1299): LP verdicts
  NEVER replace §6 SMT verdicts (SMT/Z3 cross-check deferred behind the IR-bridge backlog) → name the
  absence. Acceptance: the M1 docA×docB thread re-derived IN-LANE (legend §L·thread) — the conflict
  pair surfaces `deontic_direction_conflict` with participating rules + provenance; the control pair
  (age<18 vs age≥18, clpfd-disjoint) documents no-conflict = the standing conformance thread. Dep:
  drs-map.1a/.1b/.2.
- [ ] conformance-seed.a: synthetic conformance corpus v1 (AUTHORING + lock). Re-express the
  docA/docB/control thread (legend §L·thread) as ClinicalCNL EN documents + author ≥2 fresh scenario
  docs covering an interval bound + a multi-exception rule (exercising drs-map.2's 2×2 `exc.<k>`
  clone shape). Expected verdicts HAND-AUTHORED at corpus authoring, never route-derived (§0/§10.6
  honesty: the APE line is a user-directed bet, never "evidence-selected"). Acceptance: each document
  round-trips individually through the prior pipeline (parse → profile → map → emit byte-identical
  KB) + its hand-authored verdict holds; corpus committed + drift-guarded (locked). Dep: drs-map.*,
  conflict-queries.
- [ ] conformance-seed.b: the ONE-command conformance RUNNER + lock = the milestone acceptance core
  (FIRST cross-unit aggregation — read-cost = every prior battery's interface). A single plunit
  driver under `clinicalcnl/clinical/` aggregates: the Clex-gated upstream "APE still parses" smoke
  (ape-build) + the profile batteries (cnl-profile.*) + the mapping batteries (drs-map.*) + the
  conflict queries + the corpus round-trip (conformance-seed.a). Acceptance: the runner GREEN over
  the locked corpus (= the milestone acceptance core + the /loop round gate). Dep: all prior
  technical units.
- [ ] loop-framework: protocol validation — run ONE manual end-to-end /cnl-optimize round to confirm
  the loop drives the product line, fixing any protocol friction found; closes the milestone.
  Scaffolding already landed (`.claude/commands/cnl-optimize.md` 81 L + `.agent/cnl-queue.md` 24 L) —
  this unit EXERCISES it. R1 is bootstrapped by MANUALLY following the round law (the /loop skill
  guards post-DONE rounds R2+). Acceptance: one round PICKS an increment (a conformance-seed.a
  leftover or a generalization candidate) → lands a green commit `cnl-opt (R1)` (conformance runner
  green, tree clean) → appends one ledger line; the round law's gate stack runs clean; `/loop
  /cnl-optimize` confirmed to drive it. Closes the milestone; user enables /loop
  (skillOverrides.loop:"on"). Dep: conformance-seed.b.

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
