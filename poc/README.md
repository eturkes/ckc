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
- Mechanism confirmed (the real signal): the GBNF couples var/op/value and pins the
  action/direction enums, so for every DSL route admission EQUALS syntactic validity
  -- the grammar closes the validity->admission gap the JSON-Schema IR routes leave
  wide open (single_ir 99.8% -> 98.2%; the rev-2 IR-stack routes saturate validity
  near 100% yet admit only 69.6-85.6%). Zero ai_schema_violation and zero
  false_positive_conflict for every DSL route. Absolute levels: dsl and dslkh
  100%/100%, dslh 99.6%/99.6%, dslk 95.8%/95.8% (truncation, below).
- Null result (first-class, sec.11): no DSL beats single_ir on the discriminating
  metric, greedy verdict accuracy (single_ir 98.0%; DSL 50/50/38/50%, i.e. -48 to
  -60 pts vs single_ir and -30 to -42 vs direct). Where DSL routes do top single_ir
  -- dsl and dslkh on validity/admission/stability, dslh on admission/stability --
  the stability lead is the stability of a WRONG answer. The grammar moved the
  bottleneck from syntax/admission to semantics, where the weak model fails.
- Conflict miss is a stable direction-polarity collapse: dsl, dslh, and dslkh each
  get all 5 null groups right and all 5 conflict groups wrong (exactly 25/50 = 50%),
  identically across all 5 sources and -- for dsl and dslkh -- all k=5 samples
  (stability 100%). On a conflict pair the model emits the same forbid|require token
  for both members (no_conflict, same_direction in 23-25 of 25 conflict cells), so
  z3 never sees an opposing-direction overlap. The grammar guarantees a well-formed
  direction token, not the right one.
- Compactness (T vs K), a second and mechanical failure: the verbose keyword block
  is truncation-prone -- under max_tokens=320 the weak model loops `AND age >= 0 AND
  age <= 130 ...` and the line is cut mid-condition (21 target_parse_error / 500 for
  ckc_dsl_kw vs 0 for ckc_dsl). Singular ckc_dsl_kw therefore drops to 95.8%/95.8%
  and 38% greedy: it misses all 25 conflicts AND loses 6 null cells to incomputable
  (member_inadmissible). So T dominates K at the singular level on every metric.
- Hop-layering neutralizes the truncation but not the semantics: in the hop chain
  the typed stage sees only the short prior DSL line, not the verbose sentence, so
  ckc_dsl_kw_hop recovers to 100%/100% and its 6 lost null cells return (38% ->
  50%), landing at the SAME 50% polarity collapse as the T hops (ckc_dsl_kw_hop >=
  ckc_dsl_hop on every metric). Conflict detection is never recovered.
- DSL landing vs JSON-IR landing on the final typing hop: the JSON-IR hop
  (ir_hop_chain 66.0% greedy) beats both DSL hops (50.0%). An invented-DSL landing
  does not beat the JSON-IR landing here.

## M4 acceptance -- claim 1 extended to invented forms (run id `matrix9`)

Claim 1 asks whether staged, grammar-constrained routes beat the direct one-leap
baseline on BOTH sec.7.3 metric families: route quality (schema-valid + admission +
k-sample convergence rates -- here syntactic validity, admission, stability) and
conflict quality (conflict-task accuracy -- here greedy verdict accuracy). Greedy is
the discriminating rate: the only one tracking conflict correctness over
well-formedness. Ranked over all 9 routes on the locked z3 evaluator by greedy, the
invented-DSL routes against the full sec.10 field (the measured pooled rows are the
two tables above; report.json carries the raw per-sample rows):

| rank | route | greedy | vs direct | vs single_ir |
| --- | --- | --- | --- | --- |
| 1 | single_ir | 98.0% | +18.0 | 0.0 |
| 2 | direct_smt | 80.0% | 0.0 | -18.0 |
| 3 | ir_hop_chain | 66.0% | -14.0 | -32.0 |
| 4 | stacked_ir | 58.0% | -22.0 | -40.0 |
| 5 | ckc_dsl (T) | 50.0% | -30.0 | -48.0 |
| 5 | ckc_dsl_hop (T) | 50.0% | -30.0 | -48.0 |
| 5 | ckc_dsl_kw_hop (K) | 50.0% | -30.0 | -48.0 |
| 8 | ckc_layered | 42.0% | -38.0 | -56.0 |
| 9 | ckc_dsl_kw (K) | 38.0% | -42.0 | -60.0 |

Verdict -- claim 1 does NOT extend to the invented DSL forms in full; a first-class
null (sec.11: no invented form beats the sec.10 field), the break localized by family:
- Route quality: every DSL route BEATS direct -- validity 95.8-100% vs 93.2%,
  admission 95.8-100% vs 93.2%, stability 68-100% vs 58%. The grammar-as-admission
  mechanism closes the validity->admission gap no JSON-IR route does (admission ==
  syntactic validity on all four; 0 ai_schema_violation vs 8-149 across the four
  JSON-IR routes; 0 false_positive_conflict). The claim's route-quality half holds.
- Conflict quality: every DSL route FALLS BELOW direct -- greedy 38-50% vs 80% (-30
  to -42 pts); the best invented forms (ckc_dsl, ckc_dsl_hop, ckc_dsl_kw_hop, tied at
  50.0%) sit 48 under single_ir, ahead of only ckc_layered and the truncating
  ckc_dsl_kw. Each DSL route misses all 25 conflict cells (false_negative_conflict =
  25/25), predominantly a same-direction polarity error (same_direction in 23-25 of
  25; ckc_dsl spends 2 on different_action). The grammar pins a well-formed direction
  token, not the right one. The claim's conflict-quality half breaks.

Claim 1 needs both families, so the invented forms fail it. The DSL leads over
single_ir are route-quality-only: ckc_dsl and ckc_dsl_kw_hop top it on validity,
admission, and stability (ckc_dsl_hop on admission and stability; ckc_dsl_kw on none),
but for the 100%-stability routes that stability splits into 25 stably-RIGHT null
cells and 25 stably-WRONG conflict cells -- the wrong half is exactly what sinks
conflict quality. single_ir stays the sole route winning conflict-task accuracy; the
mechanism win is real and orthogonal to it.

sec.11 acceptance themes met: two invented candidates (T terse, K keyword), each run
singular and layered (4 routes) over inputs locked byte-identical to matrix5 (the
2500 reused rev-2 records match byte-for-byte; dataset/model/z3 identities match in
report.json); ranked against the sec.10 field with the measured rows first; recorded
model I/O replays byte-stably (replay match, gold gate pass); sec.0 vocabulary holds
-- admission decides, the null is reported, nothing promoted. Comparison reproduced
cell-for-cell from `runs/matrix9/report.json` at acceptance (pooled metrics, both
baseline-delta sets, per-source, taxonomy).

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
