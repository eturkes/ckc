# CKC-C — Clinical Knowledge Compiler, auditor-first spec — Fable 5 execution of cds_prompt.md

Provenance: produced 2026-06-12 by Claude Fable 5 inside the ckc repository, after reading
SPEC.md (spec04), SPEC_A.md, SPEC_B.md, the archive specs, and the docs/ compendium — a
post-exposure synthesis, not a blind sample of the prompt. Evidence base unavailable to the
other executions: a completed spec04 M1 (pure-Rust pipeline, byte-deterministic replay, Z3
verdicts over synthetic Japanese fixtures) and same-day container probes (clingo 5.6.2 raw-vs-
resolved defeasibility; Z3 4.13.3 `assert-soft` minimal correction sets; Lean 4.30 per-instance
reflection checks without mathlib; cvc5 1.3.4 agreeing with recorded M1 verdicts and cores).
Status: candidate input for spec refinement; not a design authority.

## §0 Mission and posture

Build a headless system that ingests licensed Japanese clinical guidelines (and, where rights
permit, medical textbooks), compiles their textual content into a deterministic, explainable,
verifiable, executable representation, and audits the corpus for logical incompatibilities and
factual inconsistencies — surfacing source-grounded revision candidates for guideline authors.
The final artifact of the initial work is a bilingual JA/EN static review report (data plus a
self-contained HTML view) strong enough to anchor a medical-journal manuscript about
high-confidence clinical autoformalization.

Posture inherited from the working lineage, kept because implementation proved it out:

- Admission over proposer identity: models, retrieval, agents, and humans all propose;
  deterministic validation (schema, grounding, canonical bytes, compile, verify, replay)
  decides. Model output is never accepted on convergence or confidence alone.
- The durable assets are the corpus snapshots, the IR and its semantics, the admitted mapping
  layer, provenance, verification evidence, and the evaluation methodology. Code is cheap and
  rebuildable; concepts are not.
- Every output is research evidence with calibrated wording (`warrants review`,
  `requires human adjudication`, `documented null result`); clinical/SaMD/deployment authority
  sits behind explicit gates.
- One asymmetry drives the architecture: runtime model calls are the principal epistemic and
  operational risk, so the system is built to move model involvement to development time —
  models help author a compact admitted mapping layer once; fresh documents then compile
  deterministically with zero runtime model calls wherever mappings cover them, and typed
  residuals where they do not.

## §1 Success criteria

A fresh checkout, one command chain, deterministic outputs:

1. Corpus loop: licensed sources enter through permission-recorded, content-addressed
   snapshots; extraction yields source graphs with exact spans down to table cells; every
   later claim resolves to quotable Japanese bytes or a typed residual.
2. Compile loop: admitted IR for every covered statement — parsed, validated, canonicalized
   idempotently (`normalize(normalize(x)) == normalize(x)`), content-hashed, replayable.
3. Audit loop: the formal portfolio (§7) executes per fixture group and per corpus slice;
   every finding carries conflict kind, severity, witness or core, involved rules, regions,
   quoted spans, and a suggested review question in Japanese and English.
4. Null results, residuals, and ambiguities are first-class typed outcomes, not log lines.
5. Bilingual static review artifact renders deterministically from canonical report bytes.
6. Locked evaluation: metrics (coverage, admission rate, idempotency, convergence, conflict
   precision/recall over seeded cases, trace completeness, replay stability) emit as exact
   raw rows under a frozen evaluator identity; the manuscript bundle (tables, figures data,
   reproducibility instructions) derives from those rows.
7. Replay: `ckc replay` reproduces all accepted artifact hashes from the snapshots alone;
   recorded model I/O replays byte-stably; adjudication records attach to findings without
   changing them.

## §2 Decisive stack

| Problem | Decision | Reason (evidence) |
| --- | --- | --- |
| Core language | Rust workspace, edition 2024; one binary `ckc` | M1 proved byte-deterministic canonical JSON, hashing, emission, replay in pure Rust with no determinism debt; agent authorship removes the "Python iterates faster" rationale, which is human bias. |
| Extraction adapters | Rust HTML/XML first (proven); PDF via a quarantined adapter process (Rust `pdfium`-family or `uv`-managed Python PyMuPDF, decided per fixture evidence) joined only through canonical artifacts | PDF/OCR ecosystems are genuinely Python-weighted; the boundary keeps adapter nondeterminism outside accepted semantics. |
| OCR | Separate low-trust lane, engine identity + confidence recorded; OCR-derived text feeds review surfaces, never accepted formalization without validation | Scanned guideline PDFs exist; the lane keeps them from contaminating determinism claims. |
| Japanese lexical layer | Project lexicon (concepts, modality phrases, units) as canonical YAML, versioned by content hash — the admitted mapping layer; morphological tokenization (Sudachi-family) only as evidence-discovery aid for mapping authoring, never in the accepted path | M1's lexicon-driven normalize stage worked deterministically; tokenizer output in the accepted path would import dictionary-version drift. |
| IR | Layered bundle per document: DocIR → SegmentIR → ClinicalIR → NormIR → FormalIR, components content-hashed and reused across documents | The layer stack is what makes mapping reuse, amortization, and per-stage failure attribution measurable; single-layer IRs (GLK/CGIR-style) make every document a model transaction forever. |
| Canonical bytes | One canonical JSON serialization per type; strict reader as writer-inverse; sorted sets; exact rationals; tagged unions; content hashes over canonical payload bytes | Implemented and round-trip-tested in M1; the determinism kernel everything else stands on. |
| Decidable checks | SMT-LIB 2 text artifacts; Z3 primary; per-query narrowest logic; named assertions mapped to rule ids and source regions | Proven in M1 end-to-end with unsat cores naming both source documents. |
| Cross-check | cvc5 replay of emitted SMT on high-severity findings; divergence or `unknown` → review item | Probe: cvc5 1.3.4 agrees with recorded M1 verdicts and cores out of the box; portfolio-friendly emission (result commands gated on expected status) is a one-line emitter rule. |
| Defeasibility | Exceptions compile to negated context conjuncts (M1 semantics) AND every eligible pair also runs exception-free — raw-vs-resolved reporting from day one; clingo ASP joins as a backend when real corpora show priority/defeat structure that conjunct semantics cannot express | Probe: stratified clingo program separates `raw_conflict`/`defeated`/`accepted` deterministically in 12 lines; adoption stays evidence-triggered to avoid a second semantics before the first one saturates. |
| Revision localization | Z3 `assert-soft` (weights from strength/certainty/source authority) for minimal correction sets over conflict clusters; MARCO-style MUS/MCS enumeration in the Rust adapter when clusters outgrow single calls | Probe: weighted MCS in one solver call, no extra dependency; "these passages jointly imply an impossibility" is the auditor's core sentence. |
| Mechanized semantics | Lean 4 package defining the NormIR/FormalIR fragment, conflict predicates, and normalizer properties; per-instance checks by `decide`/`native_decide` generated from accepted artifacts; generic theorems as explicit proof debt | Probe: single-file Lean, no mathlib, seconds per check; kernel `decide` stalls on String-order computation — `native_decide` or Nat-keyed encodings are the working patterns. This is the manuscript's mechanized-semantics anchor. |
| Reports/UI | Canonical `report.json`; deterministic renderings: `report.md`, `report.ja.md`, and one self-contained bilingual `report.html` per run (embedded data + vendored viewer bytes; JA primary, EN gloss linked per span; no server, no node toolchain in the build path) | A static review artifact is the deliverable; a web service is production bias. |
| Storage | Content-addressed files under `runs/` and `corpus/`; no database | Sufficient at audit scale; queryability lives in the lineage index. Analytic stores join only if corpus scale demands. |
| Model harness | llama.cpp-family local runtime and/or recorded hosted-model calls as subprocess adapters: grammar-constrained (GBNF/JSON-Schema from exported type schemas), greedy + fixed seed, full I/O recording; live calls only under explicit experiment flags | Recorded-bytes replay is what makes model-assisted runs auditable; grammar masks make schema validity a decoding property rather than a hope. |
| Tests | cargo test workspace suites; pinned byte fixtures from observed output; metamorphic variants as committed fixtures; property tests only where they pay (canonical bytes, normalizers) | M1's pinned-battery style caught real regressions; avoid test bloat that audits nothing. |

## §3 Determinism kernel

Adopt spec04 §4 unchanged (ids, hashes, exact rationals, seven string policies, canonical
payload bytes, envelope with origin/authority enums, total operation results, JSONL events,
replay manifests). It is implemented, byte-pinned, and archive-proven; re-deriving it would be
waste. Two auditor-stage additions:

- `unit` becomes a first-class quantity field: every quantity is an exact rational plus a
  canonical unit code with deterministic conversion factors (UCUM-compatible table committed
  as lexicon data); original Japanese unit strings persist as raw fields. Threshold conflicts
  compare only unit-normalized values.
- `AdjudicationRecord` artifact kind: reviewer role (clinician, formalist, terminology,
  adjudicator), target artifact hash, label, notes, timestamp — append-only, runtime-metadata
  dated, never mutating the target; findings render with their adjudication state.

## §4 Sources, permissions, extraction

- Source families, in adoption order: synthetic fixtures (committed, license-clean); Minds-style
  guideline HTML; PMDA e-PI XML (structured 禁忌/効能/用法 sections — the cross-source audit
  counterpart); J-STAGE/JATS XML; guideline PDF; licensed textbook EPUB/PDF as
  `restricted_internal_only` permission class (textbooks are corpus expansion, not v1
  acceptance — their licensing rarely permits redistribution and their prose is weakly
  structured; the permission machinery, not the schema, is what they need).
- PermissionRecord per source: rights holder, access ref, license label, redistribution status
  (`redistributable | reconstructable | restricted_internal_only`), allowed artifact classes;
  blocked exports emit typed residuals and the pipeline continues; reports redact by
  deterministic policy (quoted spans only where permitted, offsets+hashes otherwise).
- Extraction contract: real parsers (no regex screen-scraping); SourceGraph with nodes, spans,
  anchors, regions; tables preserve row/column/cell/header/caption relations and unit context
  — dose and threshold conflicts live in tables; every extracted unit has a span or a typed
  `extraction_uncertain` residual; identical bytes + config ⇒ identical graph bytes.
- Drift: source hash changes emit `source_drift.json` and mark dependent scores stale.

## §5 IR and the admitted mapping layer

Five layers in one bundle per document, components content-hashed with use-site records:

| Layer | Content |
| --- | --- |
| DocIR | Layout-preserving text/table view over SourceGraph refs. |
| SegmentIR | CQ, recommendation, evidence, exception, definition, table-row, metadata segments. |
| ClinicalIR | Normalized statements (population, condition, action, modality, strength, certainty, exceptions, temporal slots) + terminology bindings. |
| NormIR | Guarded rules: finite-DNF contexts, direction, action, exceptions as refs + negated conjuncts; factual rules; decision tables for table rows. |
| FormalIR | Target-independent constraints, normalized action/context keys, query plans. |

The admitted mapping layer is the system's product-defining asset: lexicon concept entries
(with interval semantics), modality phrase → (direction, strength) rows, certainty phrases,
unit rows, action-kind rows, and binding policies — each entry source-grounded at admission,
content-hashed, and reused across every document it covers. Coverage of fresh documents by the
existing mapping set, with zero apply-phase model calls, is a standing metric; unmapped and
ambiguous mentions emit typed residuals/ambiguities that seed development-time mapping
proposals (model-assisted, recorded, admitted by the same checks as everything else).
Terminology systems join behind one binding contract: project lexicon first, MEDIS masters
(病名/HOT) as first external systems, JLAC and MedDRA/J registered next; licensed
vocabularies stay registry-listed until rights evidence exists.

## §6 Translation routes

- `route.deterministic` — the default and the acceptance path: extract → segment → normalize
  via admitted mappings → assemble → compile. No model anywhere at runtime.
- `route.model_assisted` — development-time only: grammar-constrained model fills IR layers
  for passages the mapping set cannot cover; output is proposal material; every accepted
  result must re-derive deterministically after the implied mapping entries are admitted.
  Convergence across k recorded samples is a stability signal recorded with the proposal,
  never an admission criterion.
- Repair loops feed structured diagnostics (stable codes) back at most N recorded rounds;
  successful repairs revalidate from scratch.

## §7 Formal portfolio

Execution order per corpus slice, each stage producing named, replayable artifacts:

1. Self-coherence (per rule): context satisfiability check; unsat context ⇒
   `condition_unsatisfiable` finding (usually an extraction or normalization defect; sometimes
   a genuine source defect — either way the auditor's cheapest catch).
2. Eligibility scan (pairwise, blocked by normalized action key + concept overlap signature to
   control the quadratic).
3. Context overlap (Q1) twice per eligible pair: with exception conjuncts (resolved view) and
   without (raw view). Raw-sat ∧ resolved-unsat ⇒ `exception_resolved_conflict` — reported,
   not hidden; it is exactly the evidence a guideline author needs to confirm the exception is
   intentional. Raw-unsat ⇒ documented null result with the disjointness reason.
4. Polarity/factual consistency (Q2) on resolved-sat pairs: deontic direction conflicts,
   strict factual contradictions, threshold/unit conflicts (empty unit-normalized intervals),
   temporal-order conflicts (difference logic; cycles surface as joint unsatisfiability).
5. Cluster localization: for every hard finding cluster, weighted MCS via `assert-soft`
   (weights from strength, certainty, source authority class) ⇒ minimal revision candidates
   ("these k passages jointly imply an impossibility; relaxing this one restores
   consistency"), rendered with the involved spans.
6. Cross-check: cvc5 replays every blocking/major finding's queries; divergence ⇒ review item.
7. Mechanized anchor: Lean data files generated from the run's accepted rules; per-instance
   conflict/no-conflict and normalization-idempotency checks by reflection; check names and
   replay commands land in the trace.
8. ASP lane (adoption-triggered): when adjudicated corpus evidence shows priority/defeat
   structure beyond conjunct semantics, compile NormIR to clingo facts under a fixed
   meta-program (strict/defeasible/defeater, explicit priority), report
   `accepted/defeated/raw_conflict/resolved_conflict` derivations alongside — never instead
   of — the SMT raw/resolved view.

Verdict categories, outcome enums, severity (`blocking | major | moderate | minor | info`),
and finding wording follow the lineage contracts; `unknown`, timeout, and solver disagreement
are review items, never silently dropped.

## §8 Conflict taxonomy

| Kind | Detector |
| --- | --- |
| `condition_unsatisfiable` | §7.1 self-check. |
| `deontic_direction_conflict` | Q2 polarity under resolved-sat overlap. |
| `exception_resolved_conflict` | Raw-sat ∧ resolved-unsat (Q1 pair). |
| `numeric_threshold_empty_intersection` | Unit-normalized interval emptiness. |
| `temporal_constraint_conflict` | Difference-logic joint unsatisfiability. |
| `strict_factual_contradiction` | Factual consequents jointly inconsistent. |
| `terminology_incoherence` | Functional key collision; mutually exclusive bindings. |
| `table_value_disagreement` | Overlapping table guards, incompatible outputs. |
| `source_support_mismatch` | Accepted IR not traceable to its cited spans (admission audit). |
| `cross_source_conflict` | Any of the above across documents/families — guideline × package insert is the flagship. |
| `priority_ambiguity` | ASP lane only: conflicting defeasible rules with insufficient superiority metadata. |
| `replay_or_certificate_failure` | Replay mismatch; reflection-check failure. |

Every finding: kind, severity, rules, regions, quoted spans (permission-gated), assertion
names, witness/core/derivation, suggested review question (JA+EN), adjudication state,
claim-tier wording.

## §9 Evidence, reports, UI, manuscript

- Trace: derivation DAG + claim-evidence rows + lineage index; `ckc trace` walks any finding
  to source bytes and back; reuse/compactness exports quantify the mapping layer.
- `report.json` canonical; `report.md`/`report.ja.md` deterministic renderings; `report.html`
  the bilingual static review artifact: corpus overview, rule browser (span ↔ statement ↔ rule
  ↔ assertion), finding list with filters (kind, severity, document, status), finding detail
  (JA excerpts primary, EN gloss linked, formal evidence, revision candidates, review
  question), metrics page, export bundles. Annotations land as AdjudicationRecords, not edits.
- Manuscript bundle per locked run: metric tables (CSV/JSON exact rationals), figures data,
  corpus and permission summaries, replay instructions, limitations text assembled from typed
  residual/ambiguity statistics — the methods section writes itself from artifacts.

## §10 Evaluation

- Locked evaluator identity (fixture/gold/schema/metric/code hashes) materialized before any
  scored run; evaluator changes are a separately governed migration.
- Corpora: synthetic core (seeded conflicts, exceptions, dose/unit, temporal, disjoint
  controls — every taxonomy row covered both directions); licensed pilot slice; adjudicated
  gold (clinician + formalist + terminology reviewer roles, senior adjudication on
  disagreement, agreement statistics reported).
- Metamorphic suite as committed fixtures: punctuation/kana-kanji/section-order variants,
  rule-order invariance, unit-conversion invariance, irrelevant-paragraph deletion leaving
  rule hashes unchanged.
- Metrics: extraction coverage; mapping coverage (share of statements compiled with zero
  model calls); admission rate; idempotency; hash convergence across variants; conflict
  precision/recall over seeded cases; witness/core completeness; replay stability;
  adjudication agreement; per-finding reviewer yield (share of findings adjudicated
  `warrants_revision`) — the auditor's single most important number.

## §11 Future-path gates

The gate system carries every stronger claim (clinical authority, SaMD, EHR runtime, patient
data, probabilistic semantics, world models, retrieval quality, corpus-scale benchmarks,
automated promotion). Two auditor-era additions to the lineage's table: `G-ADJUDICATION`
(published claims about real-guideline defect rates require adjudicated gold evidence) and
`G-EXEC-EVAL` (any patient-context rule-evaluation runtime — the `applicable / not_applicable
/ unknown` executable semantics that a CDS backend needs — enters as a gated milestone with
three-valued evaluation semantics defined first; the audit pipeline never needs it).

## §12 Milestones

| M | Deliverable | Gate |
| --- | --- | --- |
| C1 | Determinism kernel + IR + deterministic route on synthetic fixtures; SMT raw/resolved + self-check + threshold/unit kinds; trace; replay | seeded taxonomy rows all detected; replay byte-stable |
| C2 | MCS localization + cvc5 cross-check + Lean per-instance anchor; severity + review questions; report.json/md/ja.md | every hard finding carries core + revision candidate + bilingual question |
| C3 | Bilingual static report.html + AdjudicationRecords + manuscript bundle | reviewer walks span→rule→assertion→verdict→question entirely offline |
| C4 | Real-source ingestion (Minds HTML + e-PI XML first), permission/redaction/drift live; mapping-coverage metrics; model-assisted mapping authoring (recorded) | licensed pilot slice audited end-to-end; coverage + yield reported |
| C5 | Adjudicated gold + locked evaluation + metamorphic suite at corpus scale; ASP lane if triggered | manuscript-grade tables with agreement statistics |

Build order inside any milestone: kernel types → emitters → checks → renderings; land a
compiling skeleton before batteries; pin expected bytes from observed output; one conceptual
deliverable per session-sized unit.

## §13 Relation to the research program (spec04)

This spec serves cds_prompt.md's product goal directly. It deliberately omits spec04's
research program — the lift minimal pair (M2), route/IR variation (M3), invented DSLs (M4),
autoresearch loop (M5) — which answers a prior question this spec takes on faith: that the
layered-IR + admitted-mapping design is the right translation architecture at all. Run the
research program first (it is also what makes the manuscript's novelty claims defensible);
then this spec's C1–C3 collapse to a thin increment over the proven spine, and C4–C5 merge
with spec04's M6. The synthesis recommendation accompanying this file maps each section onto
spec04 amendments.
