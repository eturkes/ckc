# Clinical Knowledge Compiler (CKC) Specification

## 1. Title, Status, Purpose

Title: Clinical Knowledge Compiler (CKC) M1 Research Harness Specification.

Status: stable M1 implementation contract for greenfield repository `.`; implementation reader: Claude Code.

Purpose: CKC is a headless research harness for reusable clinical-language mappings into compact compositional IR components verified once, reused by deterministic identity, and compiled to executable formal targets. M1 processes public Japanese guidelines, extracts text/layout/tables, compares bounded pipelines, compiles admitted IR to SMT-LIB, checks contradiction/null-result tasks, benchmarks the reusable-IR thesis, and emits replayable traces, reports, schemas, metrics, ledgers, and provenance artifacts. Its bounded autonomous loop proposes patches, runs fixed-budget locked experiments, scores with immutable evaluators, records run-local promotion/rejection, gates registry/status promotion, and preserves rejected attempts.

Defines CKC behavior, repository shape, schemas, registries, artifacts, commands, experiments, gates, acceptance criteria.

## 2. Normative Language

`MUST` is an M1 acceptance invariant. `SHOULD` is strong; alternatives are recorded in a registry, manifest, or gate evidence without weakening schema validity, replay, source grounding, traceability, benchmark comparability, authority separation. `MAY` is optional.

Implementation-defined choices are explicit degrees of freedom. Package, solver, adapter, model, prompt, source, executable, cache, and resource-limit versions belong in lockfiles, registries, manifests, or runtime metadata. Prose versions name only stable public standards or source-native identifiers.

Accepted CKC semantics come from admission, not proposer identity. AI, retrieval, agents, and humans may propose or critique semantics; they do not create accepted semantic authority.

## 3. Problem, Thesis, and Scope

Clinical guidelines express recommendations through Japanese prose, tables, exceptions, evidence summaries, CQ/PICO/EtD structures, temporal qualifiers, and modalities. Direct translation to a proof assistant, solver, CDS format, or executable target tends toward brittle one-off encodings.

CKC tests compact reusable IR mappings for population, action, condition, modality, exception, quantity, terminology binding, rule, axiom, constraint. Verified mappings are reused by normalized hash and compiled deterministically. The harness evaluates reuse, compactness/MDL proxies, hash convergence, formal compilability, contradiction precision/recall, weak-model lift, trace completeness, ablations, replay stability, and rankings.

M1 is a bounded headless harness over public Japanese guideline sources and synthetic fixtures. It reports candidates, locked smoke measurements, null results, failures, residuals, ambiguities, incoherences, traces, solver witnesses, next experiments.

M1 treats verification as open research. SMT-LIB is the first required formal target: named assertions, finite constraints, satisfiability, counterexample models, and UNSAT cores. Lean, Rocq/Coq, Isabelle/HOL, Why3, Alloy, TLA+, ASP, CQL/ELM, FHIR CPG, OWL/SHACL, DMN/FEEL, openEHR GDL2, e-graphs, optimization/probabilistic logic, agent-oriented DSLs, and invented CKC IRs are registry-backed backlog/optional candidates until gates promote stronger claims.

M1 has no clinical authority. Patient data, workflows, CDS runtime, EHR integration, SaMD behavior, UI, and production clinical decision support are backlog/gate-only design pressure.

## 4. Goals and Scope Boundaries

M1 goals:

| Goal                  | Contract                                                                                                                                            |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Reusable mapping      | Compare direct/layered IR for reuse, compactness, hash convergence, compilation, and trace quality.                                                 |
| Public corpus loop    | Retrieve, cache, process, validate, benchmark, and report public Japanese guidelines without UI/private-data dependencies.                          |
| Comparable candidates | Pipelines share schemas, fixtures, metrics, traces, replay manifests, evaluator locks, reports.                                                     |
| Kernel discipline     | Accepted artifacts use stable IDs, canonical bytes, spans/anchors/regions, permission records, typed outcomes, lineage, hashes, verifier links.     |
| Formal compilation    | At least two pipelines compile comparable artifacts to SMT-LIB and produce verifier results.                                                        |
| Authority separation  | AI, retrieval, constrained decoding, agents, and human fixtures are proposal/evidence until admission.                                              |
| Experimentation       | Matrices support compatibility filters, fixed budgets, pairwise/fractional designs, Pareto/beam narrowing, ablations, sweeps.                       |
| Locked evaluation     | Improvements score only against frozen fixtures, schemas, evaluator code, metrics, source hashes, toolchains, seeds, budgets.                       |
| Bounded autoresearch  | CKC may propose, patch, run, score, classify, record run-local promotion/rejection, replay, and ledger changes; registry/status promotion is gated. |

M1 scope boundaries:

| Area                                                                         | M1 treatment                                                                   |
| ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| Public Japanese guideline retrieval                                          | Required; autonomous, content-addressed, permission-aware.                     |
| HTML/XML/PDF/table extraction                                                | Required baselines with uncertainty diagnostics.                               |
| Japanese NLP, segmentation, retrieval, terminology grounding                 | Required baselines with registry-backed alternates.                            |
| SMT-LIB compilation and verification                                         | Required first formal target.                                                  |
| Runtime AI calls                                                             | Disabled by default; explicit experiment flag; proposal/evidence only.         |
| Autonomous candidate loop                                                    | Required bounded smoke harness with locked evaluator; no live clinical claims. |
| Large public guideline corpus processing                                     | Architecture-supported; M1 smoke remains bounded.                              |
| UI, patient data, EHR/CDS runtime, hospital operations, regulatory authority | Backlog/gate-only.                                                             |

Allowed report wording: `research harness`, `candidate`, `review candidate`, `formalization-QA`, `text-quality analysis`, `source-grounded`, `schema-valid`, `verifier-checked`, `replayable`, `requires human adjudication`, `locked smoke measurement`, `raw benchmark output`, `documented null result`.

## 5. Architecture and Stack Decision

Selected architecture: Rust typed core plus Python adapters joined only through canonical file artifacts and JSON Schema.

Pipeline shape:

```text
source registry -> fetch/cache -> permission record -> source graph extraction -> segmentation -> retrieval/evidence proposal -> terminology grounding -> semantic normalization -> IR bundle -> reusable component analysis -> formal compiler -> verifier adapter -> metrics/traces/diffs/reports/replay
```

Rust owns durable semantics: IDs, enums, schema structs, JSON Schema export, canonical JSON bytes, semantic hashes, envelope/registry/run-plan/IR validation, SMT-LIB emission, diagnostic types, property-testable invariants, future deterministic verifier adapters.

Python owns adapters: public source fetch/cache orchestration, HTML/XML/PDF/table extraction, Japanese NLP, retrieval/embeddings/reranking/graph retrieval, terminology grounding, explicitly enabled LLM/constrained-output/repair/critique integrations, solver process orchestration until Rust replacement, metrics, reports, ledgers, experiment loops, batch corpus orchestration.

Boundary invariants:

```text
Every component reads/writes declared artifact paths.
Every output validates against Rust-exported JSON Schema before downstream use.
Rust computes accepted artifact hashes; Python cannot define accepted identity.
Candidate isolation uses separate run directories.
Runtime metadata is recorded and excluded from semantic hashes.
No Python object is implicit semantic state.
Locked evaluator artifacts are read-only per candidate run.
Editable surfaces are declared before an experiment begins; undeclared edits are invalid.
```

Default tooling:

```text
Python: uv, pyproject.toml, uv.lock.
Rust: Cargo workspace, Cargo.lock.
Schemas: generated from Rust types and committed under schemas/.
CLI: Python `ckc` orchestrates; Rust `ckc-core-cli` exports schemas, validates, canonicalizes, hashes, normalizes run plans, and emits SMT.
Tests: cargo test, pytest, property tests where useful.
Formal target: SMT-LIB.
Solver adapter: one required; additional solvers schema-supported.
```

## 6. Repository Layout and CLI

Repository root: `.`.

```text
.
├── SPEC.md
├── pyproject.toml
├── uv.lock
├── Cargo.toml
├── Cargo.lock
├── crates/{ckc-core,ckc-core-cli,ckc-smt}/
├── ckc/
│   ├── adapters/{fetch,extract,segment,retrieve,ground,formalize,verify,report}/
│   ├── runner/
│   ├── metrics/
│   ├── traces/
│   └── reports/
├── registry/{methods,candidates,corpora,experiments,evaluators,prompts,policies,indexes,schemas,source_processors,gates}.yaml
├── corpus/{fixtures,gold,synthetic,raw}/
├── schemas/
├── examples/
├── runs/
├── tests/{fixtures,python,rust}/
└── Makefile
```

`corpus/raw/` and `runs/` are gitignored unless license-safe and intentionally committed.

Canonical CLI:

```text
uv run ckc registry check
uv run ckc schema export --out schemas/
uv run ckc artifact validate --schema <schema> --input <artifact>
uv run ckc artifact canonicalize --input <artifact> --out <canonical.json>
uv run ckc corpus fetch --corpus registry/corpora.yaml
uv run ckc component run --candidate-id <id> --component-id <id> --input <artifact> --output-dir <dir> --run-manifest <manifest>
uv run ckc run --experiment <experiment-id> --out runs/<run-id>
uv run ckc research loop --experiment <experiment-id> --out runs/<run-id>
uv run ckc ledger summarize --ledger runs/<run-id>/experiment_ledger.jsonl
uv run ckc gate check --evidence <gate-evidence.json>
uv run ckc compare runs/<run-id>
uv run ckc report runs/<run-id>
uv run ckc replay runs/<run-id>
uv run ckc trace query --lineage-index <path> --artifact <artifact-id-or-hash>
cargo test --workspace
uv run pytest
```

CLI invariants:

```text
Each command emits JSONL events and one outcome: ok, residual, ambiguity, incoherence, unsupported, or invalid.
Each command validates inputs, writes only under its output directory, and records producer, toolchain, environment profile, input/output hashes, diagnostics, trace refs.
Research-loop commands materialize evaluator_lock.json before attempts and reject undeclared-surface patches.
`registry check` fails when required registry entries are missing or invalid.
```

## 7. Method and Candidate Registry

`registry/methods.yaml` is the method registry, composition checklist, and benchmark discovery surface. Runnable adapters are required only for `m1_required` entries.

Method entry schema:

```yaml
id: method.smtlib
public_family: SMT-LIB
aliases: [SMT, SMTLIB]
category: automated_reasoning
candidate_roles: [compile_target, verifier_backend]
adapter_status: m1_required
benchmark_tags: [formal, smt, contradiction_detection]
compatible_input_kinds: [ir_bundle]
compatible_output_kinds: [compiled_artifact, verifier_result]
gate_refs: []
notes: "Executable and solver versions are recorded in manifests."
```

Adapter statuses: `m1_required`, `m1_optional`, `registered_backlog`, `gate_only`, `out_of_scope_m1`.

Seed method families; keywords seed discovery, aliases, benchmarks, adapters.

| Category                         | Registry seed keywords                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | M1 status                                                             |
| -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| Computable guideline/CDS IR      | CQL/ELM; QDM; FHIR Clinical Reasoning; CPG-on-FHIR/CQF; CRMI; FHIRPath; StructureMap/FML; FHIR Terminology Services; Library; PlanDefinition; ActivityDefinition; Measure; Evidence; EvidenceVariable; ValueSet; CodeSystem; ConceptMap; CDS Hooks; SMART; Arden Syntax; SAGE; GLIF3; Asbru; PROforma; openEHR ADL/AQL/GDL2; DMN/FEEL; BPMN/BPM+ Health                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | Backlog; runtime CDS gate-only.                                       |
| Japan guideline/data/terminology | Minds Guideline Library; Minds/GRADE; CQ; PICO; SR; EtD; AGREE II; GRADEpro GDT; Ichushi-Web; J-STAGE; JATS XML; guideline HTML/PDF; FHIR JP Core; JP-CLINS; JAMISDP; JAHIS; HELICS; HS001/HS005/HS024/HS026; SS-MIX2; MEDIS-DC Standard Masters; HOT; YJ; JAN; GS1; JJ1017; JLAC10/JLAC11; MERIT-9; PMDA e-PI/review reports/safety information; ICD-10 Japanese Modification; ICD-11 mapping; DPC/PDPS; K codes; receipt codes; e-prescription crosswalks; MedDRA/J; JADER; OMOP CDM; OHDSI Japan; MID-NET; NDB Open Data; JMDC; MDV; LOINC; SNOMED CT; ATC                                                                                                                                                                                                                                                                                   | Public guidelines/finite terminology required; patient/RWE gate-only. |
| Ontology/RDF/terminology         | OWL 2 EL/RL/QL/DL; SHACL Core/SPARQL/Rules; RDF/RDFS; named graphs; TriG; N-Quads; RDF-star/SPARQL-star; PROV-O; SKOS; SSSOM; FHIR ValueSet/ConceptMap governance; OBO Foundry; ROBOT; ODK; DOSDP; BFO; DOLCE; MIREOT; LogMap; AgreementMakerLight; Japanese clinical entity linking; concept normalization; terminology diffing; change-impact analysis; ELK; HermiT; Pellet/Openllet; FaCT++; Konclude; RDFox; Protégé; OWL API; owlready2; funowl; RDFLib; pySHACL                                                                                                                                                                                                                                                                                                                                                                           | Finite terminology allowed; richer ontology gated.                    |
| Proof/formal specification       | Lean family; Mathlib; Aesop; `simp`; `grind`; Rocq/Coq; Stdlib; MathComp; Iris; MetaCoq; CoqHammer; SMTCoq; Isabelle/HOL; AFP; Sledgehammer; Nitpick; TLA+; TLAPS; TLC; Apalache; Alloy; Forge; Why3/WhyML; F*; Agda; Idris; Dedukti; Lambdapi; dependent-type IR; refinement-type IR; proof by reflection; LFSC; Alethe; DRAT/LRAT; Carcara; CrossHair; SAW/Crucible/Cryptol; typed functional programming; Rust; Ada/SPARK                                                                                                                                                                                                                                                                                                                                                                                                                    | Backlog/optional; SMT-LIB emission required.                          |
| Automated reasoning              | SMT-LIB; Z3; cvc5; Bitwuzla; Yices; OpenSMT; SAT; MaxSAT; OMT; MUS/MCS; UNSAT core extraction; interpolants; Datalog; Datalog±; Soufflé; RDFox rules; OWL reasoning; ASP; Clingo; ILASP; MiniZinc; OR-Tools CP-SAT; MILP/ILP; e-graphs/equality saturation; egg; Herbie; Prolog; s(CASP); PRISM/Storm; ProbLog/cplint                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | SMT-LIB plus one solver required; alternatives registered.            |
| Clinical logic/semantics         | Strict rules; defeasible rules; defeaters; superiority relation; Defeasible Logic; Defeasible Deontic Logic; deontic logic; obligation; permission; prohibition; contrary-to-duty; reparation; Dung argumentation; ASPIC+; Carneades; Assumption-Based Argumentation; paraconsistent logic; Event Calculus; Allen interval algebra; temporal constraint networks; LTL/MTL/STL; MCDA; GRADE strength/certainty mapping                                                                                                                                                                                                                                                                                                                                                                                                                           | Finite deontic/directional checks required; richer semantics gated.   |
| Retrieval/RAG                    | BM25/BM25+/BM25F; Lucene; Anserini/Pyserini; OpenSearch; Elasticsearch; Vespa; Qdrant; Weaviate; Faiss; dense retrieval; DPR; ANCE; Contriever; SPLADE; BGE-M3; Multilingual-E5; Jina embeddings; BioBERT; PubMedBERT; MedCPT; JMedRoBERTa; JaColBERT; ColBERT; cross-encoder reranking; RRF; GraphRAG; query decomposition; retrieval routing; citation-grounded generation; ALCE; BEIR; MTEB; nDCG@k; Recall@k; MAP; MRR; RAGAS; TruLens; ARES; citation precision                                                                                                                                                                                                                                                                                                                                                                            | Sparse baseline required; dense/rerank/graph optional/gated.          |
| Extraction/layout/Japanese NLP   | HTML extraction; JATS/XML extraction; PDF extraction; table extraction; figure extraction; Docling; Marker; MinerU; GROBID; PyMuPDF; pdfplumber; pypdf; Camelot; Tabula; Table Transformer; PubTables; Unstructured; PaddleOCR; Mathpix; YomiToku/Yomitoku; LayoutLMv3; DocLayout-YOLO; Tesseract; MeCab; UniDic; IPAdic; fugashi; SudachiPy; Kuromoji; GiNZA; spaCy Japanese; SentencePiece; XLM-R tokenizer                                                                                                                                                                                                                                                                                                                                                                                                                                   | Baseline extraction/Japanese analyzer required; OCR/layout gated.     |
| AI/autoformalization             | OpenAI GPT family; Anthropic frontier model family; Google Gemini family; medical model families; Med-Gemini; MedGemma; Meditron; GatorTron; JMedLLM; UTH-BERT; proof-model families; DeepSeek-Prover; LeanDojo; ReProver; Goedel-Prover; Herald; AlphaProof; weak local model family; constrained decoding; xgrammar; Outlines; Guidance; JSON-schema decoding; grammar-state decoding; logit masks; tool-calling agents; MCP; function calling; code-execution agents; LangGraph; DSPy; TextGrad; GEPA; EvoPrompt; evaluator-cheating controls; mutation search; bounded autoresearch; self-consistency; semantic convergence; idempotency checks; premise/theorem retrieval; independent critique; adjudication; NLI critique; verifier-guided decoding; proof repair; program-aided language models; LoRA/QLoRA; recorded proposal fixtures | Proposal/evidence only; live calls explicit/recorded.                 |
| Evaluation/validation            | Gold guideline-to-IR corpus; clinician/formalist adjudication; Cohen κ; Fleiss κ; Krippendorff α; γ-agreement; semantic equivalence; idempotency; convergence; contradiction/collision benchmarks; EBM-NLP; GGPONC; CPG-on-FHIR pilots; ProofNet; FormalAlign; BEq; ProofNetVerif; CQL/FHIR/DMN conformance; Inferno; CQF Ruler; DMN TCK; JP Core/JAMISDP conformance; unit tests; metamorphic tests; property-based tests; Hypothesis; QuickCheck; weak-model lift; shadow-mode evaluation; silent trial; CDS Five Rights; explanation quality; equity/subgroup/external validation/calibration; TRIPOD+AI; PROBAST+AI; SPIRIT-AI; CONSORT-AI; DECIDE-AI; STARD-AI; CFIR; NASSS; RE-AIM                                                                                                                                                        | M1 benchmark tasks required; clinical utility/deployment gate-only.   |
| Assurance/operations             | GSN; SACM; OntoGSN; Assurance 2.0; Isabelle/SACM; D-Case; ISO 13485; ISO 14971; IEC 62304; IEC 62366; IEC 81001-5-1; FDA/PMDA/IMDRF CDS/SaMD; GMLP; N81; IDATEN; PCCP; EU AI Act; NIST AI RMF; ISO/IEC 42001; ISO/IEC 27001; APPI; Next-Generation Medical Infrastructure Act; STRIDE; LINDDUN; Zero Trust; OWASP LLM; MITRE ATLAS; de-identification; differential privacy; SBOM/AIBOM; SPDX; CycloneDX; in-toto; SLSA; Sigstore; Knowledge CI/CD; DVC; MLflow; OpenTelemetry; FHIR AuditEvent; continuous verification; drift monitoring; incident response; CAPA; post-market surveillance                                                                                                                                                                                                                                                   | Gate vocabulary only.                                                 |
| Agent-language and DSL design    | Axis; B-IR; Mog; AILANG; Aver; Vow; Vera; Magpie; Lume; Koru; typed agent DSLs; LL(1) grammar masks; prefix syntax; one-operation-per-binding; grammar state machines; per-state logit masks; LSP diagnostics; proof export; bytecode VM; spec-as-IR; orchestration bytecode; JSON AST/diagnostics; capability/effect systems; contracts/vows; bounded model checking; deterministic workflow bytecode; hash-chained evidence; typed orchestration; session types; explicit IR handoffs; generated tests                                                                                                                                                                                                                                                                                                                                        | CKC-GEN/future DSL design patterns.                                   |

`registry/candidates.yaml` defines components, evidence emitters, and pipelines.

Required pipeline IDs:

| Pipeline                          | Status        | Role                                                                                                                        |
| --------------------------------- | ------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `pipe.direct_rule_to_smt`         | `m1_required` | Deterministic extraction/segmentation/phrase normalization compiled directly to SMT-LIB named assertions.                   |
| `pipe.layered_ckcir_to_smt`       | `m1_required` | Deterministic staged pipeline through CKC IR layers before SMT-LIB.                                                         |
| `pipe.ai_structured_ckcir_to_smt` | `m1_optional` | Schema-constrained AI formalization using recorded outputs for deterministic regression; live calls require explicit flags. |

Required runnable `ckc component run` IDs:

| Role        | ID(s)                                             |
| ----------- | ------------------------------------------------- |
| Fetch/cache | `fetch.public_guideline`                          |
| Extract     | `extract.html_pdf_baseline`                       |
| Segment     | `segment.cq_recommendation_rules`                 |
| Retrieve    | `retrieve.bm25_ja_baseline`, `retrieve.off`       |
| Ground      | `ground.lexical_terminology_baseline`             |
| Formalize   | `formalize.direct_rules`, `formalize.ckcir_rules` |
| Compile     | `compile.ckcir_to_smt`                            |
| Verify      | `verify.smt`                                      |

Required runner/registry/evidence IDs, validated as registry or runner hook IDs, not `ckc component run` targets:

| Surface                   | ID(s)                                                                             |
| ------------------------- | --------------------------------------------------------------------------------- |
| Source processor registry | `source_processor.minds_html_pdf_baseline`                                        |
| Evidence emitters         | `metrics.m1_smoke`, `trace.m1_exports`, `report.m1_smoke`, `replay.deterministic` |
| Experiments               | `exp.m1_public_smoke`, `exp.m1_autonomous_smoke`                                  |

Backlog pipeline IDs:

```text
pipe.cql_elm_to_smt
pipe.fhir_cpg_to_smt
pipe.dmn_feel_to_smt
pipe.openehr_gdl2
pipe.owl_shacl
pipe.defeasible_asp
pipe.lean_proof
pipe.alloy_counterexample
pipe.why3_vc
pipe.event_calculus
pipe.tla_state_model
pipe.egraph_normalizer
pipe.smt_omt_optimizer
pipe.probabilistic_logic
pipe.ckc_gen_air_kernel
pipe.agent_dsl_axis_like
pipe.autoresearch_candidate_optimizer
```

Status-change invariants:

```text
PromotionDecision = promote | reject | quarantine | defer_gate | request_replay.
PromotionScope = run_local | registry_status.
PromotionDecision artifacts carry decision and scope.
Run-local promotion classifies an attempt inside one locked experiment and changes only ledgers/reports.
Registry/status promotion changes registry status and requires from_status, to_status, evidence hashes, replay hash, rollback instruction, status-change evidence, applicable gates.
Automated loops edit registry entries only through declared candidate patches.
Status changes affecting accepted semantics require schema, source, applicable compiler/verifier checks, trace, benchmark, replay, and applicable gates.
Changes affecting evaluator identity, metric semantics, fixtures, gold labels, schemas, or acceptance criteria cannot score improvements in the same experiment.
Rejected status changes remain in the experiment ledger.
```

## 8. Candidate Composition and Experiment Matrices

CKC models pipelines as typed graphs. End-to-end scripts are valid only as registry-declared components and runner/evidence hooks.

Component entry requirements:

```yaml
id: segment.cq_recommendation_rules
kind: component
family: segmentation
status: m1_required
deterministic_default: true
runtime_effects: []
input_artifact_kinds: [source_graph]
output_artifact_kinds: [segments]
compatible_predecessor_roles: [extract]
compatible_successor_roles: [retrieve, ground, formalize]
benchmark_tags: [japanese_guideline, cq, recommendation]
budget_class: local_cpu
gate_refs: []
```

Composition invariants:

```text
Every component declares kinds, determinism, runtime effects, benchmark tags, budgets, gate_refs, compatibility metadata.
Runner/evidence hooks declare input/output artifacts but need not be pipeline components.
The runner constructs compatible graphs and computes candidate_graph_hash before execution.
Incompatible combinations are skipped-incompatible; unsupported schema-valid combinations emit unsupported; runnable failures remain scored.
Direct pipelines emit pass-through or not_applicable artifacts for unused stages.
Registry aliases normalize to semantic families without duplicate candidate identities.
```

Experiment matrix example:

```yaml
id: matrix.m1_bounded_smoke
fixtures: [fixture.public_guideline_cq_grade, fixture.synthetic_contradiction_ja, fixture.synthetic_negative_control_ja]
component_axes:
  extract: [extract.html_pdf_baseline]
  segment: [segment.cq_recommendation_rules]
  retrieve: [retrieve.bm25_ja_baseline, retrieve.off]
  ground: [ground.lexical_terminology_baseline]
  formalize: [formalize.direct_rules, formalize.ckcir_rules]
  compile: [compile.ckcir_to_smt]
  verify: [verify.smt]
evidence_emitters: [metrics.m1_smoke, trace.m1_exports, report.m1_smoke, replay.deterministic]
max_runs: 12
repetitions: 2
budget_policy: bounded_smoke
selection_design: pairwise_then_required
runtime_ai: false
```

Matrix invariants:

```text
Matrices declare fixtures, axes, repetitions, max_runs, budget policy, compatibility filters, stopping criteria, selection design.
M1 executes a bounded proof matrix, not a method-universe sweep.
Large spaces SHOULD use baselines, ablations, pairwise/fractional coverage, beam/Pareto narrowing, targeted sweeps.
Reports classify combinations as untested, skipped-incompatible, unsupported, failed, dominated, equivalent, Pareto-front, or promising, with counts and reasons.
```

Required experiments: `exp.m1_public_smoke`, `exp.m1_autonomous_smoke`.

Bounded autonomous research loop:

```text
The loop is configured by registry/experiments.yaml and optional ResearchLoopPlan artifacts.
Admitted configuration defines objectives, editable surfaces, locked evaluator identity, budgets, promotion policy, fixtures, and stopping criteria.
The loop performs propose -> patch -> run -> score -> classify -> local_promote_or_reject -> replay -> ledger.
Proposal, patch, run, score, classification, promotion/rejection, replay, and next-attempt notes materialize as canonical artifacts or JSONL records.
Run-local promotion requires objective improvement or frontier membership, regression-threshold compliance, schema validity, trace completeness, applicable source grounding/verifier checks, replay success, authorized edits.
Registry/status promotion of accepted generators, prompts, policies, indexes, compilers, verifier adapters, metric/report code, source-processing rules, or adapters requires status-change evidence and applicable gates.
Rejected, dominated, null, crashed, timeout, unauthorized, and near-miss attempts remain queryable.
```

Editable surfaces when declared: candidate adapters/configs; accepted generators; source-processing rules; registry status artifacts; prompt templates; retrieval/index configs; normalization/terminology policies; IR transforms; compilers; verifier adapters; metric/report code; fixture/gold drafts. Same-experiment locked surfaces: evaluator adapters, scoring harness, metric definitions, evaluator identity, fixtures, gold labels, schemas, acceptance criteria, and gate definitions unless evaluator migration admits a new lock.

Fixed-budget controls:

```text
Every loop declares max_attempts, max_promotions, max_failures, wall-clock, token/cost when applicable, memory, retry, repair, timeout, stopping criteria.
M1 autonomous smoke loops use runtime_ai=false, bounded fixtures, local adapters, no live clinical claims.
Budget exhaustion returns stopped_budget with ledger records for completed and partial attempts.
```

## 9. Domain Model and Stable IDs

ID grammar:

```text
Id       = lowercase ASCII matching [a-z][a-z0-9_.:-]*
Hash     = "sha256:" + 64 lowercase hex digits
Rational = exact reduced { "num": "<int>", "den": "<positive-int>" }
```

Semantic IDs use lowercase path-like segments; deterministic disambiguation uses source order before hash order.

Core domain objects:

| Object                       | Contract                                                                                                                                              |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `SourceDocument`             | Public source identity, URI aliases, source family, access metadata, permission record, raw/content hashes, drift status.                             |
| `SourceGraph`                | Finite graph of document, section, paragraph, list, table, cell, caption, footnote, CQ, recommendation, PICO, EtD, evidence, cross-reference nodes.   |
| `SourceSpan`                 | Stable text span with source hash, node, page/block/table cell, offsets, raw/NFKC/search/display text, bbox, reading order.                           |
| `SourceAnchor`               | Subspan anchor for phrases, terms, quantities, modalities, negation, temporal cues, and table values.                                                 |
| `SourceRegion`               | Closed support set over nodes, spans, anchors, cells, headers, captions, footnotes, continuations, and cross-references.                              |
| `ClinicalSegment`            | CQ, recommendation, evidence, exception, definition, table row, PICO field, EtD field, or metadata segment.                                           |
| `TerminologyBinding`         | Mention-to-concept/code binding with alternatives, status, source support, method, trace.                                                             |
| `ClinicalStatement`          | Normalized population, condition, action, comparator, outcome, modality, strength, certainty, exception, temporal scope, assumptions, source refs.    |
| `NormRule`                   | Context, direction, action, temporal qualifiers, defeasibility, priority, exception, recommendation metadata, source refs.                            |
| `IRBundle`                   | DocIR, SegmentIR, ClinicalIR, NormIR, FormalIR, reusable components, diagnostics, hashes.                                                             |
| `CompiledArtifact`           | Formal target body, target metadata, named assertions, source/IR maps.                                                                                |
| `VerifierResult`             | Schema/compiler/target/solver status, SAT/UNSAT/unknown, model, UNSAT core, proof obligations, diagnostics.                                           |
| `TraceBundle`                | Derivation DAG, proof/witness structures, mapping hypergraph, reuse graph, axiom dependency graph, lineage index.                                     |
| `ExperimentPlan`             | Fixtures, candidates, tasks, locked evaluator, objectives, budgets, matrix, repetitions, seeds, stopping criteria.                                    |
| `EvaluatorLock`              | Immutable pre-run identity of fixtures, schemas, source hashes, scoring config, metrics, evaluator code, toolchains, seeds, budgets.                  |
| `CandidateProposal`          | Proposed change with objective, parent candidate, expected artifact diffs, editable surfaces, required gates.                                         |
| `CandidatePatch`             | Declared-surface patch with patch hash, workspace hash, candidate graph hash, rollback, changed hashes.                                               |
| `ExperimentLedger`           | Append-only JSONL evidence for proposal, patch, run, score, classification, promotion/rejection, replay.                                              |
| `PromotionDecision`          | Decision artifact carrying `decision`, `scope`, evidence refs, replay refs, and applicable gate refs.                                                 |
| `SelfImprovementEvidence`    | Gate evidence for changes to generators, prompts, policies, indexes, compilers, verifier adapters, metric/report code, or source-processing adapters. |
| `EvaluatorMigrationEvidence` | Gate evidence for future evaluator-lock, fixture/gold/schema/metric/scoring/evaluator-code, or threshold changes.                                     |
| `RunManifest`                | Run plan, candidate set, source hashes, toolchain, environment, lockfile hashes, seeds, model/tool IDs, output hashes.                                |
| `Report`                     | Markdown/JSON/CSV summary with rankings, raw rows, failures, verification outcomes, ablations, traces, null results.                                  |

Shared enums:

```text
Outcome = ok | residual | ambiguity | incoherence | unsupported | invalid
Origin = human_authored | ai_assisted | ai_generated | adapter_generated | deterministic_compiler
Authority = source_authority | mechanical_authority | evidence_discovery_only | admitted_authority | compiler_authority | verifier_authority | view_only
BindingStatus = exact | synonym | ambiguous | unmapped
Direction = for | against | contraindicate | require | permit | avoid
ClaimTier = s0_replayable | s1_admitted | s2_research_evidence | s3_clinical_regulatory
ReviewClassification = candidate | residual | ambiguity | incoherence | replay_failure | documented_null_result
AttemptClassification = improved | equivalent | dominated | regression | invalid | unsupported | timeout | crash | null_result | near_miss | unreproducible | gate_required
PromotionDecision = promote | reject | quarantine | defer_gate | request_replay
PromotionScope = run_local | registry_status
```

Outcome order: `invalid > incoherence > unsupported > ambiguity > residual > ok`.

Outcome meanings:

| Outcome       | Meaning                                                                                                                 |
| ------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `ok`          | Output is valid for the declared stage.                                                                                 |
| `residual`    | Input is schema-valid but incomplete, permission-limited, missing evidence, missing policy, or missing license support. |
| `ambiguity`   | Multiple admissible readings, bindings, spans, or normalizations remain.                                                |
| `incoherence` | Accepted inputs collide semantically.                                                                                   |
| `unsupported` | Schema-valid construction lies outside implemented M1 semantics.                                                        |
| `invalid`     | Schema, hash, canonicalization, registry, command, or proof validation fails.                                           |

## 10. Artifact, Schema, Hash, and Outcome Contracts

Every accepted artifact uses an envelope:

```json
{
  "schema_version": "ckc.m1",
  "schema_id": "schema.ir_bundle",
  "schema_hash": "sha256:...",
  "artifact_id": "artifact.semantic_id",
  "artifact_kind": "ir_bundle",
  "producer": {"candidate_id": "pipe.layered_ckcir_to_smt", "component_id": "formalize.ckcir_rules", "toolchain_manifest_hash": "sha256:..."},
  "input_hashes": ["sha256:..."],
  "content_hash": "sha256:...",
  "canonicalization_policy_hash": "sha256:...",
  "replay_manifest_hash": "sha256:...",
  "origin": "adapter_generated",
  "authority": "admitted_authority",
  "accepted_effects": [],
  "trace_refs": ["trace..."],
  "diagnostics": [],
  "runtime_metadata": {},
  "payload": {}
}
```

Envelope invariants:

```text
content_hash = sha256(canonical_payload_bytes(payload)).
Artifact path is derived from content_hash or artifact_id plus content_hash.
runtime_metadata does not affect content_hash.
Accepted semantic artifacts have accepted_effects = [].
Evidence-discovery artifacts may record Network, Clock, AI, or external Tool effects; accepted semantics begin after deterministic validation.
Fields ending in `_hash` reference accepted artifact content_hash unless their schema declares raw-byte hashing.
`compiler_authority` is reserved for compiled artifacts; `verifier_authority` is reserved for verifier results.
Accepted semantic authority begins after applicable schema validation, source grounding or explicit `synthetic_fixture_id`, canonicalization, applicable compiler/verifier/report checks, trace/replay recording, and applicable gates.
```

Schema authority:

```text
Rust types generate JSON Schema.
schemas/ contains committed generated schemas.
registry/schemas.yaml records schema_id, schema_version, Rust type manifest hash, JSON Schema hash, canonicalization policy hash, string policy bindings, and source-support aliases.
Schema changes bump schema_version and update registry/schemas.yaml.
```

Canonical JSON bytes:

```text
Objects: UTF-8 field names sorted by byte order; unknown fields rejected.
Optional fields: omitted when absent; JSON null rejected.
Arrays: ordered when order is semantic.
Sets: arrays sorted by canonical_sort_key.
Maps: identifier_ascii keys as sorted objects; other keys as sorted key/value arrays.
Strings: UTF-8 with schema-declared string policy before hashing.
Integers: decimal strings in accepted semantic artifacts.
Rationals: exact reduced objects; JSON numeric tokens rejected in accepted semantic artifacts.
Unions: tagged objects with exactly "tag" and "value".
```

String policies:

| Policy             | Contract                                                                                                                                       |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `raw_source`       | Preserve extractor-emitted Unicode scalar sequence exactly; record decoder/source byte hash when available.                                    |
| `source_nfkc`      | Apply Unicode NFKC only.                                                                                                                       |
| `semantic_ja`      | Apply NFKC, fold whitespace to U+0020, collapse runs, trim, and fold common Japanese/fullwidth punctuation to deterministic ASCII equivalents. |
| `semantic_en`      | Apply NFKC, whitespace/punctuation folding, and lowercase ASCII only for controlled-vocabulary identifiers.                                    |
| `identifier_ascii` | Require `[a-z0-9_:./-]+`; store bytes exactly.                                                                                                 |
| `diagnostic_text`  | NFKC plus semantic whitespace folding.                                                                                                         |
| `view_text`        | NFKC display text with renderer provenance.                                                                                                    |

Total operation result:

```json
{"operation_id":"compile.ckcir_to_smt","outcome":"ok","value_hashes":["sha256:..."],"diagnostic_hashes":[],"residual_hashes":[],"ambiguity_hashes":[],"incoherence_hashes":[]}
```

All stages return one total outcome. Schema-valid candidate outputs that fail validation are scored as syntax/schema failures. Governance artifacts named in §9 are schema-governed and carry canonical hashes.

## 11. Source, Corpus, Extraction, and Permission Contracts

M1 corpus shape:

| Fixture type                                                                                   | Required     |
| ---------------------------------------------------------------------------------------------- | ------------ |
| Minds/GRADE/CQ-style public guideline page or PDF                                              | At least one |
| J-STAGE/JATS-like public guideline or excerpt with HTML/XML-like layout and PDF when available | At least one |
| Table-heavy or layout-challenging public guideline excerpt                                     | At least one |
| Synthetic Japanese challenge cases                                                             | Required     |
| Injected contradiction                                                                         | Required     |
| Non-conflicting negative control                                                               | Required     |

Corpus processing is content-addressed, resumable, batchable, and reportable per document, fixture, source family, and corpus. Smoke and large-corpus runs share schemas, component graphs, matrices, traces, metrics, replay contracts.

`registry/corpora.yaml` fixture entry:

```yaml
id: fixture.public_guideline.table_heavy
source_family: jstage_or_minds
clinical_structure_tags: [guideline, cq_or_recommendation, tables, pdf, html_or_xml]
source_artifacts:
  - {kind: html, uri_alias: public_source}
  - {kind: pdf, uri_alias: public_source_pdf}
gold:
  segments: corpus/gold/fixture.../segments.jsonl
  statements: corpus/gold/fixture.../statements.jsonl
mutations: [punctuation_normalized, kana_kanji_variant, reordered_sections]
challenge_fixtures: [fixture.synthetic_contradiction_ja, fixture.synthetic_negative_control_ja]
```

Raw public documents live in a content-addressed cache. Metadata, hashes, license-safe excerpts, synthetic fixtures, and gold labels are committed. Raw bytes and extracted text are exported or committed only when allowed by the permission record. Source hash changes produce `source_drift.json`. Fixture, gold-label, and source hashes in an EvaluatorLock are immutable for that experiment; fixture proposals are scored only in evaluator-migration or fixture-development experiments.

Permission record:

```json
{
  "source_document_id": "doc...",
  "rights_holder": "...",
  "access_ref": "...",
  "license_label_or_contract_ref": "...",
  "redistribution_status": "redistributable",
  "allowed_artifacts": ["source_bytes", "source_graph", "quoted_snippets", "offsets_only", "hashes_only", "derived_labels"],
  "permission_evidence_hash": "sha256:..."
}
```

Redistribution statuses: `redistributable` permits allowed snippets/derived artifacts; `reconstructable` permits offsets, hashes, region IDs, and derived labels sufficient to reconstruct under source terms; `restricted_internal_only` exports only allowed hashes, region IDs, and derived labels. Disallowed exports emit `Residual(class=permission_limited)` and continue. Reports are redacted by deterministic policy.

Source span fields:

```text
span_id, document_id, source_hash, source_node_id, section_path, page, block_id, reading_order, bbox, char_start, char_end, raw_text, nfkc_text, search_text, display_text, text_hash
```

Source anchor fields: `anchor_id, document_id, span_id, anchor_kind, char_start, char_end, normalized_text, source_hash, confidence_or_policy, diagnostics`.

Source region fields: `region_id, document_id, member_node_ids, member_span_ids, member_anchor_ids, table_cell_refs, closure_reason, permission_export_class, region_hash`.

SourceGraph invariants:

```text
Every extracted textual unit has a SourceSpan or extraction_uncertain residual; semantic mention/value candidates have SourceAnchor records.
Permission-redacted spans retain offsets, hashes, node IDs, anchor IDs, and region IDs when export is allowed.
Every semantic claim links to at least one closed SourceRegion unless explicitly synthetic.
Same source bytes plus extraction manifest produce identical SourceGraph canonical bytes.
Tables preserve row/column/cell/header/caption/footnote relations when extracted; uncertain structure emits typed residuals.
Cross-references, continuations, captions, and footnotes close into SourceRegion or emit residuals.
```

`registry/source_processors.yaml` declares source-family adapters for fetch, extraction, segmentation, source normalization, redaction, and drift checks. Each entry records input/output artifact kinds, supported families, permission behavior, redaction policy, drift policy, diagnostics, and gate_refs:

```yaml
id: source_processor.minds_html_pdf_baseline
source_families: [minds, jstage]
stages: [fetch, extract, segment, redact, drift_check]
input_artifact_kinds: [source_uri, cached_source_bytes]
output_artifact_kinds: [source_document, permission_record, extraction_manifest, source_graph, extracted_document, source_spans, source_anchors, source_regions, extraction_diagnostics]
permission_behavior: {default_export: derived_labels, redaction_policy: policy.permission_redaction_baseline}
drift_check: {hashes: [raw_bytes, rendered_text, source_graph], on_change: emit_source_drift}
diagnostics: [permission_limited, source_drift, extraction_uncertain, table_structure_uncertain]
gate_refs: []
```

For baseline source processors, `gate_refs: []` is valid and expected. Source processor `gate_refs` apply only to stronger claims, promotions, new source families/export classes, generalized extractor-quality claims, or self-improvement. Baseline permission records, deterministic fixture extraction, redaction, drift checks, diagnostics, and locked smoke reporting remain ordinary M1 behavior.

Extraction adapters emit `source_document.json`, `permission_record.json`, `extraction_manifest.json`, `source_graph.json`, `extracted_document.jsonl`, `source_spans.jsonl`, `source_anchors.jsonl`, `source_regions.jsonl`, and `extraction_diagnostics.jsonl`. `extracted_document.jsonl` contains normalized extracted text rows used downstream. Default extraction uses deterministic HTML/JATS parsing where available, PDF text/layout via registered Python adapters, and table extraction with uncertainty diagnostics.

## 12. Terminology and Semantic Normalization

Terminology binding fields:

```text
binding_id, mention, normalized, system, code, version, concept_id, status, alternatives, method_id, source_region_ids
```

Binding status behavior:

| Status      | Consumer behavior                                                       |
| ----------- | ----------------------------------------------------------------------- |
| `exact`     | Satisfies single-concept demands.                                       |
| `synonym`   | Satisfies concept demands after representative normalization.           |
| `ambiguous` | Emits `Ambiguity(class=multiple_terms)` when one concept is required.   |
| `unmapped`  | Emits `Residual(class=missing_terminology)` when a concept is required. |

Terminology invariants:

```text
Exact, synonym, unit-equivalent, and action-kind-equivalent relations form deterministic representative classes.
Functional key collisions emit Incoherence(class=functional_key_collision).
Mutually exclusive mappings for one surface emit Incoherence(class=mutually_exclusive_term_mapping).
Terminology versions are recorded; cross-version proof transport is gate-backed future work.
```

Semantic normalization objects:

| Object              | Required fields                                                                                                                         |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `ClinicalStatement` | population, condition, action, comparator, outcome, modality, strength, certainty, exceptions, temporal scope, assumptions, source refs |
| `Action`            | action kind, target, slots, surface anchor, normalized target key                                                                       |
| `ContextExpr`       | finite disjunction of finite conjunctions of predicate, negated predicate, slot, quantity, and temporal atoms                           |
| `NormRule`          | context, direction, action, temporal qualifiers, source class, recommendation metadata                                                  |
| `FactualRule`       | context, consequent, strictness, source class                                                                                           |
| `DecisionTable`     | input variables, units, rows, guards, outputs, source rows                                                                              |

Semantic policy invariants:

```text
Action sameness normalizes terminology representatives, declared target relations, and discriminating slots.
Missing required action, output, or metadata policy emits Residual(class=missing_policy).
Duplicate policy keys with different payloads emit Incoherence(class=incompatible_policy_rows) and quarantine only conflicting rows.
Recommendation strength/certainty are proof-visible annotations; theorem predicates consume direction and normalized action/context.
Japanese modality phrase lexicons are versioned in registries/manifests and trace each match to source spans.
```

Direction groups for M1 conflict checks:

| Group            | Directions                 |
| ---------------- | -------------------------- |
| Positive         | `for`, `require`, `permit` |
| Against          | `against`, `avoid`         |
| Contraindicating | `contraindicate`, `avoid`  |

## 13. Candidate Pipeline and IR Contracts

Each runnable component implements:

```text
ckc component run --candidate-id <pipeline-or-component-id> --component-id <component-id> --input <artifact.json> --output-dir <dir> --run-manifest <manifest.json>
```

Candidate definition example:

```yaml
id: pipe.layered_ckcir_to_smt
kind: pipeline
family: layered_ir
status: m1_required
deterministic_default: true
runtime_ai: false
source_processor_id: source_processor.minds_html_pdf_baseline
components:
  fetch: fetch.public_guideline
  extract: extract.html_pdf_baseline
  segment: segment.cq_recommendation_rules
  retrieve: retrieve.bm25_ja_baseline
  ground: ground.lexical_terminology_baseline
  formalize: formalize.ckcir_rules
  compile: compile.ckcir_to_smt
  verify: verify.smt
evidence_emitters:
  metrics: metrics.m1_smoke
  trace: trace.m1_exports
  report: report.m1_smoke
  replay: replay.deterministic
emits_min_artifact_set: true
benchmark_tags: [layered_ir, smt, deterministic, japanese_guideline]
candidate_editable_surfaces: [component_config, normalization_rules, ir_transform]
```

Minimum artifact set per pipeline:

```text
pipeline_input.json
source_document.json
permission_record.json
extraction_manifest.json
source_graph.json
extracted_document.jsonl
source_spans.jsonl
source_anchors.jsonl
source_regions.jsonl
segments.jsonl
retrieval_results.jsonl
terminology_bindings.jsonl
semantic_normalization.jsonl
ir_bundle.json
compiled/smt/main.smt2
compiled/smt/assertion_map.json
verifier_results.json
trace_bundle.json
metrics.json
events.jsonl
diagnostics.jsonl
assumptions.jsonl
replay_manifest.json
```

CKC IR layers serialize as one `IRBundle`:

| Layer        | Required content                                                                                          |
| ------------ | --------------------------------------------------------------------------------------------------------- |
| `DocIR`      | Layout-preserving text, tables, source spans, anchors, regions, extraction diagnostics.                   |
| `SegmentIR`  | CQ, recommendation, evidence, exception, definition, table-row, PICO, EtD, metadata segments.             |
| `ClinicalIR` | Normalized statements, terminology bindings, GRADE/CQ/PICO/EtD slots, assumptions.                        |
| `NormIR`     | Rules with context, deontic direction, strength, certainty, temporal qualifiers, exceptions, priorities.  |
| `FormalIR`   | Target-independent constraints, named obligations, normalized actions/contexts, contradiction-query plan. |

IR invariants:

```text
Every semantic claim carries source_region_ids or explicit synthetic_fixture_id.
Every reusable action, condition, population, terminology concept, rule, axiom, equation, and constraint has a stable ID.
IRBundle computes normalized structural hashes for each layer and the whole bundle.
IRBundle preserves assumptions and uncertainty as explicit fields and validates before compilation.
Layered pipelines expose reusable component IDs, duplicate/reuse metrics, and component-combination traces.
Candidate graph hashes include component IDs, evidence emitter IDs, config/policy hashes, prompt-template hashes when used, retrieval/index hashes, compiler/verifier adapter hashes, and transform hashes.
```

IR optimization objective:

```text
CKC optimizes reusable clinical mappings, not isolated translations.
IR candidates minimize duplication while preserving source-grounded semantics, verifier success, deterministic replay, and readable traces.
Metrics include reuse, component count, normalized hash convergence, duplicate rate, compilation predictability, contradiction precision/recall, compactness, and trace explainability.
Compression, MDL, axiom minimization, semantic lattices, equality saturation, proof targets, and invented DSLs are valid registered candidate ideas.
Runtime AI between IR layers remains proposal/evidence until accuracy, determinism, replay, and applicable gates justify stronger claims.
```

Default comparison: `pipe.direct_rule_to_smt` tests direct formal-target translation; `pipe.layered_ckcir_to_smt` tests whether IR-first translation improves hash convergence, component reuse, trace completeness, ablation interpretability, predictability.

## 14. Compiler and Verifier Contracts

M1 formal target: SMT-LIB.

Compiler target interface:

```yaml
target_id: smtlib
input_kind: ir_bundle
output_kind: compiled_artifact
supports:
  named_assertions: true
  unsat_core: true
  counterexample_model: true
  proof_obligations: true
```

Compiled artifact fields: `target_id`, `logic`, `body_path`, `named_assertions`, `target_metadata`, `diagnostics`. Each named assertion records `assertion_id`, IR rule IDs, and source region IDs. Solver name, version, path, executable hash, and runtime profile belong in verifier results/manifests and run manifests. Solver-specific target-profile choices enter `target_metadata` only when they change emitted SMT.

M1 SMT profile:

```text
Default logic is the narrowest solver-supported SMT-LIB logic recorded in the compiled manifest.
Preferred encodings use quantifier-free uninterpreted predicates/functions plus linear integer/real arithmetic, finite enumerations, algebraic datatypes, Boolean connectives, and interval constraints.
String solving, quantifiers, nonlinear arithmetic, arrays, bit-vectors, optimization, probabilities, and higher-order encodings require explicit target profile declarations and may return unsupported_fragment in M1.
Every assertion that can influence a query is named and mapped to IR IDs and source regions.
Contradiction queries are separate named artifacts so syntax, context-overlap, action-sameness, and deontic/factual conflict failures remain distinguishable.
```

SMT semantics required in M1:

```text
Compile claims into named assertions.
Check condition/context overlap separately from action contradiction.
Detect obligation/prohibition and recommendation/contraindication conflicts under satisfiable shared conditions.
Represent non-overlap as documented null result.
Preserve assertion IDs mapped to source regions and IR rule IDs.
Distinguish sat, unsat, unknown, timeout, parse error, compile error, adapter error, and unsupported fragment.
Record UNSAT cores when supported and counterexample models when relevant.
```

Verifier result categories: `schema_failure`, `compiler_failure`, `target_syntax_failure`, `solver_execution_failure`, `semantic_no_conflict`, `semantic_contradiction`, `unknown`, `unsupported_fragment`.

M1 conflict and factual-inconsistency checks:

| Kind                                   | Required idea                                                                                 |
| -------------------------------------- | --------------------------------------------------------------------------------------------- |
| `context_compatibility`                | Finite context overlap via predicates, slots, quantities, and temporal constraints.           |
| `normalized_action_sameness`           | Same action kind, target relation, and discriminating slots after normalization.              |
| `deontic_direction_conflict`           | Positive versus against/contraindicating directions under compatible context and same action. |
| `strict_factual_contradiction`         | Strict factual consequents jointly inconsistent under compatible context.                     |
| `numeric_threshold_empty_intersection` | Quantity/temporal interval constraints have empty intersection.                               |
| `table_value_disagreement`             | Overlapping table guards yield incompatible outputs.                                          |
| `terminology_incoherence`              | Functional key collision or mutually exclusive mapping.                                       |
| `gloss_drift`                          | Deterministic rendered view diverges from current semantic payload.                           |
| `source_metadata_disagreement`         | Singleton metadata values disagree after normalization.                                       |
| `replay_or_certificate_failure`        | Replay mismatch or proof/certificate validation failure.                                      |
| `package_insert_vs_guideline_conflict` | Backlog/gated unless package-insert fixtures are registered.                                  |

Verifier outputs include witness paths: `source regions -> IR rules/statements -> named SMT assertions -> solver result -> core/model/proof obligation -> report finding`.

Compiler/verifier self-improvement invariant:

```text
Compilers and verifier adapters may be candidate-editable surfaces.
The locked evaluator is separate and includes schema validation, target syntax checks, solver invocation identity, expected fixture outcomes, trace completeness, and replay.
A modified compiler or verifier adapter cannot weaken the evaluator that scores its own promotion.
Accepted compiler/verifier changes require before/after evidence and rollback under G-SELF-IMPROVE; evaluator semantic changes require G-EVALUATOR-MIGRATION.
```

## 15. Benchmark Tasks and Metrics

Locked-evaluator invariant:

```text
Before candidate runs, each experiment writes evaluator_lock.json and records its hash in the run manifest.
The evaluator lock includes fixture hashes, gold-label hashes, source hashes, schema registry hash, scoring config hash, metric definitions hash, evaluator code hash, benchmark runner hash, toolchain hashes, lockfile hashes, environment profile hash, seeds, budget policy hash, and experiment plan hash.
Before each candidate run, attempt_run_lock.jsonl records evaluator_lock_hash, candidate_graph_hash, source hashes, patch/diff/workspace hashes, budget, and run-plan hash.
The runner rejects attempts that modify locked evaluator surfaces, metrics, fixtures, gold labels, schemas, acceptance criteria, or gate definitions in the same scoring loop.
Evaluator or metric evolution is a separate governed migration used only by later experiments.
```

Smoke experiment example:

```yaml
id: exp.m1_public_smoke
fixtures: [fixture.public_guideline_cq_grade, fixture.public_guideline_jstage_jats, fixture.public_guideline_table_heavy, fixture.synthetic_contradiction_ja, fixture.synthetic_negative_control_ja]
pipelines: [pipe.direct_rule_to_smt, pipe.layered_ckcir_to_smt]
tasks: [task.extract_text_layout, task.source_span_grounding, task.segment_clinical_statements, task.terminology_normalization, task.semantic_normalization, task.ir_reuse, task.component_compactness, task.formal_compilability, task.contradiction_detection, task.trace_completeness, task.determinism_idempotency]
repetitions: 2
seed: recorded_in_registry
runtime_ai: false
evaluator_lock: evaluator.m1_public_smoke.lock
budget_policy: bounded_smoke
ablate: [retrieval_off, terminology_grounding_off, exceptions_off]
```

Benchmark tasks:

| Task                                | Measures                                                                                                                                        |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `task.extract_text_layout`          | Text fidelity, reading order, page/block/table spans.                                                                                           |
| `task.source_span_grounding`        | Exact/partial span overlap, citation precision/recall.                                                                                          |
| `task.segment_clinical_statements`  | CQ/recommendation/evidence/exception boundary F1.                                                                                               |
| `task.terminology_normalization`    | Code accuracy, alias handling, MRR, unresolved mentions.                                                                                        |
| `task.semantic_normalization`       | Population/action/condition/modality/strength/certainty/exception slot F1.                                                                      |
| `task.ir_reuse`                     | Component reuse hit rate, duplicate rate, normalized hash convergence.                                                                          |
| `task.component_compactness`        | Component count, fan-in/fan-out, compression/MDL and axiom/rule minimization proxies.                                                           |
| `task.formal_compilability`         | Schema pass, compiler pass, target parse pass, solver pass.                                                                                     |
| `task.semantic_equivalence`         | IR tree distance, normalized hash agreement, entailment/convergence.                                                                            |
| `task.contradiction_detection`      | Required synthetic contradiction hit, required negative-control null, scored real/exploratory precision/recall, core quality, witness validity. |
| `task.trace_completeness`           | Required trace fields, lineage reachability, artifact link validity.                                                                            |
| `task.determinism_idempotency`      | Hash stability across repeated runs with same cache/seed.                                                                                       |
| `task.metamorphic_robustness`       | Stability under punctuation, layout, synonym, kana/kanji, section-order variants.                                                               |
| `task.source_variant_repeatability` | HTML/PDF/XML parity for the same guideline when variants exist.                                                                                 |
| `task.ai_contribution_quality`      | Schema-valid first pass, accepted artifact rate, repair cycles, hallucination defects.                                                          |
| `task.weak_model_lift`              | Gain from staged IR pipeline over direct formalization.                                                                                         |
| `task.autonomous_loop_evidence`     | Proposal validity, patch compliance, locked scoring, promotion precision, ledger completeness.                                                  |

Metric values are exact rationals; unavailable values are omitted with diagnostics. Zero denominators emit residual or not_applicable by task schema. Rankings use only locked metrics; raw rows remain.

Required score groups: extraction; grounding; segmentation; terminology; semantic slots; IR reuse; compactness; formal compilation; synthetic contradiction/null; contradiction precision/recall; trace completeness; reproducibility; AI acceptance/repair defects; research-loop outcomes. Reports show raw rows before weighted totals.

Failure taxonomy includes at least:

```text
source_fetch_failure permission_limited source_drift extraction_uncertain table_structure_uncertain cross_reference_unresolved span_grounding_missing segmentation_boundary_error terminology_unmapped terminology_ambiguous terminology_incoherent semantic_slot_missing policy_missing unsupported_ir_fragment schema_invalid compiler_error target_parse_error solver_timeout solver_unknown process_crash budget_exhausted repair_limit_exceeded near_miss null_result unauthorized_surface_edit locked_evaluator_modified false_positive_conflict false_negative_conflict trace_incomplete replay_mismatch ai_hallucinated_source ai_schema_violation
```

Benchmark outputs evaluate the reusable-IR thesis within fixture scope by reporting reuse, compactness, hash convergence, duplicate rate, formal compilability, synthetic contradiction/null outcomes, contradiction precision/recall, trace completeness, replay, weak-model lift, autonomous-loop ledger evidence, ablations, failures, rankings, and next experiments.

## 16. Trace, Proof, Replay, and Report Artifacts

Trace preserves lineage from Japanese source text to findings.

Required trace exports:

```text
trace_bundle.json
derivation_dag.json
claim_evidence_table.csv
mapping_hypergraph.json
component_reuse_graph.json
axiom_dependency_graph.json
compactness_frontier.json
proof_tree_or_solver_witness.json
candidate_diff.json
lineage_index.json
```

Optional trace export: `semantic_lattice.json`.

Required trace fields:

```text
trace_id, run_id, document_id/hash, candidate_pipeline_id, component_ids, transformation_ids, source spans/anchors/regions/text hashes, permission refs, retrieval query/index/ranker refs when used, prompt/tool/model identifiers when AI is used, assumptions, uncertainty, confidence, validation status, IR entity/rule IDs, compiled artifact refs, named assertions, solver/proof outputs, UNSAT cores, counterexamples, proof obligations, metric refs, report finding refs
```

Trace/proof structures:

| Structure               | M1 contract                                                                                                            |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `DerivationDAG`         | Source, extraction, segment, retrieval, grounding, IR, compile, verify, metric, report nodes; operation-labeled edges. |
| `ProofNode`             | Rule/operation, conclusion hash, premise hashes, source support, diagnostics, verifier/proof refs.                     |
| `ClaimEvidenceTable`    | Report claims linked to source regions, IR rules, formal assertions, verifier outputs, metrics, replay hashes.         |
| `MappingHypergraph`     | Many-to-many source/IR/formal links.                                                                                   |
| `ComponentReuseGraph`   | Reusable IR components with source support, use sites, normalized hashes, compile-target consumers.                    |
| `AxiomDependencyGraph`  | Axioms, rules, equations, constraints, obligations, derived claims, dependency edges, verifier refs.                   |
| `CompactnessFrontier`   | Candidate points comparing component count, reuse, formal compilability, contradiction precision, trace completeness.  |
| `WitnessContext`        | Context/action/table witness data for conflict checks.                                                                 |
| `ConstraintCoreWitness` | Named constraints, internal core, optional external UNSAT core hash.                                                   |
| `CandidateDiff`         | Cross-pipeline differences in segments, bindings, IR, assertions, verifier outcomes, metrics.                          |
| `LineageIndex`          | Queryable index from artifact/finding back to source spans and forward to verifier/report artifacts.                   |

Experiment ledger files:

```text
experiment_ledger.jsonl
experiment_ledger.csv
experiment_ledger.md
promotion_decisions.jsonl
evaluator_lock.json
attempt_run_lock.jsonl
```

Each JSONL attempt record includes run/attempt/candidate IDs, proposal/patch/diff/workspace hashes, candidate_graph_hash, evaluator_lock_hash, budget, outcome, classification, metrics, failures, promotion decision/scope, replay hash, trace completeness, and notes. The ledger is append-only; CSV/Markdown are derived views; rejected attempts remain queryable. Promotion decisions cite evaluator lock, candidate diff, metrics, trace completeness, replay hash, regression checks, and applicable gate evidence.

Replay manifest records command, input hashes, schema registry hash, toolchain manifest hash, environment profile hash, lockfile hashes, expected output hashes, and accepted effects. Replay compares canonical output hashes, not timestamps. Missing external tools or permissioned source bytes emit `replay_identity_unsupported`; mismatches emit symmetric-difference diagnostics. Recorded AI outputs replay as fixture bytes; live LLM calls are nondeterministic.

Claim tiers:

| Tier                     | Meaning                                                                      |
| ------------------------ | ---------------------------------------------------------------------------- |
| `s0_replayable`          | Artifact bytes replay and schema/proof/trace checks pass over frozen inputs. |
| `s1_admitted`            | S0 plus accepted deterministic validation/admission.                         |
| `s2_research_evidence`   | S1 plus valid benchmark/gate evidence for a stated research claim.           |
| `s3_clinical_regulatory` | S2 plus clinical/regulatory/deployment assurance evidence.                   |

Run output layout:

```text
runs/<run_id>/
├── manifest.json
├── replay_manifest.json
├── evaluator_lock.json
├── attempt_run_lock.jsonl
├── experiment_ledger.{jsonl,csv,md}
├── promotion_decisions.jsonl
├── report.md
├── report.json
├── ranking.csv
├── score_breakdown.json
├── failure_taxonomy.json
├── ablations.json
├── source_drift.json
├── artifacts/
├── traces/{trace_bundle.json,derivation_dag.json,claim_evidence_table.csv,mapping_hypergraph.json,component_reuse_graph.json,axiom_dependency_graph.json,compactness_frontier.json,proof_tree_or_solver_witness.json,candidate_diff.json,lineage_index.json}
├── diffs/
└── logs/events.jsonl
```

`artifacts/` contains each pipeline's minimum artifact set, including `extracted_document.jsonl`, `source_spans.jsonl`, `source_anchors.jsonl`, and `source_regions.jsonl`; pass-through and `not_applicable` artifacts use the same envelope and trace fields.

`report.md` and `report.json` include corpus/source hashes, permission/drift summary, ranking, raw/weighted scores, autonomous-loop summary when run, matrix coverage, failures, reuse/compactness, formal outcomes, syntax/semantic failures, unknowns, timeouts, available cores/models/obligations, null results, trace examples, determinism/idempotency, ablations, weak-model lift when registered, failure taxonomy, next experiments. Reports use controlled templates or deterministic renderers; free-form clinical authority claims are invalid.

## 17. AI Artifact Handling

AI artifacts include structured outputs, synthetic fixtures, gold-label drafts, repair suggestions, verifier-guided patches, candidate IR drafts, critique/adjudication notes, and generated data. Prompt text, model IDs, tool calls, and raw outputs are runtime artifacts recorded in manifests/envelopes.

AI artifacts are proposal/evidence until admitted by CKC validation and applicable gates for the artifact kind and claim tier. AI output is not source evidence for a clinical claim; it may point to source regions, propose IR/tests/patches, or critique artifacts. Automated agents may edit only experiment-declared candidate surfaces.

AI envelope fields:

```json
{
  "origin": "ai_generated",
  "authority": "evidence_discovery_only",
  "ai": {
    "model_family": "registered_model_family",
    "model_id": "recorded_in_manifest",
    "prompt_id": "prompt...",
    "prompt_template_hash": "sha256:...",
    "tool_ids": [],
    "temperature": "recorded_if_available",
    "seed": "recorded_if_available",
    "output_bytes_hash": "sha256:...",
    "generation_timestamp": "runtime_metadata_only"
  }
}
```

AI applicability rule:

```text
AI-generated artifacts validate against schemas for their artifact kind.
AI-generated semantic claims link to source regions or explicit `synthetic_fixture_id`.
When admitted into fixtures, candidate outputs, benchmark evidence, traces, reports, compiled artifacts, verifier results, or accepted semantic artifacts, AI-generated artifacts pass the same artifact-kind and claim-tier applicable schema, provenance, source-grounding or `synthetic_fixture_id`, canonicalization, trace, benchmark, applicable compiler/verifier/report, replay, and gate checks as hand-authored artifacts.
Critique notes, repair suggestions, fixture drafts, and proposal artifacts receive only artifact-kind-relevant schema, provenance, trace, and claim-tier checks until admitted into stronger artifact classes.
AI artifacts are scored for acceptance rate, repair burden, hallucinated source refs, schema violations, and verifier pass rate when those metrics apply.
Runtime LLM calls are disabled by default in M1; private clinical text and patient data are excluded from M1 AI calls.
Live LLM use requires explicit experiment flags and records prompts, tool calls, model IDs, outputs, and source context hashes.
Recorded LLM outputs may be deterministic fixtures.
Prompt templates are hash-identified runtime artifacts; accepted prompt-template changes require locked validation evidence and rollback under G-SELF-IMPROVE.
```

## 18. Reproducibility and Observability

Reproducibility invariants:

```text
Python and Rust dependencies are locked.
Schemas, registries, tests, synthetic fixtures, and gold labels are committed.
Public raw documents are content-hash cached when permission permits.
Source metadata records expected/observed hashes.
Canonical JSON uses sorted keys and stable string/rational formatting.
Run manifests record git commit, lockfile/schema hashes, environment profile, solver paths, tool versions, seeds, candidate/model/prompt IDs, source hashes, candidate graph hashes, evaluator lock hashes, and output hashes.
EvaluatorLock is materialized before attempts and immutable for that experiment.
Experiment ledgers record all attempts, including crashes, timeouts, null results, near misses, repairs, rejected regressions.
Repeated deterministic runs over the same cache/seed produce matching canonical content hashes except declared runtime metadata fields.
Source hash changes produce source_drift.json and invalidate stale source-grounded scores unless compared as drift.
```

Observability event fields:

```text
event_id, run_id, attempt_id, candidate_id, component_id, stage, level, logical_time, started_at, ended_at, duration_ms, input_hashes, output_hashes, outcome, diagnostics, budget_counters, repair_counters
```

Observability invariants:

```text
Python logging and Rust tracing emit JSONL events.
Events include run, attempt, candidate, component, stage, outcome, input/output hashes, diagnostics, budgets, repair counters, timing metadata.
Tool paths, solver/adapter versions, model identifiers, and redacted environment metadata are recorded.
Logs are not accepted semantics; accepted semantics live in validated artifacts.
```

## 19. First Milestone Acceptance Criteria

M1 is complete when:

1. `uv run ckc registry check` validates method, candidate, corpus, experiment, evaluator, prompt, policy, index, schema, `source_processors.yaml`, and gate registries. Every specified method family has category, aliases, candidate roles, adapter status, compatibility metadata, benchmark tags, and gate_refs. Runnable adapters are required only for `m1_required` method entries and required runnable component IDs; runner/registry/evidence IDs validate as registry or runner hook IDs, not `ckc component run` targets.
2. `uv run ckc schema export --out schemas/` regenerates committed JSON Schema from Rust types without uncommitted diffs.
3. `cargo test --workspace` passes for IDs, rational normalization, string policies, canonical JSON, artifact envelopes, schema export, registry validation, IR validation, run-plan canonicalization, and SMT-LIB emission.
4. `uv run pytest` passes for fetch, extraction, segmentation, retrieval baseline, terminology baseline, candidate runner, metrics, reports, trace exports, replay checks, and fixture tests.
5. `uv run ckc corpus fetch --corpus registry/corpora.yaml` fetches or reconstructs public fixtures and records content hashes, permission records, and source drift.
6. Fixture set includes one Minds/GRADE/CQ-style public guideline, one J-STAGE/JATS-like public guideline or excerpt, one table-heavy/layout-challenging public excerpt, one injected Japanese contradiction, and one non-conflicting negative control.
7. `uv run ckc run --experiment exp.m1_public_smoke --out runs/exp.m1_public_smoke` executes with `runtime_ai: false`.
8. Required pipelines `pipe.direct_rule_to_smt` and `pipe.layered_ckcir_to_smt` run on the smoke experiment and emit the minimum artifact set, including `extracted_document.jsonl`, `source_anchors.jsonl`, `source_regions.jsonl`, and pass-through or `not_applicable` artifacts where appropriate.
9. Candidate outputs validate before scoring; invalid outputs are scored as schema/syntax failures.
10. SMT-LIB compilation emits named assertions and `compiled/smt/assertion_map.json` from assertion IDs to IR rule IDs and source regions for both required pipelines.
11. SMT verification records syntax results, semantic results, named assertions, UNSAT cores or counterexamples when applicable, unknowns/timeouts distinctly, and proof obligations where available.
12. The required synthetic injected contradiction is detected as `semantic_contradiction` by at least one required pipeline with full trace through synthetic/source spans, anchors, regions, IR rules, named SMT assertions, solver output, metric row, and report finding.
13. The required synthetic non-conflicting negative control produces `semantic_no_conflict` and `documented_null_result` in each required pipeline with the same full trace. Real/public and exploratory cases retain scored false-positive/false-negative handling, not as a substitute for required synthetic hit/null acceptance.
14. `report.md` and `report.json` rank candidates, show locked raw metric rows and weighted score breakdowns, identify failures, include ablations, verifier outputs, reuse/compactness evidence, null results, next experiments without stronger ungated claims.
15. `failure_taxonomy.json`, `ablations.json`, `ranking.csv`, `score_breakdown.json`, and `source_drift.json` are emitted.
16. Required trace exports include lineage index, derivation DAG, mapping hypergraph, component reuse graph, axiom dependency graph, compactness frontier, proof/solver witness, candidate diff, links to source spans, anchors, and regions.
17. `lineage_index.json` traces at least one report finding to Japanese source spans, anchors, regions, IR rules, compiled SMT assertions, solver output, metric rows, and replay manifest.
18. `candidate_diff.json` compares direct and layered pipeline outputs at segment, terminology, IR, formal assertion, verifier, and metric levels.
19. A bounded matrix materializes stable run plans, executes compatible combinations, and reports tested, skipped-incompatible, unsupported, failed, dominated, equivalent, Pareto-front, promising combinations.
20. Repeated deterministic replay over the same cache reports stable canonical content hashes for accepted artifacts.
21. The §17 AI artifact applicability rule is enforced for admitted AI-generated CKC artifacts and draft-only AI artifacts.
22. M1 reports make no clinical, patient-care, CDS runtime, SaMD, deployment, or regulatory authority claims.
23. `uv run ckc research loop --experiment exp.m1_autonomous_smoke --out runs/exp.m1_autonomous_smoke` executes with `runtime_ai: false`, bounded fixtures, fixed `evaluator_lock.json`, declared editable surfaces, registered budgets, and no live clinical claims. It proposes at least one patch, rejects unauthorized surface edits, scores at least one valid attempt with the locked evaluator, records run-local promotion/rejection in `experiment_ledger.jsonl`, emits CSV/Markdown ledger summaries, preserves crash/timeout/null evidence if present, and replays any locally promoted attempt deterministically.

## 20. Backlog and Gates

Backlog is registered design pressure, not M1 acceptance.

Locked M1 smoke reports may include raw metric rows, rankings, reuse counts, compactness proxies, required synthetic contradiction/null outcomes, fixture contradiction precision/recall, trace completeness, deterministic replay, ablations, failures, unsupported cases, and null results as S0/S1 smoke measurements under an EvaluatorLock. Gates govern stronger claims: released benchmarks, corpus-scale/generalized/calibrated/adjudicated results, clinical/regulatory/deployment claims, new source-use/export classes, gold-corpus quality, promoted extractor quality, runtime-oracle claims, and registry/status self-improvement. Baseline permission records, smoke fixtures, synthetic labels, deterministic fixture extraction, redaction, drift checks, diagnostics, run-local research-loop promotion/rejection, and locked smoke reporting remain ordinary M1 behavior.

Gate table:

| Gate                    | Trigger                                                                                                                                                                                                       | Evidence object                  | Claims enabled                                      |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- | --------------------------------------------------- |
| `G-SOURCE-PERMISSION`   | New source family, redistribution mode, quote policy, or exported artifact class beyond baseline PermissionRecord behavior.                                                                                   | `SourcePermissionProfile`        | Source-use/export claims beyond locked M1 fixtures. |
| `G-GOLD-CORPUS`         | Adjudicated, clinician-reviewed, gold-standard, released, or reusable corpus-quality claims beyond smoke fixtures, synthetic labels, and draft labels.                                                        | `GoldCorpusEvidence`             | Gold-corpus/adjudication-quality claims.            |
| `G-EXTRACTOR-ADAPTER`   | SourceGraph-affecting extractor promotion or generalized layout/OCR/table quality claims beyond fixture diagnostics.                                                                                          | `ExtractorAdapterRecord`         | Extractor soundness for declared source profile.    |
| `G-RET-PARITY`          | Retrieval, dense retrieval, late interaction, graph retrieval, reranking, or citation-quality claims beyond raw locked fixture rows.                                                                          | `RetrievalParityReport`          | Retrieval-quality claims.                           |
| `G-PORTFOLIO`           | Independent verifier/backend agreement or solver-portfolio robustness is claimed.                                                                                                                             | `VerifierPortfolioReport`        | Portfolio verification claims.                      |
| `G-AIR-FULL`            | Non-identity AIR, ontology, argumentation, temporal, equality-saturation, or abstract-domain logic affects accepted outputs or is claimed as sound.                                                           | `AIRDomainRecord`                | Richer abstract-domain claims.                      |
| `G-REBIND`              | Proof or trace transport across source editions or terminology editions is claimed.                                                                                                                           | `RebindingEvidence`              | Rebinding/change-impact claims.                     |
| `G-EMIN`                | Released benchmark, corpus-scale evaluation, generalized semantic-equivalence/convergence, adjudicated contradiction benchmark, or calibrated empirical performance is claimed beyond locked M1 smoke tables. | `BenchmarkRelease`, `EMinReport` | S2 research measurements.                           |
| `G-EVALUATOR-MIGRATION` | Fixtures, gold labels, schemas, metric definitions, scoring semantics, evaluator code, or acceptance thresholds change for future scoring.                                                                    | `EvaluatorMigrationEvidence`     | New future-evaluator baseline.                      |
| `G-MDL`                 | Calibrated compression/reuse payoff, Pareto optimality, MDL preference, or model-selection recommendation is claimed beyond raw compactness proxies and rankings.                                             | `MDLEvidence`                    | Calibrated model selection.                         |
| `G-RUNTIME-ORACLE`      | Runtime model calls, layered runtime model calls, or IR-stage oracle fidelity is claimed beyond proposal/evidence status.                                                                                     | `RuntimeOracleReport`            | Runtime-oracle/layered-model fidelity.              |
| `G-SELF-IMPROVE`        | Automated registry/status promotion modifies accepted generators, prompt templates, terminology policies, indexes, compilers, verifier adapters, metric/report code, source-processing rules, or adapters.    | `SelfImprovementEvidence`        | Gated self-improvement under locked evaluator.      |
| `G-PROB`                | Probabilities, stochastic transitions, weights, risks, rewards, or probabilistic facts affect accepted outputs.                                                                                               | `ProbabilisticProfileRecord`     | Probabilistic claims.                               |
| `G-WORLD-MODEL`         | Latent state, trajectory, image-derived, multimodal, or world-model observations affect outputs.                                                                                                              | `WorldModelProfileRecord`        | World-model claims.                                 |
| `G-LIVE-PATIENT`        | Live, deidentified, claims, registry, linked, or real-world patient data enters CKC.                                                                                                                          | `GovernedPatientDataProfile`     | Patient-data handling.                              |
| `G-S3`                  | Clinical, regulatory, patient-care, CDS, SaMD, deployment, safety, privacy, usability, security, or post-market authority is claimed.                                                                         | `S3AssuranceEvidence`            | Clinical/regulatory/deployment.                     |

Gate invariants:

```text
GateEvidenceRef names gate, subject hash, evidence object hash, replay identity hash, enabled claims, limitations, rollback/sunset condition.
Missing required gate evidence emits Residual(class=deferred_gate_required) only for the stronger gated claim, not lower-tier locked smoke reporting.
Invalid gate evidence does not invalidate lower-tier artifacts whose schema, trace, replay, and verifier checks pass.
Gate evidence is replayable or explicitly marked non-authoritative metadata.
Candidate loops operate inside locked experiments; evaluator changes require separate governance evidence before candidate ranking.
A self-improvement candidate cannot modify the evaluator, metric definitions, fixture hashes, schemas, or acceptance criteria that score its own promotion.
```

SelfImprovementEvidence records subject surface, parent candidate, patch hash, artifact diffs, before/after locked-evaluator performance, metric deltas, regression checks, trace completeness, replay hash, rollback, limitations, and linked ledger attempts. Metric/report renderer changes affecting scoring require `G-EVALUATOR-MIGRATION` before later ranking. Registry/status self-improvement promotion requires schema validation, source grounding when semantic artifacts are affected, applicable compiler/verifier checks, trace completeness, benchmark evidence, replay, rollback, and applicable gates.

EvaluatorMigrationEvidence records old/new evaluator lock hashes, changed fixtures/gold/schemas/metrics/scoring/evaluator code, rationale, frozen baseline cohort, old-vs-new scores, regression and rank-stability analysis, source/gold provenance, limitations, replay hashes, and rollback. New evaluator identities cannot retroactively promote candidates scored under the old evaluator or score their own migration.

Registered backlog tracks:

| Track                                                                                                    | M1 disposition                                                                            |
| -------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| CKC-GEN core, accepted generator base, AIR finite-set identity, final proof-carrying compiler extraction | `pipe.ckc_gen_air_kernel`, registered backlog.                                            |
| ASP/Clingo defeasible rules and exceptions                                                               | High-value backlog after SMT harness proves artifact loop.                                |
| Lean/Rocq/Coq/Isabelle/Why3 proof assistants                                                             | Verifier-portfolio backlog and proof-carrying target candidates.                          |
| CQL/ELM and FHIR CPG                                                                                     | Computable-guideline IR backlog; clinical execution gate-only.                            |
| OWL/SHACL/RDF reasoning                                                                                  | Terminology and ontology backlog; finite resources may be used in M1.                     |
| DMN/FEEL and openEHR GDL2                                                                                | Decision-table/pathway formalization backlog.                                             |
| TLA+/Alloy/model checking                                                                                | Pipeline, temporal, state, and counterexample backlog.                                    |
| Probabilistic logic, PRISM/Storm, world models                                                           | Gated research only.                                                                      |
| E-graphs/equality saturation                                                                             | Normalization and compactness candidate backlog.                                          |
| Runtime AI, agent DSLs, layered model pipelines                                                          | Proposal/evidence only until `G-RUNTIME-ORACLE` and related gates pass.                   |
| Large public guideline corpus sweeps                                                                     | Same artifact architecture; corpus-scale claims require registered benchmark evidence.    |
| Contradiction discovery across encoded guidelines                                                        | Backlog/gated S2 research claim after corpus-scale benchmark and false-positive controls. |
| UI, visualization, semantic lattice browsers, lineage explorers                                          | Backlog view layers over trace artifacts.                                                 |
| Multimodal medicine, imaging data, biology state models                                                  | `G-WORLD-MODEL` and related gates.                                                        |
| Patient data platforms, hospital workflows, CDS Hooks, SMART, SaMD, regulatory deployment                | Excluded from M1 acceptance; `G-LIVE-PATIENT` and `G-S3` gate-only.                       |

The M1 kernel is source-grounded, schema-valid, deterministic where accepted, replayable, traceable, and comparable. Richer reasoning, automated improvement, runtime AI, patient data, multimodal inputs, proof-carrying kernels, clinical deployment, and regulatory authority require registered candidates and applicable gates without weakening M1 invariants. Autonomous self-improvement is gated capability, not permission to rewrite validators, metrics, schemas, evaluator locks, or acceptance criteria during the same scoring loop.
