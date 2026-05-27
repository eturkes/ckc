# SPEC: CKC — Clinical Knowledge Compiler

Audience: coding agent only. Optimize for deterministic parsing, formal semantics, executable evidence, reproducibility, and scientific novelty. Code is disposable. Durable assets and durable artifacts are schemas, normal forms, terminology graphs, formal corpora, proof obligations, certificates, evaluations, adjudications, and assurance evidence.

## 0. Mission

Build `CKC`: a research-grade, proof-carrying clinical knowledge compiler for Japanese clinical guidelines and medical textbooks.

Initial artifact: a headless system plus a small bilingual JA/EN result UI. The system ingests Japanese clinical text, extracts and indexes all textual content and every textual span, formalizes clinically relevant statements into deterministic executable representations, and reports source-grounded logical incompatibilities or factual inconsistencies that warrant human revision of the source text.

Initial operating mode: text-quality and formalization QA restricted to research use. All outputs are review candidates for clinician/formalist adjudication. Later CDS/SaMD deployment may discard the implementation while preserving CKC schemas, normal forms, corpora, terminology alignments, proof artifacts, conflict datasets, evaluation evidence, and assurance cases.

Ultimate ambition: a layered Japanese clinical knowledge corpus that can later evaluate EHR state, clinician notes, pharmacology, comorbidity, temporal trajectories, multimodal observations, and hospital workflow hooks under a regulated CDS/SaMD lifecycle. Keep late-stage architecture fluid; protect the formal evidence chain.

## 1. Design thesis

CKC is a compiler and assurance system.

Natural language is input. LLM output is candidate syntax. Accepted CKC output is the subset that passes the semantic firewall: source grounding, schema validation, CKC Normal Form, terminology alignment, profile admission, compiler generation, verifier execution, certificate generation, deterministic hashing, and replay.

The source of truth is `CKC Normal Form + certificate graph`. FHIR/CQL/ELM, DMN/FEEL, BPMN/ePath, openEHR/GDL, OWL/SKOS/SHACL, SMT-LIB, ASP, Datalog, Event Calculus, Lean, TLA+, Alloy/Forge, PRISM/JANI, and RDF are compilation/export/checking targets. They are valuable target languages. CKC Normal Form remains the semantic core.

Use agent-native language principles: canonical ASTs, explicit effects, finite grammars, stable diagnostics, typed artifacts, machine-checkable contracts, hash-chained provenance, translation validation, and verifier-first development.

Research contribution: the architecture and evidence chain showing deterministic, explainable, verifiable autoformalization of Japanese clinical prose. A model, prompt, UI, or codebase is an implementation detail.

## 2. Scope gates

### 2.1 Initial scope

The initial project must deliver:

- source ingestion with permission/version metadata;
- extraction and indexing of all textual content into addressable `SourceSpan` records;
- source-grounded formalization of a limited clinically relevant subset;
- deterministic CKC Normal Form and content hashes;
- executable/verifiable target artifacts;
- minimal conflict and inconsistency witnesses;
- bilingual JA/EN reports;
- a read-only UI over static report artifacts;
- a manuscript-ready reproducibility package.

### 2.2 Future continuity

The initial design must preserve a clean bridge to later production CDS/SaMD:

- JP Core and SS-MIX2 patient-data adapters;
- ePath/BPMN workflow packaging;
- audit-event mapping;
- drift and incident workflows;
- GSN/SACM assurance cases;
- ISO 14971 / IEC 62304 / IEC 62366 traceability;
- APPI-aligned privacy controls;
- PMDA-facing evidence strategy.

Future bridge code is demonstrative in this phase. Clinical deployment requires a separate QMS, clinical validation, cybersecurity/privacy program, usability engineering, hospital integration validation, and regulatory strategy.

## 3. Distilled domain commitments

This file is self-contained and immutable. You are to receive the designated commitments as the authoritative synthesis of the domain research. The underlying documents comprising this research are unavailable. Pivot from a commitment only if you identify a superior alternative during development, making note of it in a separate file and designating this file as a historical initial draft.

Core commitments:

- Minds/GRADE/CQ/PICO/EtD is the first guideline segmentation and evidence scaffold.
- Recommendation direction, recommendation strength, evidence certainty, and deontic projection are separate fields.
- MEDIS, HOT, YJ, JLAC, ICD, MedDRA/J, JP Core, SS-MIX2, ePath, and OMOP are distinct layers connected through explicit terminology bindings.
- Lucene/Pyserini BM25/BM25F is the sparse retrieval reproducibility oracle. Dense BGE-M3 and late-interaction retrieval improve recall/citation precision after parity fixtures exist.
- YomiToku is the Japanese OCR/layout baseline; MinerU is the disagreement checker for complex scientific/textbook PDFs; PyMuPDF/PDFium handles reliable embedded text.
- CQL/FHIR, DMN, BPMN/ePath, and openEHR are export/conformance targets.
- OWL/SKOS/SHACL/RDF named graphs provide terminology, provenance, structural validation, and source-scoped claim graphs.
- Deontic, defeasible, argumentation, paraconsistent, temporal, Event Calculus, SMT, ASP, Datalog, SAT/MUS, e-graphs, Lean, TLA+, and Alloy each have bounded roles.
- Gold corpora, semantic convergence, metamorphic tests, conformance suites, GSN assurance, SBOM/AIBOM, privacy, drift, and incident workflows are part of the research artifact.

## 4. Agent autonomy and implementation posture

You have full autonomy for implementation, tests, fixtures, refactors, and local scaffolding. Preserve concepts, schemas, normal forms, proof obligations, certificates, and evaluation fixtures over code.

Prefer clean reimplementation over patching a flawed design. When replacing a concept, run a reproducible experiment, update this spec, create a migration note, and preserve prior artifacts that may support publication claims.

Use current stable toolchains and libraries. Record resolved tool versions using idiomatic, but latest-gen, package managers. Use the latest project-wide command runners and orchestration tools.

Every feature reaches completion when it has typed I/O, canonical artifacts, provenance, structured diagnostics, deterministic tests, replay manifest, source-span mapping, and assurance evidence.

## 5. Core stack

### 5.1 Deterministic kernel

Use Rust for:

- CLI and orchestration: `ckc`;
- CKC types, semantic profiles, normal form, identifiers, artifact store, certificates;
- compiler frontends and verifier orchestration;
- replay, reports, static API serving, assurance artifacts;
- deterministic local joins through DuckDB.

Rust types are the in-repo schema source where practical. Generate JSON Schema from Rust or maintain tests proving Rust/JSON Schema equivalence. Kernel functions should be compatible with future verification: small pure functions, explicit preconditions, deterministic data structures, and explicit state.

Use Python for ML/OCR/NLP adapters and external ecosystem bindings. Python workers exchange explicit JSON, JSONL, Arrow, or Parquet. Rust owns canonical semantics.

Use TypeScript + SvelteKit for the read-only bilingual UI. The UI renders verified report artifacts. LLM, solver, and verifier invocations occur in pipeline commands.

### 5.2 Canonical interchange and storage

Use RFC 8785 canonical JSON bytes for accepted CKC artifacts. Default artifact ID: `sha256:<hex>`. BLAKE3 may accelerate large-file internals when the manifest records both BLAKE3 and SHA-256 identities.

Use:

- JSONL for streams;
- Arrow/Parquet for extraction, retrieval, and evaluation tables;
- DuckDB for local report joins;
- RDF/Turtle/TriG for semantic graph exports;
- a content-addressed filesystem as the primary store.

Every accepted artifact contains schema version, producer version, deterministic command manifest, source input hashes, parent hashes, stage, semantic profiles, content hash, certificate IDs, and replay command.

Stable identifiers use lowercase ASCII slugs plus source/content hash suffixes. Original Japanese strings remain labels and source text.

### 5.3 Agent-writable CKC surface syntax

Accepted CKC output is canonical JSON. Add a compact optional surface syntax only as an authoring/intermediate target for LLMs and coding agents.

Surface syntax requirements:

- prefix AST form with explicit parentheses;
- one canonical spelling per constructor;
- explicit types, profiles, source spans, and effects;
- explicit precedence through parentheses;
- macro expansion into typed JSON before firewall gates;
- deterministic parser diagnostics with source offsets;
- round-trip tests: surface -> JSON -> normal form -> surface preview.

Example shape:

```text
(rule beta_lactam_sepsis_recommendation
  :profiles [CKC-Norm CKC-Defeasible]
  :when (and (dx sepsis) (adult patient))
  :norm (recommend-for :strength strong :certainty moderate
          (administer beta_lactam))
  :source [span_s1])
```

The surface language is a convenience layer. CKC Normal Form remains authoritative.

## 6. Source substrate

### 6.1 Source classes

Register every source as a versioned, permission-tracked artifact.

Initial source classes:

- Minds and Japanese society guidelines organized around CQ, PICO, SR, EtD, GRADE recommendation strength, certainty, and rationale;
- medical textbooks and review chapters with explicit permission or usable access;
- PMDA package inserts, review reports, safety communications, and adverse-event material when permitted;
- hospital local policies in later private deployments.

Minds/GRADE CQ is the preferred segmentation unit. A recommendation may span paragraphs, tables, footnotes, figures, and evidence summaries. CKC must link all supporting spans.

### 6.2 Japanese terminology graph

Build a versioned terminology graph with provenance, license status, mapping confidence, and update cadence.

Initial anchors:

- diseases: MEDIS standard disease master, ICD-10-JP, ICD-11 when officially usable;
- drugs: HOT, YJ, generic/brand labels, package-insert names;
- labs: JLAC11 preferred where available, JLAC10 for legacy mapping, original Japanese unit labels, UCUM-compatible normalized units;
- procedures/claims: K codes, receipt codes, DPC/PDPS crosswalks where relevant;
- adverse events: MedDRA/J and JADER when license permits;
- patient data: JP Core FHIR profiles, SS-MIX2 legacy exports;
- pathways: ePath OAT and Basic Outcome Master;
- international alignment: SNOMED CT, LOINC, ICD-11, RxNorm where licensed and explicitly mapped.

Every `TerminologyBinding` has `system`, `code`, `version`, `label`, `mapping_relation`, `status`, `provenance`, `confidence`, `license_status`, `valid_from`, and `valid_to` where available.

Binding statuses: `exact | broad | narrow | related | unmapped | ambiguous | deprecated | incoherent`.

## 7. Extraction, retrieval, and grounding

### 7.0 Text-totality invariant

The system must retrieve and index all textual content from each registered source, including running text, headings, tables, footnotes, captions, figure text, appendices, evidence summaries, recommendation statements, and bibliographic text when extractable. Formalization may remain selective; extraction and addressability are total over textual content.

### 7.1 Extraction

Extraction flow:

1. PyMuPDF/PDFium for reliable embedded PDF text.
2. YomiToku for Japanese OCR/layout, especially vertical text, image PDFs, tables, and society-specific layouts.
3. MinerU for complex scientific/textbook PDFs and extraction disagreement checks.
4. Marker as a tertiary Markdown/JSON fallback when fixture evidence shows it wins on a document class.

Extraction output preserves document hierarchy, section path, page coordinates, reading order, table structure, cell coordinates, footnotes, captions, figure text, raw text, normalized text, extractor votes, and confidence.

Every textual unit becomes a `SourceSpan`. The system may formalize a subset; it must index all textual spans.

### 7.2 Retrieval

Retrieval is evidence discovery. Proof comes from CKC, compilers, solvers, source spans, and adjudication.

Primary retrieval stack:

- sparse oracle: Lucene/Pyserini BM25/BM25F with Lucene index fingerprints;
- Japanese analyzer matrix: Kuromoji, SudachiPy, MeCab-UniDic; choose by qrel performance per corpus class;
- dense recall: BGE-M3 through Python worker;
- late interaction: JaColBERT/ColBERT-style retrieval for high-value citation precision;
- vector search: exact cosine over Arrow/Parquet for initial corpora; ANN after exact-search parity tests;
- fusion: reciprocal rank fusion;
- reranking: multilingual biomedical cross-encoder as advisory evidence;
- graph retrieval: ontology-grounded GraphRAG over CKC/RDF triples;
- premise retrieval: prior accepted CKC, terminology entries, and Lean/SMT/ASP fragments.

Rust/Tantivy/Lindera may mirror sparse retrieval after parity against the Lucene/Pyserini oracle on qrels, tokenization snapshots, rank fixtures, and index fingerprints.

Retrieval artifacts include query, decomposition, analyzer, tokens, sparse rank, dense rank, late-interaction evidence, fused rank, reranker rank, span IDs, index fingerprints, model IDs, corpus hash, and source permissions.

### 7.3 Grounding invariant

A generated claim becomes an accepted CKC object only when each clinical predicate, norm, quantity, temporal anchor, table value, terminology binding, and action parameter has source-span/table-cell support or is marked as a formally introduced helper with compiler justification.

Pre-hoc source spans and table cells are the grounding substrate. Post-hoc citations are review aids.

## 8. LLM autoformalization

LLMs generate candidate CKC objects.

Generation strategy:

- structured outputs against JSON Schema for closed models;
- grammar-constrained decoding for self-hosted models;
- retrieval-augmented generation over source spans, terminology graph entries, prior accepted CKC objects, and proof premises;
- verifier-guided repair loops from structured compiler/solver diagnostics;
- proof-oriented models for Lean assistance only; Lean kernel result is authority.

N-version semantic convergence:

1. generate k candidates per target;
2. vary prompt and model family when available;
3. canonicalize every candidate;
4. cluster by CKC Normal Form digest, symbolic equivalence, solver equivalence, e-graph equivalence, and round-trip JA/EN meaning preservation;
5. promote only candidates passing the semantic firewall and convergence thresholds;
6. retain minority clusters as ambiguity evidence.

Store prompts, model identifiers, decoding settings, input span hashes, candidate hashes, cluster IDs, validator failures, repair attempts, and accepted certificate IDs. Natural-language rationales are annotations. Source spans, proof artifacts, solver outputs, and adjudications carry authority.

## 9. CKC semantic profiles

Every CKC object declares profiles that determine allowed syntax, validators, compiler targets, and certificate requirements.

Profiles:

- `CKC-Text`: documents, spans, layout, tables, extraction manifests.
- `CKC-Term`: concepts, terminology bindings, mappings, synonym sets, e-graph classes, SKOS/OWL views.
- `CKC-Evidence`: CQ/PICO/EtD/GRADE, outcome importance, effect estimates, rationale, evidence tables.
- `CKC-Classical`: strict factual, eligibility, datatype, quantity, and definitional constraints.
- `CKC-Norm`: dyadic clinical norms `(context, direction, action)` with strength, certainty, exceptions, and source evidence.
- `CKC-Defeasible`: strict/defeasible/defeater rules, superiority, rebuttal, undercut, defeat, and repair.
- `CKC-Para`: four-valued/inconsistency-tolerant labels for conflict-local review.
- `CKC-Temporal`: intervals, anchors, durations, schedules, event traces, and history invariants.
- `CKC-Event`: Event Calculus narratives for longitudinal fluents and persistence.
- `CKC-Quant`: quantities, comparators, ranges, units, specimens, methods, and normalized arithmetic.
- `CKC-Decision`: decision tables, row conditions, hit policies, cell provenance, DMN/FEEL export.
- `CKC-Workflow`: pathway fragments, ePath OAT, BPMN-compatible states, tasks, outcomes, assessments.
- `CKC-Interop`: FHIR/CQL/ELM/FHIRPath/PlanDefinition/ActivityDefinition/EvidenceVariable/DMN/BPMN/ePath/openEHR exports.
- `CKC-Prob`: probabilistic facts, weights, risk models, stochastic transitions, rewards.
- `CKC-Cert`: certificates, proof artifacts, solver outputs, replay status, assurance links.
- `CKC-Audit`: observability spans, audit events, drift events, governance evidence.

## 10. Core object model

Implement exact schemas in Rust/JSON Schema. Use this object model as the semantic minimum.

`CorpusDocument`: `doc_id`, `title_ja`, `title_en?`, `source_type`, `publisher`, `society`, `edition`, `publication_date?`, `access_date?`, `license_status`, `content_hash`, `extraction_manifest_id`, `supersedes?`, `superseded_by?`.

`SourceSpan`: `span_id`, `doc_id`, `section_path`, `cq_id?`, `page?`, `bbox?`, `table_cell?`, `raw_text`, `nfkc_text`, `search_text`, `display_text`, `language`, `previous_span_id?`, `next_span_id?`, `extractor_votes[]`, `confidence`.

`ExtractedTable`: `table_id`, `doc_id`, `caption_span_id?`, `cell_span_ids[]`, `row_headers`, `column_headers`, `reading_order`, `extraction_votes[]`, `normalized_table_hash`.

`Concept`: `concept_id`, `label_ja`, `label_en?`, `semantic_type`, `terminology_bindings[]`, `egraph_class_id?`, `source_span_ids[]`.

`TerminologyBinding`: `system`, `code?`, `version?`, `label`, `status`, `mapping_relation`, `provenance`, `confidence`, `license_status`, `valid_from?`, `valid_to?`.

`ClinicalClaim`: `claim_id`, `claim_type`, `profiles[]`, `source_span_ids[]`, `pico?`, `etd?`, `evidence_atoms[]`, `rule_ids[]`, `decision_table_ids[]`, `workflow_fragment_ids[]`, `gloss_ja`, `gloss_en`, `status`.

`PICOFrame`: `population`, `intervention`, `comparator`, `outcomes[]`, `cq_id?`, `scope`, `exclusions[]`, `source_span_ids[]`.

`EtDFrame`: `benefits`, `harms`, `certainty`, `values`, `resources`, `equity`, `acceptability`, `feasibility`, `recommendation_direction`, `recommendation_strength`, `source_span_ids[]`.

`Rule`: `rule_id`, `profiles[]`, `kind`, `context`, `antecedent`, `consequent`, `norm?`, `priority_over[]`, `exceptions[]`, `temporal_scope?`, `population_scope?`, `source_span_ids[]`, `provenance`, `certificate_ids[]`.

`Norm`: `context`, `direction`, `action`, `recommendation_strength`, `evidence_certainty`, `original_modality_phrase_ja`, `deontic_projection`, `exception_policy`, `prima_facie_or_all_things_considered`.

`Action`: `action_type`, `target_concept`, `parameters`, `temporal_constraints`, `quantity_constraints`.

`DecisionTable`: `table_id`, `hit_policy`, `input_columns[]`, `output_columns[]`, `rows[]`, `source_table_id?`, `dmn_export_id?`, `certificate_ids[]`.

`DecisionRow`: `row_id`, `conditions[]`, `outputs[]`, `priority?`, `source_span_ids[]`, `cell_refs[]`.

`WorkflowFragment`: `workflow_id`, `workflow_type`, `states[]`, `transitions[]`, `outcomes[]`, `assessments[]`, `tasks[]`, `variance_rules[]`, `source_span_ids[]`.

`EventNarrative`: `event_types[]`, `fluent_types[]`, `happens[]`, `initiates[]`, `terminates[]`, `initially[]`, `holds_queries[]`, `source_span_ids[]`.

`PatientCase`: `case_id`, `case_type`: `synthetic | fixture | deidentified_later | live_later`, `facts[]`, `events[]`, `observations[]`, `medications[]`, `conditions[]`, `allergies[]`, `time_origin?`, `source_span_ids[]`, `privacy_status`. Initial cases are synthetic or fixture-only; PHI cases belong to later governed deployments.

`ExecutionWitness`: `witness_id`, `bundle_id`, `case_id?`, `context_facts[]`, `trace[]`, `applicable_rules[]`, `defeated_rules[]`, `violated_constraints[]`, `models[]`, `unsat_cores[]`, `source_span_ids[]`, `certificate_ids[]`. A CKC artifact is executable only when at least one target can produce a replayable witness, model, proof, or explicit unsat/core result.

`EvidenceAtom`: `evidence_type`, `source_span_ids[]`, `pico_ref?`, `effect_measure?`, `value?`, `unit?`, `confidence_interval?`, `certainty`, `outcome_importance?`, `table_cell_refs[]`.

`Conflict`: `conflict_id`, `conflict_type`, `severity`, `confidence`, `minimal_artifact_set[]`, `source_spans[]`, `normalized_view`, `witness?`, `repair_candidates[]`, `solver_evidence[]`, `argument_graph_id?`, `human_review_question_ja`, `human_review_question_en`, `classification`.

`ArgumentGraph`: `argument_graph_id`, `arguments[]`, `attack_edges[]`, `support_edges[]`, `undercut_edges[]`, `defeat_edges[]`, `extension_summaries[]`, `source_span_ids[]`.

`Certificate`: `certificate_id`, `certificate_class`, `input_artifact_hashes[]`, `compiler_hash?`, `solver_or_checker`, `command_manifest`, `result`, `proof_artifact_hashes[]`, `replay_status`, `diagnostics[]`.

`AssuranceNode`: `node_id`, `node_type`, `claim`, `evidence_artifact_ids[]`, `status`, `children[]`.

`AuditTrace`: `trace_id`, `stage_spans[]`, `model_invocations[]`, `retrieval_events[]`, `verifier_events[]`, `artifact_hashes[]`, `redaction_status`, `audit_export_refs[]`.

## 11. CKC Normal Form

CKC Normal Form is the semantic canonical layer before RFC 8785 serialization. Implement it as a deterministic rewrite pipeline.

Passes:

1. preserve raw source text;
2. normalize derived fields with Unicode NFKC, Japanese punctuation normalization, full/half-width unification, and whitespace normalization;
3. alpha-normalize variables;
4. sort commutative operands for `and`, `or`, unordered sets, unordered evidence sets, and unordered source-span sets;
5. preserve order for document order, temporal sequences, priority chains, pathways, workflows, and order-sensitive decision-table rows;
6. normalize action constructors;
7. normalize quantities and units;
8. normalize terminology through e-graph classes and terminology bindings;
9. normalize clinical modality through an auditable Japanese phrase lexicon while preserving original wording;
10. canonicalize decision-table rows/cells;
11. canonicalize argument/workflow graphs;
12. generate deterministic stable IDs from normalized content and source anchors;
13. order diagnostics deterministically.

Invariants:

- `NF(NF(x)) = NF(x)`.
- Reordered commutative antecedents produce identical semantic digests.
- Order-sensitive artifacts change digest when order changes meaning.
- Raw source preservation and semantic normalization are separate.
- Canonicalization preserves source support and records every rewrite.

### 11.1 Executable witness semantics

CKC is executable exclusively through target backends. Each executable profile must define one of these witness forms: `sat_model`, `unsat_core`, `asp_model`, `prolog_justification`, `decision_table_witness`, `event_trace`, `workflow_trace`, `shacl_report`, `owl_explanation`, `lean_theorem`, or `adjudication_record`. Each witness must map target symbols back to CKC node IDs and source spans. Future EHR execution uses the same witness schema with `PatientCase` inputs.

## 12. Semantic firewall and certificates

### 12.1 Gates

A candidate becomes accepted CKC output when all applicable gates pass:

1. syntax parse;
2. Rust/JSON Schema validation;
3. source-span and table-cell grounding;
4. terminology status assignment;
5. CKC Normal Form;
6. profile admission;
7. extraction/retrieval provenance checks;
8. compiler generation;
9. target parse validation;
10. verifier execution;
11. translation validation;
12. certificate generation;
13. deterministic hashing;
14. replay manifest update.

Each failed gate emits a structured diagnostic with source spans, artifact IDs, machine-readable code, JA/EN messages, repair hints, evidence artifacts, and responsible stage.

### 12.2 Certificate classes

Every accepted artifact carries the deepest achieved certificate class. Higher classes imply lower applicable classes.

- `C0-Parsed`: deterministic parse tree exists.
- `C1-Schema`: Rust/JSON Schema validation passed.
- `C2-Normal`: CKC Normal Form and canonical hash passed.
- `C3-Grounded`: clinical content has source-span/table/terminology grounding.
- `C4-Executable`: at least one compiler target executes/checks.
- `C5-Portfolio`: independent backends agree or produce structured disagreement.
- `C6-ProofObject`: proof/certificate artifact exists, such as cvc5 proof, SHACL report, OWL explanation, DMN witness, ASP model, Alloy/TLA counterexample.
- `C7-Kernel`: Lean proof or certified reflected decision procedure checks.
- `C8-Adjudicated`: clinician/formalist adjudication is linked.
- `C9-Assured`: assurance-case node references the artifact with live evidence and all defeaters resolved.

Research claims report certificate-depth distributions.

## 13. Formal verification portfolio

Use a primary/secondary hierarchy. Backends cross-check each other where semantics overlap.

### 13.1 Proof kernel

Primary: Lean 4.

Use Lean for CKC metatheory, high-value theorem obligations, selected conflict proofs, and reflected decision procedures. Start with shallow embeddings. Use Mathlib for arithmetic/order structures and CSLib-style LTS/reduction abstractions for workflow and temporal metatheory where useful. Accepted Lean files compile with zero `sorry` or `admit`.

### 13.2 SMT, optimization, and repairs

Primary proof-producing SMT: cvc5. Preserve proof artifacts where feasible.

Primary fast model/core/optimization backend: Z3. Use it for numeric thresholds, intervals, datatype constraints, bounded traces, unsat cores, counterexamples, Optimize/MaxSMT, and repair candidates.

Use SAT/MUS/MCS/hitting-set and MaxSAT/OMT formulations for minimal conflict sets and minimal repair proposals. Repair candidates are review prompts.

Use Bitwuzla for bit-vector/floating-point subproblems when the corpus actually requires them.

### 13.3 Nonmonotonic and argumentation reasoning

Primary: Clingo/ASP.

Use ASP for defeasible/default reasoning, strict-vs-defeasible priority, exceptions, abduction, repair sets, argument graph extraction, and Event Calculus fragments.

Use Dung-style argumentation as a view over structured rules. Export attack/support/undercut/defeat graphs and grounded/preferred/stable-style summaries as diagnostics.

Use s(CASP)/Prolog when goal-directed justification trees or Event Calculus queries are clearer than whole-program stable models.

### 13.4 Temporal and workflow reasoning

Use three temporal layers:

- interval layer: Allen-style relations and numeric bounds compiled to SMT;
- event layer: `Happens`, `Initiates`, `Terminates`, `HoldsAt`, persistence, clipping compiled to ASP/s(CASP);
- workflow layer: bounded state traces compiled to SMT/ASP, and protocol transition systems compiled to TLA+/Apalache or Lean LTS when useful.

### 13.5 Terminology, ontology, and equality

Use egglog/e-graphs for deterministic equivalence saturation over synonymy, spelling variants, Greek-letter variants, unit rewrites, brand/generic mappings, and cross-target canonicalization. Bound saturation and record extraction cost functions.

Use OWL 2 EL/SKOS for large terminology hierarchy and subsumption. Use ROBOT/ELK for profile validation, classification, and explanations. Use HermiT/Openllet for DL constructs outside EL only when needed.

Use SHACL for closed-world structural validation over CKC/RDF/FHIR-shaped data.

Use ontology alignment repair before accepting cross-system equivalence.

### 13.6 Decision tables and interoperability

Use DMN/FEEL as export and verification target for decision-table-shaped content. Check hit-policy validity, overlapping rows, missing/gap witnesses, shadowed rows, type/unit inconsistencies, and cell provenance.

Use CQL/ELM and FHIR Clinical Reasoning as interop exports for compatible fragments. Validate CQL-to-ELM idempotency and FHIR packaging, while preserving CKC as the semantic core.

Use BPMN/BPM+ Health and ePath OAT for future pathway packaging. Compile workflow semantics to bounded traces, TLA+/Apalache, Lean LTS, or FHIR PlanDefinition/Task/CarePlan exports depending on scope.

### 13.7 Probabilistic and world-model profiles

Initial CKC objects use GRADE certainty as ordinal evidence metadata. Probabilistic profiles are later extensions.

Add probabilistic profiles after deterministic profiles are stable:

- ProbLog/cplint/PRISM-style probabilistic logic for uncertain facts and noisy events;
- PRISM/Storm/JANI for stochastic pathways, screening policies, and treatment-policy risk models;
- deterministic residual checks by stripping probabilities and validating the remaining CKC fragment.

World-model or multimodal paradigms are late-stage research profiles. They must compile observations or latent-state claims back into source-grounded CKC evidence before affecting CDS outputs.

## 14. Compiler and translation-validation contract

Every compiler implements:

- input: CKC Normal Form bundle;
- output: target artifact plus `CompilationMap` from CKC node IDs to target node IDs;
- deterministic diagnostics;
- replay command;
- artifact hash;
- certificate class;
- target parse check;
- normalized target result.

Target compilers:

- CKC -> SMT-LIB;
- CKC -> cvc5 proof obligations;
- CKC -> Z3 Optimize/MaxSMT;
- CKC -> ASP/Clingo;
- CKC -> s(CASP)/Prolog;
- CKC -> Datalog/Soufflé/RDFox;
- CKC -> egglog;
- CKC -> Lean;
- CKC -> RDF/SHACL;
- CKC -> OWL/SKOS;
- CKC -> DMN/FEEL;
- CKC -> TLA+/Apalache;
- CKC -> Alloy/Forge;
- CKC -> FHIR/CQL/ELM;
- CKC -> ePath/BPMN/openEHR;
- CKC -> ProbLog/PRISM/JANI for late profiles.

Translation validation checks target parse, target symbol table, source-span mapping, repeated compilation identity, comparable-backend agreement, proof artifact replay/preservation, Lean compilation, and first-class reporting of backend disagreement.

## 15. Conflict and inconsistency taxonomy

### 15.1 Logical incompatibilities

Detect:

- recommendation-for and recommendation-against projections for the same normalized action under satisfiable shared context;
- contraindication conflicting with recommendation without priority or exception;
- strict consequents that jointly contradict;
- numeric thresholds with empty intersection;
- temporal schedules with empty feasible trace;
- unsafe concurrent Event Calculus fluents;
- state invariant violations in bounded traces;
- unsatisfiable terminology classes;
- incoherent ontology alignments;
- SHACL closed-world violations;
- same-PICO mutually exclusive recommendations;
- cyclic or contradictory priority relations;
- unresolved argument graph attacks under declared semantics;
- decision-table overlaps, gaps, shadowed rows, or hit-policy violations;
- workflow/pathway dead ends, unreachable tasks, or outcome/task contradictions;
- compiler backend disagreement;
- proof/certificate replay failure.

### 15.2 Factual inconsistencies

Detect:

- incompatible quantities, dates, definitions, classifications, table values, or evidence summaries;
- evidence table values disagreeing with recommendation text;
- generated claim lacking source support;
- source term mapped to incompatible concepts;
- English gloss changing clinical meaning relative to Japanese source;
- source edition/version claims conflicting with metadata;
- same-PICO cross-guideline disagreement in direction, strength, certainty, or rationale;
- package-insert contraindication conflicting with guideline recommendation without explicit exception;
- guideline drift from superseded editions.

### 15.3 Paraconsistent review mode

When sources conflict, localize the conflict slice and allow non-explosive review queries. Four-valued labels may mark an atom as supported, refuted, both, or unknown. Use this as a review/retrieval mode while preserving classical/defeasible certificates for accepted formal claims.

### 15.4 Conflict report item

Each item includes conflict ID, type, severity, confidence, minimal conflict set, JA exact source spans, EN gloss, normalized CKC view, witness context/trace/table row/model, solver/proof evidence, certificate IDs, argument graph, repair candidates, human review prompt in JA/EN, suggested source-revision question, and classification.

Classifications: `true_conflict | likely_ambiguity | extraction_error | formalization_error | terminology_error | interop_compiler_error | stale_source | needs_clinician_adjudication`.

## 16. Evaluation and scientific evidence

### 16.1 Gold corpus

Create a small adjudicated corpus before scaling:

- 2–3 Japanese guidelines or textbook chapters with permission/usable access;
- 50–150 recommendation, factual, table, and pathway spans;
- CQ/PICO-level slicing where possible;
- dual annotation by clinical reviewer and formal-methods reviewer;
- senior adjudication for disagreement;
- adjudicated CKC gold bundle;
- synthetic contradiction set derived from real spans;
- frozen train/dev/test split by guideline, society, topic, and recommendation type.

Store annotation envelopes that can hold CKC plus selected target IR renderings.

### 16.2 Metrics

Track extraction coverage, table-cell accuracy, source-span attribution precision/recall, PICO/EtD extraction F1, terminology binding accuracy, ambiguity rate, alignment-coherence repair rate, schema validity, normal-form digest stability, semantic convergence, compiler success by profile, proof/certificate replay, translation validation, backend agreement, decision-table defect precision, temporal/event conflict precision, synthetic/adjudicated conflict precision/recall, false-positive review burden, bilingual gloss preservation, certificate-depth distribution, human adjudication agreement, assurance defeater closure, and time/cost metadata.

### 16.3 Semantic equivalence

Evaluate equivalence at these levels:

1. byte identity;
2. CKC Normal Form AST identity;
3. alpha-equivalence;
4. e-graph equivalence;
5. SMT/ASP/Lean/DMN equivalence under shared theory;
6. round-trip JA/EN meaning preservation under clinician/formalist review.

Convergence rate is the fraction of candidate runs landing in the dominant accepted equivalence class. Low convergence is review evidence.

### 16.4 Metamorphic and adversarial tests

Generate deterministic variants:

- whitespace/layout changes;
- Japanese modality paraphrases;
- Greek-letter and kana/kanji variants;
- reordered commutative antecedents;
- equivalent units;
- table row/column perturbations respecting or violating hit policy;
- exception insertion/removal;
- priority reversal;
- temporal anchor perturbation;
- terminology version changes;
- JA -> EN -> JA translation perturbation;
- source edition supersession.

Semantic no-ops preserve normal-form digest or formal-equivalence class. Semantic edits produce specific diagnostics.

### 16.5 Retrieval evaluation

Maintain qrels for Japanese analyzer variants, dense retrieval, late interaction, GraphRAG, and reranking. Report Recall@k, MRR, nDCG, citation precision, span attribution accuracy, and index reproducibility fingerprints.

## 17. Assurance and governance

Build assurance evidence from the first commit.

Trace links:

- source document -> source span -> concept -> claim -> rule/table/workflow -> target artifact -> verifier result -> certificate -> conflict report -> UI card;
- requirement -> implementation module -> test -> evaluation result;
- hazard -> mitigation -> verification evidence;
- assurance goal -> strategy -> solution/evidence -> unresolved defeaters.

Represent assurance as machine-readable GSN/SACM-style YAML/JSON and export an OntoGSN-compatible RDF view when practical. Use defeaters explicitly.

Top research assurance claim: accepted CKC artifacts are source-grounded, deterministic, formally checkable, replayable, and suitable for identifying source text requiring review.

Initial hazard candidates: incorrect extraction, incorrect formalization, missing exception, false conflict, missed conflict, mistranslation, terminology mismatch, ontology alignment incoherence, solver/compiler bug, proof replay failure, stale source guideline, licensing misuse, over-trust, PHI leakage in later deployments.

Instrument pipeline stages with OpenTelemetry-style research traces. Later production maps clinical access events to FHIR AuditEvent/IHE audit patterns. Trace payloads must support redaction before storage.

## 18. CLI

Build a CLI named `ckc`.

Commands:

- `ckc ingest <source>`: register source file, metadata, permissions, content hash.
- `ckc extract <doc_id>`: create `SourceSpan` and table graph.
- `ckc index <doc_id|corpus>`: build retrieval artifacts.
- `ckc formalize <span|section|cq|doc>`: generate candidate CKC.
- `ckc normalize <bundle>`: produce CKC Normal Form and hashes.
- `ckc validate <bundle>`: run schema, grounding, terminology, profile, and SHACL checks.
- `ckc compile <bundle>`: emit target artifacts.
- `ckc verify <bundle>`: run solver/proof checks and collect certificates.
- `ckc conflicts <bundle|corpus>`: produce incompatibility/factual inconsistency reports.
- `ckc repair <conflict>`: compute minimal repair candidates.
- `ckc certs <artifact>`: show certificate graph and replay commands.
- `ckc assure <run_id>`: produce/update assurance-case artifacts.
- `ckc eval <dataset>`: run gold, synthetic, metamorphic, retrieval, and regression suites.
- `ckc report <run_id>`: produce bilingual report JSON.
- `ckc ui`: serve/read static bilingual result UI.
- `ckc replay <manifest>`: replay a run and compare hashes.
- `ckc demo toy-research-kernel --replay --out runs/toy`: run Phase 0.

Every command emits structured diagnostics and immutable artifacts.

## 19. Repository layout

```text
.
├── SPEC.md
├── CLAUDE.md
├── crates/
│   ├── ckc-cli/
│   ├── ckc-core/          # types, profiles, NF, IDs, diagnostics
│   ├── ckc-store/         # CAS + DuckDB
│   ├── ckc-extract/       # extraction orchestration contracts
│   ├── ckc-retrieve/      # Pyserini oracle, RRF, dense/late interaction, parity tests
│   ├── ckc-term/          # terminology graph, e-graph, alignment repair
│   ├── ckc-datalog/       # Soufflé/RDFox/static-analysis adapters
│   ├── ckc-compile/       # target compilers
│   ├── ckc-verify/        # solver/checker orchestration
│   ├── ckc-cert/          # certificate graph and replay
│   ├── ckc-report/        # bilingual report JSON
│   ├── ckc-assurance/     # GSN/SACM/risk/traceability
│   └── ckc-audit/         # research observability and replay traces
├── workers/python/
│   ├── ocr_layout/
│   ├── embeddings/
│   ├── llm_formalizer/
│   └── terminology/
├── schemas/
├── dsl/ckc-syntax/
├── lean/Crl/
├── logic/{smt,asp,prolog,datalog,egglog,tla,alloy,dmn}/
├── ontologies/{alignments,terminology}/
├── certs/
├── eval/{gold,synthetic_conflicts,metamorphic,retrieval_qrels,metrics}/
├── ui/sveltekit-app/
├── examples/toy_research_kernel/
└── runs/.gitkeep
```

## 20. Development phases

### Phase 0: proof-carrying research kernel

Deliver a tiny but mathematically honest end-to-end system.

Required toy scenarios:

1. sepsis context recommends/administers beta-lactam; beta-lactam anaphylaxis context contraindicates it; shared witness context exists;
2. βラクタム / ベータラクタム / β-ラクタム / generic/brand variants normalize through e-graph/terminology bindings;
3. Japanese-style vital-sign decision table has overlapping rows with incompatible actions and a gap witness;
4. Event Calculus narrative shows allergy history persists and administration violates contraindication later;
5. MaxSMT/ASP proposes minimal priority/exception repair candidates;
6. missing provenance fails SHACL;
7. Lean proves the unprioritized norm conflict or decision-table uniqueness violation;
8. replay reproduces accepted hashes exactly.

Deliver:

- Rust workspace and `ckc` CLI skeleton;
- CKC schema v0;
- CKC Normal Form with canonical JSON hashing;
- property tests for normal-form idempotency;
- Japanese analyzer/retrieval fixture;
- e-graph synonym fixture;
- Z3 deontic, interval, and decision-table witnesses;
- Z3 Optimize/MaxSMT repair fixture;
- cvc5 unsat core/proof artifact where feasible;
- Clingo defeasible, argumentation, and Event Calculus fixtures;
- Datalog/Soufflé/RDFox cycle/static-analysis fixture;
- RDF export and SHACL violation fixture;
- Lean theorem fixture;
- generated TLA+/Alloy meta-spec stubs;
- bilingual report JSON and static UI card;
- certificate graph and assurance seed;
- replay manifest proving deterministic hashes.

Acceptance:

- `ckc demo toy-research-kernel --replay --out runs/toy` runs all toy checks;
- repeated runs produce identical accepted artifact hashes;
- every verifier output maps to source spans;
- accepted Lean files compile with zero `sorry` or `admit`;
- report JSON renders from static artifacts alone;
- backend disagreement appears as a structured diagnostic.

### Phase 1: extraction and span registry

Deliver ingest/extract/index commands, source permission ledger, text-layer PDF extraction, YomiToku adapter, MinerU disagreement adapter, source-span/table graph, extraction QA reports, and layout fixtures.

Acceptance: fixture PDFs yield addressable `span_id` records for all textual spans; raw JA text is exact; normalized/search text is deterministic; extraction disagreements are reviewable.

### Phase 2: terminology and retrieval substrate

Deliver Japanese analyzer comparison, retrieval qrels, terminology graph, MEDIS/HOT/YJ/JLAC-style fixture loaders, e-graph equivalence classes, SKOS/OWL export, alignment repair fixture, and GraphRAG schema constraints.

Acceptance: known synonym/spelling/unit variants converge; ambiguous mappings remain explicit; alignment incoherence is detected with source-linked repair suggestions.

### Phase 3: candidate formalization and semantic firewall

Deliver structured-output generation, Japanese modality lexicon, CQ/PICO/EtD extraction, table-to-decision-table extraction, grounding validator, semantic convergence clustering, verifier-guided repair loop, and rejection diagnostics.

Acceptance: a small guideline section formalizes into candidates; accepted claims cite exact spans and table cells; ambiguity is explicit; accepted artifacts pass C0-C3 gates.

### Phase 4: compiler portfolio

Deliver CKC-to-SMT, MaxSMT, ASP, s(CASP)/Prolog where feasible, Datalog/Soufflé/RDFox, egglog, Lean, RDF/SHACL, OWL/SKOS, DMN/FEEL, FHIR/CQL/ELM, and generated TLA+/Alloy stubs.

Acceptance: Section 15 conflict classes have executable tests; unsat cores/witnesses map to source spans; backend disagreements are reported; C4-C7 certificates exist for toy and selected real fixtures.

### Phase 5: corpus-scale conflict detection

Deliver corpus scans, minimal conflict sets, argument graph data, decision-table reports, temporal/event reports, repair candidates, synthetic contradiction benchmark, and adjudication queue export.

Acceptance: injected contradictions are found; false positives are categorized; conflict reports distinguish extraction, formalization, terminology, logic, interop, stale-source, and source-text causes.

### Phase 6: bilingual UI and manuscript package

Deliver SvelteKit UI, JA source-first conflict cards, EN gloss, formal IR panel, proof/certificate panel, argument graph, decision-table witness panel, filters, adjudication status export, and static manuscript supplement generation.

Acceptance: every UI claim links to source spans and evidence artifact hashes; manuscript package reproduces run configuration, metrics, limitations, and representative conflict examples.

### Phase 7: future CDS/SaMD bridge

Deliver non-production stubs: JP Core patient-context adapter, SS-MIX2-to-JP-Core mapping notes, ePath/BPMN export prototype, audit-event mapping, drift monitor skeleton, and assurance-case extension. These stubs demonstrate design continuity only.

## 21. UI requirements

The UI is evidence-first and research-only.

Views: corpus overview, extraction QA, terminology/alignment, claim list, conflict list, conflict detail, decision-table witness, temporal/event witness, proof/certificate detail, argument graph, adjudication board, reproducibility manifest, assurance snapshot.

Conflict card order:

1. JA exact source span and table cells.
2. EN gloss.
3. normalized CKC rule/table/workflow fragment.
4. conflict explanation in JA/EN.
5. minimal witness/context/trace/model.
6. evidence: unsat core, model, ASP extension, SHACL report, OWL explanation, DMN witness, Lean theorem, cvc5 proof artifact.
7. certificate-depth badge.
8. repair candidates as review prompts.
9. human review question.
10. adjudication status export.

Use text-quality/formalization language and frame outputs as source revision candidates.

## 22. Testing strategy

Required tests:

- Rust unit tests for CKC NF, IDs, schema constraints, compiler output, certificate graph, assurance graph;
- property-based tests for canonicalization and normal-form idempotency;
- golden tests for canonical JSON bytes;
- Python worker frozen I/O tests;
- Japanese tokenizer/retrieval qrel tests;
- e-graph equivalence tests;
- ontology alignment repair tests;
- Datalog/Soufflé/RDFox static-analysis tests;
- SMT tests with expected sat/unsat/core/model;
- MaxSMT/repair tests;
- cvc5 proof-artifact tests;
- Clingo defeasible/argument/Event Calculus tests;
- SHACL validation tests;
- OWL classification/explanation tests;
- DMN decision-table overlap/gap tests;
- Lean compile tests;
- TLA+/Alloy generated-spec smoke tests;
- metamorphic tests;
- end-to-end toy corpus replay;
- deterministic rerun hash comparison.

## 23. Manuscript-oriented outputs

Generate automatically:

- run manifest;
- corpus description;
- source permission ledger;
- extraction coverage/table QA report;
- CKC schema and semantic profile summary;
- normal-form/idempotency results;
- retrieval qrel report;
- terminology/alignment report;
- verifier pass/fail counts;
- certificate-depth distribution;
- semantic convergence metrics;
- conflict taxonomy counts;
- decision-table defect summary;
- temporal/event defect summary;
- representative conflicts with source/proof evidence;
- adjudication agreement tables;
- assurance-case snapshot;
- limitations ledger;
- reproducibility package manifest.

Bound manuscript claim: CKC identifies formally checkable candidate inconsistencies in Japanese clinical texts with deterministic, source-grounded, proof-carrying evidence. It claims review support; clinical correctness authority belongs to human adjudicators.

## 24. Mandate compliance checklist

Before project initiation and before each phase transition, verify that the repository still satisfies these mandate-level constraints:

- The initial artifact is research-grade, headless-first, and limited to text formalization QA.
- Every source has permission/version metadata and every extracted textual unit is addressable.
- CKC accepted artifacts are deterministic, explainable, verifiable, executable, and replayable.
- Logical incompatibilities and factual inconsistencies are phrased as source-text revision candidates.
- The UI is bilingual, read-only, evidence-first, and research-only.
- Coding-agent autonomy applies to implementation details, while stack choices, semantic profiles, normal forms, evidence chains, and evaluation obligations remain stable; changes require an experiment-backed spec update.
- Future EHR/CDS/SaMD ambitions influence abstractions but remain outside initial clinical deployment scope.

## 25. Immediate first task

Implement Phase 0.

Minimum command target:

```bash
ckc demo toy-research-kernel --replay --out runs/toy
```

The command creates accepted CKC Normal Form JSON, canonical hash manifest, source-span/table fixture JSON, retrieval output, e-graph equivalence artifact, SMT-LIB files and Z3 witnesses/cores, MaxSMT repair output, cvc5 proof artifact where feasible, Clingo programs/models, Datalog/static-analysis output, RDF/Turtle/TriG export, SHACL report, Lean theorem file and compile result, TLA+/Alloy stub specs, bilingual report JSON, assurance seed, certificate graph, and replay manifest.

Then run the same command twice and prove identical accepted artifact hashes.

Keep Phase 0 small. Make every concept explicit enough that later phases can expand while preserving the evidence chain.
