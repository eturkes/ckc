# M2 short-hop translation PoC (throwaway, branch poc-m2-oneshot)

One-shot PoC of SPEC section 9: weak local model (Qwen2.5-1.5B-Instruct Q4_K_M, llama.cpp
b9601, CPU) translates 20 synthetic Japanese clinical rules over two routes;
real z3 4.13.3 scores both against gold. Contract: `DESIGN.md`. Stack deviates
from the spec by design: Python stdlib + llama-server HTTP + z3 subprocess.

## Recorded result (run id `m2`, 160 live calls, replay match)

| metric | route.direct_smt | route.single_ir | delta |
| --- | --- | --- | --- |
| syntactic validity | 92.5% (74/80) | 100% (80/80) | +7.5 |
| admission rate | 92.5% (74/80) | 96.3% (77/80) | +3.8 |
| verdict accuracy (greedy) | 80% (8/10) | 90% (9/10) | +10.0 |
| verdict stability (k=4) | 60% (6/10) | 80% (8/10) | +20.0 |

## Use

```bash
poc/setup-note: vendor/ (llama.cpp b9601 + gguf) and runs/ are gitignored; re-fetch
  per DESIGN.md "Fixed identities" if absent, then: python3 poc/run_m2.py setup-check
python3 poc/run_m2.py run --run-id <id> [--groups g01,g02] [--k 4]   # live, records all I/O
python3 poc/run_m2.py score --run-id <id>                            # records -> reports
python3 poc/run_m2.py replay --run-id <id>                           # byte-stability gate
python3 -m http.server 8076 -d poc                                   # then /ui/?run=<id>&lang=ja
```

UI: EN|JA toggle (or `?lang=`), reads `runs/latest/report.json` by default.
Reports: `runs/<id>/report.{json,md,ja.md}` - one canonical JSON, two renderings.

Cleanup: delete the branch and `poc/` (vendor + runs go with it).
