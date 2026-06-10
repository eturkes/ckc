# Category 1: Computable Guideline IR, CDS Standards, and Workflow Interfaces

## 1. CQL/ELM Clinical Logic IR

### Purpose
Clinical Quality Language (CQL) is a high-level, domain-specific language for expressing clinical knowledge (quality measures and decision-support logic); Expression Logical Model (ELM) is its machine-readable canonical compiled form intended for sharing and execution.

### Maintainer/Standards body
HL7 International, stewarded by the Clinical Decision Support (CDS) Work Group with CQI Work Group co-sponsorship. Current normative release: CQL 1.5.3 (ANSI normative; Release 1 Errata 2, based on FHIR 4.0.1). Release 2.0.0 is in trial-use ballot (CQL 2.0.0-ballot, September 2025). Originally developed under the Clinical Quality Framework (CQF) initiative, a public-private partnership sponsored by CMS and ONC.

### Conceptual model
CQL is a declarative, expression-oriented DSL with an ANTLR4 grammar; ELM is an abstract syntax tree (AST) serialized in XML/JSON via XMI/UML schema. Logic is built from libraries of named expression definitions, parameters, contexts (Patient/Population), and queries over retrieve expressions that target a data model (typically FHIR or QDM).

### Expressiveness/Semantics
Supports set-based query, temporal operators (interval algebra over time), terminology operations, code/value-set membership, arithmetic, three-valued logic (TRUE/FALSE/NULL) with null-propagating arithmetic and short-circuit logical operators. Lacks native imperative control flow, side effects, or process/workflow constructs. Formal semantics specified in English prose plus UML; no published mechanized denotational semantics.

### Composability/Modularity
Libraries are first-class artifacts with versioning, including/using imports, parameterization, and accessible/private visibility. Expressions reference one another by name; data and terminology dependencies are declarable.

### Suitability for autoformalization to IR
High. CQL's surface syntax is amenable to LLM generation; ELM provides a normalized AST that two syntactically different CQL inputs compile to canonical form, enabling structural idempotency checks. Convergence can be measured by ELM tree-diff after canonical normalization. Library/parameter naming discipline is needed to ensure cross-run ontology stability.

### Formal verification potential
No native theorem-proving backend. The closest published work is Hekmatnejad, Simms, and Fainekos (2019), "Model Checking Clinical Decision Support Systems Using SMT" (arXiv:1901.04545), which translates HL7 CDS Knowledge Artifact logic (sibling to ELM) into Z3 SMT models. No published Coq/Isabelle/NuSMV translations of CQL/ELM directly. Three-valued logic and null semantics require careful encoding for SMT.

### Tooling/Ecosystem maturity
Mature reference implementations: cqframework/clinical_quality_language (Java/Kotlin CQL-to-ELM translator, current release 4.8.0, May 2026), cqframework/cql-execution (JavaScript engine, 80 stars), HAPI FHIR embedded CQL engine, IBM/Alvearie CQL evaluator. NIH/CMS-sponsored ecosystem (eCQM measures, opioid CDS).

### Japan-specific considerations
No known production CQL deployment in Japan. JP Core FHIR profiles do not yet bind CQL libraries. MHLW/JAHIS standardization activities focus on FHIR profiles and SS-MIX, not CQL. No published Japanese-language CQL authoring tooling or value-set bindings to MEDIS-DC standard masters.

### Interoperability with the others on this list
Embedded in FHIR Library resources (#2); referenced by PlanDefinition/Measure (#2) and CPG-on-FHIR (#3). Uses FHIRPath (#4) as an embedded sub-language; queries terminology via FHIR Terminology Services (#5). CDS Hooks services (#6) commonly back logic with CQL. Distinct from openEHR GDL2 (#8); DMN/FEEL (#9) and BPMN (#10) are orthogonal.

### Limitations/Known issues
No mechanized formal semantics; null/three-valued logic complicates verification. Limited support for temporal series and stateful reasoning. Steep learning curve outside the US eCQM community. ELM XML is verbose and version-coupled to FHIR releases.

### Training data proxy
Moderate. cqframework/clinical_quality_language has 313 stars and 143 forks (May 2026); CMS eCQM ecosystem provides several thousand published CQL files. Stack Overflow tag presence minimal. HL7 Zulip CQL stream is the primary community channel. LLMs have seen the spec text and example libraries but limited diverse codebases.

---

## 2. FHIR Clinical Reasoning Resources

### Purpose
FHIR resources (Library, PlanDefinition, ActivityDefinition, Measure) that package shareable, executable clinical knowledge artifacts — decision rules, order sets, protocols, and quality measures — within the FHIR ecosystem.

### Maintainer/Standards body
HL7 International, Clinical Decision Support and Clinical Quality Information Work Groups. Current published version: FHIR R5 (5.0.0, published 26 March 2023); R6 normative ballot in progress (currently v6.0.0-ballot4, generated 18 May 2026) targeting normative status for Library, Measure, ActivityDefinition, PlanDefinition; final R6 publication expected in 2027 at the earliest.

### Conceptual model
Resource-graph metadata wrappers. Library wraps logic (often CQL/ELM as base64-encoded attachment); ActivityDefinition is a parameterized template for a single Request resource (MedicationRequest, ServiceRequest, etc.); PlanDefinition is a hierarchical action tree with triggers, conditions, related actions, and grouping behaviors representing event-condition-action rules, order sets, or protocols; Measure binds populations and stratifiers to CQL expressions.

### Expressiveness/Semantics
Event-Condition-Action with hierarchical action grouping, related-action ordering (before/after/concurrent), selection-behavior (any/all/one-or-more/at-most-one), required-behavior, cardinality-behavior. Dynamic values via embedded expression (FHIRPath/CQL). Semantics defined by `$apply` operation that produces RequestOrchestration/CarePlan output bundles.

### Composability/Modularity
PlanDefinitions reference ActivityDefinitions and Libraries by canonical URL; nesting via action.definitionCanonical pointing to other PlanDefinitions. Versioned canonical URLs allow controlled artifact reuse. CRMI (Canonical Resource Management Infrastructure) IG governs packaging.

### Suitability for autoformalization to IR
High for the orchestration/structural layer. Action trees are straightforward LLM targets. JSON/XML canonical serialization permits structural idempotency comparison. Logic semantics delegated to embedded CQL/FHIRPath — convergence depends on those layers. PlanDefinition action graph is convergent under canonical-URL referencing.

### Formal verification potential
No native verification. Action ordering and selection-behavior have informal English semantics; would require translation to a process formalism (BPMN-like Petri nets or pi-calculus) for model checking. Contradictions across PlanDefinitions are not detectable natively.

### Tooling/Ecosystem maturity
HAPI FHIR implements `$apply`, `$package`, `$data-requirements`. CQF Ruler reference implementation. Production use in US Opioid CDS, immunization forecasting (CDC), maternal health IGs.

### Japan-specific considerations
JP Core (FHIR Japanese implementation research WG, JAMI NeXEHRS) currently profiles base FHIR resources but does not include Clinical Reasoning module profiles as of the released v1.2.0 (published 30 July 2025, still based on FHIR 4.0.1); v1.3.0-dev (also on FHIR 4.0.1) is in active development as of early 2026. No MHLW/JAHIS-mandated CDS use of PlanDefinition. Adoption requires JP-specific binding of terminology to MEDIS-DC masters.

### Interoperability with the others on this list
Library wraps CQL/ELM (#1). Uses FHIRPath (#4) for inline expressions, StructureMap (#4) for ActivityDefinition→resource transforms, Terminology Services (#5) for value sets. Invoked via CDS Hooks (#6) and SMART (#7) for app-launch. Conceptually parallel to but distinct from openEHR GDL2 (#8). CPG-on-FHIR (#3) profiles these resources.

### Limitations/Known issues
Action-graph semantics under-specified for concurrency. PlanDefinition can become unwieldy for large guidelines. No native conflict detection across artifacts. Heavy reliance on embedded CQL means tool chains must support both.

### Training data proxy
Moderate-to-high. hapifhir/hapi-fhir has 2.3k stars and 1.5k forks (May 2026); hapifhir/hapi-fhir-jpaserver-starter has 520 stars and 1.3k forks; many implementation guides on hl7.org; substantial Zulip chat archive. Stack Overflow `hl7-fhir` tag has thousands of questions but few specifically on Clinical Reasoning. LLMs reliably generate well-formed PlanDefinition skeletons.

---

## 3. CPG-on-FHIR / Clinical Quality Framework Artifact Packaging

### Purpose
HL7 implementation guide (Clinical Practice Guidelines on FHIR) defining patterns, profiles, conformance, and a levels-of-enablement model for representing entire computable clinical practice guidelines using FHIR Clinical Reasoning resources.

### Maintainer/Standards body
HL7 Clinical Decision Support WG (Clinical Quality Framework initiative). Current published version: 2.0.0 (STU2, uses FHIR R4). Builds on parallel Using-CQL-With-FHIR IG and CRMI IG.

### Conceptual model
Layered "levels of enablement" (L1 narrative → L2 semi-structured → L3 computable/structured → L4 executable). Defines a CPG knowledge-engineering process integrated with guideline authoring. Profiles PlanDefinition into CPGStrategy, CPGPathway, CPGRecommendation, CPGCaseFeatureDefinition, etc., and ActivityDefinition into typed recommendation activities.

### Expressiveness/Semantics
Inherits FHIR Clinical Reasoning semantics (#2). Adds conformance constraints on artifact packaging, identifier patterns, and metadata for provenance, evidence linkage (via EBM-on-FHIR), and recommendation strength. Manual / semi-automated / automated implementation paths defined.

### Composability/Modularity
Strong: explicit canonical-URL versioning, $package operation produces transportable Bundles. Profiles compose with EBM-on-FHIR for evidence and DEQM for measurement. Encourages pathway/strategy decomposition.

### Suitability for autoformalization to IR
High for structural/metadata layer; medium for clinical logic (depends on CQL). The layered L1-L4 model is well-suited to LLM staged generation. The profile constraints provide guardrails that improve convergence/idempotency by restricting valid shapes.

### Formal verification potential
None native. Recommendations and pathways could be lifted into process models, but the IG itself does not specify verification. Cross-guideline contradiction detection requires external reasoning over the constrained PlanDefinition graph plus shared terminology.

### Tooling/Ecosystem maturity
CQF Ruler reference implementation; sample IGs for CHF, opioid prescribing, WHO SMART Guidelines (ANC, immunization). Smaller community than base FHIR. Lichtner, Alper, Jurth, Spies, Boeker, Meerpohl, and von Dincklage (2023, "Representation of evidence-based clinical practice guideline recommendations on FHIR," J Biomed Inform, DOI 10.1016/j.jbi.2023.104305) demonstrates an EBM-on-FHIR + CPG-on-FHIR profile bundle for evidence-based guideline recommendations.

### Japan-specific considerations
No known JP localization of CPG-on-FHIR. WHO SMART Guidelines (which use CPG-on-FHIR) have not been formally adopted in Japan. Japanese guideline societies (e.g., Minds, JCS) publish narrative guidelines without computable companions; this is an open research gap.

### Interoperability with the others on this list
Directly profiles FHIR Clinical Reasoning (#2); uses CQL (#1), FHIRPath (#4), Terminology Services (#5). Recommended runtime integration via CDS Hooks (#6) and SMART (#7). WHO SMART Guidelines bundle CPG-on-FHIR with DMN (#9) decision tables.

### Limitations/Known issues
Specification complexity is high; few authored guidelines reach L3/L4. Profile churn between versions. Limited authoring tooling for clinicians. The IG itself states tooling and adoption are still maturing.

### Training data proxy
Low-to-moderate. Specification documents are public; example IGs exist on GitHub but small in number. WHO SMART Guidelines repository (WorldHealthOrganization/smart-*) provides several reference implementations. CODEX+ CELIDA project provides an execution engine for EBM-on-FHIR/CPG recommendations against OMOP. LLMs have weaker priors here than for base FHIR.

---

## 4. FHIRPath and FHIR StructureMap Transformations

### Purpose
FHIRPath is a path-based navigation/extraction expression language over hierarchical models (FHIR resources, HL7 v2, CDA). FHIR Mapping Language (FML) / StructureMap is a declarative DSL for transforming between structured models (e.g., HL7 v2 → FHIR, CDA → FHIR, FHIR version conversion).

### Maintainer/Standards body
HL7 International. FHIRPath is an ANSI Normative Standard (current Normative Release v2.0.0, designated ANSI/HL7 FHIRPath R1-2020 (R2024), reaffirmed 24 July 2024; v3.0.0 in ballot). StructureMap/FML is part of the FHIR core spec (R4 onward); FML media type `text/fhir-mapping`.

### Conceptual model
FHIRPath: fluent, collection-centric tree-traversal language with functional combinators (where, select, exists, all, repeat). Operates on a tree abstraction independent of XML/JSON. StructureMap: rule-based transformation from a source DAG to target DAG; abstract syntax in StructureMap resource, concrete syntax in FML. Rules consist of source patterns, target constructors, dependent group invocations.

### Expressiveness/Semantics
FHIRPath: side-effect-free, polymorphic over collections, includes type checking via `is`/`as`, terminology functions via `%terminologies` and `memberOf()`, model-independent. StructureMap: supports recursive group invocation, transforms (create, copy, cast, evaluate FHIRPath), implicit type conversion. Maps are unidirectional.

### Composability/Modularity
FHIRPath expressions embed inside many FHIR contexts (invariants, search parameters, ActivityDefinition dynamicValue, CQL). StructureMaps compose via `group ... extends` and `imports`; can chain transformations.

### Suitability for autoformalization to IR
FHIRPath: very high — small grammar, model-independent, normative ANSI status, well-defined operator semantics. Excellent compilation target for predicates and field extraction in a clinical IR. StructureMap: high for structural transforms but less convergent — multiple correct maps can produce identical outputs.

### Formal verification potential
FHIRPath has a formal grammar (ANTLR) and could be encoded into SMT/HOL; some research in static analysis exists but no mainstream verifier. StructureMap has declarative rule semantics amenable to relational logic but no published model-checker.

### Tooling/Ecosystem maturity
FHIRPath implementations in Java (HAPI), JavaScript (fhirpath.js), .NET (Firely SDK), Python (fhirpath-py), Rust. StructureMap engines in HAPI (FHIR Mapping Engine), Matchbox, Aidbox. Reference test suite published by HL7.

### Japan-specific considerations
FHIRPath widely usable in JP Core profiles for invariants. StructureMap is the natural tool for converting legacy SS-MIX2/HL7v2 messages and Medical Markup Language (MML) into JP Core FHIR — an active gap area (Goldsmith & Kobayashi work on MML→openEHR archetypes is parallel). No Japan-specific FHIRPath extensions.

### Interoperability with the others on this list
FHIRPath is the substrate of FHIR Reasoning (#2), CQL (#1, via translation appendix), and CPG-on-FHIR (#3). StructureMap referenced from ActivityDefinition/PlanDefinition.transform (#2). Independent of openEHR (#8), DMN (#9), BPMN (#10) but conceptually substitutable for ADL-path navigation in openEHR.

### Limitations/Known issues
FHIRPath: limited aggregation, no recursion until `repeat()`, three-valued-logic edge cases. StructureMap: difficult to author, sparse tooling, debugging is hard, performance variable. Both have small expert communities relative to base FHIR.

### Training data proxy
FHIRPath: moderate — multiple language implementations on GitHub, normative spec public, well-documented. StructureMap: low — few public examples beyond HL7 spec; very limited LLM training data; authoring guidance is thin.

---

## 5. FHIR Terminology Services

### Purpose
HTTP/REST service interface for managing and operating on coded clinical terminologies via FHIR CodeSystem, ValueSet, and ConceptMap resources — providing lookup, value-set expansion, code validation, subsumption, and concept translation.

### Maintainer/Standards body
HL7 International, Vocabulary WG. Part of FHIR core (current published R5; R6 normative ballot ongoing — v6.0.0-ballot4, May 2026). Reference operations defined in `terminology-service.html`.

### Conceptual model
Three resource types: CodeSystem (defines/exposes a coded vocabulary), ValueSet (rule-based or enumerated subset of one or more code systems, with `compose` and `expansion` sections), ConceptMap (directional mappings between concepts with equivalence relationships). Operations: `$lookup`, `$validate-code`, `$expand`, `$subsumes`, `$translate`, `$closure`.

### Expressiveness/Semantics
ValueSet composition supports include/exclude with concept lists, filters on properties, intensional definitions (e.g., SNOMED CT ECL via filter), and reference to other ValueSets. ConceptMap supports element-to-element with equivalence (equivalent, broader, narrower, related-to, not-related-to, unmatched). Closure operation maintains a client-side subsumption table.

### Composability/Modularity
ValueSets reference CodeSystems by canonical URL; can compose other ValueSets; versioning explicit. ConceptMaps are unidirectional but multiple can compose to form translation chains.

### Suitability for autoformalization to IR
High as a substrate — terminology bindings provide the lexicon for any clinical IR. LLM autoformalization should generate ValueSet references rather than inline codes to ensure cross-run consistency and ontology convergence. Standardized canonical URLs aid idempotency.

### Formal verification potential
Limited at the service level; the underlying ontologies (SNOMED CT, LOINC) have description-logic foundations (SNOMED uses OWL EL profile). ConceptMap equivalence relationships allow inference about semantic preservation but no built-in contradiction detection across mappings. SPARQL-based inconsistency detection has been demonstrated for Japanese device-adverse-event terminology (Yagahara & Yokoi 2022, BMC Med Inform Decis Mak, DOI 10.1186/s12911-022-01748-2).

### Tooling/Ecosystem maturity
Mature: HAPI FHIR terminology server, Snowstorm (SNOMED), Ontoserver (CSIRO), tx.fhir.org public service, LOINC FHIR terminology service, Termite, Aidbox terminology module.

### Japan-specific considerations
MEDIS-DC maintains Japanese standard masters: standard disease names (病名マスター), standard medication codes (HOT/YJ), JLAC10/JLAC11 lab codes, standard procedure codes. JP Core binds many of these as CodeSystems. SNOMED CT is not officially adopted in Japan (no Member). ICD-10 currently used for mortality/morbidity statistics; MHLW notification (mhlw.go.jp/stf/toukei/goriyou/sippei.html) sets ICD-11 (準拠統計分類) national application from January 2027, with code count expanding from ~16,000 (ICD-10) to ~35,000 (ICD-11) and chapter count from 22 to 28. JP Core copyright headers in v1.2.0 explicitly credit JAHIS, MEDIS-DC, and Japan Dental Association as master/code-system owners. MEDIS-DC standard masters are the de-facto Japanese terminology layer; JP Core also defines local HOT/YJ medication codes.

### Interoperability with the others on this list
Used by CQL (#1) via `[CodeSystem]`/`[ValueSet]` references; by FHIR Reasoning (#2) for action conditions; FHIRPath (#4) `memberOf()` uses terminology; CPG-on-FHIR (#3) binds recommendations to ValueSets. Independent of CDS Hooks (#6) and SMART (#7) at protocol level. openEHR (#8) has its own terminology subsystem but increasingly federates with FHIR terminology services.

### Limitations/Known issues
ValueSet expansion can be expensive/inconsistent across servers (especially intensional with version flux). ConceptMap equivalence semantics are loose. Multi-language support (Japanese designations) requires careful encoding. No standardized cross-server agreement on expansion semantics.

### Training data proxy
High. CodeSystem/ValueSet examples ubiquitous in FHIR IGs; HAPI and Ontoserver well-documented. LLMs have strong priors. Japanese-specific bindings have less public training data.

---

## 6. CDS Hooks Workflow Invocation

### Purpose
HL7 specification for "hook"-based REST invocation of remote Clinical Decision Support services from within EHR workflows; CDS services return cards (info/suggestion/app-link) that the EHR renders to clinicians in near-real-time.

### Maintainer/Standards body
HL7 International, CDS Work Group (originally SMART Health IT / Boston Children's). Current stable release: CDS Hooks 2.0.1 (STU2, generated 12 March 2025; errata release of 2.0 that moved publication to the standard HL7 FHIR IG Publisher infrastructure without substantive changes); published at cds-hooks.hl7.org. A Partial Normative Ballot is anticipated in the 2025–2026 cycle.

### Conceptual model
Workflow event triggers (hooks like `patient-view`, `order-select`, `order-sign`, `encounter-start`, `appointment-book`) cause the CDS Client (EHR) to POST a context payload + prefetch FHIR data to a CDS Service endpoint. The service returns a JSON array of "cards" (with summary, indicator, suggestion actions, app-link launches) and optional system actions. Discovery via `/cds-services` endpoint listing supported hooks and prefetch templates.

### Expressiveness/Semantics
Pattern is synchronous request/response per hook. No internal logic representation; the service's reasoning is opaque (often backed by CQL/FHIR Clinical Reasoning, rules engines, or ML). Cards are presentation/action constructs, not formal recommendations.

### Composability/Modularity
Multiple services can subscribe to the same hook; EHR aggregates cards. Suggestion actions can chain into SMART app launches for richer interaction.

### Suitability for autoformalization to IR
Low — CDS Hooks is a transport/integration spec, not a knowledge representation. Useful as a deployment target but does not store logic. An autoformalized IR must back the service implementation.

### Formal verification potential
Not applicable to the protocol itself. Each service must verify internally.

### Tooling/Ecosystem maturity
Reference sandbox at sandbox.cds-hooks.org, CDS Hooks Tools (Boston Children's), HAPI CDS-Hooks server, Cerner/Oracle Health, Epic supports several hooks. Production use: drug-drug interactions, prior authorization (Da Vinci CRD IG).

### Japan-specific considerations
No known production CDS Hooks deployment in Japan. JP Core does not specify CDS Hooks. Major Japanese EHR vendors (Fujitsu, NEC, IBM Japan, SSI) are FHIR-aware but CDS Hooks adoption is nascent. Conceptually compatible with the JP Core FHIR profile stack.

### Interoperability with the others on this list
Backed commonly by FHIR Clinical Reasoning (#2) + CQL (#1); recommended invocation pattern in CPG-on-FHIR (#3). Uses FHIR R4 resources; auth via SMART Backend Services (#7). Card "link" can launch SMART apps (#7). Orthogonal to openEHR (#8), DMN (#9), BPMN (#10) — but those engines can sit behind a CDS Hooks endpoint.

### Limitations/Known issues
Synchronous-only; ill-suited to long-running reasoning. Card UI semantics vary by EHR. No standard for action audit trail. Service-side logic is invisible to verifier. Documented alert-fatigue concerns in literature.

### Training data proxy
Moderate. cds-hooks.org spec and sandbox public; GitHub repos for sandbox and tools have moderate activity. Several published academic studies. LLMs generate hook payloads and responses reliably.

---

## 7. SMART App Launch and SMART Backend Services

### Purpose
HL7 implementation guide defining OAuth 2.0 / OpenID Connect profiles for FHIR client authorization: (a) user-facing app launch (standalone or EHR-launched) with patient/encounter context, and (b) backend services using asymmetric JWT client authentication for autonomous server-to-server access.

### Maintainer/Standards body
HL7 International / FHIR Infrastructure WG (originated at Boston Children's Hospital CHIP / SMART Health IT). Current published version: SMART App Launch 2.2.0 (STU 2.2), based on FHIR R4 (package hl7.fhir.uv.smart-app-launch#2.2.0).

### Conceptual model
Two patterns: (1) App Launch — OAuth 2.0 authorization-code grant with PKCE; discovery via `.well-known/smart-configuration`; scopes like `patient/Observation.rs`, `user/*.read`; launch context tokens conveying patient, encounter, etc. (2) Backend Services — client_credentials grant with JWT bearer assertion signed by client's private key (RS384/ES384); `system/*.rs` scopes; supports FHIR Bulk Data Access.

### Expressiveness/Semantics
Defines token formats, scope syntax (v1 vs v2), launch context parameters, refresh-token behavior, token introspection. Permission v2 grants finer-grained access (e.g., `.rs` = read+search, `.cu` = create+update).

### Composability/Modularity
Standalone vs EHR launch flows. Independent of FHIR resource semantics. Pairs with FHIR Bulk Data IG for asynchronous extraction.

### Suitability for autoformalization to IR
Not applicable — security/transport spec.

### Formal verification potential
OAuth 2.0 itself has had formal security analyses (Fett, Küsters, Schmitz 2016, "A Comprehensive Formal Security Analysis of OAuth 2.0," ACM CCS 2016, pp. 1204–1215, DOI 10.1145/2976749.2978385); SMART inherits security properties. Spec compliance is testable via the ONC-mandated Inferno test suite.

### Tooling/Ecosystem maturity
Highly mature. Inferno test suite, smarthealthit.org reference implementations (smart-launcher), Keycloak SMART module, Aidbox, HAPI, Cerner/Oracle, Epic, Allscripts native support. US ONC certification mandates SMART support.

### Japan-specific considerations
No formal MHLW/JAHIS mandate for SMART. JP Core does not yet specify SMART profiles. Japanese hospitals using cloud FHIR platforms (e.g., Fujitsu Healthcare Platform launched March 2023) are introducing SMART-style flows. The forthcoming MHLW shared EHR (電子カルテ情報共有サービス) under the Digital Agency, which began limited fiscal-2024 rollout with full-scale provision targeted from fiscal 2027, may converge on SMART patterns.

### Interoperability with the others on this list
Required substrate for CDS Hooks (#6) when services need FHIR access. Used to authorize FHIR Clinical Reasoning endpoints (#2). openEHR (#8) has a parallel "SMART on openEHR" specification (specifications.openehr.org/releases/ITS-REST/development/smart_app_launch.html) for compatible patterns. Independent of CQL (#1), FHIRPath (#4), DMN (#9), BPMN (#10).

### Limitations/Known issues
Refresh token handling varies. Asymmetric key registration is often out-of-band (no widely adopted Dynamic Client Registration). Scope syntax breaking change between v1 and v2 has caused fragmentation. Token introspection optional.

### Training data proxy
High. OAuth 2.0 is ubiquitous in LLM training; SMART-specific code on GitHub (smart-on-fhir org has many repos), HL7 Inferno test suite, extensive blog posts. Stack Overflow has many `smart-on-fhir` questions.

---

## 8. OpenEHR ADL/AQL/GDL2

### Purpose
A complete two-level modeling stack: ADL (Archetype Definition Language) and Templates define computable clinical content models, AQL (Archetype Query Language) queries archetyped data, GDL2 (Guideline Definition Language v2) expresses decision logic — together providing an alternative to FHIR-centric knowledge modeling.

### Maintainer/Standards body
openEHR International (formerly openEHR Foundation). ADL current version: ADL 2 (with ADL 1.4 still widespread); AQL current; GDL2 is published in the openEHR CDS component (CDS Release-2.0.1) with STABLE status on specifications.openehr.org. The successor Process Model (PROC) component, which contains the newer Decision Language (DL), Expression Language (EL), and Task Planning (TP), was itself moved to RETIRED status in PROC Release-1.7.0 (August 2024) pending re-scoping, while the standalone GDL2 specification remains stable in the CDS component.

### Conceptual model
Two-level model: a stable Reference Model (RM) defining persistent record structures (COMPOSITION, EVALUATION, OBSERVATION, INSTRUCTION, ACTION) plus archetypes — constraint-based, maximalist clinical concept definitions — composed into operational templates (OPT). AQL is declarative path-based query with containment and path syntax over archetyped data. GDL2 is a rule-based guideline language (JSON syntax) referencing archetype-bound input/output, with preconditions and rule expressions agnostic of natural language and reference terminology.

### Expressiveness/Semantics
Archetypes provide constraint-based information models with terminology bindings (multi-terminology). AQL provides relational-style queries with archetype-aware path semantics. GDL2 supports preconditions, rules with condition-then structure, calculations, scoring (CHA2DS2-VASc, NEWS, etc.), but limited support for time-series logic (Nan et al. 2020 explicitly noted inability to represent "last three NATs are all negative"). Semantics specified in openEHR specs; ADL has a formal grammar.

### Composability/Modularity
Archetypes are versioned and specialized via inheritance; templates compose archetypes. CKM (Clinical Knowledge Manager) provides governance. GDL2 guidelines reference archetypes as input bindings, providing model-driven separation of data and logic.

### Suitability for autoformalization to IR
High conceptually — archetypes provide a stable, ontology-aligned IR substrate with explicit constraints. ADL/GDL2 JSON serialization is amenable to LLM generation, and archetype canonical paths support cross-run idempotency. However, the maximalist archetype philosophy and the requirement to select/specialize correct archetypes from CKM may reduce semantic convergence without good retrieval.

### Formal verification potential
ADL has formal grammar and AOM (Archetype Object Model). Archetype validation tools exist. GDL2 rules are amenable to translation into rule engines (e.g., Drools) but no native verifier; Nan, Tang, Feng, Wang, Li, Lu, and Duan (2020, "A Computer-Interpretable Guideline for COVID-19: Rapid Development and Dissemination," JMIR Medical Informatics 8(10):e21628, DOI 10.2196/21628, PMC7546731) manually translated GDL2 to Drools for a COVID-19 guideline, explicitly noting absence of a publicly available GDL2 execution engine. The openEHR Decision Language (DL) successor aimed to better formalize semantics but its parent PROC component is currently retired.

### Tooling/Ecosystem maturity
Archetype Designer (Better/openEHR), ADL Workbench, GDL2 Editor (Java, open source), EHRbase (open-source openEHR CDR), Better Platform, Code24, DIPS, Cabolabs EHRServer. The openEHR Clinical Knowledge Manager (CKM) hosts hundreds of governed archetypes (Svoboda, Mautner, Mouček et al. 2017, "Applying an Archetype-Based Approach to Electroencephalography/Event-Related Potential Experiments in the EEGBase Resource," PMC5382193, describe "hundreds of archetypes describing many medical domains" in the public CKM).

### Japan-specific considerations
openEHR has measurable but limited Japan adoption. NPO openEHR Japan exists (openehr.jp); Kyoto University's 千年カルテ (Millennium Karte) / Life Course Data Healthcare project (led by Naoto Kume, Shinji Kobayashi) used openEHR for nationwide EHR infrastructure piloting; Kobayashi, Kume, Nakahara, and Yoshihara (2018, "Designing Clinical Concept Models for a Nationwide Electronic Health Records System For Japan," European Journal of Biomedical Informatics, DOI 10.24105/ejbi.2018.14.1.4) report ~6,300 patients registered in the system as of 2017. Archetypes were designed to complement Medical Markup Language (MML) semantics. No MHLW endorsement; coexists alongside dominant proprietary Japanese EHR vendors (Fujitsu HOPE, NEC MegaOak, SSI, IBM Japan).

### Interoperability with the others on this list
SMART on openEHR specification (#7-equivalent) exists. openEHR-to-FHIR mappings (archetype→FHIR profile) developed by community but lossy. GDL2 (this entry) parallels CQL (#1) / FHIR Reasoning (#2) — orthogonal stacks. AQL parallels FHIRPath/FHIR search (#4). Some GDL2 rules can input/output FHIR Resources per spec design intent.

### Limitations/Known issues
Lack of public execution engine (Nan et al. 2020) and parent-component retirement of the newer DL/TP successors. de Bruin, Chen, Rappelsberger, and Adlassnig (2020, "A Comparative Study of the Arden Syntax and GDL Clinical Knowledge Representation Languages," Stud Health Technol Inform 272:187–190, DOI 10.3233/SHTI200525, PMID 32604632) compared Arden Syntax vs GDL and concluded: "Arden Syntax is a more dynamic standard, having better readability and a higher number and more diverse operators than GDL. In contrast, GDL is a more rigid language." Authoring complexity: Grangel, Campos, and Cano (2025, SSRN abstract 5431220, "Transforming Clinical Guidelines from BPMN into Guideline Definition Language using Patterns," posted September 2025) note "creating CIGs directly in GDL is a complex task that requires intensive collaboration between clinicians and IT experts," motivating BPMN→GDL transformation patterns. Smaller global ecosystem than FHIR.

### Training data proxy
Low-to-moderate. openEHR specs publicly available; CKM archetypes downloadable; gdl-lang/common-clinical-models GitHub repo. Stack Overflow `openehr` tag exists but small. Limited LLM training data compared to FHIR; ADL/GDL2 generation by LLMs is less reliable.

---

## 9. DMN + FEEL Decision Tables

### Purpose
OMG standard for representing operational business decisions: Decision Requirements Diagrams (DRDs) link decision nodes, input data, business knowledge models, and knowledge sources; decision logic is expressed via decision tables, boxed expressions, and FEEL (Friendly Enough Expression Language).

### Maintainer/Standards body
Object Management Group (OMG). Current released version: DMN 1.5 (formal specification approved by the OMG Architecture Board on 20 March 2023, superseding earlier 1.0–1.4); DMN 1.6 is currently in beta (DMN 1.6 Beta 1) on the OMG specification page. FEEL is defined within the DMN spec.

### Conceptual model
DRD: directed acyclic graph of decisions. Each decision's logic is a "boxed expression": decision table (rows of rules with input entries and output entries, plus a hit policy: UNIQUE, ANY, PRIORITY, FIRST, COLLECT, OUTPUT ORDER, RULE ORDER), boxed literal expression (FEEL text), boxed invocation, boxed function, boxed list, boxed context. FEEL is a side-effect-free, strongly-typed functional expression language based on IEEE 754-2008 Decimal 128 numbers.

### Expressiveness/Semantics
FEEL supports literals, arithmetic, ranges, lists, contexts (records), date/time, string operations, built-in functions, externally defined PMML/Java functions, iteration via `for`/`every`/`some`. Hit policies have formally defined semantics. DMN explicitly aims at unambiguous notation with executable formal semantics.

### Composability/Modularity
DRDs compose decisions; Business Knowledge Models are reusable parameterized functions; Decision Services package subsets of a DRD for external invocation. Import other DMN models.

### Suitability for autoformalization to IR
Very high for decision-table-shaped logic — the boxed/tabular structure is highly canonical and LLMs reliably produce well-formed tables. Hit-policy semantics make tables deterministic. Idempotency is strong: two semantically equivalent guideline excerpts compile to identical or row-permutation-equivalent tables. FEEL is small enough to verify.

### Formal verification potential
Strong. Calvanese, Dumas, Laurson, Maggi, Montali, and Teinemaa (2016), "Semantics and Analysis of DMN Decision Tables" (BPM 2016, LNCS 9850, DOI 10.1007/978-3-319-45348-4_13; arXiv:1603.07466) provides first-order logic semantics and geometric (iso-oriented hyper-rectangle) algorithms for overlap and missing-rule detection. Calvanese, Dumas, Maggi, and Montali (2017, "Semantic DMN: Formalizing Decision Models with Domain Knowledge," RuleML+RR 2017, DOI 10.1007/978-3-319-61252-2_6) extend with ontological background knowledge. Tools: dmn-check (github.com/red6/dmn-check) — Maven/Gradle plugin + Camunda Modeler plugin doing static analysis for duplicates, conflicts, shadowed rules, type errors; dmn-js (Camunda OSS DMN editor) with embedded checks; Signavio DMN editor; commercial Trisotech, Camunda, Red Hat Decision Manager. Survey: Grohé, Corea, and Delfmann (2021, "DMN 1.0 Verification Capabilities: An Analysis of Current Tool Support," BPM Forum 2021, DOI 10.1007/978-3-030-85440-9_3). Della Penna and Melatti (2025, "Automating Execution and Verification of BPMN+DMN Business Processes," arXiv:2512.15214) describe BDTransTest for combined BPMN+DMN behavioral verification via statistical model checking.

### Tooling/Ecosystem maturity
Mature in business/enterprise IT. Open-source engines: Camunda DMN, Drools/Kogito DMN, OpenL Tablets, Trisotech (commercial). WHO SMART Guidelines use DMN tables.

### Japan-specific considerations
No major healthcare DMN deployment known in Japan. Used industrially in finance/insurance. No Japanese-localized authoring tools. Conceptually compatible with Japanese guideline tabular content (e.g., DPC算定 tables, drug interaction tables).

### Interoperability with the others on this list
WHO SMART Guidelines and some CPG-on-FHIR (#3) authoring chains use DMN tables alongside PlanDefinition. DMN complements BPMN (#10) as a separation-of-concerns for decisions vs process. Can sit behind CDS Hooks (#6). Independent of FHIR data layer — must be bound to terminology (#5) via inputs.

### Limitations/Known issues
Hit-policy choice can change semantics subtly. FEEL numeric precision (Decimal 128) differs from some clinical platforms. No native time-series. Cross-table contradiction detection requires aggregation outside the spec. Limited healthcare adoption.

### Training data proxy
Moderate-to-high. DMN/FEEL well-covered by Camunda/Red Hat documentation and blog ecosystem; many examples on GitHub; Stack Overflow `dmn`/`camunda-dmn` tags. LLMs reliably generate decision tables.

---

## 10. BPMN / BPM+ Health / ePath for Guideline Workflow Modeling

### Purpose
BPMN is OMG's graphical/XML notation for business process modeling; BPM+ Health is a community initiative applying BPMN + CMMN + DMN to clinical pathways; ePath is the Japanese AMED-funded electronic clinical pathway standard using OAT (Outcome-Assessment-Task) units as the primary structural element.

### Maintainer/Standards body
BPMN: OMG (current formal version 2.0.2, January 2014, also ratified as ISO/IEC 19510:2013 — the ISO publication is identical to OMG BPMN 2.0.1, the predecessor to 2.0.2). BPM+ Health: originated under OMG Healthcare Domain Task Force, transitioning into HL7 as the HL7 BPM Community. ePath: Japan Association for Medical Informatics (JAMI) Standard JAMISDP04 ("ePathのデータ要素と構造に関する仕様書"); JAHIS Standard 23-102 implementation guide ("JAHIS ePath実装ガイド Ver.1.0," October 2023, https://www.jahis.jp/standard/detail/id=1020); jointly stewarded by JAMI and the Japanese Society for Clinical Pathway (JSCP); funded by AMED FY2018–FY2020.

### Conceptual model
BPMN: token-flow process model with flow objects (events, activities, gateways), connecting objects (sequence/message flows), swimlanes (pool/lane), artifacts. Formal execution semantics specified for tokens. CMMN complements BPMN for case-driven/unstructured behaviors. BPM+ Health uses BPMN for workflow + CMMN for case management + DMN for decisions, with a "Field Guide to Shareable Clinical Pathways" (v2.0, released 27 January 2020). ePath: structures clinical pathways into OAT units where Outcome = desired patient state (e.g., "hemodynamics stable"), Assessment = observation items used as judgment criteria for outcome achievement (e.g., BP 80–180 mmHg, pulse <90/min), Task = actions/work required for the assessment or to achieve the outcome — grouped under disease-day/event info (§6.5.2.1 of JAHIS 23-102). Path types: ひな型パス (template path), 施設パス (institutional path), 適用後パス (applied path). Standardized terminology is provided by JSCP's Basic Outcome Master (BOM), certified by JAHIS as an information standard.

### Expressiveness/Semantics
BPMN: rich workflow constructs (parallel/exclusive/inclusive/event-based gateways, sub-processes, boundary events, compensation, escalation). Formal semantics for major subset given as LTS / token-flow. Christiansen, Carbone, and Hildebrandt (2010, "Formal Semantics and Implementation of BPMN 2.0 Inclusive Gateways," WS-FM 2010, LNCS 6551, DOI 10.1007/978-3-642-19589-1_10) and Corradini, Morichetta, Polini, Re, and Tiezzi (2020, "Collaboration vs. choreography conformance in BPMN," arXiv:2002.04396) provide direct LTS operational semantics; inclusive-gateway non-local semantics were formalized in Christiansen et al. (BPMN 2.0 Beta 1). ePath OAT model is a structured data model (nested message structure) rather than a process-flow notation — captures expected outcomes per disease day plus variance.

### Composability/Modularity
BPMN: call activities, sub-processes, message flows between pools. CMMN: stage decomposition. ePath: path templates → institutional paths → applied paths; OAT units composable across pathway timeline, with Evaluation and Overall Evaluation sub-elements.

### Suitability for autoformalization to IR
BPMN: moderate — XML serialization is verbose; LLM-generated diagrams often need layout/semantic cleanup; graph isomorphism makes idempotency comparison nontrivial but feasible. ePath OAT: very high for Japanese guidelines because the schema directly maps to outcome/assessment/task triples that align with clinical recommendation structure; BOM provides controlled vocabulary for cross-run convergence. BPM+ Health combines the strengths but increases artifact count.

### Formal verification potential
BPMN: strong academic basis. Operational semantics in LTS (Corradini et al., arXiv:2002.04396); description-logic formalization (Ghidini, Rospocher, and Serafini, "A formalisation of BPMN in Description Logics," arXiv:2109.10716, 2021); inclusive-gateway formal semantics work (Christiansen et al. 2010). Token semantics permit Petri-net translation and model checking (LoLA, mCRL2, ProM). BPMN+DMN combined verification via BDTransTest (Della Penna and Melatti 2025, arXiv:2512.15214). ePath data-model has no native verification but variance detection against expected outcomes provides empirical conformance checking.

### Tooling/Ecosystem maturity
BPMN: very mature — Camunda Modeler, bpmn.io (Camunda OSS), Activiti, Flowable, jBPM, Signavio, Bizagi, Bonita, Trisotech. BPM+ Health: emerging — Field Guide v2.0 published, pilot studies in emergency medicine (McClay and Goyal 2020, "Piloting Implementation and Dissemination of Best Practice Guidelines Using BPM+Health," J Clin Transl Sci 4(s1):141–142, PMC8822958, modeling first-trimester bleeding and non-traumatic low back pain protocols with ACEP) reported. ePath: production deployments at Saiseikai Kumamoto Hospital (clinical pathway program since 1997, EHR-integrated since 2010, ePath system from 2018), Kyushu University Hospital (since 2018), National Hospital Organization Shikoku Cancer Center, NTT Medical Center Tokyo, and other AMED demonstration sites; commercial implementations via Precision Co., Ltd.

### Japan-specific considerations
ePath is uniquely Japanese, funded by AMED 2018–2020 under PI Hidehisa Soejima (Saiseikai Kumamoto Hospital), jointly led by JAMI and JSCP. Native integration with DPC (Diagnosis-Procedure Combination) data; multi-vendor implementation across four demonstration hospitals using different EHR vendors. BOM (Basic Outcome Master) provides Japanese-language standardized outcome vocabulary supervised by JSCP. Soejima, Matsumoto, Nakashima, Nohara, Yamashita, Machida, and Nakaguma (2021, "A functional learning health system in Japan: Experience with processes and information infrastructure toward continuous health improvement," *Learning Health Systems* 5(4):e10252, DOI 10.1002/lrh2.10252) describes ePath as the data infrastructure for Japan's Learning Health System. JAHIS Standard 23-102 published October 2023. Tou, Matsumoto, Hashinokuchi, Kinoshita, Nohara, Yamashita, Wakata, Takenaka, Soejima, Yoshizumi, Nakashima, and Kamouchi (2025, *JMIR Medical Informatics* 13:e71617, DOI 10.2196/71617) demonstrate ML over ePath real-world data for prolonged-length-of-stay prediction in lung-cancer VATS pathways at Kyushu University Hospital. BPMN itself has limited Japanese clinical adoption.

### Interoperability with the others on this list
BPMN ↔ DMN (#9) is the canonical pairing (process invokes decisions). BPMN can wrap CDS Hooks (#6) calls as service tasks. BPM+ Health is conceptually overlapping with CPG-on-FHIR (#3) — different communities, similar goals. ePath OAT units can be expressed as FHIR PlanDefinition/Goal/Task/CarePlan (#2) but no canonical mapping exists; Grangel, Campos, and Cano (2025, SSRN abstract 5431220) demonstrate BPMN→GDL2 (#8) transformations. ePath's BOM could be exposed as FHIR CodeSystem/ValueSet (#5).

### Limitations/Known issues
BPMN: inclusive-gateway and event-subprocess semantics complex; large diagrams unmaintainable; graphical layout not part of process semantics. BPM+ Health: small healthcare adoption, limited shared-pathway library. ePath: Japanese-only specification (no English normative document — only journal-paper English descriptions in Soejima 2021 and Tou 2025), limited vendor neutrality despite multi-vendor pilots, OAT model focused on inpatient pathways rather than chronic disease management, no formal contradiction-detection layer.

### Training data proxy
BPMN: high — Camunda blog/docs, bpmn.io library widely used, Stack Overflow `bpmn` tag has thousands of questions, many GitHub modelers. BPM+ Health: low — small specialist community, few public artifacts. ePath: very low — Japanese-language specs only (JAHIS PDF, JAMI), few public examples; LLMs have minimal training data; English-language papers (Soejima 2021, Tou 2025) provide limited technical detail.
