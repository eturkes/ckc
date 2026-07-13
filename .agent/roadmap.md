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
- Fail-closed profile: unknown-word guessing disabled/rejected; profile checker validates APE
  parse output against registered sentence patterns, rejects naming sentence + construct
  (anti-ACE lesson: bare out-of-lexicon text = parse error, never a guess). EN interval
  markers: at least / at most / less than / more than ↔ ge/le/lt/gt — hand-oracled battery,
  expected values hand-written, never derived from the mapping under test.
- Determinism: same accepted document → byte-identical KB text (canonical clause order +
  emission); provenance = {document id, sentence index} carried on every emitted clause group.
- Conformance runner: ONE command (plunit driver script under `clinicalcnl/clinical/`) =
  upstream fork suite + profile battery + mapping battery + conflict queries + corpus
  round-trip; THE loop round gate. Milestone acceptance = runner green over the locked corpus
  + a ledgered manual dry-run round (loop-framework).

- [ ] ape-vendor: fetch APE + an AceRules SUBSET into the fork tree (upstream layout, strip
  `.git`). Verified facts (probed + first-hand-checked 2026-07-13; re-probe at fetch): APE =
  `github.com/Attempto/APE`, HEAD `5f4d5354a45fb772763bf1a9543f508f15b28982` (default branch;
  core version 6.7-131003, code 2008-2013, HEAD maintained through 2024) → `clinicalcnl/` ROOT
  (it is THE fork we build/patch); AceRules = `github.com/tkuhn/AceRules` →
  `clinicalcnl/vendor/acerules/`, engine-source subset ONLY (`engine/` = the DRS→rule mapping we
  adapt + `LICENSE.txt`/`README.md` for attribution), EXCLUDING `dependencies/` (bundled
  `lparse-1.1.2.tar.gz` + `smodels-2.34.tar.gz` = GPLv2-only ASP solvers — incompatible-direction
  AND unneeded: our Prolog conflict queries replace ASP solving) + `docker/`/`webapp/`/`webclient/`;
  record excluded paths. Both upstream roots collide on `.gitignore`/`LICENSE.txt`/`README.md` →
  the subdir placement + `CKC_FORK.md` naming resolve it. License = LGPL-3.0-or-later per PER-FILE
  SOURCE HEADERS ("either version 3 of the License, or (at your option) any later version" —
  verified in `ape.pl` + `engine/acerules_processor.pl`); `LICENSE.txt` carries the LGPLv3 TEXT
  but the headers are the operative grant, so GitHub's NOASSERTION metadata is accurate (not
  "wrong"). §11.5 SEQUENCING (reconciles "row recorded BEFORE acquisition" with post-fetch
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
  - Pins (deterministic): APE `5f4d5354a45fb772763bf1a9543f508f15b28982`, AceRules `5b7afb7bdfbce56027997307f9b798af53551223`. Clone each + `git checkout <pin>` → `git rev-parse HEAD HEAD^{tree}` CONFIRMS commit+tree (APE tree `ac239d2…`, AceRules tree `1cebf98…`) → place via `git archive HEAD <paths> | (cd <dest> && tar -x)` (no `.git`): APE whole repo → `clinicalcnl/`; AceRules `engine LICENSE.txt README.md` → `clinicalcnl/vendor/acerules/`. Nothing-dropped = `git status --porcelain clinicalcnl | wc -l` == placed count (NOT per-file check-ignore).
  - VERIFY grants via NARROW reads ONLY — never whole-file (`LICENSE.txt` 41.8K ea, `ape.pl` 21.6K): `clinicalcnl/ape.pl` L1-25 + `…/acerules/engine/acerules_processor.pl` L1-13 = per-file grant ("…either version 3 … or (at your option) any later version") → LGPL-3.0-or-later; each `LICENSE.txt` first ~15 lines = "GNU LESSER GENERAL PUBLIC LICENSE / Version 3" spot-check.
  - CKC_FORK.md APE version = `pack.pl` `version('6.7.180715')`, HEAD 2024-04-21; AceRules HEAD 2024-11-01, © 2008-2012 Tobias Kuhn (L60 "6.7-131003" = older CHANGES entry — use pack.pl).
  - SPEC §11.5 edit = narrow-read L1447-1456 ("Standing verdicts:" sentence): drop "no port planned" + note APE+AceRules vendored into `clinicalcnl/`; insert the 2 per-repo evidence bullets (facts above) before "Adopted ACE precedents:".
  - Marksman-clean SPEC.md + CKC_FORK.md → ONE atomic commit `clinicalcnl+spec (M3.ape-vendor): …` → close: `.agent/context.sh`, collapse this unit to a `- [x]` stub, milestone stays IN-PROGRESS.
- [ ] ape-build: build the vendored APE + prove it runs under SWI-Prolog 9.2.9 (the functional
  env-gate lands HERE; record what built/loaded/parsed in the commit). Load/build entry (verified
  — APE has NO `load.pl`): `ape.pl` = the Prolog load entry; `Makefile` + `run.sh` + `pack.pl` =
  build/run/SWI-pack scaffolding; `README.md` = build steps — read them post-fetch. Compat: core
  6.7-131003 (2008-2013), HEAD maintained through 2024 → SWI 9.2.9 compatibility UNKNOWN, determine
  here; patch only where wiring/compat demands, each edit `% CKC:`, layout else untouched.
  UPSTREAM-SUITE REALITY (verified — a zero shell exit does NOT mean green): the driver
  `tests/test_everything.sh` has no fail-fast, OVERWRITES version-controlled baselines in
  `testruns/` (pass = `git diff` on outputs empty modulo random ids, NOT exit code), and needs TWO
  external MUTABLE fetches — Clex (`tests/downloader.pl ensure_clex` ← `Clex/master`, the larger
  lexicon) + the acetexts regression set (`download_acetexts` ← attempto.ifi.uzh.ch);
  `examples/paraphrase_roundtrip.pl` prints pass/fail but exits 0 either way (weak smoke, not a
  gate). ACCEPTANCE (fail-closed, Clex-free, reproducible): (a) APE builds + `ape.pl` loads clean
  under SWI 9.2.9; (b) a smoke parse of a sentence in APE's built-in lexicon (a small one ships;
  the suite needs a LARGER one) returns a DRS — asserted on the DRS shape, not a zero exit; (c) any
  upstream test that runs Clex-FREE (the DRS→X direction — `test_drace`/`test_owlswrl`/`test_drs_to_x`
  take DRS input; confirm at build) gated fail-closed = preserve baselines, diff outputs, scan
  failure markers, per-component status + counts. DEFERRED (do NOT claim as the gate): the text→DRS
  regression (`rtest.bash`) + anything needing Clex/acetexts — if later run, wrap fail-closed AND pin
  the test-only Clex/acetexts fetches under §11.5 (Clex GPL-3.0 = compatible for test-only use, still
  needs its own row + pin; 9997607 blocks content IMPORT into the product, not test tooling).
  (Unbounded-debug half, isolated so a fresh window absorbs the build churn.)
- [ ] cnl-ulex: clinical ulex seed — the M1/M2 semantic inventory re-expressed EN (adult /
  child / sepsis / severe renal impairment / pregnancy; antibiotic A target; administer
  action; age-years quantity with unit years), entry ids mirroring committed lexicon concept
  ids (pop.*, cond.*, drug.abx_a, act.administer, q.age_years) for later IR-bridge
  alignment; APE loads it with guessing off; load-integrity battery (unknown id, duplicate,
  malformed row rejected).
- [ ] cnl-profile.1: profile checker v1 over APE output for the recommendation sentence
  pattern (population/condition context guard + action + modality direction/strength);
  fail-closed rejects naming sentence + construct; plunit accept battery (worked §8.6-shape
  sentences re-expressed EN) + reject battery (out-of-profile parse, unknown word, guess
  attempt, imperative/question forms).
- [ ] cnl-profile.2: exception sentences (labeled, own basis reference) + interval atoms
  (four EN bound markers, hand-oracled validity battery incl. bound-presence masks) + and/or
  DNF with precedence pins + anaphora/pronoun/ellipsis/definite-reference reject probes —
  completes the transplanted pre-reset sentence model (one rule = recommendation + basis +
  zero-or-more labeled exceptions).
- [ ] drs-map.1: DRS → clinical KB core (AceRules-adapted): recommendation/contraindication
  predicates {action, target, direction, strength}, population/condition guards from DNF,
  {document id, sentence index} provenance on every clause group; byte-identical re-emission
  law + battery.
- [ ] drs-map.2: labeled exception override compilation (PROLEG pattern: default rule +
  exception clauses + defeat wiring per AceRules semantics) + interval guard compilation;
  worked 2-rule × 2-exception oracle (transplant the banked bridge-oracle shape: stmt.0 owns
  exc.0/exc.1, stmt.1 owns exc.2/exc.3, trailing 1-exception rule catches per-rule counter
  resets).
- [ ] conflict-queries: query layer mirroring SPEC §6 verdict categories — deontic conflict
  on overlapping guards (same action, opposing direction), documented no-conflict; M1
  docA×docB thread re-derived in-lane (contradiction surfaced with participating rules +
  provenance; control pair documents no-conflict) = the standing conformance thread.
- [ ] conformance-seed: synthetic conformance corpus v1 — docA/docB/control content
  re-expressed as ClinicalCNL EN documents + ≥2 fresh scenario docs (interval +
  multi-exception coverage); ONE-command conformance runner wired (upstream + profile +
  mapping + queries + corpus round-trip); corpus locked (runner green = acceptance core).
- [ ] loop-framework: protocol validation — `.claude/commands/cnl-optimize.md` +
  `.agent/cnl-queue.md` stub landed at planning; this unit extends the queue from
  conformance-seed leftovers + generalization candidates, runs ONE manual end-to-end round
  (pick → land green commit `cnl-opt (R1)` → ledger entry), fixes protocol frictions found,
  confirms `/loop /cnl-optimize` drives it. Closes the milestone; user enables /loop
  (skillOverrides.loop:"on").

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
