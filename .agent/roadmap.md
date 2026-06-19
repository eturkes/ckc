# CKC roadmap — branch poc-m2-3-4

Drives the throwaway M2-M4 translation PoC in `poc/` (Python, spec-deviating by
design). The production build plan — SPEC §2's real milestones (M2 = Rust
`exp.m2_shorthop`, …) — lives on `main`; this roadmap is throwaway, do not merge
it back. Format: one open milestone over an ordered unit checklist; unchecked lines
carry the full unit spec; checked items collapse to `- [x] <id>: <gist>.`;
closed milestones persist as bare headers; git history retains removed text. PoC
units cite `poc/DESIGN.md` (+ SPEC §9-§11) as contract and Python gates
(`python3 -m py_compile` + `run_m2.py run`/`score`/`replay`), not cargo.

Status: reopened for M5 -- show single_ir is insufficient and a more sophisticated
route is needed for good performance. m5-design is DONE: on the new `oblique`
surface family the conflict verdict sits at ceiling for both single_ir and
reason_ir (1.0, too coarse to separate them), but the new `exact_ir_match`
faithfulness metric does -- single_ir 0.70, reason_then_ir 0.90 (runs/oblique_demo,
k=1, all 10 groups). Open units m5-scale + m5-doc below. M1 and M2-M4 stay closed;
production milestones live on `main`.

## M1 spine — plan 89c4cba — accept m1 — review deb485f

## M2-M4 PoC — plan 3aac156 — accept m2-3-4-poc (codex-reviewed)

## M5 single_ir-insufficiency — oblique surface + faithfulness metric

- [x] m5-design: `oblique` source (6th surface; indirect renderings of the same
  20 gold rules), `reason_ir`/`repair_ir` routes (free-text reasoning stage before
  the constrained IR commit), `exact_ir_match` greedy faithfulness metric in
  METRIC_ORDER; score.py identity synced to live 7B/b9704. Demonstrated single_ir
  0.70 < reason_ir 0.90 on oblique exact_ir_match (verdict 1.0 for both).
- [ ] m5-scale: test `repair_ir` at k=1 on full oblique (draft->audit->commit;
  terse audit wired) -- does the 2nd sophisticated route also clear single_ir on
  exact_ir_match? Then widen to more/all sources. Iterate at k=1 (DEFAULT, greedy,
  all 20 items ~2-3min; exact_ir_match is greedy so k=1 suffices for the headline);
  take k=3 stability snapshots on a `--groups` subset only when needed (k=3 ~3x
  slower). Gate: `run_m2.py run --sources oblique --routes ir,reason_ir[,repair_ir]
  --k 1` then `score`; route exact_ir_match > single_ir. `direct` only needed for
  delta-vs-direct. Combine routes into one report by merging route-named records +
  server_props.json then `score` (no re-run).
- [ ] m5-doc: sync poc/DESIGN.md + README.md to rev-4 -- the `oblique` source, the
  `exact_ir_match` metric and the verdict-coarseness lesson, the `reason_ir`/
  `repair_ir` routes, and the single_ir-insufficiency finding. Docs still describe
  the closed 9-route verdict-only PoC. NOTE: the HTML report (poc/ui/index.html) was
  already rewritten this session into a single-screen 2-metric bar comparison
  (faithful logic vs conflict call) with de-jargoned human labels; doc text +
  screenshots must describe THAT, not the old 6-section/source-tab report.
