# Japanese Clinical Guideline Autoformalization CDS POC — Agent Spec

Audience: Claude Fable 5 coding agent. This file is self-contained. Use the attached research documents as optional expansion material, but treat this spec as the controlling project brief.

## 0. Mission

Build a research-grade, headless proof-of-concept system that ingests Japanese clinical guideline/textbook text and transforms clinically relevant statements into a deterministic, explainable, verifiable, executable formal representation. The first artifact must detect logical incompatibilities and source-to-source factual inconsistencies and render a simple bilingual JA/EN review UI for manuscript support.

The initial artifact is a formalization and consistency-analysis system outside clinician-facing CDS. It must produce revision candidates for expert review; clinical advice belongs to later regulated deployments. The later ambition is a SaMD-certified real-time CDS over EHR data and clinician notes; preserve conceptual paths to that future while optimizing this phase for correctness, determinism, auditability, and publishable formal-methods novelty.

Core thesis: code is cheap; the durable assets are the corpus, formal semantics, canonical IR, provenance model, verification evidence, and evaluation methodology. Prefer rebuilding code over compromising the conceptual kernel.

## 1. Success criteria

The POC succeeds when a fresh checkout can run a deterministic pipeline over a small licensed Japanese source corpus and produce:

1. A content-addressed source-text ledger with exact provenance down to page/block/character offsets.
2. A canonical normalized IR corpus where every accepted rule parses, type-checks, canonicalizes idempotently, and links to exact source spans.
3. Machine-generated Lean, SMT-LIB, ASP, and report artifacts for each accepted rule cluster.
4. A verifier report that flags at least these classes: deontic action collision, mutually unsatisfiable conditions, satisfiable overlap between incompatible recommendations, threshold/unit conflicts, terminology grounding disagreement, and source-support ambiguity.
5. Minimal conflict explanations with solver witness models or unsat cores and source snippets in Japanese with English glosses.
6. A static bilingual UI showing documents, rules, provenance traces, solver evidence, and reviewer-facing revision candidates.
7. A repeatable evaluation harness measuring parse/type/proof pass rates, normalization idempotency, multi-run convergence, conflict precision/recall, provenance completeness, and reviewer adjudication status.

## 2. Scope boundary for the initial work

Initial scope:

- Text-only guidelines and textbooks from allowlisted local files or licensed URLs.
- Japanese primary text; English labels/glosses for UI and manuscript readers.
- Recommendation-level and statement-level extraction, especially CQ/PICO/GRADE-style sections where available.
- Deterministic formalization ledger from frozen LLM outputs. LLM calls may be nondeterministic; accepted artifacts must be deterministic once cached.
- Batch/offline execution only.
- Research review workflow only.

Future-compatible but secondary:

- FHIR JP Core patient-data binding, CDS Hooks, SMART, CPG-on-FHIR, and CQL export.
- Drug safety sources, PMDA package inserts, real-world evidence, and longitudinal EHR evaluation.
- Temporal pathways, multimodal data, comorbidity interactions, pharmacology, probabilistic risk models, and patient-state world models.

Initial implementation must keep these future paths visible as interfaces and metadata, while avoiding premature production architecture.

## 3. Non-negotiable design principles

Always preserve the original source text and a reversible chain from every formal atom back to exact source spans.

Always separate these concepts: source claim, extracted clinical proposition, normalized concept grounding, formal rule, verifier result, and reviewer judgment.

Always treat the LLM as a proposal generator. Parsers, type checkers, canonicalizers, proof checkers, solvers, and human adjudication decide acceptance.

Always use canonical serialization for accepted artifacts. Equivalent unordered structures must normalize to the same bytes and same content hash.

Always use explicit uncertainty states. Ambiguity, missing definitions, weak recommendation language, low evidence certainty, and conflicting terminology mappings must become typed IR fields instead of prose comments.

Always make every conflict report reconstructible from stored artifacts without re-calling an LLM.

Always optimize for the manuscript claim: unprecedented confidence in clinical autoformalization through explicit formal semantics, deterministic normalization, solver-backed collision detection, and clinician/formalist adjudication.

## 4. Stack decisions

Use this stack unless direct experimentation proves a replacement strictly improves the formal goals.

| Problem area | Primary choice | Role |
|---|---|---|
| Orchestration and NLP glue | Python | Fast iteration, Japanese NLP, LLM/API orchestration, extraction pipelines, evaluation harness. |
| Deterministic formal core | Rust or Python with a clean boundary; prefer Rust once schema stabilizes | Parser, type checker, canonicalizer, hash ledger, CLI stability. Early Python is acceptable if property tests guard determinism. |
| Source text extraction | PyMuPDF for PDFs; BeautifulSoup/lxml/trafilatura-style extraction for HTML; OCR only as a tagged fallback | Retrieve textual content with layout provenance. Image OCR output must be labeled lower-confidence and excluded from primary claims until validated. |
| Japanese tokenization | SudachiPy + Sudachi dictionary; project-local domain lexicon overlays | Deterministic segmentation, lexical normalization, candidate terminology matching. |
| Canonical IR | Custom GLK: Guideline Logic Kernel canonical JSON AST + JSON Schema + optional prefix surface DSL | Primary durable representation. JSON AST is authoritative; DSL is an authoring/debug surface. |
| Constrained generation | JSON Schema / grammar-constrained decoding where supported; otherwise strict parse-repair loops | Ensure every LLM proposal is syntactically valid or rejected. |
| Formal proof kernel | Lean 4 | Define GLK semantics, prove/check small semantic invariants, create machine-checkable certificates for kernel properties and selected rule facts. |
| SMT backend | SMT-LIB with Z3 as primary and cvc5 as independent cross-check | Satisfiability, overlap, arithmetic/unit/temporal constraints, unsat cores, witness models. |
| Defeasible/default reasoning | ASP with Clingo | Exceptions, priority, default applicability, explanation-friendly answer sets. |
| Minimal revision localization | MaxSAT via PySAT RC2 or equivalent, with Z3 Optimize as fallback | Minimal unsatisfied clause sets, minimal correction sets, soft weighting by recommendation/evidence strength. |
| Large rule materialization | Soufflé Datalog when scale requires it | Closure over terminology/rule graphs and fast batch analysis. |
| Clinical interoperability | FHIR JP Core concepts for future patient data; CQL/ELM and CPG-on-FHIR as export targets | Interop layer secondary to GLK primary semantics. |
| Artifact storage | Content-addressed filesystem + SQLite manifest; DuckDB for analytic views | Immutable reproducibility plus simple queryability. |
| UI | Static TypeScript site reading generated JSON artifacts | Bilingual review UI with no live clinical execution. |
| Testing | Pytest/Hypothesis, golden corpus tests, Lean checks, solver checks, schema conformance, snapshot tests | Regression and evidence generation. |

## 5. Why GLK instead of CQL/FHIR/Lean as the sole IR

CQL/ELM is excellent for clinical logic exchange and canonical AST comparison, while GLK provides the mechanized semantics required for the research kernel.

FHIR Clinical Reasoning/CPG-on-FHIR is excellent packaging and workflow structure, but it delegates core logic to CQL/FHIRPath and is too interoperability-oriented to expose every proof obligation.

Lean is the strongest proof target, but direct NL-to-Lean clinical formalization is too unconstrained as the primary extraction interface and too proof-engineering-heavy for every guideline sentence.

GLK must sit between clinical prose and formal backends. It is small, typed, provenance-rich, and intentionally agent-writable. It compiles to Lean for semantic certificates, SMT for decidable collision checks, ASP for defeasible reasoning, CQL/FHIR for interoperability, and UI JSON for review.

## 6. GLK design

GLK is a canonical AST designed for agent authorship and machine verification. Its surface syntax, if implemented, must follow these rules:

- Prefix-only operators.
- One semantic operation per node.
- No precedence rules.
- Explicit tags for modality, strength, certainty, temporal scope, exceptions, and provenance.
- Stable machine-readable error codes for every validation failure.
- Grammar-aware completions/logit masks once parser states are stable.
- JSON AST remains the single source of truth.

### 6.1 Core entities

Minimum GLK objects:

```text
SourceDocument(id, kind, title_ja, title_en?, version?, publisher?, rights, retrieval, content_hash)
TextSpan(id, document_id, page?, block?, start_offset, end_offset, raw_text, normalized_text, span_hash)
Segment(id, document_id, span_ids, heading_path, cq_id?, section_kind, language, text_hash)
Concept(id, system, code, display_ja, display_en?, aliases, mapping_confidence, mapping_source)
Quantity(value_rational, unit, comparator?)
Atom(predicate, args, concept_bindings?, quantity?, temporal?)
Condition(ast)
Action(ast)
Rule(id, source_spans, modality, strength, certainty, condition, action, exceptions, priority, scope, evidence, provenance)
FormalArtifact(id, rule_ids, target, text, hash, check_status, diagnostics)
Conflict(id, kind, rule_ids, source_spans, solver, status, witness?, unsat_core?, minimal_repair?, explanation_ja, explanation_en)
ReviewerJudgment(id, target_id, reviewer_role, label, notes, timestamp)
```

### 6.2 Modal and recommendation vocabulary

Required modalities:

```text
STATE      factual/definitional claim
OBLIGE     should/strongly recommend/must
FORBID     prohibit/contraindicated/avoid
PERMIT     may/can be considered
CONSIDER   weak/conditional recommendation, candidate option
MONITOR    surveillance/follow-up obligation
REFER      referral/consultation action
```

Required strength/certainty fields:

```text
rule_kind: strict | defeasible | defeater | preference | evidence_statement
recommendation_strength: strong_for | weak_for | weak_against | strong_against | none | unknown
certainty: high | moderate | low | very_low | unknown
priority: integer or explicit superiority relation
```

GRADE certainty and recommendation strength are metadata plus optional soft weights. Treat them as qualitative fields unless a later probabilistic model explicitly defines a quantitative interpretation.

### 6.3 Condition/action AST

Use a typed logical AST:

```text
true
false
atom(predicate, args)
and([expr...])
or([expr...])
not(expr)
implies(lhs, rhs)
exists(var, type, body)
forall(var, type, body)
compare(lhs, op, rhs) where op ∈ {eq, ne, lt, le, gt, ge}
interval(relation, lhs_interval, rhs_interval)
within(duration, event_a, event_b)
has_concept(entity, concept_id)
has_value(observation, comparator, quantity)
perform(action_type, target, parameters)
```

Represent numeric values as exact rationals or decimal strings plus normalized unit metadata. Normalize units through UCUM-compatible mappings and Japan-specific code systems where possible. Store the original Japanese unit string separately.

### 6.4 Provenance model

Every accepted GLK node must include:

```text
source_span_ids
source_text_hashes
extractor_version
prompt_hash
model_id
model_output_hash
normalizer_version
schema_version
terminology_snapshot_hashes
validator_diagnostics
human_review_status
```

For each rule, retain the exact Japanese evidence text. English translations are display artifacts subordinate to Japanese provenance.

## 7. Canonicalization and determinism

Canonicalization must be a first-class compiler pass.

Rules:

- Preserve raw source bytes and raw extracted text.
- Normalize comparison text with Unicode NFKC, stable whitespace rules, and explicit Japanese punctuation handling.
- Sort order-insensitive arrays by semantic keys.
- Preserve order-sensitive arrays such as document flow, paragraph order, temporal sequences, and ordered treatment steps.
- Generate IDs from canonical content plus type namespace; avoid random IDs in accepted artifacts.
- Serialize canonical GLK with RFC 8785-style canonical JSON.
- Hash every artifact with SHA-256 or BLAKE3 and record the algorithm.
- Store every LLM output before parsing; accepted outputs must be replayable from cache.
- Treat model temperature, seed, prompt, schema, tool versions, and source snapshots as run-ledger fields.
- Property-test `normalize(normalize(x)) = normalize(x)`.
- Snapshot-test canonical bytes for representative corpus items.

The system can be deterministic even when model calls vary. Determinism begins at the cached proposal boundary: the same frozen source snapshot and proposal ledger must yield the same accepted IR, solver outputs, and UI report.

## 8. Pipeline architecture

Implement the pipeline as explicit stages with immutable inputs/outputs.

```text
00_source_manifest
01_retrieve_snapshot
02_extract_text
03_segment_text
04_detect_statement_candidates
05_ground_concepts
06_propose_glk_candidates
07_parse_typecheck_normalize
08_compile_formal_targets
09_verify_and_collision_scan
10_generate_explanations
11_build_static_ui
12_evaluate_and_export_metrics
```

Each stage must write:

```text
stage_manifest.json
inputs.json
outputs.json
logs.jsonl
diagnostics.jsonl
artifact files addressed by content hash
```

Each stage must be rerunnable. If inputs have the same hashes, outputs must match exactly or the stage must emit a determinism failure.

## 9. Source retrieval and text extraction

Sources must enter through `sources.yaml`:

```yaml
sources:
  - id: local_minds_hf_2025
    kind: guideline_pdf
    language: ja
    path: data/raw/hf_guideline.pdf
    rights: licensed_for_research
    expected_hash: null
    priority_sections: [CQ, 推奨, 禁忌, アルゴリズム, Evidence-to-Decision]
```

Retrieval rules:

- Use allowlisted local files by default.
- Use remote retrieval only when rights metadata is explicit.
- Store raw bytes exactly once by content hash.
- Record URL, access time, ETag/Last-Modified when available, license/permission status, and parser configuration.

Extraction rules:

- PDF extraction must preserve page number, block order, character offsets when available, and layout confidence.
- HTML extraction must preserve headings, list/table structure, source URL, and DOM path.
- Tables must become structured table artifacts plus raw text spans.
- OCR must be a separate extraction method with confidence and engine metadata.
- Extraction quality failures must become reviewable diagnostics instead of silent omissions.

## 10. Japanese clinical normalization and grounding

Use deterministic lexical preprocessing before LLM formalization:

1. Unicode and whitespace normalization.
2. Sentence/paragraph segmentation with source offsets preserved.
3. Sudachi tokenization and dictionary forms.
4. Section-kind classification using headings and lexical cues.
5. Candidate clinical term detection with local dictionary overlays.
6. Terminology grounding to Japanese-relevant systems.

Minimum terminology systems to support as interfaces:

```text
MEDIS disease master
ICD-10 Japan / ICD-11 mappings
YJ and HOT medication codes
JLAC10/JLAC11 laboratory codes
MERIT-9/UCUM units where applicable
MedDRA/J for adverse events
FHIR JP Core profile/code references for future patient data
PMDA package-insert section anchors for drug facts
```

A concept grounding may have multiple candidates. GLK must store candidate sets, selected concept, confidence, source, and reviewer status. Use canonical concept IDs in formal logic; keep Japanese labels for display.

## 11. Autoformalization strategy

Use a staged, verifier-guided process.

### 11.1 Candidate detection

Classify segments into:

```text
recommendation
definition
eligibility_condition
contraindication
exception
dose_or_threshold
temporal_followup
monitoring
referral
outcome_or_evidence
background_only
ambiguous
```

Only clinically actionable or formally useful segments advance to GLK. Background text remains searchable provenance.

### 11.2 Structured proposition layer

Before GLK, produce an intermediate proposition record:

```text
proposition_id
source_span_ids
claim_type
population
condition
intervention_or_action
comparator?
recommendation_direction
strength
certainty
exceptions
temporal_scope
numeric_thresholds
concept_candidates
ambiguity_notes
```

This layer is intentionally close to the source text. It improves auditability and provides a clinician-friendly adjudication surface.

### 11.3 GLK candidate generation

For each proposition, generate `k` GLK candidates under a strict schema. Use multiple model routes if available. Every candidate must pass:

```text
schema validation
GLK type check
canonicalization
source-span completeness
concept-grounding completeness or explicit unknown marker
formal target compilation
at least one verifier-backed semantic check
```

Cluster candidates by canonical semantic equivalence. Promote a candidate only when it is in the majority semantic cluster or has formal superiority after verification. Low convergence must create a review item.

### 11.4 Repair loop

Each failed candidate receives structured diagnostics:

```text
ERR_SCHEMA_*
ERR_TYPE_*
ERR_CONCEPT_*
ERR_MODALITY_*
ERR_TEMPORAL_*
ERR_UNIT_*
ERR_LEAN_*
ERR_SMT_*
ERR_ASP_*
ERR_PROVENANCE_*
```

The repair prompt must include only the source proposition, schema excerpt, failed candidate, and diagnostics. Successful repairs must be revalidated from scratch.

## 12. Formal semantics and verification backends

### 12.1 Lean semantic kernel

Lean defines the reference semantics of GLK:

```text
ClinicalState
Concept
Predicate
Action
Modality
Strength
Rule
Applicability
Conflict
Defeat
Priority
TemporalRelation
```

Lean goals for the POC:

- GLK well-formedness predicates.
- Deontic collision definition.
- Defeasible override definition for simple priorities.
- Normalization invariants where practical.
- Executable examples generated from sample GLK.
- Machine-checkable theorem stubs or checks for each conflict kind.

Lean is the trusted semantic anchor. SMT and ASP are efficient decision procedures compiled from GLK; Lean definitions must mirror their intended meaning on the supported fragment.

### 12.2 SMT-LIB / Z3 / cvc5

Use SMT for decidable checks:

```text
condition satisfiability
condition overlap between rules
mutual exclusion of clinical states
numeric threshold contradiction
unit-normalized arithmetic comparisons
temporal interval consistency
source-to-source factual conflict encoded as incompatible predicates
```

Emit SMT-LIB with named assertions for every rule, source span, and subcondition. Required outputs:

```text
sat/unsat/unknown
model or unsat core
solver version
logic fragment
named assertions used
translation hash
```

Use Z3 as primary. Cross-check high-severity findings with cvc5. Mark solver disagreement as a review item.

### 12.3 ASP / Clingo

Use ASP for default, defeasible, and exception semantics:

```text
applicable(rule)
defeated(rule)
overrides(rule_a, rule_b)
recommended(action)
forbidden(action)
collision(rule_a, rule_b)
```

ASP outputs must include answer sets and shown predicates sufficient to explain why a recommendation applies, is defeated, or collides.

### 12.4 MaxSAT minimal repair

Translate conflict clusters into hard/soft clauses:

```text
hard: schema/type/provenance constraints, strict facts, mutually exclusive ontology facts
soft: defeasible recommendations weighted by strength/certainty/source authority/reviewer status
```

Report minimal correction sets as revision candidates. The system must leave source editing to reviewers. The UI should show which source statements would need clarification, exception refinement, or priority adjustment to resolve the conflict.

### 12.5 FHIR/CQL export

Export only after GLK validation. Treat CQL/ELM and FHIR Clinical Reasoning artifacts as interoperability products:

```text
GLK Rule -> CQL expression/library where expressible
GLK Action -> FHIR ActivityDefinition/PlanDefinition action where expressible
GLK Concept -> FHIR CodeableConcept/ValueSet/ConceptMap where expressible
GLK source metadata -> FHIR Library/PlanDefinition relatedArtifact/extension where expressible
```

Export lossiness must be explicit. A CQL/FHIR artifact may be incomplete if GLK uses semantics beyond CQL/FHIR; record `loss_report.json`.

## 13. Conflict taxonomy

Minimum conflict kinds:

```text
DEONTIC_ACTION_COLLISION
  Same or equivalent action is OBLIGE and FORBID under satisfiable overlapping conditions.

STRICT_FACT_CONTRADICTION
  Source statements assert incompatible facts after concept grounding.

CONDITION_UNSATISFIABLE
  A rule's own antecedent cannot be satisfied.

THRESHOLD_CONFLICT
  Numeric recommendations differ after unit normalization in a clinically meaningful direction.

TEMPORAL_CONFLICT
  Timing constraints cannot be jointly satisfied.

EXCEPTION_GAP
  One source contains an exception that would resolve another source's collision but the formal relation is unstated.

PRIORITY_AMBIGUITY
  Defeasible rules conflict and superiority/authority/date/strength metadata is insufficient.

TERMINOLOGY_GROUNDING_CONFLICT
  Different plausible concept mappings change the formal conclusion.

SOURCE_SUPPORT_MISMATCH
  GLK candidate cannot be traced to the cited source text.

FACTUAL_SOURCE_DISAGREEMENT
  Guideline/textbook claim conflicts with an authoritative structured source such as PMDA package-insert facts or a newer guideline version.
```

Each conflict must include severity, confidence, source authority, reviewer status, and an explanation based on formal artifacts. Use language such as `warrants review` and `potential inconsistency`; reserve `contradiction` for formally proven contradictions within the supported semantics.

## 14. Explanation and UI requirements

The UI is a static bilingual review report.

Required screens:

```text
Corpus overview: documents, versions, extraction status, formalization status.
Rule browser: source text, proposition layer, GLK, concept groundings, formal artifacts.
Conflict list: kind, severity, rule IDs, source snippets, witness/unsat core, reviewer state.
Conflict detail: JA source excerpts, EN gloss, formal explanation, solver artifacts, minimal repair candidates.
Metrics: pass rates, convergence, idempotency, reviewer adjudication summary.
Export: JSON/CSV/Markdown bundles for manuscript tables.
```

Bilingual behavior:

- Japanese source is primary.
- English is a gloss for review and manuscript readability.
- English text must link to the Japanese span it summarizes.
- Formal explanation should use stable bilingual labels for modalities, strengths, conflict kinds, and solver statuses.

UI actions can be local-only annotations in JSON during the POC. Production editing workflows are later-stage work.

## 15. Evaluation methodology

Create a gold-standard corpus incrementally.

Minimum annotation unit:

```text
source passage
structured proposition
GLK artifact
concept groundings
formal target outputs
reviewer adjudication trace
```

Reviewer roles:

```text
clinician
formalist / formal-methods reviewer
terminology reviewer
senior adjudicator for disagreements
```

Metrics:

```text
Extraction coverage: extracted textual content / expected textual content.
Source alignment: accepted GLK nodes with exact source spans.
Schema pass rate: candidates passing JSON Schema.
Typecheck pass rate: candidates passing GLK type checker.
Formal compile pass rate: GLK -> Lean/SMT/ASP success.
Verifier pass rate: generated checks returning sat/unsat as expected.
Idempotency: normalize(normalize(x)) == normalize(x); canonical hash stability.
Convergence: cluster purity across k candidates and model routes.
Semantic equivalence: accepted GLK vs gold GLK under normalization and solver checks.
Conflict precision/recall: synthetic benchmark plus adjudicated real conflicts.
Explanation quality: reviewer-rated traceability, readability, clinical actionability.
```

Evaluation datasets:

1. Synthetic micro-guidelines with known contradictions.
2. Japanese guideline passages with clear CQ/PICO/GRADE recommendations.
3. Drug contraindication/dosing passages from structured regulatory sources when licensing permits.
4. Real cross-guideline pairs selected by clinicians for likely conflict.
5. Ambiguous passages intentionally expected to fail convergence and enter review.

## 16. Repository layout

Use a structure close to this:

```text
repo/
  README.md
  CLAUDE.md
  pyproject.toml
  crates/                         # optional Rust core once stable
    glk-core/
  packages/
    ui/
  glk/
    schema/
    grammar/
    examples/
    lean/
    smt/
    asp/
  src/
    cdsaf/
      ingest/
      extract/
      segment/
      ground/
      propose/
      glk/
      compile/
      verify/
      report/
      eval/
  data/
    raw/                           # gitignored licensed files
    snapshots/                     # content-addressed raw bytes
    derived/                       # content-addressed stage artifacts
    terminology/                   # versioned snapshots
  runs/
    <run_id>/
      stage_manifests/
      diagnostics/
      artifacts/
      metrics.json
  tests/
    unit/
    property/
    golden/
    fixtures/
  docs/
    design/
    manuscript_tables/
```

## 17. CLI contract

Provide a single CLI entry point:

```bash
cdsaf doctor
cdsaf init-corpus sources.yaml
cdsaf ingest sources.yaml --out runs/<run_id>
cdsaf extract runs/<run_id>
cdsaf segment runs/<run_id>
cdsaf ground runs/<run_id>
cdsaf formalize runs/<run_id> --k 8
cdsaf verify runs/<run_id>
cdsaf report runs/<run_id> --out site/
cdsaf eval runs/<run_id>
cdsaf reproduce runs/<run_id>
```

`doctor` must check external tools, solver availability, Lean availability, schema validity, and terminology snapshots. `reproduce` must rerun deterministic stages from frozen artifacts and compare output hashes.

## 18. Seed smoke tests

Implement these before ingesting real documents.

### 18.1 Canonicalization

Input: two GLK JSON files with identical rules in different key/list order.

Expected:

```text
same canonical bytes
same hash
normalize(normalize(x)) == normalize(x)
```

### 18.2 Deontic collision

Japanese toy source:

```text
心不全患者にACE阻害薬の投与を強く推奨する。
ただし重度腎機能障害ではACE阻害薬の投与を禁忌とする。
```

Expected:

```text
R1: OBLIGE perform(start_medication, ACEI) if has_condition(heart_failure)
R2: FORBID perform(start_medication, ACEI) if has_condition(severe_renal_impairment)
SMT witness: heart_failure = true, severe_renal_impairment = true
ASP collision: collision(R1, R2)
Lean example: deonticCollision R1 R2 = true
UI conflict: DEONTIC_ACTION_COLLISION, warrants review because exception/priority must be explicit
```

### 18.3 Unsatisfiable condition

Rule condition:

```text
age >= 75 AND age < 65
```

Expected:

```text
SMT unsat
unsat core includes both threshold assertions
conflict kind CONDITION_UNSATISFIABLE
```

### 18.4 Terminology ambiguity

Source term maps to two plausible concepts.

Expected:

```text
GLK stores both candidates
formal rule uses selected concept only when confidence/reviewer status passes threshold
low-confidence mapping creates TERMINOLOGY_GROUNDING_CONFLICT or review item
```

## 19. Development workflow for Fable 5

Operate autonomously at high effort. Use subagents for independent workstreams: GLK schema, extraction, Lean kernel, SMT compiler, ASP compiler, UI, evaluation harness, and documentation. Continue working when reversible implementation choices follow from this spec. Pause only for destructive actions, legal/licensing decisions, or clinical judgments requiring a human.

Before reporting progress, tie each claim to local evidence: tests, command output, generated files, solver status, or checked artifacts. Record lessons in the project memory system, one lesson per file, when a result changes future work.

Use fresh-context verifier subagents or clean-room checks for major milestones. The verifier should read this spec and inspect artifacts directly instead of relying on the builder's summary.

Prefer concise decisions over option surveys. When a choice is necessary, choose the path that maximizes formal correctness and auditability. Document the reason in `docs/design/decisions/`.

Avoid preserving code for its own sake. If a module's abstractions obscure the formal model, replace it.

## 20. Milestones

### M0 — Formal kernel skeleton

Deliver:

```text
GLK JSON Schema
canonicalizer
property tests
Lean kernel sketch
SMT compiler for toy rules
ASP compiler for toy rules
seed smoke tests passing
```

### M1 — Source/text pipeline

Deliver:

```text
source manifest
PDF/HTML extraction
span ledger
segmenter
Japanese tokenization
extraction coverage diagnostics
```

### M2 — Proposition and GLK formalization

Deliver:

```text
candidate segment classifier
structured proposition schema
LLM proposal adapter with cached outputs
GLK parse/typecheck/normalize loop
repair diagnostics
multi-run convergence clustering
```

### M3 — Verification and conflict analysis

Deliver:

```text
Lean/SMT/ASP generation for accepted rules
condition satisfiability checks
overlap/collision checks
unsat core and model extraction
minimal repair prototype
conflict taxonomy implementation
```

### M4 — Bilingual static UI

Deliver:

```text
static report builder
rule browser
conflict browser
source-span viewer
metrics page
export tables
```

### M5 — Evaluation package for manuscript

Deliver:

```text
gold corpus annotation format
synthetic benchmark
real-source pilot corpus
metrics report
error analysis tables
reproducibility bundle
```

## 21. Manuscript-facing claims to enable

The implementation should support claims of this form:

```text
We define a provenance-preserving clinical guideline IR with mechanized semantics for a decidable fragment of deontic/defeasible recommendations.
We enforce deterministic normalization and content-addressed replay for LLM-generated autoformalizations.
We compile Japanese guideline recommendations to Lean, SMT, and ASP targets and use independent verifiers to detect formally characterized inconsistency classes.
We quantify multi-run semantic convergence and idempotency rather than reporting only raw extraction accuracy.
We provide clinician/formalist adjudication traces linking every detected inconsistency to source text and machine-checkable evidence.
```

Build toward these claims. Features that improve these claims come first; secondary features wait.

## 22. Later SaMD path

Keep future production constraints visible:

```text
FHIR JP Core patient-model bindings
SS-MIX2 import path
CDS Hooks/SMART integration
CQL/FHIR Clinical Reasoning export
APPI-aware PHI isolation
medical-device QMS evidence
risk management and clinical safety cases
shadow-mode evaluation
alert fatigue governance
human factors validation
```

The POC should produce artifacts useful to a future safety case: requirements traceability, verification evidence, versioned corpora, test results, provenance logs, and design decisions. Production runtime speed, uptime, EHR hooks, and hospital deployment are later-phase objectives.

## 23. First build order

Start here:

1. Implement GLK schema, canonicalizer, and content hash IDs.
2. Implement seed toy rules and smoke tests.
3. Implement Lean, SMT, and ASP compilers for the toy fragment.
4. Prove or check seed collision end-to-end.
5. Implement source ledger and text extraction.
6. Add Japanese tokenization and concept-grounding stubs.
7. Add cached LLM proposal interface.
8. Add convergence clustering and repair diagnostics.
9. Build verifier report JSON.
10. Build static UI from verifier report JSON.

At every step, keep the end-to-end path runnable. Expand semantics only when the previous fragment is deterministic, typed, verified, and explainable.
