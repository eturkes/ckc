# CKC roadmap — branch poc-m2-3-4

Drives the throwaway M2-M4 translation PoC in `poc/` (Python, spec-deviating by
design). The production build plan — SPEC §2's real milestones (M2 = Rust
`exp.m2_shorthop`, …) — lives on `main`; this roadmap is throwaway, do not merge
it back. Format: one open milestone over an ordered unit checklist; unchecked lines
carry the full unit spec; checked items collapse to `- [x] <id>: <gist>.`;
closed milestones persist as bare headers; git history retains removed text. PoC
units cite `poc/DESIGN.md` (+ SPEC §9-§11) as contract and Python gates
(`python3 -m py_compile` + `run_m2.py run`/`score`/`replay`), not cargo.

## M1 spine — plan 89c4cba — accept m1 — review deb485f

## M2-M4 PoC — plan 3aac156

Contract: `poc/DESIGN.md` (implements SPEC §9-§11 in Python; throwaway). Built:
M2 §9 short-hop pair + M3 §10 route trio at rev-2, run `matrix5` (28de59e
short-hop → accc7b5 widen → 1c8d276 results); M4 §11 invented-DSL field at rev-3,
run `matrix9` (4500 records / 9000 calls, replay match). Remaining: poc-accept --
rank the DSL field against single_ir/direct on the locked z3 evaluator and tag the
PoC.

- [x] dsl-design: DESIGN.md rev-3 — §11 DSL routes, two compactness candidates
  (T terse / K keyword), each singular + 3-call hop; GBNF builders, DSL parsers,
  dual baseline (vs direct + single_ir), matrix slotting; dsl-impl consolidated
  to one unit at 1M.
- [x] dsl-impl: four §11 DSL routes (dsl/dslk singular, dslh/dslkh 3-call hop)
  on a GBNF mask, parsed to the shared rev-2 IR -> compile_ir. New
  `m2/grammars.py` (terse+kw+surface+ground builders, dsl_stage_grammars,
  grammar_shas); `routes.py` DSL parsers + route_stages grammar slot + DSL
  prompts; `admit.py` admit_dsl/_admit_dsl_hop dispatch; `llm.py` chat grammar
  field; `run_m2.py` validates --routes vs route_stages + threads grammar;
  `score.py` 9-route ROUTE_KEYS/IDS + delta_ir (vs single_ir) + grammar_sha256
  + EXPERIMENT_ID exp.m2poc_dsl. rev-2 routes byte-identical (prompt/schema sha
  match matrix5). Gate green: py_compile + smoke_dslc (singular) + smoke_dslh
  (hop), each run+score+replay (replay byte-match).
- [x] dsl-run: ran the 9-route field (run `matrix9`, exp.m2poc_dsl; seeded
  matrix5's 2500 rev-2 records + 2000 fresh DSL = 4500 records / 9000 calls; score
  + replay byte-match). report.py vs-single_ir delta table + grammar_sha256
  display + 9-route widening (title de-staled); ui/ dual-baseline banner (best-DSL
  vs single_ir) + per-DSL delta_ir chips; README M4 result block (M2-M3 figures
  kept). Result: the GBNF mask makes admission == syntactic-validity for every DSL route
  (closes the validity->admission gap the JSON-Schema IR routes leave open;
  mechanism confirmed) yet no DSL beats single_ir on greedy verdict accuracy.
  dsl/dslh/dslkh collapse to 50% via a stable forbid/require polarity error on
  conflict pairs (no_conflict/same_direction); dslk drops to 38% (keyword verbosity
  truncates at max_tokens, losing all conflicts AND 6 null cells). T dominates K at
  the singular level; hop neutralizes K's truncation (dslkh back to 50%) but not the
  polarity collapse. First-class null per SPEC §11.
- [ ] poc-accept: rank the DSL route(s) against the existing five-route field on
  the locked z3 evaluator — the bar is `route.single_ir` (the rev-2 winner on
  all four metrics) and `route.direct_smt` (the M2 baseline); claim 1 extended
  to invented forms. Fold the readings into `poc/README.md`; tag
  `accept/m2-3-4-poc`. Reading: SPEC §11; the dsl-run report. Gate: comparison
  reproduced from the scored report.
