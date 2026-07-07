# Archived PoC — measured route/DSL priors (throwaway harness, 2026-06)

Provenance: `~/Projects/scratch/ckc-archive` branch `poc-m2-3-4` (`poc/DESIGN.md` = contract,
`poc/README.md` = recorded results, `.agent/memory.md` M5 section = distilled lessons,
`poc/runs/` on disk holds `report.json` per run). `ckc-archive2` = the two earlier spec
lineages (branches `archive/spec01..03`; original charters `SPEC_A/B/C.md`) — no CNL/DSL
content beyond what the current SPEC already distilled. This file carries the PoC's
inspiration-bearing measurements into the committed compendium; §10/§11 cite it as PRIORS —
a throwaway spec-deviating harness (Python stdlib + llama-server HTTP + z3 subprocess), NOT
locked measurements of this pipeline.

Harness: 20 synthetic JA rules (require/forbid drug under conjunctive conditions), 10 gold
conflict/no-conflict pairs, 5 surface families re-expressing the same rules (directive /
package-insert / table-row / verbose-prose / metamorphic), k=5 samples (s0 greedy + 4 sampled),
weak local model (Qwen2.5-1.5B Q4_K_M, llama.cpp GBNF/JSON-Schema constrained; M5 phase 7B),
z3 4.13.3 scores every route against shared gold. Metrics: syntactic validity, admission rate
(structural type coherence), greedy verdict accuracy, k=5 verdict stability.

## matrix5 — 5-route field (2500 records, replay match)

| metric | direct_smt | single_ir | stacked_ir | ir_hop_chain | ckc_layered |
| --- | --- | --- | --- | --- | --- |
| syntactic validity | 93.2 | 99.8 | 100 | 100 | 99.4 |
| admission | 93.2 | 98.2 | 76.4 | 85.6 | 69.6 |
| greedy verdict accuracy | 80.0 | 98.0 | 58.0 | 66.0 | 42.0 |
| verdict stability k=5 | 58.0 | 78.0 | 14.0 | 32.0 | 20.0 |

- single_ir beats direct on EVERY metric and every source family (+18.0 greedy pooled); basis
  of the production §9 minimal pair.
- Constrained-hop stacking HURTS at weak-model scale: stacked/hop/layered all sit below direct
  on verdict metrics; every multi-stage failure lands at the final typing stage (intermediate
  stages never fail); multi-stage compounds sampling noise (stability collapse to 14–32).
  One constrained hop with the sentence + full vocabulary in view = the reliability sweet spot.
- JSON-Schema constraint saturates syntactic validity but CANNOT couple var/op/value → the
  discriminating gate is admission-time type coherence; incoherent conditions leak through.

## matrix9 — invented-DSL field (9 routes; +4 grammar-masked DSL routes)

Two invented record-DSL candidates under a hand-written GBNF mask, each singular + 3-hop:
T terse infix (`forbid drug_aspirin when pregnant=true`) vs K verbose keyword block
(`RULE …\nGUARD … AND …`) — the token-compactness axis. Both parse deterministically into the
same IR dict; grammar couples var/op/value and pins enums.

| metric | direct | single_ir | dsl (T) | dsl_hop (T) | dsl_kw (K) | dsl_kw_hop (K) |
| --- | --- | --- | --- | --- | --- | --- |
| validity | 93.2 | 99.8 | 100 | 99.6 | 95.8 | 100 |
| admission | 93.2 | 98.2 | 100 | 99.6 | 95.8 | 100 |
| greedy verdict accuracy | 80.0 | 98.0 | 50.0 | 50.0 | 38.0 | 50.0 |
| stability k=5 | 58.0 | 78.0 | 100 | 96.0 | 68.0 | 100 |

- MECHANISM WIN: grammar-as-admission closes the validity→admission gap entirely (admission ==
  validity, zero schema violations, zero false-positive conflicts) — the gap every JSON-Schema
  route leaves open.
- DIRECTION-POLARITY COLLAPSE (the conflict-killer): every DSL route missed ALL 25 conflict
  cells — the model emits the SAME `forbid|require` token for both members of a conflict pair
  (same-direction in 23–25/25), stably across sources and samples (T routes: 100% stability =
  25 stably-right null cells + 25 stably-WRONG conflict cells). A grammar guarantees a
  well-formed direction token, never the right one; stability without faithfulness is the
  stability of a wrong answer. Abstract ASCII direction tokens carried none of the source's
  deontic register.
- TRUNCATION AT REPETITION POINTS: the verbose K form degenerately loops at the grammar's
  unbounded repetition point (`AND age >= 0 AND age <= 130 …`) and truncates mid-condition
  under the token budget (21/500 parse errors; T: 0). Terse dominates verbose singular-level on
  every metric. Hop-layering neutralizes truncation (later hops see only the short prior line,
  not the verbose sentence) but never recovers the semantics.
- LANDING DIALECT: the JSON-IR hop chain (66.0 greedy) beats both DSL hop chains (50.0) — an
  invented-DSL landing did not beat the JSON-IR landing on the final typing hop.
- Verdict: first-class null — no invented form beat single_ir; the claim-1 test splits per
  §7.3 family (route quality: every DSL beats direct; conflict quality: every DSL falls below
  direct). Assess any invented form against BOTH families before a beats/does-not-beat claim.

## M5 oblique phase — verdict coarseness + faithfulness + reasoning room (7B model)

- `oblique` 6th source family: same 20 gold rules rendered INDIRECTLY — drug synonyms
  (アセチルサリチル酸/MTX), oblique deontic phrasing (推奨されない→forbid, 適応→require), age
  conventions (高齢者=65, 後期高齢者=75, 成人=18), negated phrasing (非妊娠=false). Semantic
  indirection, a DIFFERENT axis from surface-metamorphic transforms (punctuation/width folds),
  and the one that actually dents faithfulness.
- VERDICT COARSENESS: conflict verdicts turn on drug + direction + overlap-satisfiability only
  → tolerate wrong thresholds/conventions that preserve overlap. On oblique, single_ir AND the
  reasoning route both scored verdict 1.0 while exact-IR faithfulness split them 0.70 vs 0.90
  (direct 0.20). `exact_ir_match` = action + direction + order-independent condition-set vs
  compiled gold, greedy — the metric that separates routes after verdicts saturate.
- REASONING ROOM: `reason_ir` (unconstrained free-text reasoning stage → constrained IR commit;
  a schema/grammar-free stage is the reasoning room single-shot constrained emission lacks)
  recovered hinted-convention + dropped-negation errors (0.70→0.90 faithfulness) but NOT
  genuine model knowledge gaps (後期高齢者 stayed 65), and introduced one new bound misread
  (70歳を超えない → <70). Constraint PLACEMENT is a route axis distinct from hop count:
  constrained-hop stacking hurt (matrix5) while a free stage before ONE constrained commit
  helped. Cost: sampling variance on the free stage — self-consistency 0.75 vs single_ir 0.88
  at k=3 (candidate fix: greedy reason stage). `repair_ir` (draft → free audit → commit) wired
  but never measured; PoC closed mid-M5 (open units m5-scale/m5-doc in the archive roadmap).
- Convention terms are a model KNOWLEDGE hazard → keep convention semantics in committed
  lexicon data (concept surface atomic, interval semantics lexicon-carried), never re-derived
  by the model — validates the production lexicon design.
