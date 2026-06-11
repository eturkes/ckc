# Category 1: Computable Guideline IR, CDS Standards, and Workflow Interfaces

## 1. CQL/ELM Clinical Logic IR

### Purpose
Clinical Quality Language (CQL): high-level DSL for clinical knowledge (quality measures, decision-support logic). Expression Logical Model (ELM): its machine-readable canonical compiled form for sharing/execution.

### Maintainer/Standards body
HL7 International; CDS WG steward, CQI WG co-sponsor. Normative: CQL 1.5.3 (ANSI normative; Release 1 Errata 2, FHIR 4.0.1). Release 2.0.0 in trial-use ballot (CQL 2.0.0-ballot, Sep 2025). Origin: Clinical Quality Framework (CQF) initiative, CMS+ONC public-private partnership.

### Conceptual model
Declarative, expression-oriented DSL, ANTLR4 grammar; ELM = AST serialized XML/JSON via XMI/UML schema. Libraries of named expression definitions, parameters, contexts (Patient/Population), queries over retrieve expressions targeting a data model (typically FHIR or QDM).

### Expressiveness/Semantics
Set-based query, temporal operators (interval algebra over time), terminology ops, code/value-set membership, arithmetic, three-valued logic (TRUE/FALSE/NULL) with null-propagating arithmetic and short-circuit logicals. Lacks imperative control flow, side effects, process/workflow constructs. Semantics: English prose + UML; no published mechanized denotational semantics.

### Composability/Modularity
Libraries first-class: versioned, including/using imports, parameterization, accessible/private visibility. Expressions cross-reference by name; data and terminology dependencies declarable.

### Suitability for autoformalization to IR
High. Surface syntax LLM-amenable; ELM = normalized AST — syntactically different CQL compiles to canonical form, enabling structural idempotency checks; convergence measurable by ELM tree-diff after canonical normalization. Library/parameter naming discipline needed for cross-run ontology stability.

### Formal verification potential
No native theorem-proving backend. Closest: Hekmatnejad, Simms, Fainekos (2019), "Model Checking Clinical Decision Support Systems Using SMT" (arXiv:1901.04545): HL7 CDS Knowledge Artifact logic (ELM sibling) → Z3 SMT models. No published Coq/Isabelle/NuSMV translations of CQL/ELM. Three-valued logic/null semantics require careful SMT encoding.

### Tooling/Ecosystem maturity
Mature: cqframework/clinical_quality_language (Java/Kotlin CQL-to-ELM translator, release 4.8.0, May 2026), cqframework/cql-execution (JavaScript engine, 80 stars), HAPI FHIR embedded CQL engine, IBM/Alvearie CQL evaluator. NIH/CMS-sponsored ecosystem (eCQM measures, opioid CDS).

### Japan-specific considerations
No known production CQL deployment. JP Core FHIR profiles bind no CQL libraries. MHLW/JAHIS standardization targets FHIR profiles and SS-MIX, not CQL. No published Japanese CQL authoring tooling or value-set bindings to MEDIS-DC standard masters.

### Interoperability with the others on this list
In FHIR Library (#2); referenced by PlanDefinition/Measure (#2), CPG-on-FHIR (#3). FHIRPath (#4): embedded sub-language. Terminology via FHIR Terminology Services (#5). CDS Hooks services (#6) commonly CQL-backed. Distinct from openEHR GDL2 (#8); DMN/FEEL (#9), BPMN (#10) orthogonal.

### Limitations/Known issues
No mechanized formal semantics; null/three-valued logic complicates verification. Limited temporal-series/stateful reasoning. Steep learning curve outside US eCQM community. ELM XML verbose, version-coupled to FHIR releases.

### Training data proxy
Moderate. cqframework/clinical_quality_language: 313 stars/143 forks (May 2026); CMS eCQM ecosystem: several thousand published CQL files. Stack Overflow presence minimal; HL7 Zulip CQL stream is primary community channel. LLMs saw spec text and example libraries, limited diverse codebases.

## 2. FHIR Clinical Reasoning Resources

### Purpose
FHIR resources (Library, PlanDefinition, ActivityDefinition, Measure) packaging shareable executable clinical knowledge artifacts — decision rules, order sets, protocols, quality measures.

### Maintainer/Standards body
HL7 CDS + CQI WGs. Published: FHIR R5 (5.0.0, 26 Mar 2023); R6 normative ballot in progress (v6.0.0-ballot4, generated 18 May 2026) targeting normative Library, Measure, ActivityDefinition, PlanDefinition; final R6 publication 2027 at earliest.

### Conceptual model
Resource-graph metadata wrappers. Library wraps logic (often CQL/ELM base64 attachment); ActivityDefinition: parameterized template for a single Request resource (MedicationRequest, ServiceRequest, etc.); PlanDefinition: hierarchical action tree (triggers, conditions, related actions, grouping behaviors) representing event-condition-action (ECA) rules, order sets, or protocols; Measure binds populations/stratifiers to CQL expressions.

### Expressiveness/Semantics
ECA with hierarchical action grouping; related-action ordering (before/after/concurrent); selection-behavior (any/all/one-or-more/at-most-one); required-behavior; cardinality-behavior. Dynamic values via embedded FHIRPath/CQL. Semantics defined by `$apply` producing RequestOrchestration/CarePlan output bundles.

### Composability/Modularity
PlanDefinitions reference ActivityDefinitions/Libraries by canonical URL; nesting via action.definitionCanonical to other PlanDefinitions; versioned canonical URLs for controlled reuse. CRMI (Canonical Resource Management Infrastructure) IG governs packaging.

### Suitability for autoformalization to IR
High for orchestration/structural layer; action trees straightforward LLM targets; JSON/XML canonical serialization permits structural idempotency comparison. Logic delegated to embedded CQL/FHIRPath — convergence depends on those layers. Action graph convergent under canonical-URL referencing.

### Formal verification potential
None native. Action ordering/selection-behavior: informal English semantics; model checking requires translation to process formalism (BPMN-like Petri nets, pi-calculus). Cross-PlanDefinition contradictions not natively detectable.

### Tooling/Ecosystem maturity
HAPI FHIR: `$apply`, `$package`, `$data-requirements`. CQF Ruler reference implementation. Production: US Opioid CDS, CDC immunization forecasting, maternal health IGs.

### Japan-specific considerations
JP Core (FHIR Japanese implementation research WG, JAMI NeXEHRS) profiles base resources only; no Clinical Reasoning module profiles in released v1.2.0 (published 30 Jul 2025, FHIR 4.0.1); v1.3.0-dev (also FHIR 4.0.1) in active development as of early 2026. No MHLW/JAHIS-mandated PlanDefinition CDS. Adoption requires JP-specific terminology binding to MEDIS-DC masters.

### Interoperability with the others on this list
Library wraps CQL/ELM (#1). FHIRPath (#4) inline expressions; StructureMap (#4) ActivityDefinition→resource transforms; Terminology Services (#5) value sets. Invoked via CDS Hooks (#6), SMART (#7) app-launch. Parallel to but distinct from openEHR GDL2 (#8). CPG-on-FHIR (#3) profiles these resources.

### Limitations/Known issues
Action-graph concurrency semantics under-specified. PlanDefinition unwieldy for large guidelines. No native cross-artifact conflict detection. Heavy embedded-CQL reliance — toolchains must support both.

### Training data proxy
Moderate-to-high. hapifhir/hapi-fhir: 2.3k stars/1.5k forks (May 2026); hapifhir/hapi-fhir-jpaserver-starter: 520 stars/1.3k forks; many IGs on hl7.org; substantial Zulip archive. Stack Overflow `hl7-fhir` tag: thousands of questions, few on Clinical Reasoning. LLMs reliably generate well-formed PlanDefinition skeletons.

## 3. CPG-on-FHIR / Clinical Quality Framework Artifact Packaging

### Purpose
HL7 IG (Clinical Practice Guidelines on FHIR): patterns, profiles, conformance, levels-of-enablement model for representing entire computable clinical practice guidelines via FHIR Clinical Reasoning resources.

### Maintainer/Standards body
HL7 CDS WG (CQF initiative). Published: 2.0.0 (STU2, FHIR R4). Builds on parallel Using-CQL-With-FHIR IG and CRMI IG.

### Conceptual model
Levels of enablement: L1 narrative → L2 semi-structured → L3 computable/structured → L4 executable. CPG knowledge-engineering process integrated with guideline authoring. PlanDefinition profiled into CPGStrategy, CPGPathway, CPGRecommendation, CPGCaseFeatureDefinition, etc.; ActivityDefinition into typed recommendation activities.

### Expressiveness/Semantics
Inherits FHIR Clinical Reasoning semantics (#2). Adds conformance constraints: artifact packaging, identifier patterns, metadata for provenance, evidence linkage (via EBM-on-FHIR), recommendation strength. Manual/semi-automated/automated implementation paths defined.

### Composability/Modularity
Strong: explicit canonical-URL versioning; $package produces transportable Bundles. Composes with EBM-on-FHIR (evidence), DEQM (measurement). Encourages pathway/strategy decomposition.

### Suitability for autoformalization to IR
High for structural/metadata layer; medium for clinical logic (CQL-dependent). L1-L4 layering suits LLM staged generation. Profile constraints restrict valid shapes, improving convergence/idempotency.

### Formal verification potential
None native. Recommendations/pathways liftable into process models; IG specifies no verification. Cross-guideline contradiction detection requires external reasoning over constrained PlanDefinition graph plus shared terminology.

### Tooling/Ecosystem maturity
CQF Ruler reference implementation; sample IGs: CHF, opioid prescribing, WHO SMART Guidelines (ANC, immunization). Community smaller than base FHIR. Lichtner, Alper, Jurth, Spies, Boeker, Meerpohl, von Dincklage (2023, "Representation of evidence-based clinical practice guideline recommendations on FHIR," J Biomed Inform, DOI 10.1016/j.jbi.2023.104305): EBM-on-FHIR + CPG-on-FHIR profile bundle for evidence-based guideline recommendations.

### Japan-specific considerations
No known JP localization. WHO SMART Guidelines (which use CPG-on-FHIR) not formally adopted in Japan. Japanese guideline societies (e.g., Minds, JCS) publish narrative guidelines without computable companions — open research gap.

### Interoperability with the others on this list
Directly profiles #2; uses CQL (#1), FHIRPath (#4), Terminology Services (#5). Recommended runtime integration via CDS Hooks (#6), SMART (#7). WHO SMART Guidelines bundle CPG-on-FHIR with DMN (#9) decision tables.

### Limitations/Known issues
High specification complexity; few authored guidelines reach L3/L4. Profile churn between versions. Limited clinician authoring tooling. IG itself states tooling and adoption still maturing.

### Training data proxy
Low-to-moderate. Specs public; example IGs on GitHub few. WHO SMART Guidelines repos (WorldHealthOrganization/smart-*): several reference implementations. CODEX+ CELIDA project: execution engine for EBM-on-FHIR/CPG recommendations against OMOP. LLM priors weaker than base FHIR.

## 4. FHIRPath and FHIR StructureMap Transformations

### Purpose
FHIRPath: path-based navigation/extraction expression language over hierarchical models (FHIR resources, HL7 v2, CDA). FHIR Mapping Language (FML)/StructureMap: declarative DSL transforming between structured models (HL7 v2 → FHIR, CDA → FHIR, FHIR version conversion).

### Maintainer/Standards body
HL7 International. FHIRPath: ANSI Normative Standard (Normative Release v2.0.0, designated ANSI/HL7 FHIRPath R1-2020 (R2024), reaffirmed 24 Jul 2024; v3.0.0 in ballot). StructureMap/FML: FHIR core spec (R4 onward); FML media type `text/fhir-mapping`.

### Conceptual model
FHIRPath: fluent, collection-centric tree traversal with functional combinators (where, select, exists, all, repeat); tree abstraction independent of XML/JSON. StructureMap: rule-based source-DAG→target-DAG transformation; abstract syntax in StructureMap resource, concrete syntax in FML; rules = source patterns + target constructors + dependent group invocations.

### Expressiveness/Semantics
FHIRPath: side-effect-free, polymorphic over collections, `is`/`as` type checking, terminology functions via `%terminologies` and `memberOf()`, model-independent. StructureMap: recursive group invocation; transforms (create, copy, cast, evaluate FHIRPath); implicit type conversion; maps unidirectional.

### Composability/Modularity
FHIRPath embeds in many FHIR contexts (invariants, search parameters, ActivityDefinition dynamicValue, CQL). StructureMaps compose via `group ... extends` and `imports`; chainable.

### Suitability for autoformalization to IR
FHIRPath: very high — small grammar, model-independent, normative ANSI status, well-defined operator semantics; excellent compilation target for predicates/field extraction in a clinical IR. StructureMap: high for structural transforms but less convergent — multiple correct maps can produce identical outputs.

### Formal verification potential
FHIRPath: formal ANTLR grammar; SMT/HOL-encodable; some static-analysis research, no mainstream verifier. StructureMap: declarative rule semantics amenable to relational logic; no published model-checker.

### Tooling/Ecosystem maturity
FHIRPath: Java (HAPI), JavaScript (fhirpath.js), .NET (Firely SDK), Python (fhirpath-py), Rust. StructureMap engines: HAPI (FHIR Mapping Engine), Matchbox, Aidbox. HL7-published reference test suite.

### Japan-specific considerations
FHIRPath widely usable in JP Core profile invariants. StructureMap natural for converting legacy SS-MIX2/HL7v2 and Medical Markup Language (MML) into JP Core FHIR — active gap area (Goldsmith & Kobayashi MML→openEHR archetype work is parallel). No Japan-specific FHIRPath extensions.

### Interoperability with the others on this list
FHIRPath: substrate of FHIR Reasoning (#2), CQL (#1, via translation appendix), CPG-on-FHIR (#3). StructureMap referenced from ActivityDefinition/PlanDefinition.transform (#2). Independent of openEHR (#8), DMN (#9), BPMN (#10) but conceptually substitutable for ADL-path navigation in openEHR.

### Limitations/Known issues
FHIRPath: limited aggregation, no recursion until `repeat()`, three-valued-logic edge cases. StructureMap: difficult to author, sparse tooling, hard debugging, variable performance. Both have small expert communities relative to base FHIR.

### Training data proxy
FHIRPath: moderate — multiple GitHub language implementations, public normative spec, well-documented. StructureMap: low — few public examples beyond HL7 spec; very limited LLM training data; thin authoring guidance.

## 5. FHIR Terminology Services

### Purpose
HTTP/REST service interface for coded clinical terminologies via FHIR CodeSystem, ValueSet, ConceptMap — lookup, value-set expansion, code validation, subsumption, concept translation.

### Maintainer/Standards body
HL7 Vocabulary WG. FHIR core (published R5; R6 normative ballot ongoing — v6.0.0-ballot4, May 2026). Reference operations in `terminology-service.html`.

### Conceptual model
CodeSystem (defines/exposes a coded vocabulary), ValueSet (rule-based or enumerated subset of code systems; `compose` + `expansion` sections), ConceptMap (directional concept mappings with equivalence relationships). Operations: `$lookup`, `$validate-code`, `$expand`, `$subsumes`, `$translate`, `$closure`.

### Expressiveness/Semantics
ValueSet composition: include/exclude concept lists, property filters, intensional definitions (e.g., SNOMED CT ECL via filter), references to other ValueSets. ConceptMap: element-to-element with equivalence (equivalent, broader, narrower, related-to, not-related-to, unmatched). `$closure` maintains client-side subsumption table.

### Composability/Modularity
ValueSets reference CodeSystems by canonical URL; can compose other ValueSets; explicit versioning. ConceptMaps unidirectional but composable into translation chains.

### Suitability for autoformalization to IR
High as substrate — terminology bindings provide the lexicon for any clinical IR. LLM autoformalization should emit ValueSet references rather than inline codes for cross-run consistency/ontology convergence. Standardized canonical URLs aid idempotency.

### Formal verification potential
Limited at service level; underlying ontologies (SNOMED CT, LOINC) have description-logic foundations (SNOMED uses OWL EL profile). ConceptMap equivalence relationships allow semantic-preservation inference; no built-in cross-mapping contradiction detection. SPARQL-based inconsistency detection demonstrated for Japanese device-adverse-event terminology (Yagahara & Yokoi 2022, BMC Med Inform Decis Mak, DOI 10.1186/s12911-022-01748-2).

### Tooling/Ecosystem maturity
Mature: HAPI FHIR terminology server, Snowstorm (SNOMED), Ontoserver (CSIRO), tx.fhir.org public service, LOINC FHIR terminology service, Termite, Aidbox terminology module.

### Japan-specific considerations
MEDIS-DC maintains Japanese standard masters: standard disease names (病名マスター), standard medication codes (HOT/YJ), JLAC10/JLAC11 lab codes, standard procedure codes. JP Core binds many as CodeSystems. SNOMED CT not officially adopted in Japan (no Member). ICD-10 currently used for mortality/morbidity statistics; MHLW notification (mhlw.go.jp/stf/toukei/goriyou/sippei.html) sets ICD-11 (準拠統計分類) national application from January 2027; code count ~16,000 (ICD-10) → ~35,000 (ICD-11); chapters 22 → 28. JP Core v1.2.0 copyright headers explicitly credit JAHIS, MEDIS-DC, Japan Dental Association as master/code-system owners. MEDIS-DC masters: de-facto Japanese terminology layer; JP Core also defines local HOT/YJ medication codes.

### Interoperability with the others on this list
CQL (#1) via `[CodeSystem]`/`[ValueSet]` references; FHIR Reasoning (#2) action conditions; FHIRPath (#4) `memberOf()`; CPG-on-FHIR (#3) binds recommendations to ValueSets. Protocol-independent of CDS Hooks (#6), SMART (#7). openEHR (#8) has own terminology subsystem but increasingly federates with FHIR terminology services.

### Limitations/Known issues
ValueSet expansion expensive/inconsistent across servers (especially intensional with version flux). ConceptMap equivalence semantics loose. Multi-language support (Japanese designations) requires careful encoding. No standardized cross-server agreement on expansion semantics.

### Training data proxy
High. CodeSystem/ValueSet examples ubiquitous in FHIR IGs; HAPI/Ontoserver well-documented; strong LLM priors. Japanese-specific bindings: less public training data.

## 6. CDS Hooks Workflow Invocation

### Purpose
HL7 spec for "hook"-based REST invocation of remote CDS services from EHR workflows; services return cards (info/suggestion/app-link) the EHR renders to clinicians near-real-time.

### Maintainer/Standards body
HL7 CDS WG (originally SMART Health IT / Boston Children's). Stable: CDS Hooks 2.0.1 (STU2, generated 12 Mar 2025; errata release of 2.0 moving publication to the standard HL7 FHIR IG Publisher infrastructure without substantive changes); published at cds-hooks.hl7.org. Partial Normative Ballot anticipated in 2025–2026 cycle.

### Conceptual model
Workflow event hooks (`patient-view`, `order-select`, `order-sign`, `encounter-start`, `appointment-book`): CDS Client (EHR) POSTs context payload + prefetch FHIR data to CDS Service endpoint; service returns JSON array of "cards" (summary, indicator, suggestion actions, app-link launches) + optional system actions. Discovery via `/cds-services` endpoint listing supported hooks and prefetch templates.

### Expressiveness/Semantics
Synchronous request/response per hook. No internal logic representation; service reasoning opaque (often CQL/FHIR Clinical Reasoning, rules engines, ML). Cards are presentation/action constructs, not formal recommendations.

### Composability/Modularity
Multiple services per hook; EHR aggregates cards. Suggestion actions chain into SMART app launches for richer interaction.

### Suitability for autoformalization to IR
Low — transport/integration spec, not knowledge representation. Deployment target only, stores no logic; autoformalized IR must back the service implementation.

### Formal verification potential
Not applicable to the protocol; each service must verify internally.

### Tooling/Ecosystem maturity
Reference sandbox sandbox.cds-hooks.org, CDS Hooks Tools (Boston Children's), HAPI CDS-Hooks server, Cerner/Oracle Health, Epic supports several hooks. Production: drug-drug interactions, prior authorization (Da Vinci CRD IG).

### Japan-specific considerations
No known production deployment in Japan. JP Core does not specify CDS Hooks. Major Japanese EHR vendors (Fujitsu, NEC, IBM Japan, SSI) FHIR-aware; CDS Hooks adoption nascent. Conceptually compatible with JP Core FHIR profile stack.

### Interoperability with the others on this list
Commonly backed by FHIR Clinical Reasoning (#2) + CQL (#1); recommended invocation pattern in CPG-on-FHIR (#3). Uses FHIR R4 resources; auth via SMART Backend Services (#7); card "link" can launch SMART apps (#7). Orthogonal to openEHR (#8), DMN (#9), BPMN (#10) — those engines can sit behind a CDS Hooks endpoint.

### Limitations/Known issues
Synchronous-only; ill-suited to long-running reasoning. Card UI semantics vary by EHR. No standard for action audit trail. Service-side logic invisible to verifier. Documented alert-fatigue concerns in literature.

### Training data proxy
Moderate. cds-hooks.org spec and sandbox public; GitHub sandbox/tools repos moderately active; several published academic studies. LLMs generate hook payloads and responses reliably.

## 7. SMART App Launch and SMART Backend Services

### Purpose
HL7 IG: OAuth 2.0 / OpenID Connect profiles for FHIR client authorization — (a) user-facing app launch (standalone or EHR-launched) with patient/encounter context; (b) backend services using asymmetric JWT client authentication for autonomous server-to-server access.

### Maintainer/Standards body
HL7 / FHIR Infrastructure WG (originated at Boston Children's Hospital CHIP / SMART Health IT). Published: SMART App Launch 2.2.0 (STU 2.2), FHIR R4 (package hl7.fhir.uv.smart-app-launch#2.2.0).

### Conceptual model
(1) App Launch: OAuth 2.0 authorization-code grant with PKCE; discovery via `.well-known/smart-configuration`; scopes like `patient/Observation.rs`, `user/*.read`; launch context tokens conveying patient, encounter, etc. (2) Backend Services: client_credentials grant with JWT bearer assertion signed by client's private key (RS384/ES384); `system/*.rs` scopes; supports FHIR Bulk Data Access.

### Expressiveness/Semantics
Defines token formats, scope syntax (v1 vs v2), launch context parameters, refresh-token behavior, token introspection. v2 finer-grained (e.g., `.rs` = read+search, `.cu` = create+update).

### Composability/Modularity
Standalone vs EHR launch flows. Independent of FHIR resource semantics. Pairs with FHIR Bulk Data IG for asynchronous extraction.

### Suitability for autoformalization to IR
Not applicable — security/transport spec.

### Formal verification potential
OAuth 2.0 formally analyzed (Fett, Küsters, Schmitz 2016, "A Comprehensive Formal Security Analysis of OAuth 2.0," ACM CCS 2016, pp. 1204–1215, DOI 10.1145/2976749.2978385); SMART inherits security properties. Spec compliance testable via ONC-mandated Inferno test suite.

### Tooling/Ecosystem maturity
Highly mature: Inferno test suite, smarthealthit.org reference implementations (smart-launcher), Keycloak SMART module, Aidbox, HAPI, Cerner/Oracle, Epic, Allscripts native support. US ONC certification mandates SMART.

### Japan-specific considerations
No formal MHLW/JAHIS mandate. JP Core does not yet specify SMART profiles. Japanese hospitals on cloud FHIR platforms (e.g., Fujitsu Healthcare Platform, launched Mar 2023) introducing SMART-style flows. Forthcoming MHLW shared EHR (電子カルテ情報共有サービス) under the Digital Agency — limited fiscal-2024 rollout, full-scale provision targeted from fiscal 2027 — may converge on SMART patterns.

### Interoperability with the others on this list
Required substrate for CDS Hooks (#6) when services need FHIR access. Authorizes FHIR Clinical Reasoning endpoints (#2). openEHR (#8) has parallel "SMART on openEHR" spec (specifications.openehr.org/releases/ITS-REST/development/smart_app_launch.html). Independent of CQL (#1), FHIRPath (#4), DMN (#9), BPMN (#10).

### Limitations/Known issues
Refresh-token handling varies. Asymmetric key registration often out-of-band (no widely adopted Dynamic Client Registration). v1→v2 scope-syntax breaking change caused fragmentation. Token introspection optional.

### Training data proxy
High. OAuth 2.0 ubiquitous in LLM training; smart-on-fhir GitHub org has many repos; HL7 Inferno test suite; extensive blog posts; many Stack Overflow `smart-on-fhir` questions.

## 8. OpenEHR ADL/AQL/GDL2

### Purpose
Complete two-level modeling stack: ADL (Archetype Definition Language) + Templates define computable clinical content models; AQL (Archetype Query Language) queries archetyped data; GDL2 (Guideline Definition Language v2) expresses decision logic — alternative to FHIR-centric knowledge modeling.

### Maintainer/Standards body
openEHR International (formerly openEHR Foundation). ADL current: ADL 2 (ADL 1.4 still widespread); AQL current; GDL2 published in openEHR CDS component (CDS Release-2.0.1), STABLE status on specifications.openehr.org. Successor Process Model (PROC) component — newer Decision Language (DL), Expression Language (EL), Task Planning (TP) — moved to RETIRED in PROC Release-1.7.0 (Aug 2024) pending re-scoping; standalone GDL2 spec remains stable in CDS component.

### Conceptual model
Two-level: stable Reference Model (RM) for persistent record structures (COMPOSITION, EVALUATION, OBSERVATION, INSTRUCTION, ACTION) + archetypes — constraint-based, maximalist clinical concept definitions — composed into operational templates (OPT). AQL: declarative path-based query with containment and path syntax over archetyped data. GDL2: rule-based guideline language (JSON syntax) referencing archetype-bound input/output, with preconditions and rule expressions agnostic of natural language and reference terminology.

### Expressiveness/Semantics
Archetypes: constraint-based information models with multi-terminology bindings. AQL: relational-style queries with archetype-aware path semantics. GDL2: preconditions, condition-then rules, calculations, scoring (CHA2DS2-VASc, NEWS, etc.); limited time-series logic (Nan et al. 2020 explicitly noted inability to represent "last three NATs are all negative"). Semantics in openEHR specs; ADL has formal grammar.

### Composability/Modularity
Archetypes versioned, specialized via inheritance; templates compose archetypes. CKM (Clinical Knowledge Manager) governance. GDL2 guidelines reference archetypes as input bindings — model-driven data/logic separation.

### Suitability for autoformalization to IR
High conceptually — archetypes provide stable, ontology-aligned IR substrate with explicit constraints. ADL/GDL2 JSON serialization LLM-amenable; archetype canonical paths support cross-run idempotency. Maximalist archetype philosophy plus requirement to select/specialize correct CKM archetypes may reduce semantic convergence without good retrieval.

### Formal verification potential
ADL: formal grammar + AOM (Archetype Object Model); archetype validation tools exist. GDL2 rules translatable to rule engines (e.g., Drools), no native verifier; Nan, Tang, Feng, Wang, Li, Lu, Duan (2020, "A Computer-Interpretable Guideline for COVID-19: Rapid Development and Dissemination," JMIR Medical Informatics 8(10):e21628, DOI 10.2196/21628, PMC7546731) manually translated GDL2 to Drools for a COVID-19 guideline, explicitly noting absence of a publicly available GDL2 execution engine. DL successor aimed to better formalize semantics; its parent PROC component currently retired.

### Tooling/Ecosystem maturity
Archetype Designer (Better/openEHR), ADL Workbench, GDL2 Editor (Java, open source), EHRbase (open-source openEHR CDR), Better Platform, Code24, DIPS, Cabolabs EHRServer. CKM hosts hundreds of governed archetypes (Svoboda, Mautner, Mouček et al. 2017, "Applying an Archetype-Based Approach to Electroencephalography/Event-Related Potential Experiments in the EEGBase Resource," PMC5382193: "hundreds of archetypes describing many medical domains" in public CKM).

### Japan-specific considerations
Measurable but limited Japan adoption. NPO openEHR Japan exists (openehr.jp). Kyoto University's 千年カルテ (Millennium Karte) / Life Course Data Healthcare project (led by Naoto Kume, Shinji Kobayashi) used openEHR for nationwide EHR infrastructure piloting; Kobayashi, Kume, Nakahara, Yoshihara (2018, "Designing Clinical Concept Models for a Nationwide Electronic Health Records System For Japan," European Journal of Biomedical Informatics, DOI 10.24105/ejbi.2018.14.1.4) report ~6,300 patients registered as of 2017. Archetypes designed to complement Medical Markup Language (MML) semantics. No MHLW endorsement; coexists with dominant proprietary Japanese EHR vendors (Fujitsu HOPE, NEC MegaOak, SSI, IBM Japan).

### Interoperability with the others on this list
SMART on openEHR spec (#7-equivalent) exists. Community openEHR-to-FHIR mappings (archetype→FHIR profile), lossy. GDL2 parallels CQL (#1)/FHIR Reasoning (#2) — orthogonal stacks. AQL parallels FHIRPath/FHIR search (#4). Some GDL2 rules can input/output FHIR Resources per spec design intent.

### Limitations/Known issues
Lack of public execution engine (Nan et al. 2020); parent-component retirement of newer DL/TP successors. de Bruin, Chen, Rappelsberger, Adlassnig (2020, "A Comparative Study of the Arden Syntax and GDL Clinical Knowledge Representation Languages," Stud Health Technol Inform 272:187–190, DOI 10.3233/SHTI200525, PMID 32604632) compared Arden Syntax vs GDL: "Arden Syntax is a more dynamic standard, having better readability and a higher number and more diverse operators than GDL. In contrast, GDL is a more rigid language." Authoring complexity: Grangel, Campos, Cano (2025, SSRN abstract 5431220, "Transforming Clinical Guidelines from BPMN into Guideline Definition Language using Patterns," posted Sep 2025) note "creating CIGs directly in GDL is a complex task that requires intensive collaboration between clinicians and IT experts," motivating BPMN→GDL transformation patterns. Smaller global ecosystem than FHIR.

### Training data proxy
Low-to-moderate. Specs public; CKM archetypes downloadable; gdl-lang/common-clinical-models GitHub repo. Stack Overflow `openehr` tag small. Limited LLM training data vs FHIR; ADL/GDL2 generation less reliable.

## 9. DMN + FEEL Decision Tables

### Purpose
OMG standard for operational business decisions: Decision Requirements Diagrams (DRDs) link decision nodes, input data, business knowledge models, knowledge sources; decision logic via decision tables, boxed expressions, FEEL (Friendly Enough Expression Language).

### Maintainer/Standards body
Object Management Group (OMG). Released: DMN 1.5 (formal spec approved by OMG Architecture Board 20 Mar 2023, superseding 1.0–1.4); DMN 1.6 in beta (DMN 1.6 Beta 1) on OMG spec page. FEEL defined within DMN spec.

### Conceptual model
DRD: directed acyclic graph of decisions. Per-decision "boxed expression": decision table (rule rows with input/output entries plus hit policy: UNIQUE, ANY, PRIORITY, FIRST, COLLECT, OUTPUT ORDER, RULE ORDER), boxed literal expression (FEEL text), boxed invocation, boxed function, boxed list, boxed context. FEEL: side-effect-free, strongly-typed functional expression language on IEEE 754-2008 Decimal 128 numbers.

### Expressiveness/Semantics
FEEL: literals, arithmetic, ranges, lists, contexts (records), date/time, string operations, built-in functions, externally defined PMML/Java functions, iteration via `for`/`every`/`some`. Hit policies have formally defined semantics. DMN explicitly aims at unambiguous notation with executable formal semantics.

### Composability/Modularity
DRDs compose decisions; Business Knowledge Models: reusable parameterized functions; Decision Services package DRD subsets for external invocation. Imports other DMN models.

### Suitability for autoformalization to IR
Very high for decision-table-shaped logic — boxed/tabular structure highly canonical; LLMs reliably produce well-formed tables. Hit-policy semantics make tables deterministic. Strong idempotency: semantically equivalent guideline excerpts compile to identical or row-permutation-equivalent tables. FEEL small enough to verify.

### Formal verification potential
Strong. Calvanese, Dumas, Laurson, Maggi, Montali, Teinemaa (2016), "Semantics and Analysis of DMN Decision Tables" (BPM 2016, LNCS 9850, DOI 10.1007/978-3-319-45348-4_13; arXiv:1603.07466): first-order logic semantics + geometric (iso-oriented hyper-rectangle) algorithms for overlap and missing-rule detection. Calvanese, Dumas, Maggi, Montali (2017, "Semantic DMN: Formalizing Decision Models with Domain Knowledge," RuleML+RR 2017, DOI 10.1007/978-3-319-61252-2_6): extension with ontological background knowledge. Tools: dmn-check (github.com/red6/dmn-check) — Maven/Gradle plugin + Camunda Modeler plugin, static analysis for duplicates, conflicts, shadowed rules, type errors; dmn-js (Camunda OSS DMN editor) embedded checks; Signavio DMN editor; commercial Trisotech, Camunda, Red Hat Decision Manager. Survey: Grohé, Corea, Delfmann (2021, "DMN 1.0 Verification Capabilities: An Analysis of Current Tool Support," BPM Forum 2021, DOI 10.1007/978-3-030-85440-9_3). Della Penna and Melatti (2025, "Automating Execution and Verification of BPMN+DMN Business Processes," arXiv:2512.15214): BDTransTest for combined BPMN+DMN behavioral verification via statistical model checking.

### Tooling/Ecosystem maturity
Mature in business/enterprise IT. Open-source engines: Camunda DMN, Drools/Kogito DMN, OpenL Tablets; Trisotech (commercial). WHO SMART Guidelines use DMN tables.

### Japan-specific considerations
No major healthcare DMN deployment known in Japan. Used industrially in finance/insurance. No Japanese-localized authoring tools. Conceptually compatible with Japanese guideline tabular content (e.g., DPC算定 tables, drug interaction tables).

### Interoperability with the others on this list
WHO SMART Guidelines and some CPG-on-FHIR (#3) authoring chains use DMN tables alongside PlanDefinition. Complements BPMN (#10) as separation-of-concerns: decisions vs process. Can sit behind CDS Hooks (#6). Independent of FHIR data layer — inputs must bind to terminology (#5).

### Limitations/Known issues
Hit-policy choice can change semantics subtly. FEEL numeric precision (Decimal 128) differs from some clinical platforms. No native time-series. Cross-table contradiction detection requires aggregation outside the spec. Limited healthcare adoption.

### Training data proxy
Moderate-to-high. DMN/FEEL well-covered by Camunda/Red Hat docs and blogs; many GitHub examples; Stack Overflow `dmn`/`camunda-dmn` tags. LLMs reliably generate decision tables.

## 10. BPMN / BPM+ Health / ePath for Guideline Workflow Modeling

### Purpose
BPMN: OMG graphical/XML notation for business process modeling. BPM+ Health: community initiative applying BPMN + CMMN + DMN to clinical pathways. ePath: Japanese AMED-funded electronic clinical pathway standard using OAT (Outcome-Assessment-Task) units as the primary structural element.

### Maintainer/Standards body
BPMN: OMG, current formal 2.0.2 (Jan 2014); also ratified as ISO/IEC 19510:2013 — ISO publication identical to OMG BPMN 2.0.1, predecessor to 2.0.2. BPM+ Health: originated under OMG Healthcare Domain Task Force, transitioning into HL7 as the HL7 BPM Community. ePath: JAMI Standard JAMISDP04 ("ePathのデータ要素と構造に関する仕様書"); JAHIS Standard 23-102 implementation guide ("JAHIS ePath実装ガイド Ver.1.0," Oct 2023, https://www.jahis.jp/standard/detail/id=1020); jointly stewarded by JAMI and the Japanese Society for Clinical Pathway (JSCP); AMED-funded FY2018–FY2020.

### Conceptual model
BPMN: token-flow process model — flow objects (events, activities, gateways), connecting objects (sequence/message flows), swimlanes (pool/lane), artifacts; formal token execution semantics. CMMN complements BPMN for case-driven/unstructured behaviors. BPM+ Health: BPMN (workflow) + CMMN (case management) + DMN (decisions), with "Field Guide to Shareable Clinical Pathways" (v2.0, released 27 Jan 2020). ePath: pathways structured into OAT units — Outcome = desired patient state (e.g., "hemodynamics stable"); Assessment = observation items as judgment criteria for outcome achievement (e.g., BP 80–180 mmHg, pulse <90/min); Task = actions/work required for the assessment or to achieve the outcome — grouped under disease-day/event info (§6.5.2.1 of JAHIS 23-102). Path types: ひな型パス (template path), 施設パス (institutional path), 適用後パス (applied path). Standardized terminology: JSCP's Basic Outcome Master (BOM), certified by JAHIS as an information standard.

### Expressiveness/Semantics
BPMN: rich workflow constructs (parallel/exclusive/inclusive/event-based gateways, sub-processes, boundary events, compensation, escalation); formal semantics for major subset as LTS/token-flow. Christiansen, Carbone, Hildebrandt (2010, "Formal Semantics and Implementation of BPMN 2.0 Inclusive Gateways," WS-FM 2010, LNCS 6551, DOI 10.1007/978-3-642-19589-1_10) and Corradini, Morichetta, Polini, Re, Tiezzi (2020, "Collaboration vs. choreography conformance in BPMN," arXiv:2002.04396) provide direct LTS operational semantics; inclusive-gateway non-local semantics formalized in Christiansen et al. (BPMN 2.0 Beta 1). ePath OAT: structured data model (nested message structure), not process-flow notation — captures expected outcomes per disease day plus variance.

### Composability/Modularity
BPMN: call activities, sub-processes, message flows between pools. CMMN: stage decomposition. ePath: path templates → institutional paths → applied paths; OAT units composable across pathway timeline, with Evaluation and Overall Evaluation sub-elements.

### Suitability for autoformalization to IR
BPMN: moderate — verbose XML serialization; LLM-generated diagrams often need layout/semantic cleanup; graph isomorphism makes idempotency comparison nontrivial but feasible. ePath OAT: very high for Japanese guidelines — schema maps directly to outcome/assessment/task triples aligned with clinical recommendation structure; BOM provides controlled vocabulary for cross-run convergence. BPM+ Health combines the strengths but increases artifact count.

### Formal verification potential
BPMN: strong academic basis. LTS operational semantics (Corradini et al., arXiv:2002.04396); description-logic formalization (Ghidini, Rospocher, Serafini, "A formalisation of BPMN in Description Logics," arXiv:2109.10716, 2021); inclusive-gateway formal semantics (Christiansen et al. 2010). Token semantics permit Petri-net translation and model checking (LoLA, mCRL2, ProM). BPMN+DMN combined verification via BDTransTest (Della Penna and Melatti 2025, arXiv:2512.15214). ePath: no native verification; variance detection against expected outcomes provides empirical conformance checking.

### Tooling/Ecosystem maturity
BPMN very mature: Camunda Modeler, bpmn.io (Camunda OSS), Activiti, Flowable, jBPM, Signavio, Bizagi, Bonita, Trisotech. BPM+ Health emerging: Field Guide v2.0 published; emergency-medicine pilots (McClay and Goyal 2020, "Piloting Implementation and Dissemination of Best Practice Guidelines Using BPM+Health," J Clin Transl Sci 4(s1):141–142, PMC8822958, modeling first-trimester bleeding and non-traumatic low back pain protocols with ACEP). ePath production: Saiseikai Kumamoto Hospital (clinical pathway program since 1997, EHR-integrated since 2010, ePath system from 2018), Kyushu University Hospital (since 2018), National Hospital Organization Shikoku Cancer Center, NTT Medical Center Tokyo, other AMED demonstration sites; commercial implementations via Precision Co., Ltd.

### Japan-specific considerations
ePath uniquely Japanese: AMED-funded 2018–2020 under PI Hidehisa Soejima (Saiseikai Kumamoto Hospital), jointly led by JAMI and JSCP. Native integration with DPC (Diagnosis-Procedure Combination) data; multi-vendor implementation across four demonstration hospitals using different EHR vendors. BOM provides Japanese-language standardized outcome vocabulary supervised by JSCP. Soejima, Matsumoto, Nakashima, Nohara, Yamashita, Machida, Nakaguma (2021, "A functional learning health system in Japan: Experience with processes and information infrastructure toward continuous health improvement," *Learning Health Systems* 5(4):e10252, DOI 10.1002/lrh2.10252) describes ePath as data infrastructure for Japan's Learning Health System. JAHIS Standard 23-102 published Oct 2023. Tou, Matsumoto, Hashinokuchi, Kinoshita, Nohara, Yamashita, Wakata, Takenaka, Soejima, Yoshizumi, Nakashima, Kamouchi (2025, *JMIR Medical Informatics* 13:e71617, DOI 10.2196/71617) demonstrate ML over ePath real-world data for prolonged-length-of-stay prediction in lung-cancer VATS pathways at Kyushu University Hospital. BPMN itself: limited Japanese clinical adoption.

### Interoperability with the others on this list
BPMN ↔ DMN (#9) canonical pairing (process invokes decisions). BPMN can wrap CDS Hooks (#6) calls as service tasks. BPM+ Health conceptually overlaps CPG-on-FHIR (#3) — different communities, similar goals. ePath OAT units expressible as FHIR PlanDefinition/Goal/Task/CarePlan (#2) but no canonical mapping exists; Grangel, Campos, Cano (2025, SSRN abstract 5431220) demonstrate BPMN→GDL2 (#8) transformations. ePath's BOM could be exposed as FHIR CodeSystem/ValueSet (#5).

### Limitations/Known issues
BPMN: inclusive-gateway and event-subprocess semantics complex; large diagrams unmaintainable; graphical layout not part of process semantics. BPM+ Health: small healthcare adoption, limited shared-pathway library. ePath: Japanese-only specification (no English normative document — only journal-paper English descriptions in Soejima 2021 and Tou 2025); limited vendor neutrality despite multi-vendor pilots; OAT model focused on inpatient pathways rather than chronic disease management; no formal contradiction-detection layer.

### Training data proxy
BPMN: high — Camunda blog/docs, bpmn.io widely used, Stack Overflow `bpmn` tag thousands of questions, many GitHub modelers. BPM+ Health: low — small specialist community, few public artifacts. ePath: very low — Japanese-language specs only (JAHIS PDF, JAMI), few public examples; minimal LLM training data; English papers (Soejima 2021, Tou 2025) provide limited technical detail.
