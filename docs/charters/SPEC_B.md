# CDS Guideline Autoformalization Research Prototype Spec

Audience: Claude Fable 5 coding agent. This file is self-contained and should be treated as the project-specific technical charter. The later `CLAUDE.md` may add general repo discipline; this spec controls domain architecture and formal-methods choices.

## 0. Operating frame for Fable 5

Use high effort by default; use the highest available effort for formal semantics, verifier failures, schema design, and evaluation design. Proceed autonomously for reversible implementation choices. Pause only for credentials, licensed source files unavailable to the environment, destructive actions, or a genuine scope change.

When enough information exists to act, act. Prefer a checked implementation, a failing proof, or a benchmark result over a plan. Ground every progress claim in artifacts from the current run: test output, solver output, generated files, or logs. Store design decisions, invariants, unresolved proof debt, and benchmark results in the repo memory system once the supplied `CLAUDE.md` defines it.

Code is cheap. Concepts, formal semantics, canonical representations, provenance, tests, and verifier harnesses are valuable. Favor rebuilding modules cleanly over preserving poor abstractions. Keep low-level coding freedom: choose concrete package names, module boundaries, and implementation techniques that satisfy the contracts below.

## 1. Project goal

Build a research-grade, non-clinical prototype for Japanese clinical guideline/textbook autoformalization. The initial artifact is a headless pipeline plus a simple bilingual JA/EN results UI. It ingests textual content from licensed Japanese clinical guidelines and medical textbooks, extracts source-grounded statements, converts them into a deterministic clinical guideline intermediate representation, compiles that representation into executable/checkable formal targets, detects logical incompatibilities and factual inconsistencies, and reports revision candidates against the original text.

The system is not a clinician-facing CDS and not a medical device. All outputs are revision-support evidence for researchers and guideline authors. Future production SaMD/EHR integration matters only insofar as early choices should preserve clean semantics, provenance, and Japan-local interoperability.

## 2. Decisive architecture choices

Canonical knowledge representation: build a custom JSON AST named `CGIR` (Clinical Guideline Intermediate Representation). CGIR is the source of truth. It is constrained by Pydantic/JSON Schema, canonicalized by a deterministic normalizer, and rendered to other targets. CQL/ELM, FHIR Clinical Reasoning, Lean, SMT-LIB, and ASP are compiler targets or interoperability exports, not the primary editable IR.

Formal authority: use a two-layer strategy. Lean 4 defines the deep semantics and proves/refutes properties of the object language where feasible. SMT-LIB with Z3 and cvc5 performs scalable decidable checks, unsat cores, overlap witnesses, arithmetic constraints, and finite-domain reasoning. clingo ASP handles defeasible/deontic rule acceptance, priority, and exception reasoning. PySAT/MaxSAT or a MARCO-style MUS/MCS enumerator localizes minimal conflict/repair sets.

Implementation substrate: use Python for orchestration, schemas, ingestion adapters, canonicalization, solver invocation, and API. Use Lean 4 for semantics/proofs. Use TypeScript/Vite/React for the bilingual UI. Use SQLite or DuckDB for local reproducible storage and content-addressed files for immutable artifacts. Use current stable releases and lockfiles/manifests, while keeping this early prototype free of brittle version pinning in the spec.

Model interface: implement a provider-neutral LLM formalizer with structured-output schemas. Accept API structured output/tool schemas and local constrained decoding backends where available. The LLM generates candidate CGIR data only. The LLM never serves as the verifier.

## 3. Scope boundaries

Initial scope includes:

- Japanese guideline/textbook source acquisition from local files and permitted URLs.
- Deterministic text extraction from embedded text in PDF, HTML, XML/JATS, Markdown, plain text, and EPUB.
- Separate OCR lane for scanned pages, with OCR output flagged as low-trust until human validation; OCR-derived text can be stored and reviewed, while accepted formalization must carry an extraction-confidence flag.
- Passage segmentation, table-cell preservation, source-span IDs, bilingual labels/glosses, and provenance.
- CGIR schema, canonicalizer, validator, hash/id policy, and diff tools.
- LLM candidate generation into CGIR using constrained schemas and source-grounded prompts.
- Candidate clustering, idempotency checks, round-trip checks, and critic/adjudication passes.
- Compilation to Lean, SMT-LIB, ASP, and report JSON.
- Detection of logical incompatibilities, modal/deontic conflicts, unresolved defeasible conflicts, impossible numeric intervals, impossible temporal constraints, and internally inconsistent factual claims.
- A simple bilingual web UI to inspect source spans, formal statements, solver traces, witnesses, and suggested revision targets.
- Evaluation harness suitable for a manuscript: gold fixtures, synthetic contradiction cases, metamorphic tests, run manifests, reproducibility scripts, and exported result tables.

Future-aligned but outside the initial implementation target:

- Real-time EHR operation, clinician alerts, orders, hooks, or automation.
- Patient-specific advice on live PHI.
- SaMD certification evidence beyond traceability/reproducibility groundwork.
- Full pharmacologic interaction modeling, multimodal data, learned risk models, or clinical world models.
- Claims that detected issues are clinically wrong without domain-expert adjudication.

## 4. Success criteria

Deterministic means every accepted artifact is content-addressed and reproducible from `(raw source bytes, source adapter version, schema version, prompt version, model/provider metadata, decoding parameters, normalizer version, terminology snapshots, compiler versions)`. The final accepted CGIR package must canonicalize to byte-identical JSON across repeated normalization. IDs must derive from stable content/provenance hashes, not from run order.

Explainable means every rule, fact, condition, action, modality, code binding, threshold, and conflict must point to exact source spans. Every detected issue must include a human-readable JA/EN explanation, the involved formal atoms, the source text, and either a solver witness, an unsat core, an ASP derivation, or a Lean theorem/check result.

Verifiable means accepted CGIR must pass schema validation, internal type checking, canonicalization idempotency, and at least one formal backend check. Key theorem families must be represented in Lean. Scalable conflict checks must emit solver artifacts that can be rerun.

Executable means CGIR must evaluate over synthetic or structured patient/context records and produce a trace. Execution is for research tests and examples only. The evaluator must return explicit statuses such as `applicable`, `not_applicable`, `unknown`, `conflict_raw`, `conflict_resolved`, and `inconsistent`.

Research-grade means the prototype can run end-to-end on a small licensed corpus and on synthetic fixtures, produce reproducible reports, and support a manuscript demonstrating formal autoformalization confidence. It does not need production performance, broad specialty coverage, or polished UX.

## 5. Core data model: CGIR

CGIR is a strictly typed JSON AST. It should be easy for LLMs to emit under schema constraints and easy for compilers to consume. Prefer enums and prefix-operator trees over free strings. Preserve raw source text separately from normalized labels.

Top-level package shape:

```json
{
  "schema": "cgir.v0",
  "package_id": "cgirpkg_<hash>",
  "run_id": "run_<hash>",
  "source_documents": [],
  "text_spans": [],
  "concepts": [],
  "rules": [],
  "terminology_bindings": [],
  "compiler_artifacts": [],
  "checks": []
}
```

### 5.1 SourceDocument

Fields:

- `document_id`: deterministic hash of raw bytes plus adapter namespace.
- `title_ja`, `title_en`, `version_label`, `publication_date`, `source_type`.
- `license_status`: `user_supplied`, `public`, `licensed`, `unknown`, `blocked`.
- `adapter`: source adapter name and version.
- `raw_uri_or_path`: stored separately from publication output when copyrighted.
- `content_hash`: cryptographic hash of raw bytes.
- `metadata`: publisher, society, guideline version, ISBN/DOI/URL, Minds CQ identifiers when available.

### 5.2 TextSpan

A `TextSpan` is the atomic provenance unit. It can be a sentence, paragraph, heading, table cell, figure caption, footnote, or list item.

Fields:

- `span_id`: deterministic from document ID, page/section/table coordinates, and normalized text.
- `document_id`.
- `span_kind`: `heading`, `paragraph`, `sentence`, `table_cell`, `table_row`, `caption`, `footnote`, `reference`, `other`.
- `raw_text_ja`: exact extracted Japanese text.
- `normalized_text_ja`: Unicode-normalized text for processing.
- `gloss_en`: English gloss generated for review; never the formal authority.
- `location`: page, section path, bounding boxes, row/column coordinates, byte offsets when available.
- `extraction_confidence`: `embedded_text`, `structured_xml`, `html_text`, `ocr_high`, `ocr_low`, `manual`.
- `parent_span_ids` and `child_span_ids`.

### 5.3 ClinicalConcept

Concepts normalize clinical terms without forcing premature ontology lock-in.

Fields:

- `concept_id`: deterministic from normalized label and accepted code bindings.
- `label_ja`, `label_en`, `aliases_ja`, `aliases_en`.
- `semantic_type`: `disease`, `finding`, `drug`, `procedure`, `lab_test`, `lab_value`, `adverse_event`, `population`, `outcome`, `time`, `dose`, `unit`, `care_setting`, `other`.
- `bindings`: list of code bindings with `system`, `code`, `display`, `version`, `confidence`, `source_span_id`.
- `jp_priority`: prefer Japanese operational codes where appropriate: ICD-10 JP/MEDIS disease master for diagnoses, HOT/YJ for drugs, JLAC10/11 for labs, MERIT-9/UCUM for units, MedDRA/J for adverse events, K-codes only for claims/procedure evaluation.
- `status`: `candidate`, `accepted`, `needs_review`, `rejected`.

### 5.4 Rule

A rule is the main formal object.

Fields:

- `rule_id`: deterministic hash of canonical payload and provenance.
- `source_span_ids`: exact spans supporting the rule.
- `rule_kind`: `definition`, `factual_claim`, `eligibility`, `recommendation`, `contraindication`, `dose`, `monitoring`, `temporal_pathway`, `evidence_relation`, `terminology_mapping`.
- `context`: formula AST over patient/population/source conditions.
- `action`: action AST for intervention, test, monitoring, avoidance, referral, diagnosis, documentation, or follow-up.
- `modality`: `obligatory`, `recommended`, `permitted`, `discouraged`, `forbidden`, `informational`, `factual`.
- `strength`: `strong`, `weak`, `conditional`, `none`, `unspecified`.
- `certainty`: `high`, `moderate`, `low`, `very_low`, `not_appraised`, `unspecified`.
- `defeasibility`: `strict`, `defeasible`, `defeater`.
- `priority`: explicit override/exception relationships to other rule IDs or rule classes.
- `temporal_scope`: interval/duration/order constraints when applicable.
- `quantitative_constraints`: exact rational values and units; store original text and normalized unit.
- `exceptions`: formula AST or references to exception rules.
- `status`: `candidate`, `schema_valid`, `type_valid`, `source_aligned`, `accepted`, `needs_review`, `rejected`.
- `confidence`: extraction/model confidence metadata only; confidence must never override formal verification.

### 5.5 Formula AST

Use prefix trees.

Primitive nodes:

- `atom`: concept or field predicate.
- `comparison`: field/concept value compared by `eq`, `ne`, `lt`, `le`, `gt`, `ge`, `in`, `not_in`, `subset`, `overlaps`.
- `and`, `or`, `not`, `implies`, `iff`.
- `exists`, `forall` only over finite named domains in the initial system.
- `unknown_if_missing` marker for data-availability semantics.
- `temporal`: `before`, `after`, `during`, `within`, `overlaps`, `starts_before`, `ends_after`.

Truth values: use `true`, `false`, `unknown`, and optionally `both` in conflict-tolerant reporting. Solver encodings may use two-valued approximations, but CGIR must preserve unknown vs false.

### 5.6 Action AST

Action shape:

- `action_type`: `administer_drug`, `avoid_drug`, `perform_test`, `monitor`, `refer`, `diagnose`, `treat`, `educate`, `document`, `schedule_followup`, `other`.
- `target_concept_id`.
- `parameters`: dose, route, frequency, duration, timing, comparator, threshold, location, care setting.
- `intended_outcome_concept_ids`.
- `harm_concept_ids`.

### 5.7 CheckResult and RevisionCandidate

A check result is a machine artifact; a revision candidate is a manuscript/UI artifact.

CheckResult fields:

- `check_id`, `check_type`, `backend`, `status`, `input_artifact_ids`, `output_artifact_ids`.
- `formal_statement`: compact representation.
- `solver_result`: `sat`, `unsat`, `unknown`, `valid`, `invalid`, `accepted`, `rejected`.
- `witness`: model/counterexample when applicable.
- `unsat_core`: named constraints/source rule IDs when applicable.
- `proof_or_derivation`: Lean theorem, ASP model, derivation tree, or replay command.

RevisionCandidate fields:

- `candidate_id`, `severity`: `blocking`, `major`, `moderate`, `minor`, `info`.
- `category`: `logical_incompatibility`, `modal_conflict`, `defeasible_unresolved`, `factual_numeric`, `factual_terminology`, `temporal_impossible`, `source_ambiguity`, `model_instability`, `extraction_uncertain`.
- `title_ja`, `title_en`, `explanation_ja`, `explanation_en`.
- `source_span_ids`, `rule_ids`, `check_ids`.
- `suggested_review_question_ja`, `suggested_review_question_en`.
- `machine_confidence`: confidence in detection, not clinical truth.

## 6. Formal semantics

### 6.1 Object language

Define a finite, typed clinical object language:

- Sorts: `PatientContext`, `Concept`, `Code`, `Unit`, `Quantity`, `Time`, `Interval`, `Action`, `Formula`, `Norm`, `Rule`.
- Values: exact integers/rationals for thresholds and doses; normalized units; symbolic concept IDs; time intervals.
- Formula semantics: three-valued evaluation over partially known clinical contexts.
- Norm semantics: dyadic form `(context, modality(action))`.
- Defeasible semantics: `strict`, `defeasible`, and `defeater` rules with explicit superiority/priority relation.
- Conflict semantics: raw conflicts are detected before priority; resolved conflicts are computed after priority; both are reported.

### 6.2 Modality conflict lattice

Hard conflicts:

- `obligatory(A)` conflicts with `forbidden(A)` when contexts overlap.
- `recommended(A)` conflicts with `forbidden(A)` when contexts overlap.
- `obligatory(A)` conflicts with `discouraged(A)` when contexts overlap.
- `recommended(A)` conflicts with `discouraged(A)` when both are strong or when policy says weak-vs-weak warrants review.

Soft/review conflicts:

- `permitted(A)` vs `forbidden(A)` is a review conflict unless the permission is explicitly exception-scoped.
- `recommended(A)` vs `recommended(not A)` is a review conflict, then can become hard when action complement is formally known.
- Strength/certainty mismatches alone are not contradictions; they are revision candidates only when the action/context pair is otherwise equivalent.

### 6.3 Factual inconsistency classes

- Empty numeric interval: same context yields constraints such as `dose >= 10 mg` and `dose <= 5 mg`.
- Mutually exclusive classifications: same concept instance asserted into disjoint categories under same scope.
- Unit-incompatible claims: same quantity compared after unit normalization yields impossibility.
- Temporal cycle: required event order produces `t1 < t2 < ... < t1` or impossible duration.
- Terminology clash: two code bindings assert equivalence between concepts declared disjoint or non-overlapping in the local terminology snapshot.
- Source self-inconsistency: same document/version makes incompatible claims under overlapping conditions.
- Cross-source inconsistency: different documents/societies make incompatible claims under overlapping conditions.

### 6.4 Open-world, closed-world, and unknown

Guidelines often omit patient facts. CGIR must distinguish absent information from falsehood. Use open-world semantics for missing clinical facts in source interpretation. Use finite closed-world domains only inside solver tasks where the domain is explicitly enumerated. Every solver result must record the approximation used.

### 6.5 Priority and exception policy

Priority must be explicit or mechanically derived from source text. Derived priority candidates include `unless`, `except`, `contraindicated`, population-specific statements overriding general statements, and explicit guideline hierarchy. Automatic priority can resolve a conflict for execution, but the raw conflict remains available for review.

GRADE strength/certainty is metadata for explanation and prioritization. It must not be treated as a probability in the initial prototype. Probabilistic logic can be added later after a validated numeric interpretation is defined.

## 7. Compiler targets

### 7.1 Lean 4

Purpose: trusted semantic anchor and proof-by-reflection layer.

Implement a Lean package with:

- Inductive definitions for CGIR formula/action/rule fragments.
- Evaluators such as `evalFormula`, `applies`, `modalConflict`, `incompatible`, `acceptedRules` for finite fragments.
- Normalization functions and theorem statements for idempotency and semantic preservation.
- Generated Lean data files for accepted CGIR packages.
- Theorem files proving key synthetic and corpus-derived claims where tractable.

Initial theorem families:

- `normalize(normalize(x)) = normalize(x)` for each normalizer fragment.
- `eval(normalize(x), ctx) = eval(x, ctx)` for formula fragments.
- `incompatible(r1, r2) = true` for detected finite examples.
- `incompatible(r1, r2) = false` for verified non-conflict regression examples.
- `compiler_smt_soundness_stub` theorem statements with proof debt explicitly recorded when full proof is pending.

Use proof by reflection: the LLM emits rule data, while Lean computes/decides. Generated free-form Lean proofs are optional repair artifacts; generated proof terms are not part of the primary autoformalization acceptance path.

### 7.2 SMT-LIB with Z3 and cvc5

Purpose: scalable decidable checks, witnesses, unsat cores, and cross-checking.

Use canonical SMT-LIB output:

- Fixed `set-logic` per task; prefer quantifier-free fragments (`QF_LIA`, `QF_LRA`, `QF_UFLIA`, `QF_DT`) before quantified logic.
- Deterministic symbol naming: `r_<rulehash>__<field>`.
- All source constraints wrapped with named annotations or solver assumptions.
- Exact rational arithmetic for thresholds; avoid floating-point unless source text requires IEEE semantics.
- Push/pop for incremental pairwise and group checks.
- Run both Z3 and cvc5 where feasible; record divergence or `unknown` as a review candidate.

Primary checks:

- Context overlap for pairs/groups of rules.
- Modal conflict satisfiability: `context_i ∧ context_j ∧ same_action ∧ incompatible_modality`.
- Numeric interval satisfiability.
- Temporal constraint satisfiability.
- Terminology disjointness checks where local concept maps provide disjointness.
- Unsat cores for impossible factual bundles.

### 7.3 ASP/clingo

Purpose: defeasible/deontic execution with exceptions, priorities, and accepted/rejected rule sets.

Compile CGIR to ASP facts plus a fixed meta-program:

- `rule(R)`, `strict(R)`, `defeasible(R)`, `defeater(R)`.
- `applies(R, Context)`.
- `concludes(R, modality(Action), Context)`.
- `priority(R1, R2)`.
- `defeated(R, Context)`, `accepted(R, Conclusion, Context)`.
- `raw_conflict(Action, Context)` and `resolved_conflict(Action, Context)`.

Use clingo stable-model semantics for the initial backend. Use deterministic `#show` projections and stable naming. ASP explains how exceptions resolve conflicts; SMT explains whether contexts overlap and whether numeric/temporal constraints are possible.

### 7.4 MUS/MCS and MaxSAT

Purpose: minimal revision localization.

Group constraints by source span/rule. When an inconsistency is found, compute minimal unsatisfiable subsets or minimal correction sets at the group level. The UI should say “these source passages together imply an impossibility,” not “this passage is wrong.”

Start with Z3/cvc5 unsat cores and PySAT-backed MARCO-style enumeration for small fixtures. Expand only when needed.

### 7.5 FHIR, FHIRPath, CQL/ELM exports

Purpose: interoperability and future CDS alignment.

Use FHIR JP Core as the future patient-context substrate. Use base FHIR Clinical Reasoning resources (PlanDefinition, ActivityDefinition, Library, Measure) only as export/package formats. JP Core does not define knowledge artifacts; Clinical Reasoning logic must be layered above it.

Use FHIRPath for patient-field references and future EHR data access because it is compact, model-independent, and maps cleanly to FHIR profiles. Use CQL/ELM export for straightforward clinical logic fragments when useful for comparison with CDS standards. Treat CQL/ELM as an interoperability target, because the initial project needs deontic, defeasible, and proof-assistant semantics that CQL alone does not provide.

## 8. Source ingestion and text extraction

### 8.1 Source adapters

Implement adapters as pure functions from source bytes/URIs to normalized document artifacts. Each adapter must emit a manifest with raw hash, adapter version, extraction method, and warnings.

Initial adapters:

- `local_pdf`: Japanese guidelines/textbooks with embedded text.
- `local_epub`: licensed textbook EPUB.
- `local_html`: saved HTML.
- `jats_xml`: J-STAGE or publisher full-text XML when available.
- `markdown_text`: research fixtures and manually curated samples.
- `pmda_epi_xml`: future useful adapter for package inserts; keep separate from guideline corpus.

Preferred extraction tools:

- PDF: PyMuPDF as primary; pdfplumber for table geometry if needed.
- HTML/XML: lxml/selectolax/BeautifulSoup with adapter-specific selectors.
- EPUB: ebooklib plus lxml/BeautifulSoup.
- Japanese tokenization/normalization: SudachiPy for deterministic lexical assistance; tokenization is auxiliary, not a formal source of truth.

### 8.2 Text preservation

Always store:

- raw source bytes;
- exact extracted text;
- normalized processing text;
- page/section/table location;
- extraction warnings;
- hash of each extracted span.

Text normalization must preserve raw text. Use Unicode normalization only in separate fields and in canonical IDs where specified. Medical symbols, Greek letters, subscripts, inequalities, units, and Japanese era dates require explicit normalization tests.

### 8.3 Segmentation

Segment by structure before language. Prefer document headings, CQ numbers, recommendation boxes, table boundaries, figure captions, and list nesting over sentence splitting. Sentence-level segmentation is a child layer inside paragraph/table spans.

Tables are first-class. Each cell must retain row/column headers and unit context. Many dose/threshold inconsistencies live in tables; flattening tables into plain text loses semantics.

## 9. Autoformalization pipeline

### 9.1 Pipeline stages

1. `ingest`: source bytes to document artifacts.
2. `segment`: document artifacts to TextSpan graph.
3. `retrieve`: gather local context, terminology candidates, prior similar rules, and source metadata.
4. `formalize`: LLM emits candidate CGIR objects under schema constraints.
5. `validate`: schema/type/source-alignment validation.
6. `canonicalize`: deterministic normalization and stable IDs.
7. `cluster`: repeated candidates grouped by canonical hash and formal equivalence.
8. `compile`: generate Lean, SMT-LIB, ASP, report JSON, optional FHIR/CQL exports.
9. `verify`: run Lean/SMT/ASP/MUS checks.
10. `report`: emit CheckResults and RevisionCandidates.
11. `review_ui`: serve bilingual inspection UI.
12. `evaluate`: compute metrics and benchmark artifacts.

### 9.2 LLM formalization contract

The LLM receives source spans, local headings/tables, glossary candidates, and the CGIR JSON Schema. It returns candidate CGIR only. It must include provenance for every claim. Invalid or unsupported claims are retained as rejected candidates with reasons.

Use candidate sampling and convergence:

- Generate multiple candidates per source unit with fixed prompt/schema versions.
- Canonicalize each candidate.
- Cluster by canonical JSON hash, AST isomorphism, and solver/Lean equivalence where available.
- Promote the dominant valid cluster only when it passes thresholds.
- Flag low-convergence passages for human review and manuscript analysis.

Self-consistency is evidence of stability, not proof of correctness. Source alignment and formal verification are required for acceptance.

### 9.3 Retrieval and terminology

Retrieval must improve consistency without becoming an authority. Build indexes over:

- accepted prior CGIR rules;
- Japanese clinical terms and aliases;
- JP code systems and local terminology snapshots;
- guideline CQ/recommendation IDs;
- unit normalization rules;
- source-specific abbreviations.

Use deterministic BM25/lexical retrieval as a baseline. Add embeddings for recall where available. Store retrieved context IDs in the run manifest.

### 9.4 Source alignment

Every generated atom must be aligned to one or more source spans. Alignment checks:

- Numeric values match source text after unit parsing.
- Modality words map to the selected modality and strength.
- Condition clauses map to context predicates.
- Exceptions map to `exceptions` or `priority` fields.
- Drug/procedure/disease terms map to accepted or candidate concepts.
- English glosses never introduce formal content absent from Japanese source.

## 10. Canonicalization and determinism

Canonicalization is a compiler pass, not a pretty-printer.

Required rules:

- Sort object keys lexicographically in serialized canonical JSON.
- Sort unordered arrays by semantic key; preserve ordered arrays only for source order, temporal order, or proof traces.
- Normalize Unicode in canonical labels while retaining raw text.
- Normalize units to canonical unit codes and exact rational conversion factors.
- Normalize comparisons: prefer `lt/le/eq/ge/gt` with explicit values; convert textual ranges to closed/open interval objects.
- Normalize Boolean formulas: flatten nested `and`/`or`, sort commutative children, remove identities, preserve explicit `unknown_if_missing` semantics.
- Normalize actions: represent “avoid X” as `forbidden(administer X)` where clinically equivalent; keep `avoid_drug` action label for UI display.
- Normalize terminology bindings by code-system canonical URI/version.
- Stable IDs use canonical payload plus provenance, not model output order.

Idempotency tests:

- `normalize(normalize(x)) == normalize(x)` for every CGIR type.
- Canonical serialization round-trip is byte-identical.
- Reordered JSON input yields the same canonical hash.
- Unit-equivalent quantities yield the same canonical quantity.
- Formula-equivalent commutative expressions yield the same canonical formula when within the rewrite fragment.

## 11. Verification and contradiction detection

### 11.1 Check taxonomy

Run these checks on every accepted CGIR package:

- Schema validation.
- Type validation.
- Source-alignment validation.
- Canonicalization idempotency.
- Lean compilation/checking.
- SMT syntax and solver replay.
- Pairwise context-overlap checks for rules sharing action/concept families.
- Numeric interval consistency checks.
- Temporal ordering consistency checks.
- ASP accepted/resolved conflict checks.
- Minimal conflict localization for every hard inconsistency.
- Metamorphic regression suite.

### 11.2 Conflict search strategy

Use blocking and indexing to control combinatorics:

- Compare rules only when action/concept/terminology overlap, or when one rule has broad scope.
- Use context signatures to prefilter pairs.
- Use SMT for overlap satisfiability.
- Use ASP to determine raw vs priority-resolved conflict.
- Use MUS/MCS for groups after pairwise checks.
- Use human-review queues for solver `unknown`, low source alignment, or model instability.

### 11.3 Witnesses and cores

For satisfiable conflicts, show a witness context such as a synthetic patient satisfying both contexts. For unsatisfiable factual bundles, show an unsat core with rule IDs and source spans. For ASP results, show accepted/defeated rule derivations. For Lean results, show theorem/check names and replay commands.

### 11.4 Severity policy

Severity is a research triage label:

- `blocking`: formal inconsistency prevents package acceptance.
- `major`: hard modal/factual inconsistency with clear source spans.
- `moderate`: raw conflict resolved only by inferred priority or weak source alignment.
- `minor`: terminology/unit/convergence ambiguity.
- `info`: explanatory output, unsupported candidate, or extraction warning.

The report must avoid clinical truth claims. It should say “the formalized text implies…” or “these passages warrant review because…”

## 12. Evaluation design

### 12.1 Gold and fixture corpora

Create three corpora:

- `synthetic_core`: small JA/EN fixtures with known formal answers, contradictions, exceptions, temporal cases, and dose/unit cases.
- `licensed_pilot`: a narrow real Japanese guideline/textbook subset supplied by the user or available under appropriate terms.
- `adjudicated_gold`: clinician/formalist-reviewed source spans paired with accepted CGIR.

Gold annotation item:

```json
{
  "source_span_ids": [],
  "gold_cgir": {},
  "clinical_reviewer_notes": "",
  "formalist_reviewer_notes": "",
  "adjudication_status": "accepted|needs_revision|rejected"
}
```

### 12.2 Metrics

Core metrics:

- extraction coverage by page/section/table;
- schema-valid candidate rate;
- type-valid candidate rate;
- source-aligned atom rate;
- canonical idempotency pass rate;
- syntactic identity rate across reruns;
- canonical-hash convergence rate;
- formal-equivalence convergence rate;
- Lean check pass rate;
- SMT replay pass rate;
- ASP replay pass rate;
- contradiction benchmark precision/recall/F1;
- revision-candidate reviewer yield;
- proportion of findings with witness/core/proof;
- human adjudication agreement.

### 12.3 Metamorphic tests

Implement property-based/metamorphic tests:

- Paraphrase invariance for controlled Japanese surface variants.
- JA source plus EN gloss must formalize to equivalent CGIR where the gloss is accurate.
- Rule order invariance.
- Independent CQ merge commutativity.
- Unit conversion invariance.
- Table row order invariance where row order is semantically irrelevant.
- Exception wording variants map to the same priority/defeater structure.
- Re-formalizing accepted CGIR through an informalized summary returns equivalent CGIR.
- Removing an irrelevant paragraph leaves existing rule hashes unchanged.

### 12.4 Manuscript artifacts

The pipeline must export:

- run manifests;
- schema and compiler versions;
- corpus summary tables;
- metric tables as CSV/JSON;
- conflict/revision candidate tables;
- formal replay commands;
- figures data for pipeline diagrams and convergence plots;
- anonymized synthetic examples suitable for publication.

## 13. UI requirements

Build a minimal bilingual JA/EN web UI after the headless pipeline works.

Required views:

- Run overview: documents, extraction coverage, accepted rules, checks, revision candidates.
- Document/source view: original Japanese spans with highlights and table locations.
- Rule view: JA source, EN gloss, CGIR rendering, concept/code bindings, modality, strength, certainty, exceptions.
- Conflict view: involved spans/rules, conflict category, severity, witness/core/derivation, suggested review question.
- Formal artifact view: Lean/SMT/ASP snippets and replay command.
- Filters: document, rule kind, severity, backend, status, concept, source section.
- Export: JSON, CSV, and static HTML report bundle.

UI copy must state research use only. It must show “source-grounded formalization” rather than “clinical recommendation.” It should help authors revise text; it should not advise clinicians.

## 14. Repository shape

Recommended layout:

```text
/cgir_spec/              JSON Schema, examples, canonicalization notes
/src/cds_auto/           Python package
  /ingest/               source adapters
  /segment/              span graph builders
  /terminology/          code/alias/unit normalization
  /formalize/            LLM provider interface and prompt packs
  /cgir/                 Pydantic models, canonicalizer, validators
  /compile/              Lean/SMT/ASP/FHIR/CQL emitters
  /verify/               solver runners and replay parsers
  /report/               CheckResult and RevisionCandidate builders
  /api/                  FastAPI backend
/lean/ClinicalIR/        Lean object language and generated packages
/ui/                     Vite/React/TypeScript frontend
/tests/                  unit, property, metamorphic, integration tests
/fixtures/               synthetic public fixtures
/corpus/                 gitignored raw licensed sources + manifests
/runs/                   gitignored generated run outputs
/docs/                   manuscript-oriented technical notes
```

CLI commands:

```text
cds-auto ingest <source> --out runs/<run>/ingest
cds-auto segment runs/<run>/ingest --out runs/<run>/spans
cds-auto formalize runs/<run>/spans --schema cgir.v0 --out runs/<run>/candidates
cds-auto canonicalize runs/<run>/candidates --out runs/<run>/cgir
cds-auto compile runs/<run>/cgir --targets lean,smt,asp --out runs/<run>/formal
cds-auto verify runs/<run>/formal --out runs/<run>/checks
cds-auto report runs/<run> --out runs/<run>/report
cds-auto ui runs/<run>/report
cds-auto evaluate runs/<run> --gold fixtures/gold --out runs/<run>/eval
```

## 15. Implementation phases

### Phase 0: Reproducible formal smoke tests

Build a tiny probe before full pipeline code:

- Pydantic CGIR fragment with stable canonical hash.
- Z3 and cvc5 contradiction/unsat-core examples.
- Lean file proving/executing a tiny modal conflict check.
- clingo program showing raw conflict and priority-resolved conflict.
- Vite/npm and Python environment smoke checks.

Commit these as permanent regression fixtures.

### Phase 1: CGIR schema and canonicalizer

Implement strict Pydantic models, JSON Schema export, canonical JSON serialization, stable IDs, unit normalization skeleton, formula normalization, and property tests. This phase is the foundation. All later phases should adapt to it.

Done when synthetic rules round-trip, canonicalize idempotently, and produce stable hashes across reordered input.

### Phase 2: Ingestion and segmentation

Implement local PDF/HTML/XML/EPUB/Markdown ingestion, TextSpan graph, table preservation, source manifests, and extraction coverage reports.

Done when synthetic Japanese documents and at least one real provided document produce stable spans with traceable page/section/table coordinates.

### Phase 3: Formal compilers

Implement CGIR to SMT-LIB, Lean data, and ASP facts/meta-program. Add replay commands and parser for solver output. Keep generated artifacts readable and deterministic.

Done when fixtures detect: hard modal conflict, resolved exception, impossible dose interval, non-conflict due disjoint context, and temporal cycle.

### Phase 4: LLM formalizer

Implement provider-neutral structured-output formalizer. Use prompt packs with schema, examples, retrieval context, and source-alignment requirements. Add candidate clustering and status promotion. Provide a mock provider for tests.

Done when mock provider, checked fixtures, and one live provider adapter all produce candidate CGIR that passes validation. Live provider tests should be optional in CI.

### Phase 5: Evaluation harness

Implement gold-corpus schema, metric computation, metamorphic tests, and report exports. Add synthetic benchmark cases for contradiction precision/recall.

Done when `cds-auto evaluate` produces a full metric table and catches seeded regressions.

### Phase 6: UI

Implement minimal FastAPI + Vite/React UI using report JSON as input. Avoid complex state. Prioritize trace views over styling.

Done when a reviewer can inspect every revision candidate from source text to formal artifact to solver result.

### Phase 7: Manuscript support

Generate static report bundles, figure-ready CSV/JSON, and reproducibility instructions. Include a “methods artifact” page summarizing formal semantics, deterministic pipeline, evaluation metrics, and limitations.

## 16. Acceptance tests

Minimum end-to-end acceptance:

1. A Japanese synthetic guideline fixture with two overlapping rules produces a modal conflict with a concrete witness.
2. A fixture with a contraindication exception produces a raw conflict and an ASP-resolved conflict with defeated rule trace.
3. A dose fixture with incompatible bounds produces an unsat core naming exactly the involved source spans.
4. A disjoint-context fixture produces no conflict and emits the disjointness reason.
5. Reordering rules and JSON keys yields identical package hash.
6. Re-running canonicalization changes no bytes.
7. Lean checks pass for generated tiny examples.
8. Z3 and cvc5 replay commands succeed for SMT artifacts.
9. The UI displays source JA, EN gloss, CGIR, solver result, and suggested review question for each issue.
10. The evaluation harness reports idempotency and convergence metrics.

## 17. Risk controls

Source licensing: raw licensed content stays in gitignored corpus storage. Reports for manuscripts should use short excerpts only when rights allow, or synthetic/public examples.

Hallucinated formalization: all accepted atoms require source spans. Unsupported atoms become rejected candidates. LLM confidence never substitutes for source alignment.

Japanese ambiguity: preserve original text; store English glosses as review aids. Formal authority remains the Japanese source span plus accepted CGIR.

OCR errors: store OCR lane separately with confidence. Require manual or high-confidence validation before accepted formalization.

Unit and threshold errors: use exact rationals, canonical unit tables, and property tests for conversions.

Solver incompleteness: prefer decidable fragments. Treat `unknown`, timeout, or solver disagreement as review candidates. Record logic fragment and approximation.

Priority overreach: report raw conflicts even when a priority rule resolves execution. Avoid hiding genuine guideline ambiguity behind inferred exceptions.

Model drift: record model/provider metadata, prompt versions, schema versions, retrieval context, and candidate clusters. Compare model versions through evaluation runs rather than replacing baselines silently.

Clinical overclaiming: UI and reports frame outputs as formal inconsistency/revision candidates. Human clinical adjudication is required before manuscript claims about guideline correctness.

## 18. Future production alignment

Keep interfaces clean enough for later replacement:

- Patient context should align with JP Core/FHIRPath and eventually SS-MIX2/JP Core adapters.
- Clinical actions should map toward FHIR ActivityDefinition/Request resources.
- Knowledge packages should export to FHIR Clinical Reasoning/Library where possible.
- Execution traces should be compatible with future CDS Hooks/cards, but initial UI remains research-only.
- EHR notes, multimodal data, pharmacology graphs, comorbidity models, probabilistic risk models, and clinical world models should enter through new compiler/evaluator layers, not by weakening CGIR semantics.
- Production SaMD implementation may replace the codebase; preserve CGIR lessons, formal semantics, corpus, test fixtures, and evaluation methodology.

## 19. First task for the coding agent

Start by building Phase 0 and Phase 1. Use the smallest end-to-end vertical slice:

- Define the minimal CGIR Pydantic models for predicates, quantities, actions, modalities, rules, packages, check results, and revision candidates.
- Implement canonical JSON and stable IDs.
- Create four synthetic Japanese source spans.
- Compile two rules to SMT and detect a modal conflict witness.
- Compile two dose facts to SMT and detect an unsat core.
- Compile a priority exception case to ASP and show accepted/defeated rules.
- Generate a tiny Lean file that checks modal conflict by computation.
- Add pytest/Hypothesis tests for idempotency and replay.
- Produce a `runs/demo/report.json` that the later UI can consume.

After this slice passes, expand ingestion and LLM formalization. The formal slice is the core of the research contribution; build around it.
