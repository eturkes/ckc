# CKC fork provenance — `clinicalcnl/`

This tree is the CKC ClinicalCNL product-line host (SPEC §10.6). It vendors a fork of the
Attempto Parsing Engine (APE) plus an engine-source subset of AceRules, and holds CKC's own
clinical additions. Licensing evidence rows live in SPEC §11.5; this file is the per-repo
corresponding-source provenance record (GPL-3.0-or-later obligation). Vendored trees were
placed via `git archive` (upstream `.git` stripped); the manifests below fix the exact
snapshots.

## Three zones (fork-vs-vendored-vs-ours auditable)

- `clinicalcnl/` root = the **APE fork** — upstream layout preserved verbatim (diffability +
  GPL corresponding-source clarity). This is THE engine we build and patch.
- `clinicalcnl/vendor/acerules/` = the **AceRules adaptation-source subset** — the `engine/`
  DRS-to-rule mapping we adapt, plus upstream `LICENSE.txt` + `README.md`.
- `clinicalcnl/clinical/` = **CKC clinical additions ONLY** (profile checker, DRS-to-KB
  mapping, conflict queries, ulex, corpus, conformance runner) — populated by later M3 units;
  absent until then.

Upstream files are edited only to wire in CKC demands; each such edit is commented `% CKC:`.

## APE (fork root)

- Upstream: `github.com/Attempto/APE`
- Commit: `5f4d5354a45fb772763bf1a9543f508f15b28982` (= `master` HEAD, 2024-04-21)
- Tree: `ac239d2efef730fad7240acae502a5b3ffa86a23` (recorded pre-strip; 132 files)
- Vendored as-of: 2026-07-14
- Version: SWI-Prolog pack `6.7.180715` (`pack.pl`); release `6.7-180714` (`CHANGES.md`)
- Rights holders (per per-file source headers, manifest-wide sweep): Attempto Group /
  University of Zurich (© 2008-2013), Tobias Kuhn (© 2008-2012), and Kaarel Kaljurand
  (© 2008-2013 and 2008-2010; sole holder on the `prolog/utils/owlswrl/` subtree and some
  `prolog/utils/` + `tests/` files) — all under one operative grant.
- License: **LGPL-3.0-or-later** — per-file header grant ("GNU Lesser General Public License
  … either version 3 … or (at your option) any later version"), verified first-hand in
  `ape.pl` and the `prolog/utils/owlswrl/` headers; `LICENSE.txt` carries the LGPLv3 text.
  GitHub's `NOASSERTION` metadata is a non-authoritative auto-detector artifact, not the grant.
- What / why: APE is the ACE parser (raw ACE text → DRS). It is the whole product-line host —
  the CNL profile checker, DRS-to-KB mapper, and conflict queries all call APE's
  `get_ape_results/2,3` (module `ape`, `prolog/ape.pl`).

## AceRules (engine subset, `vendor/acerules/`)

- Upstream: `github.com/tkuhn/AceRules`
- Commit: `5b7afb7bdfbce56027997307f9b798af53551223` (= `master` HEAD, 2024-11-01)
- Tree: `1cebf98b450c6ed0dc88355beec266dd18270378` (recorded pre-strip; subset = 158 files)
- Vendored as-of: 2026-07-14
- Rights holder: Tobias Kuhn (© 2008-2012; sole holder across the engine subset).
- License: **LGPL-3.0-or-later** — per-file header grant, verified first-hand in
  `engine/acerules_processor.pl`; `LICENSE.txt` carries the LGPLv3 text.
- Vendored subset: `engine/` (the DRS-to-rule mapping CKC adapts) + `LICENSE.txt` + `README.md`.
- Excluded (⇒ no obligation): `dependencies/` (bundled GPL-2.0-or-later ASP solvers
  `lparse`/`smodels` — unneeded; CKC's Prolog `court` conflict queries replace ASP solving),
  `docker/` / `webapp/` / `webclient/` (deployment + UI, outside the engine subset), and the
  top-level `.gitignore` (repo metadata).
- What / why: only the AceRules DRS-to-rule engine is adaptation source for CKC's clinical
  KB mapping; the deployment, UI, and ASP-solver layers are not part of the CKC build.

## License obligations (met)

Both upstream projects grant LGPL-3.0-or-later per their per-file source headers. CKC retains
each upstream notice and `LICENSE.txt` unchanged; the corresponding source for each is its
vendored subtree above (APE at the root, AceRules at `vendor/acerules/`); this file records
the per-repo provenance. CKC's own additions and any `% CKC:`-marked edits to upstream files
convey under CKC's LICENSE (GPL-3.0-or-later), compatible with the LGPL-3.0-or-later inputs.
