# CKC roadmap — branch poc-m2-3-4

Drives the throwaway M2-M4 translation PoC in `poc/` (Python, spec-deviating by
design). The production build plan — SPEC §2's real milestones (M2 = Rust
`exp.m2_shorthop`, …) — lives on `main`; this roadmap is throwaway, do not merge
it back. Format and stamp semantics are unchanged (see /session-prompt): one open
milestone over an ordered unit checklist; unchecked lines carry the full unit
spec; checked items collapse to `- [x] <id>: <gist>. NN% NNNK/200K <hash>`;
closed milestones persist as bare headers; git history retains removed text. PoC
units cite `poc/DESIGN.md` (+ SPEC §9-§11) as contract and Python gates
(`python3 -m py_compile` + `run_m2.py run`/`score`/`replay`), not cargo.

## M1 spine — plan 89c4cba — accept m1 — review deb485f

## M2-M4 PoC — plan _

Contract: `poc/DESIGN.md` (implements SPEC §9-§11 in Python; throwaway). Built:
M2 §9 short-hop pair + M3 §10 route trio at rev-2, run `matrix5` (28de59e
short-hop → accc7b5 widen → 1c8d276 results). Remaining: M4 §11 invented-DSL
routes, ranked against the existing five-route field on the locked z3 evaluator.

- [ ] dsl-design: revise `poc/DESIGN.md` to revision 3 — specify the §11
  invented-DSL route shapes (grammar-masked concrete syntax; deterministic
  parse → IR → compile; singular + layered configs), their fixed identities
  (route key / full id / stages / calls-per-sample), the GBNF grammar surface,
  and how they slot into the matrix beside the existing five-route field. May
  refine the units below (e.g. split dsl-impl into singular/layered). Reading:
  SPEC §11 (read in full), §9-§10; `poc/DESIGN.md` rev-2; `poc/m2/routes.py`.
  Gate: ASCII + vocabulary self-check, internal consistency; present the rev-3
  contract for user sign-off before dsl-impl. No code.
- [ ] dsl-impl: implement the §11 DSL route(s) per the rev-3 contract — GBNF
  grammar, deterministic parser (DSL → IR), IR → SMT compile, and route /
  admit / score wiring (`m2/routes.py`, `m2/admit.py`, `m2/score.py`, grammar
  asset). Reading: rev-3 `DESIGN.md`; `git show HEAD` route/admit patterns;
  existing `single_ir` route. Gate: `python3 -m py_compile` on changed modules
  + a k=1 smoke `run`+`score` on one DSL route (one group, one source).
- [ ] dsl-run: full matrix including the DSL route(s) over 5 sources × k5;
  `score` → `report.{json,md,ja.md}`; `replay` byte-stability; add the DSL
  rows/columns to `poc/README.md` results, keeping the M2-M3 figures. Reading:
  rev-3 `DESIGN.md`; `m2/report.py`, `ui/`. Gate: `run_m2.py run`/`score`/
  `replay` green (replay byte-match).
- [ ] poc-accept: rank the DSL route(s) against the existing five-route field on
  the locked z3 evaluator — the bar is `route.single_ir` (the rev-2 winner on
  all four metrics) and `route.direct_smt` (the M2 baseline); claim 1 extended
  to invented forms. Fold the readings into `poc/README.md`; tag
  `accept/m2-3-4-poc`. Reading: SPEC §11; the dsl-run report. Gate: comparison
  reproduced from the scored report.
