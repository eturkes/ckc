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

## M2-M4 PoC — plan 3aac156

Contract: `poc/DESIGN.md` (implements SPEC §9-§11 in Python; throwaway). Built:
M2 §9 short-hop pair + M3 §10 route trio at rev-2, run `matrix5` (28de59e
short-hop → accc7b5 widen → 1c8d276 results). Remaining: M4 §11 invented-DSL
routes, ranked against the existing five-route field on the locked z3 evaluator.

- [x] dsl-design: DESIGN.md rev-3 — §11 DSL routes, two compactness candidates
  (T terse / K keyword), each singular + 3-call hop; GBNF builders, DSL parsers,
  dual baseline (vs direct + single_ir), matrix slotting; dsl-impl consolidated
  to one unit at 1M. 20% 205K/1M _
- [ ] dsl-impl: implement the four §11 DSL routes per the rev-3 contract
  ("Revision 3" section) in two internally-gated passes — singular (dsl, dslk)
  then hop (dslh, dslkh): `m2/grammars.py` (GBNF builders + grammar_shas), the
  DSL parsers, chat grammar field, run threading, admit dispatch, and score
  ROUTE_KEYS/ROUTE_IDS + dual baseline (`m2/routes.py`, `m2/grammars.py`,
  `m2/admit.py`, `m2/llm.py`, `m2/run_m2.py`, `m2/score.py`). Reading: rev-3
  `DESIGN.md`; `git show HEAD` route/admit/score patterns; existing `single_ir`
  + `ir_hop_chain` routes. Gate: `python3 -m py_compile` on changed modules +
  the rev-3 smoke recipes (`smoke_dslc` singular, `smoke_dslh` hop), each
  `run`+`score`+`replay` green.
- [ ] dsl-run: run the 9-route field (`exp.m2poc_dsl`) over 5 sources × k5 (MAY
  reuse matrix5's five-route records); `score` → `report.{json,md,ja.md}`;
  `replay` byte-stability; add to `m2/report.py` the vs-single_ir delta table +
  9-route widening and to `ui/` the 9 columns + dual-baseline banner; add the DSL
  rows/columns to `poc/README.md`, keeping the M2-M3 figures. Reading: rev-3
  `DESIGN.md`; `m2/report.py`, `ui/`. Gate: `run_m2.py run`/`score`/`replay`
  green (replay byte-match).
- [ ] poc-accept: rank the DSL route(s) against the existing five-route field on
  the locked z3 evaluator — the bar is `route.single_ir` (the rev-2 winner on
  all four metrics) and `route.direct_smt` (the M2 baseline); claim 1 extended
  to invented forms. Fold the readings into `poc/README.md`; tag
  `accept/m2-3-4-poc`. Reading: SPEC §11; the dsl-run report. Gate: comparison
  reproduced from the scored report.
