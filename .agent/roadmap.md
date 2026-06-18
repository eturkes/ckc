# CKC roadmap — branch poc-m2-3-4

Drives the throwaway M2-M4 translation PoC in `poc/` (Python, spec-deviating by
design). The production build plan — SPEC §2's real milestones (M2 = Rust
`exp.m2_shorthop`, …) — lives on `main`; this roadmap is throwaway, do not merge
it back. Format: one open milestone over an ordered unit checklist; unchecked lines
carry the full unit spec; checked items collapse to `- [x] <id>: <gist>.`;
closed milestones persist as bare headers; git history retains removed text. PoC
units cite `poc/DESIGN.md` (+ SPEC §9-§11) as contract and Python gates
(`python3 -m py_compile` + `run_m2.py run`/`score`/`replay`), not cargo.

Status: both milestones closed; the throwaway PoC is complete (tag
`accept/m2-3-4-poc`), no open units. Retire by deleting `poc/` + the branch
(`poc/README.md` "Cleanup"). A fresh `/session-prompt` with an empty task has
nothing to take here; production milestones live on `main`.

## M1 spine — plan 89c4cba — accept m1 — review deb485f

## M2-M4 PoC — plan 3aac156 — accept m2-3-4-poc (codex-reviewed)
