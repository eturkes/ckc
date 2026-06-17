# M2-M4 translation PoC (throwaway, branch poc-m2-3-4)

Scope: SPEC M2-M4 on one throwaway branch. M2 (sec.9 short-hop pair) and M3
(sec.10 route trio) are built and measured below at revision 2; M4 (sec.11
invented-DSL routes) is the planned extension and is not yet in the matrix.

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
