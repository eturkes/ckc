# M2-M4 translation PoC (throwaway, branch poc-m2-3-4)

Scope: SPEC M2-M4 on one throwaway branch. M2 (sec.9 short-hop pair) and M3
(sec.10 route trio) are built and measured at revision 2 (run `matrix5`); M4
(sec.11 invented-DSL routes) widens the field to 9 routes at revision 3 (run
`matrix9`). Both fields are measured below.

Revision 2 widens the SPEC sec.9 PoC to a 5 x 5 x 5 matrix: 5 routes (sec.9 pair
+ sec.10 trio at PoC scale), 5 surface-style source families re-expressing the
same 20 synthetic Japanese rules and 10 gold groups, k=5 samples. Weak local model
(Qwen2.5-1.5B-Instruct Q4_K_M, llama.cpp b9601, CPU) translates; real z3 4.13.3
scores every route against shared gold. Contract: `DESIGN.md`. Stack deviates from
the spec by design: Python stdlib + llama-server HTTP + z3 subprocess.

## Recorded result — M2-M3 route field (run id `matrix5`, 2500 records / 5000 live calls, replay match)

Pooled over 5 sources (100 outputs per route cell, 50 group cells):

| metric | direct_smt | single_ir | stacked_ir | ir_hop_chain | ckc_layered |
| --- | --- | --- | --- | --- | --- |
| syntactic validity | 93.2% | 99.8% | 100% | 100% | 99.4% |
| admission rate | 93.2% | 98.2% | 76.4% | 85.6% | 69.6% |
| verdict accuracy (greedy) | 80.0% | 98.0% | 58.0% | 66.0% | 42.0% |
| verdict stability (k=5) | 58.0% | 78.0% | 14.0% | 32.0% | 20.0% |
| mean ms / sample | 1243 | 1537 | 2470 | 4334 | 4148 |

Greedy verdict accuracy by source family (n/10 groups):

| source | direct | ir | stacked | hop | layered |
| --- | --- | --- | --- | --- | --- |
| directive | 8 | 10 | 5 | 5 | 6 |
| package_insert | 8 | 10 | 7 | 8 | 6 |
| table_row | 8 | 9 | 7 | 8 | 3 |
| verbose_prose | 8 | 10 | 5 | 7 | 2 |
| metamorphic | 8 | 10 | 5 | 5 | 4 |

Readings (measured rates on synthetic fixtures; no clinical claims):
- route.single_ir beats route.direct_smt on every metric and every source
  (pooled deltas +6.6 validity, +5.0 admission, +18.0 greedy, +20.0 stability).
- More hops hurt at this model scale: stacked/hop/layered sit BELOW direct on
  verdict metrics. Grammar saturates syntactic validity for every IR route; the
  discriminating gate is admission-time type coherence (var/op/value coupling the
  JSON grammar cannot express), and every multi-stage failure lands at the final
  typing stage (rows/typed/rule; intermediate stages never fail). One constrained
  hop with the sentence and full vocabulary in view is the reliability sweet spot.
- Source robustness: single_ir holds 9-10/10 on all five families; direct is flat
  at 8/10; the multi-stage routes swing 2-8/10 (verbose_prose worst for layered).
- Multi-stage routes also compound sampling noise: 2-3 chances per sample per
  member to emit one incoherent condition, hence the stability collapse.

## Recorded result — M4 DSL route field (run id `matrix9`, 4500 records / 9000 calls, replay match)

Revision 3 widens the field to 9 routes: the five above plus four invented-DSL
routes under a hand-written GBNF mask, parsed deterministically into the SAME IR
the rev-2 routes share, then compiled to z3. Two compactness candidates -- T
(terse infix, `forbid drug_aspirin when pregnant=true`) and K (verbose keyword
block, `RULE ... / GUARD ...`) -- each in a singular and a 3-call hop config.
matrix9 reuses matrix5's 2500 rev-2 records (byte-identical inputs) and adds 2000
fresh DSL records (4000 DSL calls). Bars: route.single_ir (rev-2 winner) and
route.direct_smt (M2 baseline).

Pooled (100 outputs per route cell, 50 group cells); deltas in points:

| metric | direct_smt | single_ir | ckc_dsl (T) | ckc_dsl_hop (T) | ckc_dsl_kw (K) | ckc_dsl_kw_hop (K) |
| --- | --- | --- | --- | --- | --- | --- |
| syntactic validity | 93.2% | 99.8% | 100% | 99.6% | 95.8% | 100% |
| admission rate | 93.2% | 98.2% | 100% | 99.6% | 95.8% | 100% |
| verdict accuracy (greedy) | 80.0% | 98.0% | 50.0% | 50.0% | 38.0% | 50.0% |
| verdict stability (k=5) | 58.0% | 78.0% | 100% | 96.0% | 68.0% | 100% |
| greedy delta vs single_ir | -18.0 | 0.0 | -48.0 | -48.0 | -60.0 | -48.0 |
| greedy delta vs direct | 0.0 | +18.0 | -30.0 | -30.0 | -42.0 | -30.0 |

Greedy verdict accuracy by source family (n/10 groups), DSL routes:

| source | ckc_dsl | ckc_dsl_hop | ckc_dsl_kw | ckc_dsl_kw_hop |
| --- | --- | --- | --- | --- |
| directive | 5 | 5 | 4 | 5 |
| package_insert | 5 | 5 | 4 | 5 |
| table_row | 5 | 5 | 5 | 5 |
| verbose_prose | 5 | 5 | 3 | 5 |
| metamorphic | 5 | 5 | 3 | 5 |

Readings (measured rates on synthetic fixtures; no clinical claims):
- Mechanism confirmed: the GBNF couples var/op/value and pins the action/direction
  enums, so the DSL routes saturate BOTH syntactic validity and admission at ~100%
  (T-singular and K-hop hit 100%/100%) -- closing the admission gap the JSON-Schema
  IR routes leave open (single_ir 98.2%, rev-2 IR-stack routes 69.6-85.6%). Zero
  ai_schema_violation and zero false_positive_conflict for every DSL route.
- Null result (first-class, sec.11): no invented DSL beats single_ir; all four sit
  far below it (greedy -48.0 to -60.0 vs single_ir, -30.0 to -42.0 vs direct). The
  grammar moved the bottleneck from syntax/admission to semantics, where the weak
  model fails.
- The failure is systematic and stable, not noisy: every DSL route gets all 5 null
  groups right and all 5 conflict groups wrong (exactly 5/10), identically across
  all 5 sources and -- for T-singular and K-hop -- all k=5 samples (stability
  100%). Cause is a direction-polarity collapse: on a conflict pair the model emits
  the same forbid|require token for both members (no_conflict, same_direction in
  23-25 of 25 cells), so z3 never sees an opposing-direction overlap. The grammar
  guarantees a well-formed direction token, not the right one.
- Compactness (T vs K): T dominates. ckc_dsl_kw is truncation-prone -- under
  max_tokens=320 the weak model loops `AND age >= 0 AND age <= 130 ...` and the
  line is cut mid-condition (21 target_parse_error / 500 vs 0 for ckc_dsl), costing
  validity, admission, accuracy, and stability. Hop-layering rescues K's robustness
  (ckc_dsl_kw_hop back to 100%/100%, since the typed stage sees only the short
  prior DSL line, not the verbose sentence) but not its accuracy.
- DSL landing vs JSON-IR landing on the final typing hop: the JSON-IR hop
  (ir_hop_chain 66.0% greedy) beats both DSL hops (50.0%). An invented-DSL landing
  does not beat the JSON-IR landing here.

## Use

```bash
poc/setup-note: vendor/ (llama.cpp b9601 + gguf) and runs/ are gitignored; re-fetch
  per DESIGN.md "Fixed identities" if absent, then: python3 poc/run_m2.py setup-check
python3 poc/run_m2.py run --run-id <id> [--groups g01,..] [--sources directive,..] [--routes direct,ir,..] [--k 5]
python3 poc/run_m2.py score --run-id <id>      # records -> report.{json,md,ja.md}
python3 poc/run_m2.py replay --run-id <id>     # byte-stability gate
python3 -m http.server 8076 -d poc             # then /ui/?run=<id>&lang=ja
```

Smoke recipe: `run --run-id smoke2 --groups g01 --sources directive,table --routes all --k 1`.
UI: scope tabs (pooled + per source), EN|JA toggle, reads `runs/latest/report.json`.
Reports: one canonical JSON, two markdown renderings.

Cleanup: delete the branch and `poc/` (vendor + runs go with it).
