# CKC fork provenance — `clinicalcnl/`

This tree is the CKC ClinicalCNL product-line host (SPEC §10.6). It vendors a fork of the
Attempto Parsing Engine (APE), an engine-source subset of AceRules, and the full Clex lexicon,
and holds CKC's own clinical additions. Licensing evidence rows live in SPEC §11.5; this file is
the per-repo corresponding-source provenance record — a voluntary compliance aid (no GPL/LGPL
provision mandates it) that fixes the exact snapshots. Vendored trees were placed via
`git archive` (upstream `.git` stripped); each is byte-identical to upstream at its pin.

## Four zones (fork-vs-vendored-vs-ours auditable)

- `clinicalcnl/` root = the **APE fork** — upstream layout preserved verbatim (diffability +
  GPL corresponding-source clarity). THE engine we build and patch.
- `clinicalcnl/vendor/acerules/` = the **AceRules engine subset** — the `engine/` DRS-to-rule
  mapping we adapt, plus upstream `LICENSE.txt` + `README.md`.
- `clinicalcnl/vendor/clex/` = the **full Clex lexicon** (~97.5K entries) — APE's large drop-in
  lexicon; ape-build wires it over APE's reduced one.
- `clinicalcnl/clinical/` = **CKC clinical additions ONLY** (profile checker, DRS-to-KB mapping,
  conflict queries, ulex, corpus, conformance runner) — populated by later M3 units; absent until then.

Upstream files are edited only to wire in CKC demands; each such edit carries a `CKC:` marker in
the file's native comment syntax + date + one-line reason (GPL §5 modification notice). No CKC
edits exist yet — the vendored trees are pristine upstream snapshots.

## APE (fork root)

- Upstream: `https://github.com/Attempto/APE`
- Commit: `5f4d5354a45fb772763bf1a9543f508f15b28982` (= `master` HEAD observed 2026-07-14; 2024-04-21)
- Tree: `ac239d2efef730fad7240acae502a5b3ffa86a23` = the commit's root tree (whole repo, 132 files)
- Vendored as-of: 2026-07-14
- Version: SWI-Prolog pack `6.7.180715` (`pack.pl`); release `6.7-180714` (`CHANGES.md`)
- © notices of record (per-file source headers, manifest-wide sweep): Attempto Group /
  University of Zurich (© 2008-2013) and Kaarel Kaljurand (© 2008-2013 and 2008-2010, named on
  the `prolog/utils/owlswrl/` subtree — except `transform_anonymous.pl` there, which is
  Attempto's). Tobias Kuhn and Kaljurand appear as `@author`s across the tree, but no APE ©
  notice names Kuhn — the copyright holders are Attempto/UZH + Kaljurand.
- License: **LGPL-3.0-or-later** — project-level via `LICENSE.txt` (LGPLv3) + per-file header
  grants ("... version 3 ... or (at your option) any later version"), verified first-hand in
  `ape.pl` and the `prolog/utils/owlswrl/` headers. 82 of 132 files carry the explicit per-file
  grant; the remaining data/grammar/fixture files ride the project license. GitHub's
  `NOASSERTION` metadata is a non-authoritative auto-detector artifact, not the grant.
- Bundled third-party sub-content (redistributed under APE's grant with the whole-repo vendoring):
  - `prolog/lexicon/clex_lexicon.pl` — a REDUCED Clex (2011 entries, Attempto/UZH © under APE's
    LGPL), loaded by default (`prolog/lexicon/clex.pl`). ape-build replaces it drop-in with the
    full Clex (see the Clex zone).
  - `tests/acetexts.pl` — APE's user-submitted regression corpus (3779 `text_drs_eval` records).
    Its `Author` field records the submitter's IP; ~2558 IPv4 addresses are present. This is
    verbatim public upstream data, redistributed unchanged; ape-build's regression suite consumes it.
  - `examples/the_lol_policy.ace.txt` — Kaljurand's ACE rendering of if-then sentences from
    J.L. De Coi's "Further Notes. Draft December 19, 2007", attributed in-file.
- What / why: APE is the ACE parser (raw ACE text → DRS) and the whole product-line host — the
  CNL profile checker, DRS-to-KB mapper, and conflict queries all call `get_ape_results/2,3`
  (module `ape`, `prolog/ape.pl`).

## AceRules (engine subset, `vendor/acerules/`)

- Upstream: `https://github.com/tkuhn/AceRules`
- Commit: `5b7afb7bdfbce56027997307f9b798af53551223` (= `master` HEAD observed 2026-07-14; 2024-11-01)
- Tree: `1cebf98b450c6ed0dc88355beec266dd18270378` = the commit's FULL root tree (274 files).
  Vendored SELECTION = 158 files: `engine/` (156) + `LICENSE.txt` + `README.md`.
- Vendored as-of: 2026-07-14
- © notice: Tobias Kuhn (© 2008-2012) across the engine `.pl`. Sole ownership of every selected
  byte is not independently established — `engine/webservice/acerules_server.pl` is a copy of the
  APE server (names Kaljurand), and `engine/court_interpreter/transform_naf.pl` names Marc
  Doerflinger as author.
- License: **LGPL-3.0-or-later** — project-level via `LICENSE.txt` (LGPLv3) + per-file grants,
  verified first-hand in `engine/acerules_processor.pl`. 45 of 46 engine `.pl` carry the explicit
  grant (`engine/parameters.pl` excepted).
- Vendored subset detail: `engine/` is taken WHOLESALE for upstream diffability. It carries the
  parser/court DRS-to-rule mapping CKC adapts, plus `webservice/` (SOAP server),
  `stable_interpreter/` (ASP adapters that shell out to the excluded solvers), and `testcases/` —
  CKC consumes only the parser/court mapping.
- Excluded (⇒ no conveyance obligation): `dependencies/` (bundled GPL-2.0-or-later ASP solvers
  `lparse`/`smodels` — unneeded; CKC's Prolog `court` conflict queries replace ASP solving),
  `docker/` / `webapp/` / `webclient/` (deployment + UI), the top-level `.gitignore`.
- What / why: the AceRules DRS-to-rule engine is adaptation source for CKC's clinical KB mapping.

## Clex (full lexicon, `vendor/clex/`)

- Upstream: `https://github.com/Attempto/Clex`
- Commit: `20960a5ce07776cb211a8cfb25dc8c81fcdf25e2` (= `master` HEAD observed 2026-07-14)
- Tree: `210d7ea09671309e2c39fb4fd87fc609d9fff1e7` = the commit's root tree (whole repo, 3 files)
- Vendored as-of: 2026-07-14
- Holder: Attempto Group / University of Zurich (© 2008-2013), "Derived from: COMLEX, Copyright
  2005 LDC, University of Pennsylvania" (COMLEX = NYU Proteus under LDC, catalog LDC98L21).
- License: **GPL-3.0-or-later** — verified first-hand in the `clex_lexicon.pl` header + `LICENSE`
  (GPLv3); GPL-3.0-or-later == CKC's own LICENSE. Derivation authority: Attempto publishes Clex
  under GPL with explicit COMLEX/LDC attribution; COMLEX (LDC98L21) permits research + commercial
  use under minimal LDC-member restrictions — a distinct framework, recorded transparently; CKC's
  use rides Attempto's published grant.
- Vendored: whole repo — `clex_lexicon.pl` (~97.5K entries) + `LICENSE` + `README.md`.
- What / why: the large ACE English lexicon. Per Clex's README it is the intended drop-in
  replacement for APE's reduced `prolog/lexicon/clex_lexicon.pl`; ape-build copies it in and
  recompiles, giving APE/CKC full English vocabulary + the in-tree (reproducible) lexicon the
  upstream regression suite needs — superseding APE's live-download `ensure_clex`.

## License obligations (met, per reuse mode)

APE + AceRules grant LGPL-3.0-or-later; Clex grants GPL-3.0-or-later. For the current reuse mode
— **pristine-source redistribution** — obligations are met: every upstream notice + `LICENSE(.txt)`
is retained unchanged, and the corresponding source for each is its vendored subtree (APE at the
root, AceRules at `vendor/acerules/`, Clex at `vendor/clex/`). Modified-source, object-code, and
combined-work duties (GPL §5 modification notices + dates; LGPL §4 combined-work notices,
relinkability / corresponding source) attach when first exercised — when CKC edits an upstream
file or ships a built artifact; each `CKC:`-marked edit will carry a native-syntax notice + date.
CKC's own additions convey under CKC's LICENSE (GPL-3.0-or-later), compatible with the
LGPL-3.0-or-later and GPL-3.0-or-later inputs. This file is a voluntary provenance/compliance aid,
not itself a license-mandated artifact.
